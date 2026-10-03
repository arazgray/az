//! Wayland clipboard via `ext-data-control-v1`.
//!
//! Compositors that speak this protocol (KDE Plasma, wlroots) let a client
//! read and replace the system clipboard without `wl-copy`. The client that
//! owns a selection must stay alive to answer `send` events, so `copy` hands
//! the bytes to `az --clipboard-hold`, which outlives the editor.

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
#[cfg(not(test))]
use std::process::{Command, Stdio};
#[cfg(not(test))]
use std::sync::Mutex;
#[cfg(test)]
use std::thread;
use std::time::{Duration, Instant};

#[cfg(not(test))]
static CLIPBOARD_HOLDS: Mutex<Vec<std::process::Child>> = Mutex::new(Vec::new());

/// Put `text` on the Wayland clipboard. False when no compositor is available
/// or it does not advertise `ext-data-control-v1`.
pub(crate) fn copy(text: &str) -> bool {
    if env::var_os("WAYLAND_DISPLAY").is_none() {
        return false;
    }
    #[cfg(test)]
    {
        copy_inprocess(text)
    }
    #[cfg(not(test))]
    {
        copy_detached(text)
    }
}

/// Read the Wayland clipboard. `None` when there is no compositor, the
/// selection is empty, or the offer is not text.
pub(crate) fn paste() -> Option<String> {
    if env::var_os("WAYLAND_DISPLAY").is_none() {
        return None;
    }
    let mut conn = Conn::connect()?;
    conn.read_clipboard().ok().flatten()
}

/// `az --clipboard-hold`: read the payload on stdin, report success on
/// stdout, then serve the selection until another client replaces it.
pub(crate) fn hold_and_serve() -> ! {
    let mut text = String::new();
    let _ = io::stdin().read_to_string(&mut text);
    let Some(conn) = Conn::connect_and_copy(&text) else {
        let _ = io::stdout().write_all(b"0");
        let _ = io::stdout().flush();
        std::process::exit(1);
    };
    let _ = io::stdout().write_all(b"1");
    let _ = io::stdout().flush();
    conn.serve();
    std::process::exit(0);
}

#[cfg(test)]
fn copy_inprocess(text: &str) -> bool {
    let (tx, rx) = std::sync::mpsc::channel();
    let text = text.to_string();
    thread::spawn(move || match Conn::connect_and_copy(&text) {
        Some(conn) => {
            let _ = tx.send(true);
            conn.serve();
        }
        None => {
            let _ = tx.send(false);
        }
    });
    rx.recv_timeout(Duration::from_secs(2)).unwrap_or(false)
}

#[cfg(not(test))]
fn copy_detached(text: &str) -> bool {
    let Ok(exe) = env::current_exe() else {
        return false;
    };
    if let Ok(mut holds) = CLIPBOARD_HOLDS.lock() {
        holds.retain_mut(|child| matches!(child.try_wait(), Ok(None)));
    }
    let mut child = match Command::new(exe)
        .arg("--clipboard-hold")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
    };
    let wrote = child
        .stdin
        .as_mut()
        .map(|stdin| stdin.write_all(text.as_bytes()).is_ok())
        .unwrap_or(false);
    // The helper reads stdin through EOF before it contacts the compositor.
    drop(child.stdin.take());
    if !wrote {
        let _ = child.wait();
        return false;
    }
    let mut status = [0u8; 1];
    let ack = child.stdout.as_mut().and_then(|out| out.read_exact(&mut status).ok());
    drop(child.stdout.take());
    if ack.is_some() && status[0] == b'1' {
        if let Ok(mut holds) = CLIPBOARD_HOLDS.lock() {
            holds.push(child);
        }
        true
    } else {
        let _ = child.wait();
        false
    }
}

struct Conn {
    stream: UnixStream,
    buf: Vec<u8>,
    fds: Vec<OwnedFd>,
    next_id: u32,
    registry: u32,
    device: u32,
    source: u32,
    globals: Vec<(u32, String, u32)>,
    offers: HashMap<u32, Vec<String>>,
    /// `None` until the first selection event. Inner `None` is an empty clipboard.
    selection: Option<Option<u32>>,
    /// Current primary-selection offer. Kept until the next primary event.
    primary: Option<u32>,
    text: String,
    failed: bool,
    cancelled: bool,
    sync_id: u32,
    sync_done: bool,
}

impl Conn {
    fn connect() -> Option<Self> {
        let path = wayland_socket_path()?;
        let stream = UnixStream::connect(&path).ok()?;
        let _ = stream.set_read_timeout(Some(Duration::from_millis(800)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(800)));
        Some(Self {
            stream,
            buf: Vec::new(),
            fds: Vec::new(),
            next_id: 2,
            registry: 0,
            device: 0,
            source: 0,
            globals: Vec::new(),
            offers: HashMap::new(),
            selection: None,
            primary: None,
            text: String::new(),
            failed: false,
            cancelled: false,
            sync_id: 0,
            sync_done: false,
        })
    }

    fn alloc(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn connect_and_copy(text: &str) -> Option<Self> {
        let mut conn = Self::connect()?;
        conn.text = text.to_string();
        if conn.bootstrap().is_err() {
            return None;
        }
        let manager = conn.global("ext_data_control_manager_v1")?;
        let seat = conn.global("wl_seat")?;
        let manager_id = conn.bind(manager.0, &manager.1, manager.2.min(1)).ok()?;
        let seat_id = conn.bind(seat.0, &seat.1, seat.2.min(1).max(1)).ok()?;
        conn.device = conn.get_data_device(manager_id, seat_id).ok()?;
        conn.source = conn.create_source(manager_id).ok()?;
        for mime in CLIP_MIMES {
            conn.offer(conn.source, mime).ok()?;
        }
        conn.set_selection(conn.device, conn.source).ok()?;
        conn.roundtrip().ok()?;
        if conn.failed {
            return None;
        }
        Some(conn)
    }

    fn read_clipboard(&mut self) -> io::Result<Option<String>> {
        self.bootstrap()?;
        let manager = self
            .global("ext_data_control_manager_v1")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no data control"))?;
        let seat = self
            .global("wl_seat")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no seat"))?;
        let manager_id = self.bind(manager.0, &manager.1, 1)?;
        let seat_id = self.bind(seat.0, &seat.1, 1)?;
        self.device = self.get_data_device(manager_id, seat_id)?;
        self.roundtrip()?;
        let Some(Some(offer)) = self.selection else {
            return Ok(None);
        };
        let mimes = self.offers.get(&offer).cloned().unwrap_or_default();
        let Some(mime) = pick_mime(&mimes).map(str::to_string) else {
            return Ok(None);
        };
        let (read_fd, write_fd) = make_pipe()?;
        self.send(offer, 0, &string_bytes(&mime), &[write_fd.as_raw_fd()])?;
        drop(write_fd);
        let bytes = read_fd_all(&read_fd, 800)?;
        Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
    }

    fn bootstrap(&mut self) -> io::Result<()> {
        self.registry = self.alloc();
        self.send(1, 1, &self.registry.to_ne_bytes(), &[])?;
        self.roundtrip()?;
        if self.failed {
            return Err(io::Error::new(io::ErrorKind::Other, "wayland error"));
        }
        Ok(())
    }

    fn global(&self, iface: &str) -> Option<(u32, String, u32)> {
        self.globals.iter().find(|g| g.1 == iface).cloned()
    }

    fn bind(&mut self, name: u32, iface: &str, version: u32) -> io::Result<u32> {
        let id = self.alloc();
        let mut payload = Vec::new();
        payload.extend_from_slice(&name.to_ne_bytes());
        push_string(&mut payload, iface);
        payload.extend_from_slice(&version.to_ne_bytes());
        payload.extend_from_slice(&id.to_ne_bytes());
        self.send(self.registry, 0, &payload, &[])?;
        Ok(id)
    }

    fn get_data_device(&mut self, manager: u32, seat: u32) -> io::Result<u32> {
        let id = self.alloc();
        let mut payload = Vec::new();
        payload.extend_from_slice(&id.to_ne_bytes());
        payload.extend_from_slice(&seat.to_ne_bytes());
        self.send(manager, 1, &payload, &[])?;
        Ok(id)
    }

    fn create_source(&mut self, manager: u32) -> io::Result<u32> {
        let id = self.alloc();
        self.send(manager, 0, &id.to_ne_bytes(), &[])?;
        Ok(id)
    }

    fn offer(&mut self, source: u32, mime: &str) -> io::Result<()> {
        self.send(source, 0, &string_bytes(mime), &[])
    }

    fn set_selection(&mut self, device: u32, source: u32) -> io::Result<()> {
        self.send(device, 0, &source.to_ne_bytes(), &[])
    }

    fn roundtrip(&mut self) -> io::Result<()> {
        let id = self.alloc();
        self.sync_id = id;
        self.sync_done = false;
        self.send(1, 0, &id.to_ne_bytes(), &[])?;
        let deadline = Instant::now() + Duration::from_millis(800);
        while !self.sync_done && !self.failed {
            self.dispatch_buffered()?;
            if self.sync_done || self.failed {
                break;
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "wayland sync"));
            }
            match self.read_more() {
                Ok(true) => {}
                Ok(false) => {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "wayland hangup"));
                }
                Err(err) if is_timeout(&err) => {}
                Err(err) => return Err(err),
            }
        }
        if self.failed {
            Err(io::Error::new(io::ErrorKind::Other, "wayland error"))
        } else {
            Ok(())
        }
    }

    fn serve(mut self) {
        let _ = self.stream.set_read_timeout(Some(Duration::from_secs(30)));
        while !self.cancelled && !self.failed {
            match self.read_more() {
                Ok(true) => {
                    if self.dispatch_buffered().is_err() {
                        break;
                    }
                }
                Ok(false) => break,
                Err(err) if is_timeout(&err) => {}
                Err(_) => break,
            }
        }
    }

    fn read_more(&mut self) -> io::Result<bool> {
        let mut tmp = [0u8; 4096];
        let (n, fds) = recv_with_fds(self.stream.as_raw_fd(), &mut tmp)?;
        if n == 0 && fds.is_empty() {
            return Ok(false);
        }
        self.buf.extend_from_slice(&tmp[..n]);
        self.fds.extend(fds);
        Ok(true)
    }

    fn dispatch_buffered(&mut self) -> io::Result<()> {
        while self.buf.len() >= 8 {
            let header = u32::from_ne_bytes(self.buf[4..8].try_into().unwrap());
            let size = (header >> 16) as usize;
            if size < 8 || self.buf.len() < size {
                return Ok(());
            }
            let object = u32::from_ne_bytes(self.buf[0..4].try_into().unwrap());
            let opcode = (header & 0xffff) as u16;
            let payload = self.buf[8..size].to_vec();
            self.buf.drain(..size);
            let fd = if object == self.source && self.source != 0 && opcode == 0 {
                if self.fds.is_empty() {
                    None
                } else {
                    Some(self.fds.remove(0))
                }
            } else {
                None
            };
            self.handle_event(object, opcode, &payload, fd);
        }
        Ok(())
    }

    fn handle_event(&mut self, object: u32, opcode: u16, payload: &[u8], fd: Option<OwnedFd>) {
        if object == 1 && opcode == 0 {
            self.failed = true;
            return;
        }
        if object == 1 && opcode == 1 {
            return;
        }
        if object == self.sync_id && self.sync_id != 0 && opcode == 0 {
            self.sync_done = true;
            return;
        }
        if object == self.registry && self.registry != 0 && opcode == 0 {
            if let Some(global) = parse_global(payload) {
                self.globals.push(global);
            }
            return;
        }
        if object == self.device && self.device != 0 && opcode == 0 && payload.len() >= 4 {
            let id = u32::from_ne_bytes(payload[0..4].try_into().unwrap());
            self.offers.entry(id).or_default();
            return;
        }
        if object == self.device && self.device != 0 && opcode == 1 && payload.len() >= 4 {
            let id = u32::from_ne_bytes(payload[0..4].try_into().unwrap());
            // The previous offer is dead once a new selection arrives.
            if let Some(Some(old)) = self.selection {
                if old != id {
                    let _ = self.send(old, 1, &[], &[]);
                    self.offers.remove(&old);
                }
            }
            self.selection = Some(if id == 0 { None } else { Some(id) });
            return;
        }
        if object == self.device && self.device != 0 && opcode == 2 {
            self.cancelled = true;
            return;
        }
        // Keep the current primary offer and destroy only the previous one.
        // Destroying the offer we were just given is a protocol error.
        if object == self.device && self.device != 0 && opcode == 3 && payload.len() >= 4 {
            let id = u32::from_ne_bytes(payload[0..4].try_into().unwrap());
            if let Some(old) = self.primary {
                if old != id {
                    let _ = self.send(old, 1, &[], &[]);
                    self.offers.remove(&old);
                }
            }
            self.primary = if id == 0 { None } else { Some(id) };
            return;
        }
        if self.offers.contains_key(&object) && opcode == 0 {
            if let Some((mime, _)) = parse_string(payload, 0) {
                self.offers.entry(object).or_default().push(mime);
            }
            return;
        }
        if object == self.source && self.source != 0 && opcode == 0 {
            if let Some(fd) = fd {
                let _ = write_all_fd(fd, self.text.as_bytes());
            }
            return;
        }
        if object == self.source && self.source != 0 && opcode == 1 {
            self.cancelled = true;
        }
    }

    fn send(&mut self, object: u32, opcode: u16, payload: &[u8], fds: &[RawFd]) -> io::Result<()> {
        let size = (8 + payload.len()) as u32;
        let mut msg = Vec::with_capacity(size as usize);
        msg.extend_from_slice(&object.to_ne_bytes());
        msg.extend_from_slice(&(size << 16 | u32::from(opcode)).to_ne_bytes());
        msg.extend_from_slice(payload);
        send_with_fds(self.stream.as_raw_fd(), &msg, fds)
    }
}

const CLIP_MIMES: &[&str] = &[
    "text/plain;charset=utf-8",
    "text/plain",
    "UTF8_STRING",
    "TEXT",
    "STRING",
];

fn pick_mime(mimes: &[String]) -> Option<&str> {
    for pref in CLIP_MIMES {
        if mimes.iter().any(|mime| mime == pref) {
            return Some(*pref);
        }
    }
    mimes.iter().find(|mime| mime.starts_with("text/")).map(String::as_str)
}

fn wayland_socket_path() -> Option<PathBuf> {
    let display = PathBuf::from(env::var_os("WAYLAND_DISPLAY")?);
    if display.is_absolute() {
        return Some(display);
    }
    Some(PathBuf::from(env::var_os("XDG_RUNTIME_DIR")?).join(display))
}

fn push_string(buf: &mut Vec<u8>, text: &str) {
    buf.extend_from_slice(&string_bytes(text));
}

fn string_bytes(text: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    let len = text.len() + 1;
    buf.extend_from_slice(&(len as u32).to_ne_bytes());
    buf.extend_from_slice(text.as_bytes());
    buf.push(0);
    while buf.len() % 4 != 0 {
        buf.push(0);
    }
    buf
}

fn parse_string(buf: &[u8], at: usize) -> Option<(String, usize)> {
    if at + 4 > buf.len() {
        return None;
    }
    let len = u32::from_ne_bytes(buf[at..at + 4].try_into().ok()?) as usize;
    let start = at + 4;
    if start + len > buf.len() {
        return None;
    }
    let bytes = if len == 0 { &[][..] } else { &buf[start..start + len - 1] };
    let text = String::from_utf8_lossy(bytes).into_owned();
    let padded = (4 + len + 3) & !3;
    Some((text, at + padded))
}

fn parse_global(payload: &[u8]) -> Option<(u32, String, u32)> {
    if payload.len() < 4 {
        return None;
    }
    let name = u32::from_ne_bytes(payload[0..4].try_into().ok()?);
    let (iface, next) = parse_string(payload, 4)?;
    if next + 4 > payload.len() {
        return None;
    }
    let version = u32::from_ne_bytes(payload[next..next + 4].try_into().ok()?);
    Some((name, iface, version))
}

fn is_timeout(err: &io::Error) -> bool {
    matches!(err.kind(), io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock)
}

fn make_pipe() -> io::Result<(OwnedFd, OwnedFd)> {
    let mut fds = [0i32; 2];
    if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((
        unsafe { OwnedFd::from_raw_fd(fds[0]) },
        unsafe { OwnedFd::from_raw_fd(fds[1]) },
    ))
}

fn write_all_fd(fd: OwnedFd, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::from(fd);
    file.write_all(bytes)
}

fn read_fd_all(fd: &OwnedFd, timeout_ms: i32) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    let mut wait = timeout_ms;
    loop {
        if !poll_in(fd.as_raw_fd(), wait)? {
            break;
        }
        let n = unsafe { libc_read(fd.as_raw_fd(), buf.as_mut_ptr(), buf.len()) };
        if n < 0 {
            return Err(io::Error::last_os_error());
        }
        if n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n as usize]);
        wait = 200;
    }
    Ok(out)
}

fn poll_in(fd: RawFd, timeout_ms: i32) -> io::Result<bool> {
    let mut pfd = PollFd { fd, events: 1, revents: 0 };
    let rc = unsafe { poll(&mut pfd, 1, timeout_ms) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(rc > 0)
}

#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}

#[repr(C)]
struct IoVec {
    base: *mut u8,
    len: usize,
}

/// Linux x86_64 `struct msghdr` (56 bytes). `socklen_t` is 4 bytes, padded.
#[repr(C)]
struct MsgHdr {
    name: *mut u8,
    namelen: u32,
    pad0: u32,
    iov: *mut IoVec,
    iovlen: usize,
    control: *mut u8,
    controllen: usize,
    flags: i32,
    pad1: u32,
}

fn send_with_fds(socket: RawFd, bytes: &[u8], fds: &[RawFd]) -> io::Result<()> {
    let mut storage = bytes.to_vec();
    let mut iov = IoVec { base: storage.as_mut_ptr(), len: storage.len() };
    let mut control = [0u8; 24];
    let mut hdr = MsgHdr {
        name: std::ptr::null_mut(),
        namelen: 0,
        pad0: 0,
        iov: &mut iov,
        iovlen: 1,
        control: std::ptr::null_mut(),
        controllen: 0,
        flags: 0,
        pad1: 0,
    };
    if let Some(fd) = fds.first() {
        let len: usize = 20; // CMSG_LEN(sizeof(int))
        control[..8].copy_from_slice(&len.to_ne_bytes());
        control[8..12].copy_from_slice(&1i32.to_ne_bytes()); // SOL_SOCKET
        control[12..16].copy_from_slice(&1i32.to_ne_bytes()); // SCM_RIGHTS
        control[16..20].copy_from_slice(&fd.to_ne_bytes());
        hdr.control = control.as_mut_ptr();
        hdr.controllen = 24; // CMSG_SPACE(sizeof(int))
    }
    let rc = unsafe { sendmsg(socket, &hdr, 0x4000) }; // MSG_NOSIGNAL
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    if rc as usize != bytes.len() {
        return Err(io::Error::new(io::ErrorKind::WriteZero, "short wayland write"));
    }
    Ok(())
}

fn recv_with_fds(socket: RawFd, buf: &mut [u8]) -> io::Result<(usize, Vec<OwnedFd>)> {
    let mut iov = IoVec { base: buf.as_mut_ptr(), len: buf.len() };
    let mut control = [0u8; 256];
    let mut hdr = MsgHdr {
        name: std::ptr::null_mut(),
        namelen: 0,
        pad0: 0,
        iov: &mut iov,
        iovlen: 1,
        control: control.as_mut_ptr(),
        controllen: control.len(),
        flags: 0,
        pad1: 0,
    };
    let rc = unsafe { recvmsg(socket, &mut hdr, 0) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut owned = Vec::new();
    let mut off = 0usize;
    let controllen = hdr.controllen.min(control.len());
    while off + 16 <= controllen {
        let clen = usize::from_ne_bytes(control[off..off + 8].try_into().unwrap());
        if clen < 16 {
            break;
        }
        let level = i32::from_ne_bytes(control[off + 8..off + 12].try_into().unwrap());
        let kind = i32::from_ne_bytes(control[off + 12..off + 16].try_into().unwrap());
        if level == 1 && kind == 1 && off + 20 <= control.len() {
            let fd = i32::from_ne_bytes(control[off + 16..off + 20].try_into().unwrap());
            owned.push(unsafe { OwnedFd::from_raw_fd(fd) });
        }
        let step = (clen + 7) & !7;
        if step == 0 {
            break;
        }
        off += step;
    }
    Ok((rc as usize, owned))
}

unsafe extern "C" {
    fn pipe(fds: *mut i32) -> i32;
    fn sendmsg(fd: i32, msg: *const MsgHdr, flags: i32) -> isize;
    fn recvmsg(fd: i32, msg: *mut MsgHdr, flags: i32) -> isize;
    fn poll(fds: *mut PollFd, nfds: u64, timeout: i32) -> i32;
    #[link_name = "read"]
    fn libc_read(fd: i32, buf: *mut u8, count: usize) -> isize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_padding_includes_nul() {
        let bytes = string_bytes("hi");
        assert_eq!(bytes.len() % 4, 0);
        assert_eq!(&bytes[..4], &3u32.to_ne_bytes());
        assert_eq!(&bytes[4..7], b"hi\0");
    }

    #[test]
    fn scm_rights_roundtrip() {
        let (a, b) = UnixStream::pair().unwrap();
        let (read_fd, write_fd) = make_pipe().unwrap();
        send_with_fds(a.as_raw_fd(), b"ping", &[write_fd.as_raw_fd()]).unwrap();
        drop(write_fd);
        let mut buf = [0u8; 8];
        let (n, fds) = recv_with_fds(b.as_raw_fd(), &mut buf).unwrap();
        assert_eq!(&buf[..n], b"ping");
        assert_eq!(fds.len(), 1);
        write_all_fd(fds.into_iter().next().unwrap(), b"data").unwrap();
        let mut got = String::new();
        File::from(read_fd).read_to_string(&mut got).unwrap();
        assert_eq!(got, "data");
    }

    /// The unit-test binary is `target/debug/deps/az-<hash>`. The real editor,
    /// which understands `--clipboard-hold`, is `target/debug/az`.
    fn az_binary() -> Option<std::path::PathBuf> {
        let mut path = env::current_exe().ok()?;
        path.pop();
        path.pop();
        path.push("az");
        path.is_file().then_some(path)
    }

    fn spawn_hold(text: &str, detach: bool) -> Option<std::process::Child> {
        let bin = az_binary()?;
        let mut cmd = std::process::Command::new(bin);
        cmd.arg("--clipboard-hold")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());
        if detach {
            // A new process group survives the test process so the restored
            // clipboard still has an owner after this test exits.
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd.spawn().ok()?;
        let wrote = child
            .stdin
            .as_mut()
            .map(|stdin| stdin.write_all(text.as_bytes()).is_ok())
            .unwrap_or(false);
        drop(child.stdin.take());
        if !wrote {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        let mut status = [0u8; 1];
        let ack = child.stdout.as_mut().and_then(|out| out.read_exact(&mut status).ok());
        if ack.is_none() || status[0] != b'1' {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        Some(child)
    }

    struct ClipboardGuard {
        previous: Option<String>,
        changed: bool,
        child: Option<std::process::Child>,
    }

    impl Drop for ClipboardGuard {
        fn drop(&mut self) {
            if self.changed {
                if let Some(text) = self.previous.take() {
                    // Ack means the new owner is live. Drop the handle without
                    // killing it so the user's previous clipboard text stays.
                    let _ = spawn_hold(&text, true);
                }
            }
            if let Some(mut child) = self.child.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    #[test]
    #[ignore = "replaces the real Wayland clipboard; run explicitly"]
    fn wayland_clipboard_roundtrip() {
        if env::var_os("WAYLAND_DISPLAY").is_none() {
            return;
        }
        let previous = Conn::connect().and_then(|mut conn| conn.read_clipboard().ok()).flatten();
        let marker = format!("az-clip-{}", std::process::id());
        let mut guard = ClipboardGuard { previous, changed: false, child: None };
        guard.child = spawn_hold(&marker, false);
        guard.changed = guard.child.is_some();
        assert!(guard.changed, "ext-data-control did not accept the selection");
        let got = Conn::connect().and_then(|mut conn| conn.read_clipboard().ok()).flatten();
        assert_eq!(got.as_deref(), Some(marker.as_str()));
    }
}
