use std::cmp::{max, min};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
#[cfg(unix)]
use std::os::unix::io::AsRawFd;
#[cfg(unix)]
use std::os::raw::{c_int, c_ulong, c_ushort};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod plugins;
mod wayland_clip;

const BG: &str = "#1a1b26";
const BG_DARK: &str = "#16161e";
const BG_FLOAT: &str = "#1f2335";
const BG_HIGHLIGHT: &str = "#292e42";
const BG_TAB: &str = "#3a405c";
const FG: &str = "#c0caf5";
const FG_DARK: &str = "#a9b1d6";
const GUTTER: &str = "#3b4261";
const COMMENT: &str = "#565f89";
const BLUE: &str = "#7aa2f7";
const CYAN: &str = "#7dcfff";
const GREEN: &str = "#9ece6a";
const GREEN2: &str = "#73daca";
const MAGENTA: &str = "#bb9af7";
const PURPLE: &str = "#9d7cd8";
const ORANGE: &str = "#ff9e64";
const YELLOW: &str = "#e0af68";
const RED: &str = "#f7768e";
const ACCENT: &str = BLUE;
const HISTORY_LIMIT: usize = 400;
const QUICK_OPEN_LIMIT: usize = 2500;
const PROJECT_SEARCH_LIMIT: usize = 80;
const REPLACE_MATCH_LIMIT: usize = 10_000;
const HUGE_SCAN_LIMIT: usize = 20_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SyntaxMode {
    Php,
    Blade,
    Html,
    Css,
    JavaScript,
    TypeScript,
    Xml,
    Markdown,
    Json,
    Toml,
    Yaml,
    Bash,
    Dotenv,
    Ini,
    Log,
    Rust,
    Nginx,
    Apache,
    Dockerfile,
    Systemd,
    Sql,
    Python,
    Java,
    Csharp,
    Cpp,
    C,
    Go,
    Kotlin,
    Swift,
    Ruby,
    Dart,
    Scala,
    R,
    Lua,
    Perl,
    Haskell,
    Elixir,
    Clojure,
    Zig,
    Julia,
    Objc,
    Plain,
}

impl SyntaxMode {
    fn label(self) -> &'static str { plugins::mode_label(self) }
    fn from_word(word: &str) -> Option<Self> { plugins::from_word(word) }
    fn from_path(path: Option<&Path>) -> Self { plugins::from_path(path) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pos {
    line: usize,
    col: usize,
}

#[derive(Clone)]
enum TextOp {
    Insert { pos: Pos, text: String },
    Delete { pos: Pos, text: String },
}

#[derive(Clone)]
struct HistoryEntry {
    ops: Vec<TextOp>,
    before: Pos,
    after: Pos,
}

struct Tab {
    path: Option<PathBuf>,
    name: String,
    lines: Vec<String>,
    cursor: Pos,
    row_offset: usize,
    col_offset: usize,
    modified: bool,
    revision: u64,
    saved_revision: u64,
    /// Hash of `lines` at the last successful save. `modified` compares this,
    /// so undo back to the saved text clears the dirty flag. `revision` stays
    /// monotonic (decrementing it would break redo).
    saved_hash: u64,
    syntax_mode: Option<SyntaxMode>,
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    large_file: bool,
    /// File on disk used CRLF. The buffer is LF; save writes CRLF back.
    crlf: bool,
}

impl Tab {
    fn empty() -> Self {
        let lines = vec![String::new()];
        let saved_hash = buffer_hash(&lines);
        Self {
            path: None,
            name: "Untitled".to_string(),
            lines,
            cursor: Pos { line: 0, col: 0 },
            row_offset: 0,
            col_offset: 0,
            modified: false,
            revision: 0,
            saved_revision: 0,
            saved_hash,
            syntax_mode: None,
            undo: Vec::new(),
            redo: Vec::new(),
            large_file: false,
            crlf: false,
        }
    }

    fn from_path(path: PathBuf) -> io::Result<Self> {
        let bytes = fs::read(&path)?;
        let large_file = bytes.len() >= 2 * 1024 * 1024;
        let crlf = bytes.windows(2).any(|w| w == b"\r\n");
        let mut text = String::from_utf8_lossy(&bytes).into_owned();
        text = text.replace("\r\n", "\n").replace('\r', "\n");
        let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let name = path.file_name().and_then(OsStr::to_str).unwrap_or("Untitled").to_string();
        let saved_hash = buffer_hash(&lines);
        Ok(Self {
            path: Some(path),
            name,
            lines,
            cursor: Pos { line: 0, col: 0 },
            row_offset: 0,
            col_offset: 0,
            modified: false,
            revision: 0,
            saved_revision: 0,
            saved_hash,
            syntax_mode: None,
            undo: Vec::new(),
            redo: Vec::new(),
            large_file,
            crlf,
        })
    }

    fn syntax(&self) -> SyntaxMode {
        if let Some(mode) = self.syntax_mode {
            return mode;
        }
        let named = SyntaxMode::from_path(self.path.as_deref());
        if named != SyntaxMode::Plain {
            return named;
        }
        self.lines
            .first()
            .and_then(|line| plugins::syntax_from_shebang(line))
            .unwrap_or(SyntaxMode::Plain)
    }

    fn text(&self) -> String {
        let sep = if self.crlf { "\r\n" } else { "\n" };
        self.lines.join(sep)
    }
}

#[derive(Clone)]
struct TreeRow {
    path: PathBuf,
    is_dir: bool,
    depth: usize,
    name: String,
}

#[derive(Clone)]
struct PickerItem {
    label: String,
    detail: String,
    path: Option<PathBuf>,
    line: Option<usize>,
    action: Option<String>,
}

#[derive(Clone)]
pub(crate) struct CompletionItem {
    pub(crate) label: String,
    pub(crate) insert: String,
    pub(crate) detail: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Editor,
    Tree,
}

struct Editor {
    root: PathBuf,
    tabs: Vec<Tab>,
    tab_index: usize,
    running: bool,
    original_stty: String,
    message: String,
    clipboard: String,
    last_find: String,
    focus: Focus,
    expanded: HashSet<PathBuf>,
    tree_rows: Vec<TreeRow>,
    tree_index: usize,
    tree_scroll: usize,
    needs_tree_refresh: bool,
    rows: usize,
    cols: usize,
    tree_width: usize,
    base_tree_width: usize,
    sidebar_hidden: bool,
    content_height: usize,
    status_line: usize,
    selection_anchor: Option<Pos>,
    show_welcome: bool,
    hide_initial_untitled: bool,
    alt_tab_hints_until: Option<Instant>,
    autocomplete_visible: bool,
    autocomplete_items: Vec<CompletionItem>,
    autocomplete_index: usize,
    autocomplete_line: usize,
    autocomplete_start_col: usize,
    autocomplete_prefix: String,
    last_recovery_write: HashMap<String, Instant>,
    cached_clock_minute: u64,
    cached_clock_text: String,
    last_tree_click_time: Option<Instant>,
    last_tree_click_path: Option<PathBuf>,
    mouse_drag_start: Option<(usize, Pos)>,
    last_editor_click: Option<(Instant, Pos, u8)>,
    last_rendered_message: String,
    message_is_error: bool,
    rendered_message_is_error: bool,
    status_flash_until: Option<Instant>,
    pending_input: VecDeque<u8>,
    pending_update: Option<String>,
    /// Last copy was confirmed by a clipboard tool or the compositor.
    /// An OSC52-only copy leaves this false so paste keeps the editor text.
    clipboard_verified: bool,
    /// When false, wheel and scrollbar movement keep the caret where it is
    /// and `ensure_editor_visible` does not pull the viewport back.
    follow_cursor: bool,
    /// Sidebar horizontal offset, in terminal columns.
    tree_h_offset: usize,
    show_hscroll: bool,
    editor_max_visual: usize,
    tree_max_visual: usize,
    line_width_cache: Option<(usize, u64, usize)>,
    prompt_history: Vec<String>,
    /// `Some(true)` drags the editor bar, `Some(false)` the sidebar bar.
    hscroll_drag: Option<bool>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--clipboard-hold") {
        wayland_clip::hold_and_serve();
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("az {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let mut editor = Editor::new(args);
    // Check GitHub for a newer release before taking over the terminal.
    // Slow/offline networks fail silently (short curl timeout); the notice
    // is shown as a modal inside run().
    editor.pending_update = check_for_updates();
    if let Err(err) = editor.run() {
        let _ = editor.cleanup();
        eprintln!("az: {err}");
    }
}

fn print_help() {
    println!("az {} - a fast, small & sane text editor", env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE:");
    println!("  az [OPTIONS] [PATH]");
    println!();
    println!("ARGS:");
    println!("  [PATH]  File, folder, file:line (e.g. main.rs:20), or :line for current file");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help     Show this help");
    println!("  -V, --version  Show version");
    println!();
    println!("KEYS:");
    println!("  Ctrl+S save, Ctrl+O quick open, Ctrl+P commands, Ctrl+F find/replace dialog, Ctrl+L find next,");
    println!("  Ctrl+Shift+O find in files (%Foo = case-sensitive), Ctrl+R replace dialog, Ctrl+G go to line,");
    println!("  Ctrl+Shift+H replace in files, Ctrl+E end of line, Ctrl+Home/End or Alt+Up/Down top/bottom,");
    println!("  Ctrl+T tree focus, Ctrl+H tree hide (tree), Ctrl+D close tab, Ctrl+N new empty tab,");
    println!("  Ctrl+Q quit, Ctrl+K shortcuts, Alt+1-9 visible tabs, Ctrl+Tab cycle tabs");
    println!("MOUSE:");
    println!("  Click sidebar: expand dir / open file, double-click file: rename, wheel: scroll one row;");
    println!("  Click editor: move cursor, drag: select, double-click: word, triple-click: line;");
    println!("  Click tab: switch, + : new tab, middle-click tab: close. Wheel on the tab bar cycles tabs.");
    println!("  Click outside a dialog: close it.");
    println!("  Right-click sidebar/editor/tab: context menu (open, copy path, rename, delete, search).");
}

impl Editor {
    fn new(args: Vec<String>) -> Self {
        let raw_target = args.get(1).cloned().unwrap_or_else(|| ".".to_string());
        // Support `file:line` and `:line` CLI forms. `:line` alone opens CWD.
        let (target_str, cli_line) = parse_cli_path(&raw_target);
        let target = PathBuf::from(&target_str);
        let abs = absolute_path(&target, None);
        let mut tabs = Vec::new();
        let root;
        let mut focus = Focus::Editor;
        let mut show_welcome = false;
        let mut hide_initial_untitled = false;
        let mut message = "Welcome to Az".to_string();
        let pending_go_line: Option<usize> = cli_line;

        if abs.is_file() {
            root = abs.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
            match Tab::from_path(abs.clone()) {
                Ok(mut tab) => {
                    if let Some(n) = pending_go_line {
                        let max_line = tab.lines.len().saturating_sub(1);
                        tab.cursor.line = min(n.saturating_sub(1), max_line);
                        tab.cursor.col = 0;
                        message = format!("Opened {}:{}", tab.name, n);
                    }
                    tabs.push(tab);
                }
                Err(_) => tabs.push(Tab::empty()),
            }
        } else if abs.is_dir() {
            root = abs;
            tabs.push(Tab::empty());
            focus = Focus::Tree;
            show_welcome = true;
            hide_initial_untitled = true;
            message = "Folder opened. Ctrl+O quick open, Ctrl+P commands, Ctrl+T switches tree/editor".to_string();
        } else {
            // Non-existent path: treat as a new file to create (e.g. `az newfile.txt`
            // or `az sub/dir/file.rs:10`). Root is the nearest existing ancestor or CWD.
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let mut ancestor = abs.parent();
            let mut root_candidate: Option<PathBuf> = None;
            while let Some(a) = ancestor {
                if a.is_dir() {
                    root_candidate = Some(a.to_path_buf());
                    break;
                }
                ancestor = a.parent();
            }
            root = root_candidate.unwrap_or(cwd);
            let name = abs.file_name().and_then(OsStr::to_str).unwrap_or("Untitled").to_string();
            let mut tab = Tab::empty();
            tab.path = Some(abs.clone());
            tab.name = name;
            if let Some(n) = pending_go_line {
                // New file only has 1 line; keep cursor at top but remember intent.
                message = format!("New file {} (line {} beyond EOF)", tab.name, n);
            } else {
                message = format!("New file {}", tab.name);
            }
            tabs.push(tab);
        }

        let mut expanded = HashSet::new();
        expanded.insert(root.clone());

        Self {
            root,
            tabs,
            tab_index: 0,
            running: true,
            original_stty: String::new(),
            message,
            clipboard: String::new(),
            last_find: String::new(),
            focus,
            expanded,
            tree_rows: Vec::new(),
            tree_index: 0,
            tree_scroll: 0,
            needs_tree_refresh: true,
            rows: 24,
            cols: 80,
            tree_width: 28,
            base_tree_width: 28,
            sidebar_hidden: false,
            content_height: 20,
            status_line: 23,
            selection_anchor: None,
            show_welcome,
            hide_initial_untitled,
            alt_tab_hints_until: None,
            autocomplete_visible: false,
            autocomplete_items: Vec::new(),
            autocomplete_index: 0,
            autocomplete_line: 0,
            autocomplete_start_col: 0,
            autocomplete_prefix: String::new(),
            last_recovery_write: HashMap::new(),
            cached_clock_minute: 0,
            cached_clock_text: String::new(),
            last_tree_click_time: None,
            last_tree_click_path: None,
            mouse_drag_start: None,
            last_editor_click: None,
            last_rendered_message: String::new(),
            message_is_error: false,
            rendered_message_is_error: false,
            status_flash_until: None,
            pending_input: VecDeque::new(),
            pending_update: None,
            clipboard_verified: false,
            follow_cursor: true,
            tree_h_offset: 0,
            show_hscroll: false,
            editor_max_visual: 0,
            tree_max_visual: 0,
            line_width_cache: None,
            prompt_history: Vec::new(),
            hscroll_drag: None,
        }
    }

    fn run(&mut self) -> io::Result<()> {
        self.try_restore_session();
        self.enable_raw_mode()?;
        print!("\x1b[2J\x1b[H");
        io::stdout().flush()?;
        self.offer_recovery();

        if self.show_welcome {
            self.render()?;
            self.render_welcome_screen()?;
            self.show_welcome = false;
            if let Ok(key) = self.read_key_blocking() {
                self.handle_key(key);
            }
        }

        if self.running {
            if let Some(remote) = self.pending_update.take() {
                self.render()?;
                self.render_update_notice(&remote)?;
                if let Ok(key) = self.read_key_blocking() {
                    self.handle_key(key);
                }
            }
        }

        let mut needs_render = true;
        let mut last_minute = current_minute();
        let mut was_flashing = false;
        while self.running {
            let minute = current_minute();
            let flashing = self.status_flash_until.map(|t| Instant::now() < t).unwrap_or(false);
            let resized = self.terminal_resized();
            // `was_flashing` paints one more frame after the timer ends so the
            // highlight does not stick on the last "on" phase.
            if needs_render || minute != last_minute || flashing || was_flashing || resized {
                self.render()?;
                needs_render = false;
                last_minute = minute;
            }
            was_flashing = flashing;
            if let Some(key) = self.read_key()? {
                self.handle_key(key);
                needs_render = true;
            }
        }
        self.cleanup()
    }

    /// True when the terminal size changed since the last check.
    /// The input loop wakes about every 100ms (`stty time 1`), so this
    /// picks up a resize without a SIGWINCH handler.
    fn terminal_resized(&mut self) -> bool {
        let (rows, cols) = (self.rows, self.cols);
        self.read_terminal_size();
        self.rows != rows || self.cols != cols
    }

    fn enable_raw_mode(&mut self) -> io::Result<()> {
        if let Ok(out) = stty_output(["-g"]) {
            self.original_stty = String::from_utf8_lossy(&out).trim().to_string();
        }
        let _ = stty_status(["-echo", "-icanon", "-isig", "-ixon", "-ixoff", "min", "0", "time", "1"]);
        print!("\x1b[?1049h\x1b[2J\x1b[H\x1b[?25l\x1b[?2004h\x1b[?1000h\x1b[?1002h\x1b[?1006h");
        io::stdout().flush()
    }

    fn cleanup(&mut self) -> io::Result<()> {
        self.save_session();
        print!("\x1b[?1002l\x1b[?1006l\x1b[?1000l\x1b[?2004l\x1b[0m\x1b[?25h\x1b[?1049l\r\n");
        io::stdout().flush()?;
        if !self.original_stty.is_empty() {
            if stty_status([self.original_stty.as_str()]).is_err() {
                let _ = stty_status(["sane"]);
            }
        } else {
            let _ = stty_status(["sane"]);
        }
        Ok(())
    }

    fn read_terminal_size(&mut self) {
        if let Some((r, c)) = terminal_size_from_ioctl() {
            self.apply_terminal_size(r, c);
            return;
        }
        if let Ok(out) = stty_output(["size"]) {
            let s = String::from_utf8_lossy(&out);
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(r), Ok(c)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                    self.apply_terminal_size(r, c);
                    return;
                }
            }
        }
        let rows = env::var("LINES").ok().and_then(|v| v.parse().ok()).unwrap_or(24);
        let cols = env::var("COLUMNS").ok().and_then(|v| v.parse().ok()).unwrap_or(80);
        self.apply_terminal_size(rows, cols);
    }

    fn apply_terminal_size(&mut self, rows: usize, cols: usize) {
        self.rows = max(10, rows);
        self.cols = max(30, cols);
        self.status_line = self.rows;
        self.content_height = self.rows.saturating_sub(5);
    }

    fn tab(&self) -> &Tab {
        &self.tabs[self.tab_index]
    }

    fn tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.tab_index]
    }

    fn read_key_blocking(&mut self) -> io::Result<String> {
        loop {
            if let Some(k) = self.read_key()? {
                return Ok(k);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// Fill `pending_input` from whatever stdin already has (or one VTIME wait).
    fn pull_stdin(&mut self) -> io::Result<bool> {
        let mut buf = [0u8; 8192];
        let n = io::stdin().read(&mut buf)?;
        if n == 0 {
            return Ok(false);
        }
        self.pending_input.extend(&buf[..n]);
        Ok(true)
    }

    fn next_byte(&mut self) -> io::Result<Option<u8>> {
        if self.pending_input.is_empty() && !self.pull_stdin()? {
            return Ok(None);
        }
        Ok(self.pending_input.pop_front())
    }

    fn prepend_pending(&mut self, bytes: &[u8]) {
        for byte in bytes.iter().rev() {
            self.pending_input.push_front(*byte);
        }
    }

    fn read_key(&mut self) -> io::Result<Option<String>> {
        let Some(first) = self.next_byte()? else {
            return Ok(None);
        };
        if first == 0x1b {
            return self.read_escape();
        }
        let mut bytes = vec![first];
        if first >= 0xC0 {
            let want = utf8_sequence_len(first);
            while bytes.len() < want {
                match self.next_byte()? {
                    Some(b) => bytes.push(b),
                    None => break,
                }
            }
        }
        let text = String::from_utf8_lossy(&bytes).into_owned();
        if is_fast_paste_byte(first) {
            self.coalesce_burst(text)
        } else {
            Ok(Some(text))
        }
    }

    /// A long paste that the terminal did not wrap in bracketed-paste markers
    /// arrives as a burst of ordinary bytes. Inserting each one redraws the
    /// screen, which looks like the editor is typing the paste. A typed
    /// character plus Enter is not a paste (that skipped auto-indent). A burst
    /// is a paste once 16 bytes are waiting, or a newline arrives with enough
    /// other text to be a real paste.
    fn coalesce_burst(&mut self, first: String) -> io::Result<Option<String>> {
        if stdin_pending() {
            let _ = self.pull_stdin()?;
        }
        if !pending_is_paste_burst(&self.pending_input) {
            return Ok(Some(first));
        }
        let mut raw = Vec::new();
        loop {
            while self.pending_input.front().copied().is_some_and(is_fast_paste_byte) {
                raw.push(self.pending_input.pop_front().unwrap());
            }
            if self.pending_input.front().is_some() || !stdin_pending() {
                break;
            }
            if !self.pull_stdin()? {
                break;
            }
        }
        if raw.is_empty() {
            return Ok(Some(first));
        }
        let mut text = first;
        text.push_str(&String::from_utf8_lossy(&raw));
        Ok(Some(format!("\0AZPASTE:{text}")))
    }

    fn read_escape(&mut self) -> io::Result<Option<String>> {
        let mut bytes = vec![0x1b];
        let start = Instant::now();
        let mut paste_stalls = 0u8;
        loop {
            if let Some((body, rest)) = take_bracketed_paste(&bytes) {
                self.prepend_pending(&rest);
                let text = String::from_utf8_lossy(&body).into_owned();
                return Ok(Some(format!("\0AZPASTE:{text}")));
            }
            // `\x1b[200~` is a prefix of a bracketed paste even when the body
            // arrived in the same read. Keep going until the end marker;
            // the old 60ms escape timeout used to drop the head of a long
            // paste and then insert the tail one byte at a time.
            let in_paste = bytes.starts_with(PASTE_START) || PASTE_START.starts_with(&bytes);
            if bytes.starts_with(PASTE_START) && bytes.len() > 5 * 1024 * 1024 {
                let text = String::from_utf8_lossy(&bytes[PASTE_START.len()..]).into_owned();
                return Ok(Some(format!("\0AZPASTE:{text}")));
            }
            if !in_paste && escape_sequence_done(&bytes) {
                return Ok(Some(String::from_utf8_lossy(&bytes).into_owned()));
            }
            if !in_paste && bytes.len() > 1 && start.elapsed() > Duration::from_millis(60) {
                return Ok(Some(String::from_utf8_lossy(&bytes).into_owned()));
            }
            if bytes.len() == 1
                && start.elapsed() > Duration::from_millis(25)
                && self.pending_input.is_empty()
                && !stdin_pending()
            {
                return Ok(Some("\x1b".to_string()));
            }
            if self.pending_input.is_empty() {
                if !self.pull_stdin()? {
                    if bytes.starts_with(PASTE_START) {
                        // `stty time 1` already waited ~100ms. A few quiet
                        // reads means the terminal never sent the end marker.
                        paste_stalls += 1;
                        if paste_stalls >= 3 {
                            let text = String::from_utf8_lossy(&bytes[PASTE_START.len()..]).into_owned();
                            return Ok(Some(format!("\0AZPASTE:{text}")));
                        }
                        continue;
                    }
                    if start.elapsed() > Duration::from_millis(25) {
                        return Ok(Some(String::from_utf8_lossy(&bytes).into_owned()));
                    }
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }
            }
            if let Some(byte) = self.pending_input.pop_front() {
                paste_stalls = 0;
                bytes.push(byte);
            }
        }
    }

    fn handle_key(&mut self, key: String) {
        if let Some(pasted) = key.strip_prefix("\0AZPASTE:") {
            self.close_autocomplete();
            if self.focus == Focus::Editor {
                self.insert_text(pasted);
                self.message = "Pasted".to_string();
            }
            return;
        }
        if key.starts_with("\x1b[<") || key.starts_with("\x1b[M") {
            // Fast wheel scrolling (and touchpads) deliver several mouse
            // reports in one stdin read; dispatch each instead of parsing the
            // whole chunk as one event (which fails and drops the burst).
            for ev in parse_mouse_events(&key) {
                self.handle_mouse(ev);
            }
            return;
        }

        if self.focus == Focus::Editor && self.handle_autocomplete_key(&key) {
            return;
        }
        if self.focus != Focus::Editor {
            self.close_autocomplete();
        }
        if self.handle_global_shortcut(&key) {
            return;
        }
        match self.focus {
            Focus::Tree => self.handle_tree_key(&key),
            Focus::Editor => self.handle_editor_key(&key),
        }
    }

    fn handle_global_shortcut(&mut self, key: &str) -> bool {
        if key == "\x1b" {
            self.alt_tab_hints_until = Some(Instant::now() + Duration::from_millis(1200));
            self.message = "Alt+1-9 switches tabs".to_string();
            return true;
        }
        match key {
            "\x11" => { self.confirm_quit(); true }
            "\x13" => { self.save_current_tab(); true }
            "\x1a" => { self.undo(); true }
            "\x19" => { self.redo(); true }
            "\x0f" => { self.quick_open(); true }
            "\x10" => { self.command_palette(); true }
            "\x07" => { self.go_to_line_prompt(); true }
            "\x14" => { self.toggle_tree_focus(); true }
            // NOTE: 0x08 is both Ctrl+H and Backspace-on-some-terminals.
            // In Editor focus, let it fall through to backspace handling so
            // Backspace never toggles the tree. In Tree focus, toggle sidebar.
            "\x08" => {
                if self.focus == Focus::Editor {
                    return false;
                } else {
                    self.toggle_sidebar();
                    return true;
                }
            }
            "\x06" => { self.find_prompt(); true }
            "\x0c" => {
                // Ctrl+L: find next using last pattern (F3-style). Falls back to prompt.
                if self.last_find.is_empty() {
                    self.find_prompt();
                } else {
                    let q = self.last_find.clone();
                    self.find_next(&q);
                }
                true
            }
            "\x12" => { self.replace_prompt(); true }
            "\x0e" => { self.new_tab(true); self.message = "New file. Ctrl+S to choose a filename".to_string(); true }
            "\x04" => { self.close_current_tab(); true }
            "\x03" => { self.copy_selection_or_line(); true }
            "\x18" => { self.cut_selection_or_line(); true }
            "\x16" => { self.paste_clipboard(); true }
            "\x01" => { self.select_all(); true }
            "\x17" => { self.delete_current_line(); true }
            _ => {
                if is_ctrl_k(key) { self.shortcuts_dialog(); return true; }
                if is_ctrl_shift_o(key) { self.project_search_prompt(); return true; }
                if is_ctrl_shift_h(key) { self.replace_in_files_prompt(); return true; }
                if is_ctrl_shift_z(key) { self.redo(); return true; }
                if is_ctrl_backspace(key) { self.delete_current_line(); return true; }
                if let Some(n) = tab_number(key) { self.switch_to_tab_number(n); return true; }
                if is_ctrl_tab(key) { self.cycle_tab(1); return true; }
                if is_ctrl_shift_tab(key) { self.cycle_tab(-1); return true; }
                false
            }
        }
    }

    fn handle_mouse(&mut self, ev: MouseEvent) {
        if ev.is_release {
            self.mouse_drag_start = None;
            self.hscroll_drag = None;
            return;
        }
        if ev.is_scroll() {
            self.handle_mouse_wheel(ev);
            return;
        }
        // Button-drag motion (1002 tracking): extend an editor drag selection
        // or drag a horizontal scrollbar.
        if ev.button & 32 != 0 {
            if ev.button & 3 == 0 {
                if self.hscroll_drag.is_some() {
                    self.handle_hscroll_click(ev.x, ev.y);
                } else {
                    self.handle_mouse_drag(ev.x, ev.y);
                }
            }
            return;
        }
        match ev.button & 3 {
            1 => {
                self.handle_mouse_middle(ev);
                return;
            }
            2 => {
                self.handle_mouse_right(ev);
                return;
            }
            _ => {}
        }
        // Left press (+modifiers) falls through.
        if ev.y == 1 {
            if let Some(action) = self.titlebar_button_action(ev.x) {
                match action {
                    "quick-open" => self.quick_open(),
                    "commands" => self.command_palette(),
                    _ => self.shortcuts_dialog(),
                }
            }
            return;
        }
        if ev.y == 2 {
            return;
        }
        if ev.y == 3 {
            self.handle_mouse_topbar(ev.x);
            return;
        }
        if ev.y < 4 || ev.y >= 4 + self.content_height {
            return;
        }
        if self.show_hscroll && ev.y == 3 + self.content_height {
            self.handle_hscroll_click(ev.x, ev.y);
            return;
        }
        if self.autocomplete_click(ev.x, ev.y) {
            return;
        }
        if !self.sidebar_hidden && ev.x <= self.tree_width.max(1) {
            self.handle_mouse_tree(ev.y);
        } else {
            self.handle_mouse_editor(ev.x, ev.y);
        }
    }

    fn handle_mouse_wheel(&mut self, ev: MouseEvent) {
        let down = !ev.scroll_up();
        if ev.y == 3 {
            self.cycle_tab(if down { 1 } else { -1 });
            return;
        }
        if ev.y < 4 || ev.y >= 4 + self.content_height {
            return;
        }
        if self.show_hscroll && ev.y == 3 + self.content_height {
            self.scroll_hscroll_by(ev.x, if down { 4 } else { -4 });
            return;
        }
        if !self.sidebar_hidden && ev.x <= self.tree_width.max(1) {
            let line = self.tree_index as isize + if down { 1 } else { -1 };
            self.tree_index = line.clamp(0, self.tree_rows.len().saturating_sub(1) as isize) as usize;
            return;
        }
        // Pan the editor. The caret and the selection stay put.
        self.close_autocomplete();
        self.follow_cursor = false;
        let view = self.editor_view_rows().max(1);
        let max_off = self.tab().lines.len().saturating_sub(view);
        let tab = self.tab_mut();
        if down {
            tab.row_offset = min(max_off, tab.row_offset + 1);
        } else {
            tab.row_offset = tab.row_offset.saturating_sub(1);
        }
    }

    fn cycle_tab(&mut self, delta: isize) {
        let all = self.open_tab_indexes();
        if all.is_empty() {
            return;
        }
        let pos = all.iter().position(|i| *i == self.tab_index).unwrap_or(0);
        let n = all.len() as isize;
        let next = (pos as isize + delta).rem_euclid(n) as usize;
        self.tab_index = all[next];
        self.clear_selection();
        self.follow_cursor = true;
        self.close_autocomplete();
        self.message = format!("Tab {}/{}", next + 1, all.len());
    }

    fn titlebar_button_action(&mut self, col: usize) -> Option<&'static str> {
        for (_, action, start, end) in self.titlebar_button_regions() {
            if col >= start && col <= end {
                return Some(action);
            }
        }
        None
    }

    fn handle_mouse_topbar(&mut self, col: usize) {
        let (prefix_w, widths) = self.topbar_tab_widths();
        let visible = self.visible_tab_indexes();
        if let Some(pos) = tab_hit_index(prefix_w, &widths, col) {
            if pos == visible.len() {
                self.new_tab(true);
                self.message = "New file. Ctrl+S to choose a filename".to_string();
                return;
            }
            if let Some(idx) = visible.get(pos) {
                self.tab_index = *idx;
                self.focus = Focus::Editor;
                self.follow_cursor = true;
                self.clear_selection();
                self.close_autocomplete();
                self.message = format!("Tab {}", pos + 1);
            }
        }
    }

    fn topbar_tab_widths(&self) -> (usize, Vec<usize>) {
        // Tabs live above the editor, not the sidebar. The last width is the `+` button.
        let prefix_w = self.editor_start_col().saturating_sub(1);
        let visible = self.visible_tab_indexes();
        let mut widths: Vec<usize> = visible
            .iter()
            .enumerate()
            .map(|(pos, tab_index)| {
                let tab = &self.tabs[*tab_index];
                let modified = if tab.modified { "*" } else { "" };
                visual_width(&format!(" {}:{}{modified} ", pos + 1, escape_control(&tab.name)))
            })
            .collect();
        widths.push(visual_width(" + "));
        (prefix_w, widths)
    }

    fn handle_mouse_tree(&mut self, row: usize) {
        self.refresh_tree();
        let idx = self.tree_scroll + row.saturating_sub(4);
        let Some(entry) = self.tree_rows.get(idx).cloned() else { return; };
        // Double-click (same path <500ms) => rename instead of toggle/open.
        let now = Instant::now();
        let double = self.last_tree_click_path.as_ref() == Some(&entry.path)
            && self
                .last_tree_click_time
                .map(|t| now.duration_since(t) < Duration::from_millis(500))
                .unwrap_or(false);
        self.tree_index = idx;
        self.ensure_tree_visible();
        if double {
            self.last_tree_click_time = None;
            self.last_tree_click_path = None;
            self.focus = Focus::Tree;
            if entry.path == self.root {
                self.message = "Cannot rename project root".to_string();
                return;
            }
            self.rename_tree_path_prompt(false);
            return;
        }
        self.last_tree_click_time = Some(now);
        self.last_tree_click_path = Some(entry.path.clone());
        self.focus = Focus::Tree;
        if entry.is_dir {
            if self.expanded.contains(&entry.path) {
                self.expanded.remove(&entry.path);
            } else {
                self.expanded.insert(entry.path);
            }
            self.needs_tree_refresh = true;
            return;
        }
        self.open_file(entry.path, true);
    }

    /// Map a 1-based content click to a buffer position (None past EOF).
    fn click_to_pos(&self, col: usize, row: usize) -> Option<Pos> {
        let line_no = self.tab().row_offset + row.saturating_sub(4);
        if line_no >= self.tab().lines.len() {
            return None;
        }
        let text_first = self.editor_start_col() + self.line_number_gutter_width();
        // Gutter clicks go to line start.
        let byte = if col < text_first {
            0
        } else {
            let line = &self.tab().lines[line_no];
            let start = byte_at_visual(line, self.tab().col_offset);
            editor_click_col(line, start, col - text_first)
        };
        Some(Pos { line: line_no, col: byte })
    }

    fn handle_mouse_editor(&mut self, col: usize, row: usize) {
        let Some(pos) = self.click_to_pos(col, row) else { return; };
        self.focus = Focus::Editor;
        self.follow_cursor = true;
        self.close_autocomplete();
        // Single click moves; double-click selects the word, triple-click the line.
        let now = Instant::now();
        let line = self.tab().lines[pos.line].clone();
        let (quick, same_line, prev_count, prev_col) = match &self.last_editor_click {
            Some((t, p, c)) => (t.elapsed() < Duration::from_millis(500), p.line == pos.line, *c, p.col),
            None => (false, false, 0, 0),
        };
        let in_same_word = quick
            && same_line
            && word_range_at(&line, pos.col)
                .map(|(s, e)| prev_col >= s && prev_col < e)
                .unwrap_or(false);
        let count = if in_same_word {
            prev_count.saturating_add(1).min(3)
        } else if quick && same_line && prev_count >= 2 {
            3
        } else {
            1
        };
        self.last_editor_click = Some((now, pos, count));
        match (count, word_range_at(&line, pos.col)) {
            (2, Some((s, e))) => {
                self.selection_anchor = Some(Pos { line: pos.line, col: s });
                self.tab_mut().cursor = Pos { line: pos.line, col: e };
                self.mouse_drag_start = Some((self.tab_index, Pos { line: pos.line, col: s }));
            }
            (3, _) => {
                self.selection_anchor = Some(Pos { line: pos.line, col: 0 });
                self.tab_mut().cursor = Pos { line: pos.line, col: line.len() };
                self.mouse_drag_start = Some((self.tab_index, Pos { line: pos.line, col: 0 }));
            }
            _ => {
                self.clear_selection();
                self.tab_mut().cursor = pos;
                self.mouse_drag_start = Some((self.tab_index, pos));
            }
        }
    }

    fn handle_mouse_drag(&mut self, col: usize, row: usize) {
        let Some((tab_index, start)) = self.mouse_drag_start else { return; };
        if tab_index != self.tab_index {
            self.mouse_drag_start = None;
            return;
        }
        let Some(pos) = self.click_to_pos(col, row) else { return; };
        self.focus = Focus::Editor;
        self.selection_anchor = Some(start);
        self.tab_mut().cursor = pos;
    }

    fn handle_mouse_middle(&mut self, ev: MouseEvent) {
        if ev.y != 3 {
            return;
        }
        let (prefix_w, widths) = self.topbar_tab_widths();
        let visible = self.visible_tab_indexes();
        if let Some(pos) = tab_hit_index(prefix_w, &widths, ev.x) {
            if pos == visible.len() {
                return;
            }
            if let Some(&idx) = visible.get(pos) {
                self.close_tab_at(idx);
            }
        }
    }

    fn close_tab_at(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        self.tab_index = idx;
        self.clear_selection();
        self.close_current_tab();
    }

    fn handle_mouse_right(&mut self, ev: MouseEvent) {
        if ev.y == 3 {
            let (prefix_w, widths) = self.topbar_tab_widths();
            let visible = self.visible_tab_indexes();
            let Some(pos) = tab_hit_index(prefix_w, &widths, ev.x) else { return; };
            if pos == visible.len() {
                return;
            }
            let Some(&idx) = visible.get(pos) else { return; };
            self.tab_index = idx;
            self.clear_selection();
            let items = vec!["Close tab".to_string(), "Copy file path".to_string()];
            match self.context_menu(&items, ev.x, 4) {
                Some(0) => self.close_current_tab(),
                Some(1) => self.copy_current_tab_path(),
                _ => {}
            }
            return;
        }
        if ev.y < 4 || ev.y >= 4 + self.content_height {
            return;
        }
        if !self.sidebar_hidden && ev.x <= self.tree_width.max(1) {
            self.handle_tree_right_click(ev.x, ev.y);
        } else {
            self.handle_editor_right_click(ev.x, ev.y);
        }
    }

    fn handle_tree_right_click(&mut self, x: usize, y: usize) {
        self.refresh_tree();
        let idx = self.tree_scroll + y.saturating_sub(4);
        let Some(entry) = self.tree_rows.get(idx).cloned() else { return; };
        self.tree_index = idx;
        self.focus = Focus::Tree;
        let items = vec![
            "Open".to_string(),
            "Copy file path".to_string(),
            "Rename".to_string(),
            "Delete".to_string(),
            "Search here".to_string(),
            "Search & Replace here".to_string(),
        ];
        let Some(choice) = self.context_menu(&items, x, y) else { return; };
        match choice {
            0 => {
                if entry.is_dir {
                    if self.expanded.contains(&entry.path) {
                        self.expanded.remove(&entry.path);
                    } else {
                        self.expanded.insert(entry.path);
                    }
                    self.needs_tree_refresh = true;
                } else {
                    self.open_file(entry.path, true);
                }
            }
            1 => self.copy_file_path_to_clipboard(&entry.path),
            2 => self.rename_tree_path_prompt(false),
            3 => self.delete_tree_path_prompt(false),
            4 => {
                let scope = self.search_scope_for_path(&entry.path);
                self.project_search_prompt_in(scope);
            }
            _ => {
                let scope = self.search_scope_for_path(&entry.path);
                self.replace_in_files_prompt_scoped(scope);
            }
        }
    }

    fn handle_editor_right_click(&mut self, x: usize, y: usize) {
        if self.selection_range().is_none() {
            self.handle_mouse_editor(x, y);
        } else {
            self.focus = Focus::Editor;
        }
        let items = vec![
            "Cut".to_string(),
            "Copy".to_string(),
            "Paste".to_string(),
            "Select All".to_string(),
            "Find in File".to_string(),
            "Replace in File".to_string(),
            "Find in Files".to_string(),
            "Replace in Files".to_string(),
            "Go to Line".to_string(),
        ];
        let Some(choice) = self.context_menu(&items, x, y) else { return; };
        match choice {
            0 => self.cut_selection_or_line(),
            1 => self.copy_selection_or_line(),
            2 => self.paste_clipboard(),
            3 => self.select_all(),
            4 => self.find_prompt(),
            5 => self.replace_prompt(),
            6 => self.project_search_prompt(),
            7 => self.replace_in_files_prompt(),
            _ => self.go_to_line_prompt(),
        }
    }

    /// Scope dir for "Search here": the dir itself, or the parent of a file.
    fn search_scope_for_path(&self, path: &Path) -> PathBuf {
        if path.is_dir() {
            return path.to_path_buf();
        }
        path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| self.root.clone())
    }

    fn copy_file_path_to_clipboard(&mut self, path: &Path) {
        let s = path.to_string_lossy().into_owned();
        self.clipboard = s.clone();
        let rel = relative_path(&self.root, path);
        self.message = if self.copy_to_system_clipboard(&s) {
            format!("Copied {rel}")
        } else {
            format!("Copied {rel} (OSC52)")
        };
    }

    fn copy_current_tab_path(&mut self) {
        let path = self.tab().path.clone();
        match path {
            Some(p) => self.copy_file_path_to_clipboard(&p),
            None => self.message = "No path to copy (unsaved file)".to_string(),
        }
    }

    fn context_menu(&mut self, items: &[String], col: usize, row: usize) -> Option<usize> {
        if items.is_empty() {
            return None;
        }
        self.mouse_drag_start = None;
        let max_w = items.iter().map(|s| visual_width(s)).max().unwrap_or(0) + 4;
        let (sc, sr, width, height) = context_menu_geometry(items.len(), max_w, col, row, self.cols, self.rows);
        let mut selected = 0usize;
        let mut offset = 0usize;
        loop {
            let vis = height.saturating_sub(2).max(1);
            if selected < offset {
                offset = selected;
            }
            if selected >= offset + vis {
                offset = selected + 1 - vis;
            }
            let _ = self.render();
            self.render_context_menu(items, selected, offset, sc, sr, width, height);
            let key = self.read_key_blocking().unwrap_or_default();
            if let Some(m) = parse_sgr_mouse(&key).or_else(|| parse_legacy_mouse(&key)) {
                if m.is_release || m.button & 32 != 0 {
                    continue;
                }
                if m.is_scroll() {
                    if m.scroll_up() {
                        selected = selected.saturating_sub(1);
                    } else {
                        selected = min(items.len().saturating_sub(1), selected + 1);
                    }
                    continue;
                }
                let btn = m.button & 3;
                if btn == 0 && m.button & 32 == 0 {
                    if m.x >= sc && m.x < sc + width && m.y > sr && m.y < sr + height - 1 {
                        let idx = offset + (m.y - sr - 1);
                        if idx < items.len() {
                            return Some(idx);
                        }
                        continue;
                    }
                    return None;
                }
                if btn == 2 {
                    return None;
                }
                continue;
            }
            match key.as_str() {
                "\r" | "\n" => return Some(selected),
                "\x1b" => return None,
                "\x1b[A" | "\x10" => selected = selected.saturating_sub(1),
                "\x1b[B" | "\x0e" => selected = min(items.len() - 1, selected + 1),
                _ => {
                    if key.len() == 1 && key.as_bytes()[0].is_ascii_digit() {
                        let idx = (key.as_bytes()[0] - b'0') as usize;
                        if idx >= 1 && idx <= items.len() {
                            return Some(idx - 1);
                        }
                    }
                }
            }
        }
    }

    fn render_context_menu(&self, items: &[String], selected: usize, offset: usize, start_col: usize, start_row: usize, width: usize, height: usize) {
        let inner = width.saturating_sub(2);
        let border = ansi_style(Some(BLUE), Some(BG_FLOAT), true, false, false);
        let mut out = String::new();
        out.push_str("\x1b[?25l");
        out.push_str(&format!("\x1b[{start_row};{start_col}H{border}╔{}╗\x1b[0m", "═".repeat(inner)));
        let vis = height.saturating_sub(2);
        for i in 0..vis {
            let r = start_row + 1 + i;
            let cell = if let Some(label) = items.get(offset + i) {
                let plain = format!(" {} {}", offset + i + 1, label);
                let style = if offset + i == selected {
                    ansi_style(Some(BG_DARK), Some(ACCENT), true, false, false)
                } else {
                    ansi_style(Some(FG), Some(BG_FLOAT), false, false, false)
                };
                format!("{style}{}\x1b[0m", fit_plain(&plain, inner))
            } else {
                format!("{}\x1b[0m", fit_plain("", inner))
            };
            out.push_str(&format!("\x1b[{r};{start_col}H{border}║\x1b[0m{cell}{border}║\x1b[0m"));
        }
        out.push_str(&format!("\x1b[{};{start_col}H{border}╚{}╝\x1b[0m", start_row + height - 1, "═".repeat(inner)));
        print!("{out}");
        let _ = io::stdout().flush();
    }

    fn handle_tree_key(&mut self, key: &str) {
        self.refresh_tree();
        match key {
            "n" => self.new_tree_file_prompt(),
            "N" => self.new_tree_folder_prompt(),
            "r" | "R" => self.rename_tree_path_prompt(false),
            "+" | "=" => {
                self.base_tree_width = min(44, self.base_tree_width + 2);
                self.message = format!("Tree width {}", self.base_tree_width);
            }
            "-" | "_" => {
                self.base_tree_width = max(18, self.base_tree_width.saturating_sub(2));
                self.message = format!("Tree width {}", self.base_tree_width);
            }
            "\x1b[3~" => self.delete_tree_path_prompt(false),
            "\x1b[A" => self.tree_index = self.tree_index.saturating_sub(1),
            "\x1b[B" => self.tree_index = min(self.tree_rows.len().saturating_sub(1), self.tree_index + 1),
            "\x1b[5~" => self.tree_index = self.tree_index.saturating_sub(max(1, self.content_height)),
            "\x1b[6~" => self.tree_index = min(self.tree_rows.len().saturating_sub(1), self.tree_index + max(1, self.content_height)),
            "\r" | "\n" => {
                if let Some(row) = self.tree_rows.get(self.tree_index).cloned() {
                    if row.is_dir {
                        if self.expanded.contains(&row.path) { self.expanded.remove(&row.path); } else { self.expanded.insert(row.path); }
                        self.needs_tree_refresh = true;
                    } else {
                        self.open_file(row.path, true);
                        self.focus = Focus::Editor;
                    }
                }
            }
            "\x1b[D" => {
                if let Some(row) = self.tree_rows.get(self.tree_index) {
                    if row.is_dir {
                        self.expanded.remove(&row.path);
                        self.needs_tree_refresh = true;
                    }
                }
            }
            "\x1b[C" => {
                if let Some(row) = self.tree_rows.get(self.tree_index) {
                    if row.is_dir {
                        self.expanded.insert(row.path.clone());
                        self.needs_tree_refresh = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: &str) {
        if is_ctrl_left(key) { self.move_word_left(false); return; }
        if is_ctrl_right(key) { self.move_word_right(false); return; }
        if is_ctrl_shift_left(key) { self.move_word_left(true); return; }
        if is_ctrl_shift_right(key) { self.move_word_right(true); return; }
        if is_ctrl_home(key) || is_alt_up(key) { self.go_to_file_top(false); return; }
        if is_ctrl_end(key) || is_alt_down(key) { self.go_to_file_bottom(false); return; }
        if is_ctrl_shift_home(key) || is_alt_shift_up(key) { self.go_to_file_top(true); return; }
        if is_ctrl_shift_end(key) || is_alt_shift_down(key) { self.go_to_file_bottom(true); return; }
        if is_ctrl_shift_e(key) { self.end(true); return; }

        match key {
            "\x05" => self.go_to_line_end(),
            "\x1b[A" => { self.close_autocomplete(); self.move_cursor(-1, 0, false); }
            "\x1b[B" => { self.close_autocomplete(); self.move_cursor(1, 0, false); }
            "\x1b[C" => { self.close_autocomplete(); self.move_right(false); }
            "\x1b[D" => { self.close_autocomplete(); self.move_left(false); }
            "\x1b[1;2A" => self.move_cursor(-1, 0, true),
            "\x1b[1;2B" => self.move_cursor(1, 0, true),
            "\x1b[1;2C" => self.move_right(true),
            "\x1b[1;2D" => self.move_left(true),
            "\x1b[H" | "\x1bOH" | "\x1b[1~" => self.home(false),
            "\x1b[F" | "\x1bOF" | "\x1b[4~" => self.end(false),
            "\x1b[1;2H" => self.home(true),
            "\x1b[1;2F" => self.end(true),
            "\x1b[5~" => self.page(-1, false),
            "\x1b[6~" => self.page(1, false),
            "\x1b[3~" => { self.delete_forward(); self.refresh_autocomplete(false); }
            "\x7f" | "\x08" => { self.backspace(); self.refresh_autocomplete(false); }
            "\r" | "\n" => { self.close_autocomplete(); self.insert_newline(); }
            "\x1b" => { self.close_autocomplete(); self.clear_selection(); self.message = "Selection cleared".to_string(); }
            _ => {
                if is_printable(key) {
                    if self.selection_range().is_some() {
                        let close = match key {
                            "(" => Some(")"),
                            "{" => Some("}"),
                            "[" => Some("]"),
                            "\"" => Some("\""),
                            "'" => Some("'"),
                            _ => None,
                        };
                        if let Some(close) = close {
                            self.wrap_selection(key, close);
                            self.refresh_autocomplete(false);
                            return;
                        }
                    }
                    if (key == "(" || key == "{" || key == "[") && self.insert_auto_closed_pair(key) {
                        self.refresh_autocomplete(false);
                        return;
                    }
                    self.insert_text(key);
                    if key == ">" { self.auto_close_html_tag(); }
                    if key != "\t" { self.refresh_autocomplete(false); }
                }
            }
        }
    }

    fn render(&mut self) -> io::Result<()> {
        self.read_terminal_size();
        self.refresh_tree();
        self.update_tree_width();
        self.refresh_hscroll();
        self.ensure_editor_visible();
        self.ensure_tree_visible();
        // Any new status message flashes light blue (red on errors) to grab attention.
        if self.message != self.last_rendered_message {
            self.last_rendered_message = self.message.clone();
            self.status_flash_until = Some(Instant::now() + Duration::from_millis(750));
            self.rendered_message_is_error = self.message_is_error;
            self.message_is_error = false;
        }

        let mut out = String::new();
        out.push_str("\x1b[?25l\x1b[H");
        out.push_str(&ansi_fg(FG));
        out.push_str(&ansi_bg(BG));
        out.push_str(&self.render_titlebar());
        out.push_str(&self.render_topbar_separator());
        out.push_str(&self.render_tabbar());
        out.push_str(&self.render_content());
        out.push_str(&self.render_status_separator());
        out.push_str(&self.render_status_line());

        if let Some((cursor_row, cursor_col)) = self.cursor_screen_position() {
            out.push_str(&self.render_autocomplete_dropdown(cursor_row, cursor_col));
            out.push_str(&format!("\x1b[{};{}H\x1b[?25h", cursor_row, cursor_col));
        }
        print!("{out}");
        io::stdout().flush()
    }

    fn clock_text(&mut self) -> String {
        let minute = current_minute();
        if minute != self.cached_clock_minute || self.cached_clock_text.is_empty() {
            self.cached_clock_minute = minute;
            self.cached_clock_text = time_date_text();
        }
        self.cached_clock_text.clone()
    }

    /// Titlebar buttons: (label with padding, action, 1-based start/end cols).
    /// Layout: ` az `, then buttons separated by single spaces; rest is filler.
    fn focus_label(&self) -> &'static str {
        if self.focus == Focus::Tree { "tree" } else { "editor" }
    }

    fn titlebar_button_regions(&mut self) -> Vec<(&'static str, &'static str, usize, usize)> {
        let defs = [(" Open ", "quick-open"), (" Commands ", "commands"), (" Shortcuts ", "shortcuts")];
        let right_w = visual_width(&format!(" {} ", self.clock_text()));
        let mut out = Vec::new();
        let mut x = 4 + 1 + self.focus_label().len() + 2 + 1;
        for (label, action) in defs {
            if x + label.len() > self.cols.saturating_sub(right_w) + 1 {
                break;
            }
            out.push((label, action, x + 1, x + label.len()));
            x += 1 + label.len();
        }
        out
    }

    fn render_titlebar(&mut self) -> String {
        let style = ansi_style(Some(ACCENT), Some(BG_FLOAT), true, false, false);
        let chip = ansi_style(Some(BG_DARK), Some(ACCENT), true, false, false);
        let mode_chip = ansi_style(Some(ACCENT), Some(BG_HIGHLIGHT), true, false, false);
        let button = ansi_style(Some(ACCENT), Some(BG_FLOAT), false, false, false);
        let right = format!(" {} ", self.clock_text());
        let right_w = visual_width(&right);
        let mode = self.focus_label();
        let mut out = format!("\x1b[1;1H{chip} az {style} {mode_chip} {mode} ");
        let mut used = 4 + 1 + mode.len() + 2;
        for (label, _, _, _) in self.titlebar_button_regions() {
            out.push_str(&format!("{style} {button}{label}"));
            used += 1 + label.len();
        }
        let mid = " ".repeat(self.cols.saturating_sub(used + right_w));
        out.push_str(&format!("{style}{mid}{right}\x1b[0m"));
        out
    }

    /// Permanent full-width separator between titlebar and tab bar.
    fn render_topbar_separator(&self) -> String {
        let style = ansi_style(Some(GUTTER), Some(BG_FLOAT), false, false, false);
        format!("\x1b[2;1H{style}{}\x1b[0m", "─".repeat(self.cols))
    }

    /// Permanent full-width separator above the status bar.
    fn render_status_separator(&self) -> String {
        let style = ansi_style(Some(GUTTER), Some(BG_FLOAT), false, false, false);
        format!("\x1b[{};1H{style}{}\x1b[0m", self.rows - 1, "─".repeat(self.cols))
    }

    fn render_tabbar(&self) -> String {
        let style = ansi_style(Some(ACCENT), Some(BG_FLOAT), true, false, false);
        let (prefix_w, widths) = self.topbar_tab_widths();
        let tabs = fit_ansi(&self.render_tabs_text(&style), self.cols.saturating_sub(prefix_w));
        let used: usize = prefix_w + widths.iter().sum::<usize>().min(self.cols.saturating_sub(prefix_w));
        let pad = " ".repeat(self.cols.saturating_sub(used));
        format!("\x1b[3;1H{style}{}{tabs}{style}{pad}\x1b[0m", " ".repeat(prefix_w))
    }

    fn render_tabs_text(&self, base_style: &str) -> String {
        let mut out = String::new();
        let show_alt = self.alt_tab_hints_until.map(|t| Instant::now() < t).unwrap_or(false);
        let visible = self.visible_tab_indexes();
        for (visible_index, tab_index) in visible.iter().enumerate() {
            let tab = &self.tabs[*tab_index];
            let mut number = (visible_index + 1).to_string();
            if show_alt { number = format!("\x1b[4m{number}\x1b[24m"); }
            let modified = if tab.modified { "*" } else { "" };
            let name = escape_control(&tab.name);
            if *tab_index == self.tab_index {
                out.push_str(&ansi_style(Some(BG_DARK), Some(ACCENT), true, false, false));
                out.push_str(&format!(" {number}:{name}{modified} \x1b[0m{base_style}"));
            } else {
                out.push_str(&ansi_style(Some(FG), Some(BG_TAB), false, false, false));
                out.push_str(&format!(" {number}:{name}{modified} {base_style}"));
            }
        }
        out.push_str(&ansi_style(Some(BG_DARK), Some(GREEN), true, false, false));
        out.push_str(" + ");
        out.push_str(base_style);
        out
    }

    fn open_tab_indexes(&self) -> Vec<usize> {
        self.tabs.iter().enumerate()
            .filter(|(i, t)| !self.is_hidden_initial_tab(*i, t))
            .map(|(i, _)| i)
            .collect()
    }

    /// At most 9 tabs, windowed around the current one so a later tab stays reachable.
    fn visible_tab_indexes(&self) -> Vec<usize> {
        let all = self.open_tab_indexes();
        if all.is_empty() {
            return all;
        }
        let pos = all.iter().position(|i| *i == self.tab_index).unwrap_or(0);
        let range = window_indexes(all.len(), pos, 9);
        all[range].to_vec()
    }

    fn is_hidden_initial_tab(&self, index: usize, tab: &Tab) -> bool {
        self.hide_initial_untitled && index == 0 && tab.path.is_none() && !tab.modified
    }

    fn render_content(&self) -> String {
        let mut out = String::new();
        let tab = self.tab();
        let gutter = self.line_number_gutter_width();
        let editor_start = self.editor_start_col();
        let text_width = self.editor_text_width();
        let syntax = tab.syntax();

        let visible = self.visible_line_range();
        let text_rows = self.editor_view_rows();
        for screen_line in 0..self.content_height {
            let row_no = screen_line + 4;
            out.push_str(&format!("\x1b[{row_no};1H"));
            if self.show_hscroll && screen_line + 1 == self.content_height {
                if !self.sidebar_hidden {
                    out.push_str(&self.render_hscroll_bar(1, row_no, self.tree_width.max(1), self.tree_width.max(1), self.tree_max_visual, self.tree_h_offset));
                }
                let x = self.editor_start_col();
                let track = self.cols.saturating_sub(x.saturating_sub(1)).max(1);
                out.push_str(&self.render_hscroll_bar(x, row_no, track, self.editor_text_width(), self.editor_max_visual, tab.col_offset));
                continue;
            }
            if !self.sidebar_hidden {
                out.push_str(&self.render_tree_line(screen_line));
            }
            let line_no = visible.0 + screen_line;
            if screen_line < text_rows && line_no < tab.lines.len() {
                let gutter_text = format!("{:>width$} ", line_no + 1, width = gutter.saturating_sub(1));
                out.push_str(&format!("\x1b[{row_no};{editor_start}H{}{}\x1b[0m", ansi_style(Some(GUTTER), Some(BG), false, true, false), fit_plain(&gutter_text, gutter)));
                let line = &tab.lines[line_no];
                let rendered = self.render_editor_line(line, line_no, syntax, text_width);
                out.push_str(&rendered);
            } else {
                out.push_str(&format!("\x1b[{row_no};{editor_start}H{}~", ansi_style(Some(GUTTER), Some(BG), false, true, false)));
                out.push_str(&format!("{}{}{}", ansi_style(Some(FG), Some(BG), false, false, false), fit_plain("", self.cols.saturating_sub(editor_start).saturating_add(1)), reset_fg_bg()));
            }
        }
        out
    }

    fn visible_line_range(&self) -> (usize, usize) {
        let tab = self.tab();
        let start = min(tab.row_offset, tab.lines.len().saturating_sub(1));
        let end = min(tab.lines.len(), start + self.editor_view_rows());
        (start, end)
    }

    fn editor_view_rows(&self) -> usize {
        let h = self.content_height.max(1);
        if self.show_hscroll { h.saturating_sub(1).max(1) } else { h }
    }

    fn line_number_gutter_width(&self) -> usize {
        let digits = self.tab().lines.len().max(1).to_string().len();
        max(4, digits + 2)
    }

    fn editor_start_col(&self) -> usize {
        if self.sidebar_hidden { 1 } else { self.tree_width + 1 }
    }

    fn editor_text_width(&self) -> usize {
        let start = self.editor_start_col();
        self.cols.saturating_sub(start + self.line_number_gutter_width()).saturating_add(1).max(1)
    }

    fn render_tree_line(&self, screen_line: usize) -> String {
        let width = self.tree_width.max(1);
        let row_idx = self.tree_scroll + screen_line;
        let mut text = String::new();
        if let Some(row) = self.tree_rows.get(row_idx) {
            let prefix = if row.is_dir {
                if self.expanded.contains(&row.path) { "▾ " } else { "▸ " }
            } else { "  " };
            let indent = "  ".repeat(row.depth);
            text = format!("{indent}{prefix}{}", row.name);
            text = shift_visual(&text, self.tree_h_offset);
        } else if self.content_height >= 4 && screen_line + 3 >= self.content_height {
            let lines = self.sidebar_shortcut_lines();
            let idx = screen_line + 3 - self.content_height;
            if idx < lines.len() { text = lines[idx].clone(); }
        }
        let selected = row_idx == self.tree_index && self.focus == Focus::Tree;
        let style = if selected {
            ansi_style(Some(BG_DARK), Some(ACCENT), true, false, false)
        } else if row_idx < self.tree_rows.len() {
            let row = &self.tree_rows[row_idx];
            let is_open = !row.is_dir && self.tab().path.as_ref() == Some(&row.path);
            if is_open {
                ansi_style(Some(FG), Some(BG_HIGHLIGHT), true, false, false)
            } else {
                ansi_style(Some(plugins::tree_color(&row.path, row.is_dir)), Some(BG_DARK), row.is_dir, false, false)
            }
        } else {
            ansi_style(Some(FG_DARK), Some(BG_DARK), false, false, false)
        };
        format!("{style}{}\x1b[0m", fit_plain(&text, width))
    }

    fn sidebar_shortcut_lines(&self) -> Vec<String> {
        if self.tree_width < 18 {
            vec!["Enter open".to_string(), "N file  R rename".to_string(), "Del delete".to_string()]
        } else {
            vec![
                "Enter open/fold".to_string(),
                "N file  Shift+N folder".to_string(),
                "R rename  Del delete".to_string(),
                "+/- tree width".to_string(),
            ]
        }
    }

    fn render_editor_line(&self, line: &str, line_no: usize, syntax: SyntaxMode, width: usize) -> String {
        let tab = self.tab();
        let start = byte_at_visual(line, tab.col_offset);
        let mut out = String::new();
        let segs = highlight_segments(line, syntax);
        let sel = self.selection_range();
        let mut byte_i = start;
        let mut used = 0usize;

        while byte_i < line.len() {
            let ch = next_char(line, byte_i);
            let next_i = byte_i + ch.len();
            let rendered = display_cell(ch);
            let cell_w = visual_width(&rendered);
            if used + cell_w > width { break; }

            let selected = sel.map(|(a, b)| range_overlaps_selection(line_no, byte_i, next_i, a, b)).unwrap_or(false);
            let fg = color_at(&segs, byte_i).unwrap_or(FG);
            if selected {
                out.push_str(&ansi_style(Some(BG_DARK), Some(ACCENT), false, false, false));
            } else {
                out.push_str(&ansi_style(Some(fg), Some(BG), false, false, false));
            }
            out.push_str(&rendered);
            used += cell_w;
            byte_i = next_i;
        }

        if used < width {
            out.push_str(&ansi_style(Some(FG), Some(BG), false, false, false));
            out.push_str(&" ".repeat(width - used));
        }
        out.push_str("\x1b[0m");
        out
    }

    fn render_status_line(&self) -> String {
        let tab = self.tab();
        let path = tab.path.as_ref().map(|p| relative_path(&self.root, p)).unwrap_or_else(|| tab.name.clone());
        let syntax_label = if tab.syntax_mode.is_some() { format!("{} manual", tab.syntax().label()) } else { tab.syntax().label().to_string() };
        let tree_label = if self.sidebar_hidden { "tree hidden" } else { "tree shown" };
        let large = if tab.large_file { "  LARGE" } else { "" };
        let line = tab.lines.get(tab.cursor.line).map(String::as_str).unwrap_or("");
        let stats = format!("Ln {}, Col {}  Lines {}  Words {}{}", tab.cursor.line + 1, visual_at_byte(line, tab.cursor.col) + 1, tab.lines.len(), word_count(&tab.lines), large);
        // One chip per item, all from the Tokyo Night palette (dark text on
        // bright chips, light text on dark ones). State color follows content.
        let mut chips: Vec<(String, &str, &str, bool)> = vec![
            (format!(" {path} "), FG, BG_FLOAT, false),
        ];
        if tab.modified {
            chips.push((" modified ".to_string(), BG_DARK, ORANGE, true));
        }
        chips.push((format!(" {syntax_label} "), BG_DARK, PURPLE, true));
        chips.push((format!(" {tree_label} "), BG_DARK, CYAN, false));
        let base = ansi_style(Some(FG), Some(BG_HIGHLIGHT), false, false, false);
        // Keep a slice of the bar for the message. Drop the tree chip, then the
        // syntax chip, when a narrow terminal would otherwise hide "Saved" / "Copied".
        let msg_reserve = if self.message.is_empty() { 0 } else { min(24, self.cols / 3).max(8).min(self.cols) };
        let mut stats_text = format!(" {stats} ");
        let mut left_w = chip_row_width(&chips);
        while !chips.is_empty() && left_w + visual_width(&stats_text) + msg_reserve + 2 > self.cols {
            if chips.len() > 2 {
                chips.pop();
            } else if !stats_text.contains("Ln") || stats_text.matches(' ').count() < 4 {
                break;
            } else {
                stats_text = format!(" Ln {} Col {} ", tab.cursor.line + 1, visual_at_byte(line, tab.cursor.col) + 1);
            }
            left_w = chip_row_width(&chips);
            if chips.len() <= 1 && visual_width(&stats_text) + msg_reserve + 2 <= self.cols {
                break;
            }
            if chips.len() > 1 && left_w + visual_width(&stats_text) + msg_reserve + 2 > self.cols {
                chips.pop();
                left_w = chip_row_width(&chips);
            } else {
                break;
            }
        }
        let mut left = String::new();
        left_w = 0;
        for (i, (text, fg, bg, bold)) in chips.iter().enumerate() {
            if i > 0 {
                left.push_str(&format!("{base} "));
                left_w += 1;
            }
            left.push_str(&format!("{}{text}\x1b[0m", ansi_style(Some(*fg), Some(*bg), *bold, false, false)));
            left_w += visual_width(text);
        }
        let stats_w = visual_width(&stats_text);
        let stats_rendered = format!("{}{stats_text}\x1b[0m", ansi_style(Some(BG_DARK), Some(YELLOW), true, false, false));
        // Fresh messages blink for 750ms (~250ms phases): red on errors, light blue otherwise.
        let blink_on = match self.status_flash_until {
            Some(t) => {
                let remaining = t.saturating_duration_since(Instant::now()).as_millis().min(750);
                remaining > 0 && (750 - remaining) / 250 % 2 == 0
            }
            None => false,
        };
        let alert = if self.rendered_message_is_error { RED } else { CYAN };
        let msg_style = if blink_on {
            ansi_style(Some(BG_DARK), Some(alert), true, false, false)
        } else {
            base.clone()
        };
        // Message takes whatever is left; chips always fit via fit_ansi below.
        let msg_w = self.cols.saturating_sub(left_w + stats_w + 2);
        let mut line = format!("{base}{left}{base} ");
        if msg_w > 0 {
            line.push_str(&format!("{msg_style}{}\x1b[0m", fit_plain(&format!("{} ", self.message), msg_w)));
        }
        line.push_str(&format!("{base} {stats_rendered}"));
        format!("\x1b[{};1H{}\x1b[0m", self.status_line, fit_ansi(&line, self.cols))
    }

    fn cursor_screen_position(&self) -> Option<(usize, usize)> {
        if self.focus == Focus::Tree && !self.sidebar_hidden {
            let visible = self.tree_index.saturating_sub(self.tree_scroll);
            let row_limit = self.editor_view_rows().saturating_sub(1);
            if visible > row_limit {
                return None;
            }
            return Some((4 + visible, 1));
        }
        let tab = self.tab();
        let view = self.editor_view_rows();
        if tab.cursor.line < tab.row_offset || tab.cursor.line >= tab.row_offset + view {
            return None;
        }
        let row = 4 + tab.cursor.line - tab.row_offset;
        let line = tab.lines.get(tab.cursor.line).map(String::as_str).unwrap_or("");
        let visual = visual_at_byte(line, tab.cursor.col).saturating_sub(tab.col_offset);
        let width = self.editor_text_width().max(1);
        if visual >= width {
            return None;
        }
        let col = self.editor_start_col() + self.line_number_gutter_width() + visual;
        Some((max(1, min(self.rows, row)), max(1, min(self.cols, col))))
    }

    fn render_welcome_screen(&mut self) -> io::Result<()> {
        let hint = "  Ctrl+K for all shortcuts (searchable) ".to_string();
        let lines = vec![
            String::new(),
            "  A fast, small & sane text editor.".to_string(),
            String::new(),
            "    Ctrl+O  Quick open file/symbol".to_string(),
            "    Ctrl+P  Command palette".to_string(),
            "    Ctrl+S  Save".to_string(),
            "    Ctrl+Q  Quit".to_string(),
            String::new(),
            hint.clone(),
            String::new(),
            "  Press any key to continue ...".to_string(),
        ];
        let hint_idx = lines.iter().position(|l| *l == hint).unwrap_or(0);
        self.render_popup_box(&lines, &[hint_idx], &ttfx_logo())
    }

    fn render_update_notice(&self, remote: &str) -> io::Result<()> {
        let title = format!("  A new version of az is available: {remote} (you have {}).", env!("CARGO_PKG_VERSION"));
        let lines = vec![
            title.clone(),
            String::new(),
            "  Upgrade with:".to_string(),
            "  curl -fsSL https://raw.githubusercontent.com/arazgholami/az/refs/heads/main/install.sh | sh".to_string(),
            String::new(),
            "  Press any key to continue ...".to_string(),
        ];
        self.render_popup_box(&lines, &[0], &[])
    }

    fn render_popup_box(&self, lines: &[String], highlight_lines: &[usize], logo: &[(String, usize)]) -> io::Result<()> {
        let content_w = lines.iter().map(|l| visual_width(l)).max().unwrap_or(30) + 4;
        let need = max(56, max(content_w, if logo.is_empty() { 0 } else { TTFX_LOGO_WIDTH + 4 }));
        let width = min(self.cols.saturating_sub(4), need);
        let height = min(self.rows.saturating_sub(2), lines.len() + logo.len() + 2);
        let start_col = max(1, (self.cols.saturating_sub(width)) / 2 + 1);
        let start_row = max(1, (self.rows.saturating_sub(height)) / 2 + 1);
        let inner = width.saturating_sub(2);
        let border = ansi_style(Some(BLUE), Some(BG_FLOAT), true, false, false);
        let logo_bg = ansi_style(Some(FG), Some(BG_FLOAT), false, false, false);
        let mut out = String::new();
        out.push_str("\x1b[?25l");
        out.push_str(&format!("\x1b[{start_row};{start_col}H{border}╔{}╗\x1b[0m", "═".repeat(inner)));
        let mut drawn = 0usize;
        for (styled, w) in logo.iter() {
            if drawn >= height.saturating_sub(2) {
                break;
            }
            let row = start_row + 1 + drawn;
            drawn += 1;
            let pad = " ".repeat(inner.saturating_sub(*w));
            out.push_str(&format!("\x1b[{row};{start_col}H{border}║\x1b[0m{logo_bg}{styled}{pad}\x1b[0m{border}║\x1b[0m"));
        }
        for i in 0..height.saturating_sub(2).saturating_sub(drawn) {
            let row = start_row + 1 + drawn + i;
            let raw = lines.get(i).map(String::as_str).unwrap_or("");
            let style = if highlight_lines.contains(&i) {
                ansi_style(Some(BG_DARK), Some(ACCENT), true, false, false)
            } else {
                ansi_style(Some(FG), Some(BG_FLOAT), false, false, false)
            };
            out.push_str(&format!("\x1b[{row};{start_col}H{border}║\x1b[0m{style}{}\x1b[0m{border}║\x1b[0m", fit_plain(raw, inner),));
        }
        out.push_str(&format!("\x1b[{};{start_col}H{border}╚{}╝\x1b[0m", start_row + height - 1, "═".repeat(inner)));
        print!("{out}");
        io::stdout().flush()
    }

    fn refresh_tree(&mut self) {
        if !self.needs_tree_refresh && !self.tree_rows.is_empty() { return; }
        self.tree_rows.clear();
        self.add_tree_rows(self.root.clone(), 0);
        if self.tree_rows.is_empty() {
            let name = self.root.file_name().and_then(OsStr::to_str).unwrap_or("/").to_string();
            self.tree_rows.push(TreeRow { path: self.root.clone(), is_dir: true, depth: 0, name });
        }
        self.tree_index = min(self.tree_index, self.tree_rows.len().saturating_sub(1));
        self.needs_tree_refresh = false;
    }

    fn add_tree_rows(&mut self, dir: PathBuf, depth: usize) {
        let name = if depth == 0 {
            dir.file_name().and_then(OsStr::to_str).unwrap_or_else(|| dir.to_str().unwrap_or("/")).to_string()
        } else {
            dir.file_name().and_then(OsStr::to_str).unwrap_or("?").to_string()
        };
        self.tree_rows.push(TreeRow { path: dir.clone(), is_dir: true, depth, name });
        if !self.expanded.contains(&dir) { return; }
        let Ok(read) = fs::read_dir(&dir) else { return; };
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for entry in read.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == "." || name == ".."
                || matches!(
                    name.as_str(),
                    ".git"
                        | "node_modules"
                        | "vendor"
                        | ".idea"
                        | ".vscode"
                        | "target"
                        | "dist"
                        | "build"
                        | "__pycache__"
                        | ".next"
                        | ".nuxt"
                )
            {
                continue;
            }
            if path.is_dir() { dirs.push(path); } else if path.is_file() { files.push(path); }
        }
        dirs.sort_by_key(|p| p.file_name().map(|s| s.to_string_lossy().to_ascii_lowercase()).unwrap_or_default());
        files.sort_by_key(|p| p.file_name().map(|s| s.to_string_lossy().to_ascii_lowercase()).unwrap_or_default());
        for p in dirs { self.add_tree_rows(p, depth + 1); }
        for p in files {
            let name = p.file_name().and_then(OsStr::to_str).unwrap_or("?").to_string();
            self.tree_rows.push(TreeRow { path: p, is_dir: false, depth: depth + 1, name });
        }
    }

    fn update_tree_width(&mut self) {
        if self.sidebar_hidden {
            self.tree_width = 0;
            return;
        }
        let mut width = self.base_tree_width;
        let visible_end = min(self.tree_rows.len(), self.tree_scroll + self.content_height);
        for row in &self.tree_rows[self.tree_scroll..visible_end] {
            width = max(width, min(44, row.depth * 2 + row.name.len() + 4));
        }
        self.tree_width = min(width, self.cols / 2).max(18);
    }

    fn ensure_tree_visible(&mut self) {
        if self.tree_index < self.tree_scroll {
            self.tree_scroll = self.tree_index;
        } else if self.tree_index >= self.tree_scroll + self.content_height {
            self.tree_scroll = self.tree_index.saturating_sub(self.content_height.saturating_sub(1));
        }
    }

    fn ensure_editor_visible(&mut self) {
        let height = max(1, self.editor_view_rows());
        let width = max(1, self.editor_text_width());
        let follow = self.follow_cursor;
        let max_visual = self.editor_max_visual;
        let tab = self.tab_mut();
        tab.cursor.line = min(tab.cursor.line, tab.lines.len().saturating_sub(1));
        tab.cursor.col = clamp_char_boundary(&tab.lines[tab.cursor.line], min(tab.cursor.col, tab.lines[tab.cursor.line].len()));
        let max_row = tab.lines.len().saturating_sub(height);
        if follow {
            if tab.cursor.line < tab.row_offset {
                tab.row_offset = tab.cursor.line;
            } else if tab.cursor.line >= tab.row_offset + height {
                tab.row_offset = tab.cursor.line.saturating_sub(height - 1);
            }
        } else {
            tab.row_offset = min(tab.row_offset, max_row);
        }
        // `col_offset` is a visual column. One subtraction in bytes used to
        // leave the caret off-screen on tab-heavy lines (1 byte = 4 columns).
        let line = tab.lines[tab.cursor.line].clone();
        if follow {
            tab.col_offset = fit_visual_offset(&line, tab.cursor.col, width);
        }
        let max_off = max_visual.saturating_sub(width);
        tab.col_offset = min(tab.col_offset, max_off);
    }

    fn open_file(&mut self, path: PathBuf, announce: bool) {
        let path = absolute_path(&path, Some(&self.root));
        if let Some(idx) = self.tabs.iter().position(|t| t.path.as_ref() == Some(&path)) {
            self.tab_index = idx;
            self.focus = Focus::Editor;
            self.clear_selection();
            if announce { self.message = format!("Opened {}", relative_path(&self.root, &path)); }
            return;
        }
        match Tab::from_path(path.clone()) {
            Ok(tab) => {
                if self.is_hidden_initial_tab(0, &self.tabs[0]) {
                    self.tabs[0] = tab;
                    self.tab_index = 0;
                    self.hide_initial_untitled = false;
                } else {
                    self.tabs.push(tab);
                    self.tab_index = self.tabs.len() - 1;
                }
                self.focus = Focus::Editor;
                self.clear_selection();
                self.reveal_path_in_tree(&path);
                if announce { self.message = format!("Opened {}", relative_path(&self.root, &path)); }
            }
            Err(_) => self.message = "Could not open file".to_string(),
        }
    }

    fn new_tab(&mut self, visible: bool) {
        self.tabs.push(Tab::empty());
        self.tab_index = self.tabs.len() - 1;
        self.hide_initial_untitled = !visible && self.tabs.len() == 1;
        self.focus = Focus::Editor;
        self.follow_cursor = true;
        self.clear_selection();
    }

    fn close_current_tab(&mut self) {
        if self.tab().modified {
            let ans = self.prompt("Unsaved changes. Close? y/N: ", "");
            if ans.to_ascii_lowercase() != "y" {
                self.message = "Close cancelled".to_string();
                return;
            }
        }
        let tab = self.tabs.remove(self.tab_index);
        self.delete_recovery_for_tab(&tab);
        if self.tabs.is_empty() { self.tabs.push(Tab::empty()); }
        self.tab_index = min(self.tab_index, self.tabs.len().saturating_sub(1));
        self.clear_selection();
        self.message = "Tab closed".to_string();
    }

    fn save_current_tab(&mut self) {
        if self.tab().path.is_none() {
            self.save_current_tab_as();
            return;
        }
        let path = self.tab().path.clone().unwrap();
        let text = self.tab().text();
        self.write_tab_file(&path, &text, false);
    }

    fn save_current_tab_as(&mut self) {
        let untitled = self.tab().path.is_none();
        let default = self.tab().path.as_ref().map(|p| relative_path(&self.root, p)).unwrap_or_default();
        let label = if untitled { "Filename: " } else { "Save as: " };
        let name = self.prompt(label, &default);
        if name.trim().is_empty() {
            self.message = if untitled { "Save cancelled".to_string() } else { "Save as cancelled".to_string() };
            return;
        }
        let path = absolute_path(Path::new(name.trim()), Some(&self.root));
        let text = self.tab().text();
        // Write first; only switch tab path on success so a failed save
        // never orphans the original path.
        self.write_tab_file(&path, &text, true);
    }

    fn write_tab_file(&mut self, path: &Path, text: &str, save_as: bool) {
        match atomic_write_file(path, text.as_bytes()) {
            Ok(()) => {
                self.finish_save(path, save_as);
                self.message = format!("Saved {}", relative_path(&self.root, path));
            }
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied && !current_user_is_root() => {
                self.save_using_root_password(path, text, save_as);
            }
            Err(_) => {
                self.message = if save_as { "Save as failed".to_string() } else { "Save failed".to_string() };
            }
        }
    }

    fn finish_save(&mut self, path: &Path, save_as: bool) {
        let old_path = if save_as { self.tab().path.clone() } else { None };
        let rev = self.tab().revision;
        {
            let tab = self.tab_mut();
            if save_as {
                tab.path = Some(path.to_path_buf());
                tab.name = path.file_name().and_then(OsStr::to_str).unwrap_or("Untitled").to_string();
            }
            tab.saved_revision = rev;
            tab.saved_hash = buffer_hash(&tab.lines);
            tab.modified = false;
        }
        self.delete_recovery_for_tab(self.tab());
        if let Some(old) = old_path {
            if old.as_path() != path {
                self.delete_recovery_file(&old);
            }
        }
        if save_as {
            self.reveal_path_in_tree(path);
        }
        self.needs_tree_refresh = true;
    }

    /// Permission denied: blink the status line red, ask for the root
    /// password, and write the buffer with `sudo` once it is accepted.
    fn save_using_root_password(&mut self, path: &Path, text: &str, save_as: bool) {
        self.message = "Permission denied".to_string();
        self.message_is_error = true;
        let mut password = self.prompt_secret("Permission denied. Root password: ");
        if password.is_empty() {
            self.message = "Save cancelled".to_string();
            self.message_is_error = true;
            return;
        }
        let result = write_file_with_sudo(path, text.as_bytes(), &password);
        password.clear();
        match result {
            Ok(()) => {
                self.finish_save(path, save_as);
                self.message = format!("Saved {} as root", relative_path(&self.root, path));
            }
            Err(err) => {
                self.message = format!("Save failed: {err}");
                self.message_is_error = true;
            }
        }
    }

    fn confirm_quit(&mut self) {
        if self.tabs.iter().any(|t| t.modified) {
            let ans = self.prompt("Unsaved changes. Quit? y/N: ", "");
            if ans.to_ascii_lowercase() != "y" {
                self.message = "Quit cancelled".to_string();
                return;
            }
        }
        self.running = false;
    }

    fn mark_edited(&mut self) {
        let tab = self.tab_mut();
        tab.revision += 1;
        tab.modified = buffer_hash(&tab.lines) != tab.saved_hash;
        self.line_width_cache = None;
        self.write_recovery_for_current_tab();
    }

    fn push_history(&mut self, entry: HistoryEntry) {
        if entry.ops.is_empty() { return; }
        let tab = self.tab_mut();
        tab.undo.push(entry);
        if tab.undo.len() > HISTORY_LIMIT { tab.undo.remove(0); }
        tab.redo.clear();
    }

    fn insert_text(&mut self, text: &str) {
        self.follow_cursor = true;
        let before = self.tab().cursor;
        let mut ops = Vec::new();
        if let Some((a, b)) = self.selection_range() {
            let deleted = self.apply_delete_range(a, b);
            ops.push(TextOp::Delete { pos: a, text: deleted });
        }
        let pos = self.tab().cursor;
        let end = self.apply_insert_at(pos, text);
        ops.push(TextOp::Insert { pos, text: text.to_string() });
        self.tab_mut().cursor = end;
        self.clear_selection();
        let after = self.tab().cursor;
        self.push_history(HistoryEntry { ops, before, after });
        self.mark_edited();
    }

    fn insert_newline(&mut self) {
        let cursor = self.tab().cursor;
        let line = self.tab().lines[cursor.line].clone();
        let before = &line[..cursor.col];
        let after = &line[cursor.col..];
        let indent = indent_for_newline(before, after);
        self.insert_text(&format!("\n{indent}"));
    }

    fn insert_auto_closed_pair(&mut self, open: &str) -> bool {
        let close = match open { "(" => ")", "{" => "}", "[" => "]", _ => return false };
        if self.selection_range().is_some() {
            return self.wrap_selection(open, close);
        }
        let before = self.tab().cursor;
        let pos = self.tab().cursor;
        let pair = format!("{open}{close}");
        let _end = self.apply_insert_at(pos, &pair);
        self.tab_mut().cursor = Pos { line: pos.line, col: pos.col + open.len() };
        let after = self.tab().cursor;
        self.push_history(HistoryEntry { ops: vec![TextOp::Insert { pos, text: pair }], before, after });
        self.mark_edited();
        self.message = format!("Closed {open}{close}");
        true
    }

    /// Put `open`/`close` around the selection instead of deleting it.
    fn wrap_selection(&mut self, open: &str, close: &str) -> bool {
        let Some((a, b)) = self.selection_range() else { return false; };
        self.follow_cursor = true;
        let before = self.tab().cursor;
        let _ = self.apply_insert_at(a, open);
        let close_pos = if a.line == b.line {
            Pos { line: b.line, col: b.col + open.len() }
        } else {
            b
        };
        let after = self.apply_insert_at(close_pos, close);
        self.tab_mut().cursor = after;
        self.push_history(HistoryEntry {
            ops: vec![
                TextOp::Insert { pos: a, text: open.to_string() },
                TextOp::Insert { pos: close_pos, text: close.to_string() },
            ],
            before,
            after,
        });
        self.clear_selection();
        self.mark_edited();
        self.message = format!("Wrapped with {open}{close}");
        true
    }

    fn auto_close_html_tag(&mut self) {
        let syntax = self.tab().syntax();
        if !matches!(syntax, SyntaxMode::Php | SyntaxMode::Blade | SyntaxMode::Html | SyntaxMode::Xml) {
            return;
        }
        let cursor = self.tab().cursor;
        let line = self.tab().lines[cursor.line].clone();
        let before = &line[..cursor.col];
        if before.trim_end().ends_with("/>") { return; }
        let Some(tag) = plugins::html::last_unclosed_tag(before) else { return; };
        if plugins::html::is_void_tag(&tag) { return; }
        let closing = format!("</{tag}>");
        let pos = self.tab().cursor;
        let end = self.apply_insert_at(pos, &closing);
        self.tab_mut().cursor = pos;
        self.push_history(HistoryEntry { ops: vec![TextOp::Insert { pos, text: closing.clone() }], before: pos, after: end });
        self.mark_edited();
        self.message = format!("Closed <{tag}>");
    }

    fn backspace(&mut self) {
        if let Some((a, b)) = self.selection_range() {
            let before = self.tab().cursor;
            let deleted = self.apply_delete_range(a, b);
            self.push_history(HistoryEntry { ops: vec![TextOp::Delete { pos: a, text: deleted }], before, after: a });
            self.clear_selection();
            self.mark_edited();
            return;
        }
        let cursor = self.tab().cursor;
        if cursor.line == 0 && cursor.col == 0 { return; }
        let start = if cursor.col > 0 {
            Pos { line: cursor.line, col: prev_char_boundary(&self.tab().lines[cursor.line], cursor.col) }
        } else {
            let prev_len = self.tab().lines[cursor.line - 1].len();
            Pos { line: cursor.line - 1, col: prev_len }
        };
        let deleted = self.apply_delete_range(start, cursor);
        self.push_history(HistoryEntry { ops: vec![TextOp::Delete { pos: start, text: deleted }], before: cursor, after: start });
        self.clear_selection();
        self.mark_edited();
    }

    fn delete_forward(&mut self) {
        if let Some((a, b)) = self.selection_range() {
            let before = self.tab().cursor;
            let deleted = self.apply_delete_range(a, b);
            self.push_history(HistoryEntry { ops: vec![TextOp::Delete { pos: a, text: deleted }], before, after: a });
            self.clear_selection();
            self.mark_edited();
            return;
        }
        let cursor = self.tab().cursor;
        if cursor.line == self.tab().lines.len() - 1 && cursor.col == self.tab().lines[cursor.line].len() { return; }
        let end = if cursor.col < self.tab().lines[cursor.line].len() {
            Pos { line: cursor.line, col: next_char_boundary(&self.tab().lines[cursor.line], cursor.col) }
        } else {
            Pos { line: cursor.line + 1, col: 0 }
        };
        let deleted = self.apply_delete_range(cursor, end);
        self.push_history(HistoryEntry { ops: vec![TextOp::Delete { pos: cursor, text: deleted }], before: cursor, after: cursor });
        self.clear_selection();
        self.mark_edited();
    }

    fn delete_current_line(&mut self) {
        let before = self.tab().cursor;
        let line = before.line;
        let start = Pos { line, col: 0 };
        let end = if line + 1 < self.tab().lines.len() { Pos { line: line + 1, col: 0 } } else { Pos { line, col: self.tab().lines[line].len() } };
        let deleted = self.apply_delete_range(start, end);
        let after = Pos { line: min(line, self.tab().lines.len().saturating_sub(1)), col: 0 };
        self.tab_mut().cursor = after;
        self.push_history(HistoryEntry { ops: vec![TextOp::Delete { pos: start, text: deleted }], before, after });
        self.clear_selection();
        self.mark_edited();
        self.message = "Line removed".to_string();
    }

    fn apply_insert_at(&mut self, pos: Pos, text: &str) -> Pos {
        let text = text.replace("\r\n", "\n").replace('\r', "\n");
        let parts: Vec<&str> = text.split('\n').collect();
        let tab = self.tab_mut();
        let line = min(pos.line, tab.lines.len().saturating_sub(1));
        let col = clamp_char_boundary(&tab.lines[line], min(pos.col, tab.lines[line].len()));
        if parts.len() == 1 {
            tab.lines[line].insert_str(col, parts[0]);
            return Pos { line, col: col + parts[0].len() };
        }
        // One allocation. Inserting each line with Vec::insert is quadratic
        // and made a multi-thousand-line paste feel stuck.
        let original = tab.lines[line].clone();
        let before = &original[..col];
        let after = &original[col..];
        let mut new_lines = Vec::with_capacity(tab.lines.len() + parts.len());
        new_lines.extend(tab.lines[..line].iter().cloned());
        new_lines.push(format!("{before}{}", parts[0]));
        for mid in &parts[1..parts.len() - 1] {
            new_lines.push((*mid).to_string());
        }
        let last_col = parts.last().unwrap().len();
        new_lines.push(format!("{}{after}", parts.last().unwrap()));
        let end_line = line + parts.len() - 1;
        new_lines.extend(tab.lines[line + 1..].iter().cloned());
        tab.lines = new_lines;
        Pos { line: end_line, col: last_col }
    }

    fn apply_delete_range(&mut self, mut start: Pos, mut end: Pos) -> String {
        if pos_gt(start, end) { std::mem::swap(&mut start, &mut end); }
        let tab = self.tab_mut();
        start.line = min(start.line, tab.lines.len().saturating_sub(1));
        end.line = min(end.line, tab.lines.len().saturating_sub(1));
        start.col = clamp_char_boundary(&tab.lines[start.line], min(start.col, tab.lines[start.line].len()));
        end.col = clamp_char_boundary(&tab.lines[end.line], min(end.col, tab.lines[end.line].len()));
        let deleted = text_between(&tab.lines, start, end);
        if start.line == end.line {
            tab.lines[start.line].replace_range(start.col..end.col, "");
        } else {
            let before = tab.lines[start.line][..start.col].to_string();
            let after = tab.lines[end.line][end.col..].to_string();
            tab.lines[start.line] = before + &after;
            for _ in start.line + 1..=end.line {
                tab.lines.remove(start.line + 1);
            }
        }
        if tab.lines.is_empty() { tab.lines.push(String::new()); }
        tab.cursor = start;
        deleted
    }

    fn undo(&mut self) {
        let Some(entry) = self.tab_mut().undo.pop() else { self.message = "Nothing to undo".to_string(); return; };
        for op in entry.ops.iter().rev() {
            match op {
                TextOp::Insert { pos, text } => {
                    let end = end_pos_for_text(*pos, text);
                    self.apply_delete_range(*pos, end);
                }
                TextOp::Delete { pos, text } => {
                    self.apply_insert_at(*pos, text);
                }
            }
        }
        self.tab_mut().cursor = entry.before;
        self.tab_mut().redo.push(entry);
        self.mark_edited();
        self.message = "Undo".to_string();
    }

    fn redo(&mut self) {
        let Some(entry) = self.tab_mut().redo.pop() else { self.message = "Nothing to redo".to_string(); return; };
        for op in &entry.ops {
            match op {
                TextOp::Insert { pos, text } => { self.apply_insert_at(*pos, text); }
                TextOp::Delete { pos, text } => {
                    let end = end_pos_for_text(*pos, text);
                    self.apply_delete_range(*pos, end);
                }
            }
        }
        self.tab_mut().cursor = entry.after;
        self.tab_mut().undo.push(entry);
        self.mark_edited();
        self.message = "Redo".to_string();
    }

    fn selected_text(&self) -> Option<String> {
        self.selection_range().map(|(a, b)| text_between(&self.tab().lines, a, b))
    }

    fn selection_range(&self) -> Option<(Pos, Pos)> {
        let anchor = self.selection_anchor?;
        let cursor = self.tab().cursor;
        if anchor == cursor { return None; }
        if pos_gt(anchor, cursor) { Some((cursor, anchor)) } else { Some((anchor, cursor)) }
    }

    fn prepare_selection(&mut self, select: bool) {
        if select {
            if self.selection_anchor.is_none() { self.selection_anchor = Some(self.tab().cursor); }
        } else {
            self.selection_anchor = None;
        }
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    fn copy_selection_or_line(&mut self) {
        let text = self.selected_text().unwrap_or_else(|| {
            line_as_clipboard(&self.tab().lines, self.tab().cursor.line)
        });
        self.clipboard = text.clone();
        self.clipboard_verified = self.copy_to_system_clipboard(&text);
        self.message = if self.clipboard_verified {
            "Copied".to_string()
        } else {
            "Copied (OSC52)".to_string()
        };
    }

    fn cut_selection_or_line(&mut self) {
        let before = self.tab().cursor;
        let verified: bool;
        if let Some((a, b)) = self.selection_range() {
            let deleted = self.apply_delete_range(a, b);
            self.clipboard = deleted.clone();
            verified = self.copy_to_system_clipboard(&deleted);
            self.push_history(HistoryEntry { ops: vec![TextOp::Delete { pos: a, text: deleted }], before, after: a });
        } else {
            let line = before.line;
            let start = Pos { line, col: 0 };
            let end = if line + 1 < self.tab().lines.len() { Pos { line: line + 1, col: 0 } } else { Pos { line, col: self.tab().lines[line].len() } };
            let deleted = self.apply_delete_range(start, end);
            self.clipboard = deleted.clone();
            verified = self.copy_to_system_clipboard(&deleted);
            self.push_history(HistoryEntry { ops: vec![TextOp::Delete { pos: start, text: deleted }], before, after: start });
        }
        self.clipboard_verified = verified;
        self.clear_selection();
        self.mark_edited();
        self.message = if verified { "Cut".to_string() } else { "Cut (OSC52)".to_string() };
    }

    fn paste_clipboard(&mut self) {
        // A confirmed OS copy prefers the system clipboard (another app may
        // have replaced it since). An OSC52-only copy did not change the OS
        // clipboard, so paste the editor text instead of stale system text.
        if self.clipboard_verified {
            if let Some(text) = read_system_clipboard() {
                if !text.is_empty() {
                    self.clipboard = text.clone();
                    self.insert_text(&text);
                    self.message = "Pasted".to_string();
                    return;
                }
            }
        }
        if !self.clipboard.is_empty() {
            let text = self.clipboard.clone();
            self.insert_text(&text);
            self.message = "Pasted".to_string();
            return;
        }
        if let Some(text) = read_system_clipboard() {
            if !text.is_empty() {
                self.clipboard = text.clone();
                self.clipboard_verified = true;
                self.insert_text(&text);
                self.message = "Pasted".to_string();
                return;
            }
        }
        self.message = "Clipboard empty".to_string();
    }

    fn select_all(&mut self) {
        self.selection_anchor = Some(Pos { line: 0, col: 0 });
        let last_line = self.tab().lines.len().saturating_sub(1);
        let last_col = self.tab().lines[last_line].len();
        self.tab_mut().cursor = Pos { line: last_line, col: last_col };
        self.message = "Selected all".to_string();
    }

    /// Sends `text` to the system clipboard. Returns true when an external
    /// tool accepted it (verified); otherwise best-effort OSC52 was emitted.
    fn copy_to_system_clipboard(&self, text: &str) -> bool {
        // Over SSH only OSC52 reaches the local machine; try it first there.
        // A true result means a clipboard tool (or the Wayland protocol)
        // accepted the text. OSC52 alone is a best-effort fallback.
        let ssh = env::var("SSH_CONNECTION").is_ok() || env::var("SSH_TTY").is_ok();
        if ssh {
            print!("{}", osc52_sequence(text));
            let _ = io::stdout().flush();
        }
        if try_clipboard_tool(text) || wayland_clip::copy(text) {
            return true;
        }
        if !ssh {
            print!("{}", osc52_sequence(text));
            let _ = io::stdout().flush();
        }
        false
    }

    fn move_cursor(&mut self, line_delta: isize, col_delta: isize, select: bool) {
        self.follow_cursor = true;
        self.prepare_selection(select);
        let tab = self.tab_mut();
        let line = if line_delta < 0 {
            tab.cursor.line.saturating_sub((-line_delta) as usize)
        } else {
            min(tab.lines.len().saturating_sub(1), tab.cursor.line + line_delta as usize)
        };
        let mut col = if col_delta < 0 {
            tab.cursor.col.saturating_sub((-col_delta) as usize)
        } else {
            tab.cursor.col + col_delta as usize
        };
        col = min(col, tab.lines[line].len());
        col = clamp_char_boundary(&tab.lines[line], col);
        tab.cursor = Pos { line, col };
    }

    fn move_left(&mut self, select: bool) {
        self.follow_cursor = true;
        self.prepare_selection(select);
        let cursor = self.tab().cursor;
        if cursor.col > 0 {
            let col = prev_char_boundary(&self.tab().lines[cursor.line], cursor.col);
            self.tab_mut().cursor.col = col;
        } else if cursor.line > 0 {
            let prev_len = self.tab().lines[cursor.line - 1].len();
            self.tab_mut().cursor = Pos { line: cursor.line - 1, col: prev_len };
        }
    }

    fn move_right(&mut self, select: bool) {
        self.follow_cursor = true;
        self.prepare_selection(select);
        let cursor = self.tab().cursor;
        let len = self.tab().lines[cursor.line].len();
        if cursor.col < len {
            let col = next_char_boundary(&self.tab().lines[cursor.line], cursor.col);
            self.tab_mut().cursor.col = col;
        } else if cursor.line + 1 < self.tab().lines.len() {
            self.tab_mut().cursor = Pos { line: cursor.line + 1, col: 0 };
        }
    }

    fn home(&mut self, select: bool) {
        self.follow_cursor = true;
        self.prepare_selection(select);
        self.tab_mut().cursor.col = 0;
    }

    fn end(&mut self, select: bool) {
        self.follow_cursor = true;
        self.prepare_selection(select);
        let line = self.tab().cursor.line;
        let len = self.tab().lines[line].len();
        self.tab_mut().cursor.col = len;
    }

    fn go_to_line_start(&mut self) {
        self.home(false);
        self.message = "Start of line".to_string();
    }

    fn go_to_line_end(&mut self) {
        self.end(false);
        self.message = "End of line".to_string();
    }

    fn go_to_file_top(&mut self, select: bool) {
        self.prepare_selection(select);
        self.tab_mut().cursor = Pos { line: 0, col: 0 };
        if !select {
            self.message = "Top of file".to_string();
        }
    }

    fn go_to_file_bottom(&mut self, select: bool) {
        self.prepare_selection(select);
        let last = self.tab().lines.len().saturating_sub(1);
        let col = self.tab().lines[last].len();
        self.tab_mut().cursor = Pos { line: last, col };
        if !select {
            self.message = "End of file".to_string();
        }
    }

    fn page(&mut self, direction: isize, select: bool) {
        let delta = direction * self.content_height as isize;
        self.move_cursor(delta, 0, select);
    }

    fn move_word_left(&mut self, select: bool) {
        self.prepare_selection(select);
        let mut pos = self.tab().cursor;
        if pos.line == 0 && pos.col == 0 { return; }
        if pos.col == 0 {
            pos.line -= 1;
            pos.col = self.tab().lines[pos.line].len();
        }
        let line = &self.tab().lines[pos.line];
        let mut col = prev_char_boundary(line, pos.col);
        while col > 0 && is_space_char(prev_char(line, col)) {
            col = prev_char_boundary(line, col);
        }
        while col > 0 && is_word_char(prev_char(line, col)) {
            col = prev_char_boundary(line, col);
        }
        self.tab_mut().cursor = Pos { line: pos.line, col };
    }

    fn move_word_right(&mut self, select: bool) {
        self.prepare_selection(select);
        let mut pos = self.tab().cursor;
        let lines_len = self.tab().lines.len();
        loop {
            let line = &self.tab().lines[pos.line];
            if pos.col >= line.len() {
                if pos.line + 1 >= lines_len { break; }
                pos.line += 1;
                pos.col = 0;
                break;
            }
            while pos.col < line.len() && is_word_char(next_char(line, pos.col)) {
                pos.col = next_char_boundary(line, pos.col);
            }
            while pos.col < line.len() && is_space_char(next_char(line, pos.col)) {
                pos.col = next_char_boundary(line, pos.col);
            }
            break;
        }
        self.tab_mut().cursor = pos;
    }

    fn go_to_line_prompt(&mut self) {
        let value = self.prompt("Go to line: ", "");
        if let Ok(n) = value.trim().parse::<usize>() {
            self.go_to_line(n);
        } else {
            self.message = "Go to line cancelled".to_string();
        }
    }

    fn go_to_line(&mut self, line: usize) {
        self.follow_cursor = true;
        let line = line.saturating_sub(1);
        let max_line = self.tab().lines.len().saturating_sub(1);
        self.tab_mut().cursor.line = min(line, max_line);
        let l = self.tab().cursor.line;
        let c = min(self.tab().cursor.col, self.tab().lines[l].len());
        self.tab_mut().cursor.col = clamp_char_boundary(&self.tab().lines[l], c);
        self.clear_selection();
        self.message = format!("Line {}", self.tab().cursor.line + 1);
    }

    fn parse_find_query(&self, query: &str) -> (String, bool) {
        parse_search_query(query)
    }

    fn search_replace_dialog(&mut self, initial: PathBuf, focus: usize) {
        let mut path = relative_path(&self.root, &initial);
        if path == "." { path.clear(); }
        let mut find = self.last_find.clone();
        let mut replace = String::new();
        let mut field = focus.min(2);
        let mut cursors = [path.chars().count(), find.chars().count(), 0usize];
        let mut job = ReplaceCount::idle();
        loop {
            self.advance_replace_count(&mut job, &path, &find);
            let geom = self.render_search_replace(&path, &find, &replace, field, &cursors, &job);
            let key = match self.read_key() {
                Ok(Some(k)) => k,
                Ok(None) => continue,
                Err(_) => return,
            };
            if key.starts_with("\x1b[<") || key.starts_with("\x1b[M") {
                let mut closed = false;
                let mut do_replace = false;
                for ev in parse_mouse_events(&key) {
                    if ev.is_release || ev.button & 32 != 0 || ev.is_scroll() || ev.button & 3 != 0 {
                        continue;
                    }
                    if !geom.contains(ev.x, ev.y) || geom.hits(ev.x, ev.y, geom.cancel_btn) {
                        closed = true;
                        break;
                    }
                    if geom.hits(ev.x, ev.y, geom.replace_btn) {
                        do_replace = true;
                        break;
                    }
                    if let Some(f) = geom.field_at(ev.y) {
                        field = f;
                    }
                }
                if closed {
                    self.message = "Replace cancelled".to_string();
                    return;
                }
                if do_replace {
                    self.commit_replace(&path, &find, &replace);
                    job.invalidate();
                }
                continue;
            }
            match key.as_str() {
                "\x1b" => {
                    self.message = "Replace cancelled".to_string();
                    return;
                }
                "\t" => field = (field + 1) % 5,
                "\x1b[Z" => field = (field + 4) % 5,
                "\r" | "\n" => {
                    if field == 4 {
                        self.message = "Replace cancelled".to_string();
                        return;
                    }
                    self.commit_replace(&path, &find, &replace);
                    job.invalidate();
                }
                "\x0c" => {
                    self.last_find = find.clone();
                    let q = find.clone();
                    self.find_next(&q);
                }
                _ => {
                    if field <= 2 {
                        let cursor = &mut cursors[field];
                        let edited = match field {
                            0 => apply_line_edit(&mut path, cursor, &key),
                            1 => apply_line_edit(&mut find, cursor, &key),
                            _ => apply_line_edit(&mut replace, cursor, &key),
                        };
                        if edited {
                            if field == 1 {
                                self.last_find = find.clone();
                            }
                            if field != 2 {
                                job.invalidate();
                            }
                        }
                    }
                }
            }
        }
    }

    fn commit_replace(&mut self, path: &str, find: &str, replacement: &str) {
        if parse_search_query(find).0.is_empty() {
            self.message = "Empty search".to_string();
            return;
        }
        self.last_find = find.to_string();
        match self.replace_target(path) {
            ReplaceTarget::Buffer => self.replace_all(find, replacement),
            ReplaceTarget::File(p) | ReplaceTarget::Dir(p) => {
                self.message = self.apply_project_replace(&p, find, replacement);
            }
        }
    }

    fn replace_target(&self, path_text: &str) -> ReplaceTarget {
        let trimmed = path_text.trim();
        if trimmed.is_empty() {
            return ReplaceTarget::Buffer;
        }
        let abs = absolute_path(Path::new(trimmed), Some(&self.root));
        if self.tab().path.as_ref() == Some(&abs) {
            return ReplaceTarget::Buffer;
        }
        if abs.is_dir() {
            ReplaceTarget::Dir(abs)
        } else {
            ReplaceTarget::File(abs)
        }
    }

    fn advance_replace_count(&self, job: &mut ReplaceCount, path: &str, find: &str) {
        let (needle, ignore_case) = parse_search_query(find);
        let key = format!("{path}\0{find}");
        if job.key != key {
            *job = ReplaceCount::idle();
            job.key = key;
            if needle.is_empty() {
                job.done = true;
                return;
            }
            match self.replace_target(path) {
                ReplaceTarget::Buffer => {
                    job.count = count_in_lines(&self.tab().lines, &needle, ignore_case).min(REPLACE_MATCH_LIMIT);
                    job.capped = job.count >= REPLACE_MATCH_LIMIT;
                    job.done = true;
                    return;
                }
                ReplaceTarget::File(p) => job.files.push(p),
                ReplaceTarget::Dir(p) => {
                    job.files = self.collect_files_under(&p, 3000).into_iter().filter_map(|item| item.path).collect();
                }
            }
        }
        if job.done || needle.is_empty() {
            return;
        }
        let end = min(job.files.len(), job.index + 20);
        for file in &job.files[job.index..end] {
            if job.count >= REPLACE_MATCH_LIMIT {
                job.capped = true;
                job.done = true;
                break;
            }
            let Ok(text) = fs::read_to_string(file) else { continue; };
            if text.len() > 5 * 1024 * 1024 { continue; }
            for line in text.replace("\r\n", "\n").replace('\r', "\n").lines() {
                job.count += count_matches_in_line(line, &needle, ignore_case);
                if job.count >= REPLACE_MATCH_LIMIT {
                    job.count = REPLACE_MATCH_LIMIT;
                    job.capped = true;
                    break;
                }
            }
        }
        job.index = end;
        if job.index >= job.files.len() {
            job.done = true;
        }
    }

    fn render_search_replace(&mut self, path: &str, find: &str, replace: &str, field: usize, cursors: &[usize; 3], job: &ReplaceCount) -> ReplaceGeom {
        let _ = self.render();
        let frame = picker_frame(self.cols, self.rows);
        let width = frame.width.max(28).min(self.cols.saturating_sub(2).max(20));
        let height = 8usize;
        let col = max(1, (self.cols.saturating_sub(width)) / 2 + 1);
        let row = max(2, (self.rows.saturating_sub(height)) / 2 + 1);
        let inner = width.saturating_sub(2);
        let border = ansi_style(Some(BLUE), Some(BG_FLOAT), true, false, false);
        let label_style = ansi_style(Some(FG_DARK), Some(BG_FLOAT), false, false, false);
        let active = ansi_style(Some(BG_DARK), Some(ACCENT), true, false, false);
        let idle = ansi_style(Some(FG), Some(BG_HIGHLIGHT), false, false, false);
        let fields = [path, find, replace];
        let names = ["Path", "Find", "Replace"];
        let mut out = String::new();
        out.push_str("\x1b[?25l");
        out.push_str(&format!("\x1b[{row};{col}H{border}╔{}╗\x1b[0m", "═".repeat(inner)));
        out.push_str(&format!(
            "\x1b[{};{col}H{border}║\x1b[0m{}{}\x1b[0m{border}║\x1b[0m",
            row + 1,
            ansi_style(Some(ACCENT), Some(BG_FLOAT), true, false, false),
            fit_plain(" Search & Replace ", inner)
        ));
        for i in 0..3 {
            let y = row + 2 + i;
            let style = if field == i { &active } else { &idle };
            let room = inner.saturating_sub(10);
            let (shown, _) = field_window(fields[i], cursors[i], room);
            let text = format!(" {:<7} {}", names[i], shown);
            out.push_str(&format!("\x1b[{y};{col}H{border}║\x1b[0m{style}{}\x1b[0m{border}║\x1b[0m", fit_plain(&text, inner)));
        }
        let count = if find.is_empty() {
            " 0 matches".to_string()
        } else if !job.done {
            format!(" {} matches, counting…", job.count)
        } else if job.capped {
            format!(" {}+ matches", job.count)
        } else {
            format!(" {} match{}", job.count, if job.count == 1 { "" } else { "es" })
        };
        out.push_str(&format!(
            "\x1b[{};{col}H{border}║\x1b[0m{}{}\x1b[0m{border}║\x1b[0m",
            row + 5,
            label_style,
            fit_plain(&count, inner)
        ));
        let replace_label = " Replace ";
        let cancel_label = " Cancel ";
        let replace_style = if field == 3 { &active } else { &idle };
        let cancel_style = if field == 4 { &active } else { &idle };
        let gap = "  ";
        let buttons_w = visual_width(replace_label) + visual_width(gap) + visual_width(cancel_label);
        let pad = " ".repeat(inner.saturating_sub(buttons_w));
        let button_row = format!("{replace_style}{replace_label}\x1b[0m{label_style}{gap}\x1b[0m{cancel_style}{cancel_label}\x1b[0m{label_style}{pad}\x1b[0m");
        out.push_str(&format!("\x1b[{};{col}H{border}║\x1b[0m{button_row}{border}║\x1b[0m", row + 6));
        out.push_str(&format!("\x1b[{};{col}H{border}╚{}╝\x1b[0m", row + 7, "═".repeat(inner)));
        print!("{out}");
        if field <= 2 {
            let (shown_off, caret) = field_window(fields[field], cursors[field], inner.saturating_sub(10));
            let _ = shown_off;
            let cursor_col = col + 1 + 1 + 7 + 1 + caret;
            print!("\x1b[{};{}H\x1b[?25h", row + 2 + field, min(self.cols, cursor_col.max(1)));
        }
        let _ = io::stdout().flush();
        let replace_x = col + 1;
        let cancel_x = replace_x + replace_label.len() + 2;
        ReplaceGeom {
            col,
            row,
            width,
            height,
            replace_btn: (replace_x, row + 6, replace_label.len()),
            cancel_btn: (cancel_x, row + 6, cancel_label.len()),
            field_rows: [row + 2, row + 3, row + 4],
        }
    }

    fn refresh_hscroll(&mut self) {
        let rev = self.tab().revision;
        let max_v = match self.line_width_cache {
            Some((idx, cached_rev, w)) if idx == self.tab_index && cached_rev == rev => w,
            _ => {
                let w = self.tab().lines.iter().map(|l| visual_width(l)).max().unwrap_or(0);
                self.line_width_cache = Some((self.tab_index, rev, w));
                w
            }
        };
        self.editor_max_visual = max_v;
        let tree_max = if self.sidebar_hidden {
            0
        } else {
            self.tree_rows.iter().map(|row| row.depth * 2 + 2 + visual_width(&row.name)).max().unwrap_or(0)
        };
        self.tree_max_visual = tree_max;
        let editor_need = max_v > self.editor_text_width();
        let tree_need = !self.sidebar_hidden && tree_max > self.tree_width.max(1);
        self.show_hscroll = editor_need || tree_need;
        if !tree_need {
            self.tree_h_offset = 0;
        }
        self.tree_h_offset = min(self.tree_h_offset, tree_max.saturating_sub(self.tree_width.max(1)));
    }

    fn render_hscroll_bar(&self, x: usize, y: usize, track: usize, view: usize, content: usize, offset: usize) -> String {
        let (start, thumb) = scrollbar_thumb(track, view, content, offset);
        let mut cells = String::new();
        for i in 0..track {
            if content > view && i >= start && i < start + thumb {
                cells.push('━');
            } else {
                cells.push('─');
            }
        }
        let style = ansi_style(Some(ACCENT), Some(BG_DARK), false, false, false);
        format!("\x1b[{y};{x}H{style}{cells}\x1b[0m")
    }

    fn handle_hscroll_click(&mut self, x: usize, _y: usize) {
        self.follow_cursor = false;
        let editor = self.sidebar_hidden || x > self.tree_width.max(1);
        self.hscroll_drag = Some(editor);
        if editor {
            let origin = self.editor_start_col();
            let track = self.cols.saturating_sub(origin.saturating_sub(1)).max(1);
            let local = x.saturating_sub(origin);
            let view = self.editor_text_width().max(1);
            self.tab_mut().col_offset = scrollbar_offset(track, view, self.editor_max_visual, local);
        } else {
            let track = self.tree_width.max(1);
            let local = x.saturating_sub(1);
            self.tree_h_offset = scrollbar_offset(track, track, self.tree_max_visual, local);
        }
    }

    fn scroll_hscroll_by(&mut self, x: usize, delta: isize) {
        self.follow_cursor = false;
        let editor = self.sidebar_hidden || x > self.tree_width.max(1);
        if editor {
            let view = self.editor_text_width().max(1);
            let max_off = self.editor_max_visual.saturating_sub(view);
            let tab = self.tab_mut();
            if delta < 0 {
                tab.col_offset = tab.col_offset.saturating_sub((-delta) as usize);
            } else {
                tab.col_offset = min(max_off, tab.col_offset + delta as usize);
            }
        } else if delta < 0 {
            self.tree_h_offset = self.tree_h_offset.saturating_sub((-delta) as usize);
        } else {
            let max_off = self.tree_max_visual.saturating_sub(self.tree_width.max(1));
            self.tree_h_offset = min(max_off, self.tree_h_offset + delta as usize);
        }
    }

    /// `true` when the click landed on the autocomplete list and was consumed.
    fn autocomplete_click(&mut self, x: usize, y: usize) -> bool {
        if !self.autocomplete_visible {
            return false;
        }
        let Some((row, col)) = self.cursor_screen_position() else {
            self.close_autocomplete();
            return false;
        };
        let Some((start_col, start_row, width, count, first)) = self.autocomplete_geometry(row, col) else {
            return false;
        };
        let inside = x >= start_col && x < start_col + width && y >= start_row && y < start_row + count;
        if !inside {
            self.close_autocomplete();
            return false;
        }
        let index = first + (y - start_row);
        if index < self.autocomplete_items.len() {
            self.autocomplete_index = index;
            self.accept_autocomplete();
        }
        true
    }

    fn autocomplete_geometry(&self, cursor_row: usize, cursor_col: usize) -> Option<(usize, usize, usize, usize, usize)> {
        if !self.autocomplete_visible || self.autocomplete_items.is_empty() {
            return None;
        }
        let max_items = min(8, self.autocomplete_items.len());
        let width = min(52, max(28, self.editor_text_width() / 2));
        let mut start_col = min(cursor_col, self.cols.saturating_sub(width).max(1));
        if start_col < self.editor_start_col() {
            start_col = self.editor_start_col();
        }
        let mut start_row = cursor_row + 1;
        if start_row + max_items >= self.status_line {
            start_row = max(2, cursor_row.saturating_sub(max_items));
        }
        let first = self.autocomplete_index.saturating_sub(4);
        let count = self.autocomplete_items.len().saturating_sub(first).min(max_items);
        Some((start_col, start_row, width, count, first))
    }

    fn picker_mouse(&self, key: &str, selected: usize, len: usize) -> PickerMouse {
        if !(key.starts_with("\x1b[<") || key.starts_with("\x1b[M")) {
            return PickerMouse::NotMouse;
        }
        let frame = picker_frame(self.cols, self.rows);
        let mut selected = selected;
        let mut activate = None;
        let mut cancel = false;
        for ev in parse_mouse_events(key) {
            if ev.is_release || ev.button & 32 != 0 {
                continue;
            }
            if ev.is_scroll() {
                if ev.scroll_up() {
                    selected = selected.saturating_sub(1);
                } else {
                    selected = min(len.saturating_sub(1), selected + 1);
                }
                continue;
            }
            if ev.button & 3 != 0 {
                continue;
            }
            let inside = ev.x >= frame.start_col
                && ev.x < frame.start_col + frame.width
                && ev.y >= frame.start_row
                && ev.y < frame.start_row + frame.height;
            if !inside {
                cancel = true;
                break;
            }
            let top = frame.start_row + 4;
            if ev.y >= top && ev.y < top + frame.list_rows {
                let i = ev.y - top;
                if i < len {
                    activate = Some(i);
                }
            }
        }
        if cancel {
            return PickerMouse::Cancel;
        }
        if let Some(i) = activate {
            return PickerMouse::Activate(i);
        }
        PickerMouse::Handled(selected)
    }

    fn find_prompt(&mut self) {
        let path = self.tab().path.clone().unwrap_or_else(|| self.root.clone());
        self.search_replace_dialog(path, 1);
    }

    fn find_next(&mut self, query: &str) -> bool {
        self.follow_cursor = true;
        let (needle, ignore_case) = self.parse_find_query(query);
        if needle.is_empty() { self.message = "Empty search".to_string(); return false; }
        // If selection is exactly the previous match, start from its end so we
        // never re-find the same match.
        let mut start = self.tab().cursor;
        if let Some((a, b)) = self.selection_range() {
            let sel = text_between(&self.tab().lines, a, b);
            let is_match = if ignore_case {
                sel.to_ascii_lowercase() == needle.to_ascii_lowercase()
            } else {
                sel == needle
            };
            if is_match {
                start = b;
            } else {
                // Fresh cursor: nudge past cursor so a match exactly at cursor
                // is still found (use cursor pos, not +1, to handle multi-byte).
                // Keep start as cursor.
            }
        }
        let total = self.tab().lines.len();
        // Pass 0: start.line..end from start.col. Pass 1 (wrap): 0..start.line
        // plus start.line[0..start.col] for true wrap matches.
        for line_no in start.line..total {
            let offset = if line_no == start.line { min(start.col, self.tab().lines[line_no].len()) } else { 0 };
            if let Some(pos) = find_in_line(&self.tab().lines[line_no], &needle, offset, ignore_case) {
                self.tab_mut().cursor = Pos { line: line_no, col: pos };
                self.selection_anchor = Some(Pos { line: line_no, col: pos + needle.len() });
                self.message = format!("Found {}", needle);
                return true;
            }
        }
        for line_no in 0..=start.line.min(total.saturating_sub(1)) {
            let limit = if line_no == start.line { start.col } else { usize::MAX };
            if let Some(pos) = find_in_line(&self.tab().lines[line_no], &needle, 0, ignore_case) {
                if line_no == start.line && pos >= limit {
                    // Match is at/after start; already covered in pass 0.
                    // Look for earlier wrap match only; check if another match
                    // exists before limit.
                    let mut off = 0usize;
                    let mut best: Option<usize> = None;
                    while let Some(p) = find_in_line(&self.tab().lines[line_no], &needle, off, ignore_case) {
                        if p < limit {
                            best = Some(p);
                            off = p + needle.len().max(1);
                            if off >= self.tab().lines[line_no].len() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if let Some(p) = best {
                        self.tab_mut().cursor = Pos { line: line_no, col: p };
                        self.selection_anchor = Some(Pos { line: line_no, col: p + needle.len() });
                        self.message = format!("Found {} (wrapped)", needle);
                        return true;
                    }
                    continue;
                }
                self.tab_mut().cursor = Pos { line: line_no, col: pos };
                self.selection_anchor = Some(Pos { line: line_no, col: pos + needle.len() });
                self.message = format!("Found {} (wrapped)", needle);
                return true;
            }
        }
        self.message = "Not found".to_string();
        false
    }

    fn replace_prompt(&mut self) {
        let path = self.tab().path.clone().unwrap_or_else(|| self.root.clone());
        self.search_replace_dialog(path, 2);
    }

    fn replace_all(&mut self, query: &str, replacement: &str) {
        let (needle, ignore_case) = self.parse_find_query(query);
        if needle.is_empty() { self.message = "Empty search".to_string(); return; }
        let before = self.tab().cursor;
        let mut ops = Vec::new();
        let mut count = 0usize;
        let mut line_no = 0usize;
        while line_no < self.tab().lines.len() {
            let mut offset = 0usize;
            while let Some(pos) = find_in_line(&self.tab().lines[line_no], &needle, offset, ignore_case) {
                let start = Pos { line: line_no, col: pos };
                let end = Pos { line: line_no, col: pos + needle.len() };
                let deleted = self.apply_delete_range(start, end);
                let end_pos = self.apply_insert_at(start, replacement);
                ops.push(TextOp::Delete { pos: start, text: deleted });
                ops.push(TextOp::Insert { pos: start, text: replacement.to_string() });
                count += 1;
                offset = end_pos.col;
                if replacement.is_empty() && offset >= self.tab().lines[line_no].len() { break; }
                if ops.len() > 20_000 { break; }
            }
            if ops.len() > 20_000 { break; }
            line_no += 1;
        }
        if count > 0 {
            let after = self.tab().cursor;
            self.push_history(HistoryEntry { ops, before, after });
            self.mark_edited();
            self.message = format!("Replaced {count}");
        } else {
            self.message = "No matches".to_string();
        }
    }

    fn prompt(&mut self, label: &str, default: &str) -> String {
        self.prompt_line(label, default, false, CYAN)
    }

    fn prompt_secret(&mut self, label: &str) -> String {
        self.prompt_line(label, "", true, RED)
    }

    fn prompt_line(&mut self, label: &str, default: &str, secret: bool, alert_color: &str) -> String {
        let mut value = default.to_string();
        let mut cursor = value.chars().count();
        let mut history_at: Option<usize> = None;
        // Prompts block awaiting input, so the prompt line blinks until answered.
        let normal = ansi_style(Some(FG), Some(BG_HIGHLIGHT), true, false, false);
        let alert = ansi_style(Some(BG_DARK), Some(alert_color), true, false, false);
        let mut phase = false;
        let mut last_toggle = Instant::now();
        let mut drawn_phase = true;
        let mut redraw = true;
        loop {
            let now = Instant::now();
            if now.duration_since(last_toggle) > Duration::from_millis(350) {
                phase = !phase;
                last_toggle = now;
            }
            if redraw || phase != drawn_phase {
                redraw = false;
                drawn_phase = phase;
                let _ = self.render();
                let style = if phase { &alert } else { &normal };
                let shown = if secret { "*".repeat(value.chars().count()) } else { value.clone() };
                let prefix: String = shown.chars().take(cursor.min(shown.chars().count())).collect();
                let text = format!(" az> {label}{shown}");
                print!("\x1b[{};1H{}{}\x1b[0m", self.status_line, style, fit_plain(&text, self.cols));
                let cursor_col = min(self.cols, 6 + label.len() + visual_width(&prefix)).max(1);
                print!("\x1b[{};{}H\x1b[?25h", self.status_line, cursor_col);
                let _ = io::stdout().flush();
            }
            let key = match self.read_key() {
                Ok(Some(k)) => k,
                _ => continue,
            };
            match key.as_str() {
                "\r" | "\n" => {
                    if !secret {
                        let trimmed = value.trim();
                        if !trimmed.is_empty() && self.prompt_history.last().map(String::as_str) != Some(trimmed) {
                            self.prompt_history.push(trimmed.to_string());
                            if self.prompt_history.len() > 50 {
                                self.prompt_history.remove(0);
                            }
                        }
                    }
                    return value;
                }
                "\x1b" => return String::new(),
                "\x1b[A" if !secret && !self.prompt_history.is_empty() => {
                    let i = history_at.unwrap_or(self.prompt_history.len()).saturating_sub(1);
                    history_at = Some(i);
                    value = self.prompt_history[i].clone();
                    cursor = value.chars().count();
                }
                "\x1b[B" if !secret => {
                    if let Some(i) = history_at {
                        if i + 1 >= self.prompt_history.len() {
                            history_at = None;
                            value.clear();
                            cursor = 0;
                        } else {
                            history_at = Some(i + 1);
                            value = self.prompt_history[i + 1].clone();
                            cursor = value.chars().count();
                        }
                    }
                }
                _ => {
                    if apply_line_edit(&mut value, &mut cursor, &key) {
                        history_at = None;
                    }
                }
            }
            redraw = true;
        }
    }

    fn quick_open(&mut self) {
        let files = self.collect_quick_open_files(QUICK_OPEN_LIMIT);
        let mut symbols = Vec::new();
        let mut symbol_at = 0usize;
        let mut query = String::new();
        let mut selected = 0usize;
        loop {
            if symbol_at < files.len() && symbols.len() < QUICK_OPEN_LIMIT {
                let next = min(files.len(), symbol_at + 20);
                symbols.extend(self.collect_quick_open_symbols(&files[symbol_at..next], QUICK_OPEN_LIMIT - symbols.len()));
                symbol_at = next;
            }
            let (bare_line, q, line) = parse_quick_open_query(&query);
            let matches = if bare_line { Vec::new() } else { self.filter_quick_open_items(&files, &symbols, &q) };
            selected = min(selected, matches.len().saturating_sub(1));
            self.render_quick_open(&query, &matches, selected);
            // Non-blocking so symbol chunks keep scanning between keystrokes.
            let key = match self.read_key() {
                Ok(Some(k)) => k,
                Ok(None) => continue,
                Err(_) => return,
            };
            let key = match self.picker_mouse(&key, selected, matches.len()) {
                PickerMouse::NotMouse => key,
                PickerMouse::Handled(s) => { selected = s; continue; }
                PickerMouse::Cancel => { self.message = "Quick open cancelled".to_string(); return; }
                PickerMouse::Activate(i) => { selected = i; "\n".to_string() }
            };
            match key.as_str() {
                "\r" | "\n" => {
                    if bare_line {
                        if let Some(n) = line { self.go_to_line(n); }
                    } else if let Some(item) = matches.get(selected) {
                        if let Some(path) = &item.path {
                            self.open_file(path.clone(), false);
                            if let Some(n) = item.line { self.go_to_line(n); }
                        }
                    } else {
                        self.message = "No match".to_string();
                    }
                    return;
                }
                "\x1b" => { self.message = "Quick open cancelled".to_string(); return; }
                "\x1b[A" | "\x10" => selected = selected.saturating_sub(1),
                "\x1b[B" | "\x0e" => selected = min(matches.len().saturating_sub(1), selected + 1),
                "\x7f" | "\x08" => { remove_last_char(&mut query); selected = 0; }
                "\x15" => { query.clear(); selected = 0; }
                _ => {
                    if let Some(pasted) = key.strip_prefix("\0AZPASTE:") {
                        query.push_str(&pasted.replace('\r', " ").replace('\n', " "));
                        selected = 0;
                    } else if is_printable(&key) {
                        query.push_str(&key);
                        selected = 0;
                    }
                }
            }
        }
    }

    fn collect_quick_open_files(&self, limit: usize) -> Vec<PickerItem> {
        let root = self.root.clone();
        self.collect_files_under(&root, limit)
    }

    fn collect_files_under(&self, dir: &Path, limit: usize) -> Vec<PickerItem> {
        let mut out = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        let skip: HashSet<&str> = [
            ".git",
            "node_modules",
            "vendor",
            ".idea",
            ".vscode",
            "target",
            "dist",
            "build",
            "__pycache__",
            ".next",
            ".nuxt",
        ]
        .into_iter()
        .collect();
        while let Some(dir) = stack.pop() {
            if out.len() >= limit { break; }
            let Ok(read) = fs::read_dir(&dir) else { continue; };
            let mut dirs = Vec::new();
            let mut files = Vec::new();
            for entry in read.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();
                if path.is_dir() {
                    if !skip.contains(name.as_str()) { dirs.push(path); }
                } else if path.is_file() { files.push(path); }
            }
            dirs.sort_by_key(|p| p.to_string_lossy().to_ascii_lowercase());
            dirs.reverse();
            for d in dirs { stack.push(d); }
            files.sort_by_key(|p| p.to_string_lossy().to_ascii_lowercase());
            for p in files {
                let label = relative_path(&self.root, &p);
                out.push(PickerItem { label, detail: "file".to_string(), path: Some(p), line: None, action: None });
                if out.len() >= limit { break; }
            }
        }
        out.sort_by_key(|i| i.label.to_ascii_lowercase());
        out
    }

    fn collect_quick_open_symbols(&self, files: &[PickerItem], limit: usize) -> Vec<PickerItem> {
        let mut out = Vec::new();
        for file in files.iter().take(600) {
            if out.len() >= limit { break; }
            let Some(path) = &file.path else { continue; };
            let syntax = SyntaxMode::from_path(Some(path));
            if !plugins::is_programming_mode(syntax) { continue; }
            let Ok(meta) = fs::metadata(path) else { continue; };
            if meta.len() > 1024 * 1024 { continue; }
            let Ok(text) = fs::read_to_string(path) else { continue; };
            for (name, line) in extract_symbols(&text, syntax) {
                out.push(PickerItem {
                    label: format!("{}  #{}", file.label, name),
                    detail: format!("symbol line {line}"),
                    path: Some(path.clone()),
                    line: Some(line),
                    action: None,
                });
                if out.len() >= limit { break; }
            }
        }
        out
    }

    fn filter_quick_open_items(&self, files: &[PickerItem], symbols: &[PickerItem], query: &str) -> Vec<PickerItem> {
        let items: Vec<&PickerItem> = files.iter().chain(symbols.iter()).collect();
        if query.trim().is_empty() { return files.iter().take(14).cloned().collect(); }
        let mut ranked: Vec<(i32, PickerItem)> = Vec::new();
        for item in items {
            if let Some(score) = quick_score(&format!("{} {}", item.label, item.detail), query) {
                ranked.push((score, item.clone()));
            }
        }
        ranked.sort_by_key(|x| x.0);
        ranked.into_iter().map(|x| x.1).take(14).collect()
    }

    fn render_quick_open(&mut self, query: &str, matches: &[PickerItem], selected: usize) {
        let old = self.message.clone();
        self.message = "Quick open".to_string();
        let _ = self.render();
        self.message = old;
        // Bold the file/symbol part only, not the `:line` suffix.
        let (_, highlight, _) = parse_quick_open_query(query);
        self.render_simple_picker(" Quick Open ", if query.is_empty() { "type file, symbol, file:line, or :line" } else { query }, matches, selected, "No matching files", &highlight);
    }

    fn command_palette(&mut self) {
        let commands = self.command_items();
        let mut query = String::new();
        let mut selected = 0usize;
        loop {
            let matches = self.filter_command_items(&commands, &query);
            selected = min(selected, matches.len().saturating_sub(1));
            let old = self.message.clone();
            self.message = "Command palette".to_string();
            let _ = self.render();
            self.message = old;
            self.render_simple_picker(" Command Palette ", if query.is_empty() { "type a command" } else { &query }, &matches, selected, "No matching commands", &query);
            let key = self.read_key_blocking().unwrap_or_default();
            let key = match self.picker_mouse(&key, selected, matches.len()) {
                PickerMouse::NotMouse => key,
                PickerMouse::Handled(s) => { selected = s; continue; }
                PickerMouse::Cancel => { self.message = "Command cancelled".to_string(); return; }
                PickerMouse::Activate(i) => { selected = i; "\n".to_string() }
            };
            match key.as_str() {
                "\r" | "\n" => {
                    if let Some(item) = matches.get(selected) {
                        if let Some(action) = &item.action { self.run_command(action); }
                    }
                    return;
                }
                "\x1b" => { self.message = "Command cancelled".to_string(); return; }
                "\x1b[A" | "\x10" => selected = selected.saturating_sub(1),
                "\x1b[B" | "\x0e" => selected = min(matches.len().saturating_sub(1), selected + 1),
                "\x7f" | "\x08" => { remove_last_char(&mut query); selected = 0; }
                "\x15" => { query.clear(); selected = 0; }
                _ => {
                    if let Some(pasted) = key.strip_prefix("\0AZPASTE:") {
                        query.push_str(&pasted.replace('\r', " ").replace('\n', " "));
                        selected = 0;
                    } else if is_printable(&key) {
                        query.push_str(&key);
                        selected = 0;
                    }
                }
            }
        }
    }

    fn shortcut_items(&self) -> Vec<PickerItem> {
        shortcut_defs().iter().map(|(l, d)| PickerItem { label: (*l).to_string(), detail: (*d).to_string(), path: None, line: None, action: None }).collect()
    }

    fn shortcuts_dialog(&mut self) {
        let shortcuts = self.shortcut_items();
        let mut query = String::new();
        let mut selected = 0usize;
        loop {
            let matches = self.filter_command_items(&shortcuts, &query);
            selected = min(selected, matches.len().saturating_sub(1));
            let old = self.message.clone();
            self.message = "Keyboard shortcuts".to_string();
            let _ = self.render();
            self.message = old;
            self.render_simple_picker(" Keyboard Shortcuts ", if query.is_empty() { "type to filter (Enter/Esc closes)" } else { &query }, &matches, selected, "No matching shortcuts", &query);
            let key = self.read_key_blocking().unwrap_or_default();
            let key = match self.picker_mouse(&key, selected, matches.len()) {
                PickerMouse::NotMouse => key,
                PickerMouse::Handled(s) => { selected = s; continue; }
                PickerMouse::Cancel => { self.message = "Shortcuts closed".to_string(); return; }
                PickerMouse::Activate(i) => { selected = i; "\n".to_string() }
            };
            match key.as_str() {
                "\r" | "\n" | "\x1b" => { self.message = "Shortcuts closed".to_string(); return; }
                "\x1b[A" | "\x10" => selected = selected.saturating_sub(1),
                "\x1b[B" | "\x0e" => selected = min(matches.len().saturating_sub(1), selected + 1),
                "\x7f" | "\x08" => { remove_last_char(&mut query); selected = 0; }
                "\x15" => { query.clear(); selected = 0; }
                _ => {
                    if let Some(pasted) = key.strip_prefix("\0AZPASTE:") {
                        query.push_str(&pasted.replace('\r', " ").replace('\n', " "));
                        selected = 0;
                    } else if is_printable(&key) {
                        query.push_str(&key);
                        selected = 0;
                    }
                }
            }
        }
    }

    fn command_items(&self) -> Vec<PickerItem> {
        let defs = [
            ("Save", "Ctrl+S", "save"),
            ("Save as", "save current tab to a new path", "save-as"),
            ("New file", "create file in project", "new-file"),
            ("New folder", "create folder in project", "new-folder"),
            ("Rename selected file or folder", "tree/current file", "rename-path"),
            ("Delete selected file or folder", "asks first", "delete-path"),
            ("Go to line", "Ctrl+G", "go-line"),
            ("Go to Start of Line", "Home", "go-line-start"),
            ("Go to End of Line", "End / Ctrl+E", "go-line-end"),
            ("Go to Start of File", "Ctrl+Home / Alt+Up", "go-file-top"),
            ("Go to End of File", "Ctrl+End / Alt+Down", "go-file-bottom"),
            ("Find in files", "search project files (Ctrl+Shift+O, %term = case-sensitive)", "project-search"),
            ("Replace in files", "search & replace across project (Ctrl+Shift+H, %term = case-sensitive)", "replace-in-files"),
            ("Set syntax PHP", "force current tab to PHP", "set-syntax-php"),
            ("Set syntax Blade", "force current tab to Blade", "set-syntax-blade"),
            ("Set syntax HTML", "force current tab to HTML", "set-syntax-html"),
            ("Set syntax CSS", "force current tab to CSS", "set-syntax-css"),
            ("Set syntax JavaScript", "force current tab to JavaScript", "set-syntax-javascript"),
            ("Set syntax TypeScript", "force current tab to TypeScript", "set-syntax-typescript"),
            ("Set syntax XML", "force current tab to XML", "set-syntax-xml"),
            ("Set syntax Markdown", "force current tab to Markdown", "set-syntax-markdown"),
            ("Set syntax JSON", "force current tab to JSON", "set-syntax-json"),
            ("Set syntax TOML", "force current tab to TOML", "set-syntax-toml"),
            ("Set syntax YAML", "force current tab to YAML", "set-syntax-yaml"),
            ("Set syntax Bash", "force current tab to Shell", "set-syntax-bash"),
            ("Set syntax Dotenv", "force current tab to .env", "set-syntax-dotenv"),
            ("Set syntax INI", "force current tab to INI/conf", "set-syntax-ini"),
            ("Set syntax Log", "force current tab to log view", "set-syntax-log"),
            ("Set syntax Rust", "force current tab to Rust", "set-syntax-rust"),
            ("Set syntax Nginx", "force current tab to Nginx", "set-syntax-nginx"),
            ("Set syntax Apache", "force current tab to Apache", "set-syntax-apache"),
            ("Set syntax Dockerfile", "force current tab to Dockerfile", "set-syntax-dockerfile"),
            ("Set syntax Systemd", "force current tab to systemd", "set-syntax-systemd"),
            ("Set syntax SQL", "force current tab to SQL", "set-syntax-sql"),
            ("Set syntax Python", "force current tab to Python", "set-syntax-python"),
            ("Set syntax Java", "force current tab to Java", "set-syntax-java"),
            ("Set syntax C#", "force current tab to C#", "set-syntax-csharp"),
            ("Set syntax C++", "force current tab to C++", "set-syntax-cpp"),
            ("Set syntax C", "force current tab to C", "set-syntax-c"),
            ("Set syntax Go", "force current tab to Go", "set-syntax-go"),
            ("Set syntax Kotlin", "force current tab to Kotlin", "set-syntax-kotlin"),
            ("Set syntax Swift", "force current tab to Swift", "set-syntax-swift"),
            ("Set syntax Ruby", "force current tab to Ruby", "set-syntax-ruby"),
            ("Set syntax Dart", "force current tab to Dart", "set-syntax-dart"),
            ("Set syntax Scala", "force current tab to Scala", "set-syntax-scala"),
            ("Set syntax R", "force current tab to R", "set-syntax-r"),
            ("Set syntax Lua", "force current tab to Lua", "set-syntax-lua"),
            ("Set syntax Perl", "force current tab to Perl", "set-syntax-perl"),
            ("Set syntax Haskell", "force current tab to Haskell", "set-syntax-haskell"),
            ("Set syntax Elixir", "force current tab to Elixir", "set-syntax-elixir"),
            ("Set syntax Clojure", "force current tab to Clojure", "set-syntax-clojure"),
            ("Set syntax Zig", "force current tab to Zig", "set-syntax-zig"),
            ("Set syntax Julia", "force current tab to Julia", "set-syntax-julia"),
            ("Set syntax Objective-C", "force current tab to Objective-C", "set-syntax-objc"),
            ("Set syntax Auto", "use file extension again", "set-syntax-auto"),
            ("Set syntax Plain", "disable highlighting/completion", "set-syntax-plain"),
            ("Find in current file", "Ctrl+F", "find"),
            ("Replace in current file", "Ctrl+R", "replace"),
            ("Toggle sidebar", "Ctrl+H", "toggle-tree"),
            ("Focus tree/editor", "Ctrl+T", "focus-tree"),
            ("Close tab", "Ctrl+D", "close-tab"),
            ("Demo mode", "show welcome, command palette, quick open", "demo-mode"),
            ("Keyboard shortcuts", "searchable list (Ctrl+K)", "help"),
            ("Quit", "Ctrl+Q", "quit"),
        ];
        defs.iter().map(|(l, d, a)| PickerItem { label: (*l).to_string(), detail: (*d).to_string(), path: None, line: None, action: Some((*a).to_string()) }).collect()
    }

    fn filter_command_items(&self, commands: &[PickerItem], query: &str) -> Vec<PickerItem> {
        if query.trim().is_empty() { return commands.iter().take(18).cloned().collect(); }
        let mut ranked = Vec::new();
        for cmd in commands {
            if let Some(score) = quick_score(&format!("{} {}", cmd.label, cmd.detail), query) {
                ranked.push((score, cmd.clone()));
            }
        }
        ranked.sort_by_key(|x| x.0);
        ranked.into_iter().map(|x| x.1).take(12).collect()
    }

    fn run_command(&mut self, action: &str) {
        match action {
            "save" => self.save_current_tab(),
            "save-as" => self.save_current_tab_as(),
            "new-file" => self.create_file_prompt(None),
            "new-folder" => self.create_folder_prompt(None),
            "rename-path" => self.rename_tree_path_prompt(true),
            "delete-path" => self.delete_tree_path_prompt(true),
            "go-line" => self.go_to_line_prompt(),
            "go-line-start" => self.go_to_line_start(),
            "go-line-end" => self.go_to_line_end(),
            "go-file-top" => self.go_to_file_top(false),
            "go-file-bottom" => self.go_to_file_bottom(false),
            "project-search" => self.project_search_prompt(),
            "replace-in-files" => self.replace_in_files_prompt(),
            "set-syntax-php" => self.set_current_syntax(Some(SyntaxMode::Php)),
            "set-syntax-blade" => self.set_current_syntax(Some(SyntaxMode::Blade)),
            "set-syntax-html" => self.set_current_syntax(Some(SyntaxMode::Html)),
            "set-syntax-css" => self.set_current_syntax(Some(SyntaxMode::Css)),
            "set-syntax-javascript" => self.set_current_syntax(Some(SyntaxMode::JavaScript)),
            "set-syntax-typescript" => self.set_current_syntax(Some(SyntaxMode::TypeScript)),
            "set-syntax-xml" => self.set_current_syntax(Some(SyntaxMode::Xml)),
            "set-syntax-markdown" => self.set_current_syntax(Some(SyntaxMode::Markdown)),
            "set-syntax-json" => self.set_current_syntax(Some(SyntaxMode::Json)),
            "set-syntax-toml" => self.set_current_syntax(Some(SyntaxMode::Toml)),
            "set-syntax-yaml" => self.set_current_syntax(Some(SyntaxMode::Yaml)),
            "set-syntax-bash" => self.set_current_syntax(Some(SyntaxMode::Bash)),
            "set-syntax-dotenv" => self.set_current_syntax(Some(SyntaxMode::Dotenv)),
            "set-syntax-ini" => self.set_current_syntax(Some(SyntaxMode::Ini)),
            "set-syntax-log" => self.set_current_syntax(Some(SyntaxMode::Log)),
            "set-syntax-rust" => self.set_current_syntax(Some(SyntaxMode::Rust)),
            "set-syntax-nginx" => self.set_current_syntax(Some(SyntaxMode::Nginx)),
            "set-syntax-apache" => self.set_current_syntax(Some(SyntaxMode::Apache)),
            "set-syntax-dockerfile" => self.set_current_syntax(Some(SyntaxMode::Dockerfile)),
            "set-syntax-systemd" => self.set_current_syntax(Some(SyntaxMode::Systemd)),
            "set-syntax-sql" => self.set_current_syntax(Some(SyntaxMode::Sql)),
            "set-syntax-python" => self.set_current_syntax(Some(SyntaxMode::Python)),
            "set-syntax-java" => self.set_current_syntax(Some(SyntaxMode::Java)),
            "set-syntax-csharp" => self.set_current_syntax(Some(SyntaxMode::Csharp)),
            "set-syntax-cpp" => self.set_current_syntax(Some(SyntaxMode::Cpp)),
            "set-syntax-c" => self.set_current_syntax(Some(SyntaxMode::C)),
            "set-syntax-go" => self.set_current_syntax(Some(SyntaxMode::Go)),
            "set-syntax-kotlin" => self.set_current_syntax(Some(SyntaxMode::Kotlin)),
            "set-syntax-swift" => self.set_current_syntax(Some(SyntaxMode::Swift)),
            "set-syntax-ruby" => self.set_current_syntax(Some(SyntaxMode::Ruby)),
            "set-syntax-dart" => self.set_current_syntax(Some(SyntaxMode::Dart)),
            "set-syntax-scala" => self.set_current_syntax(Some(SyntaxMode::Scala)),
            "set-syntax-r" => self.set_current_syntax(Some(SyntaxMode::R)),
            "set-syntax-lua" => self.set_current_syntax(Some(SyntaxMode::Lua)),
            "set-syntax-perl" => self.set_current_syntax(Some(SyntaxMode::Perl)),
            "set-syntax-haskell" => self.set_current_syntax(Some(SyntaxMode::Haskell)),
            "set-syntax-elixir" => self.set_current_syntax(Some(SyntaxMode::Elixir)),
            "set-syntax-clojure" => self.set_current_syntax(Some(SyntaxMode::Clojure)),
            "set-syntax-zig" => self.set_current_syntax(Some(SyntaxMode::Zig)),
            "set-syntax-julia" => self.set_current_syntax(Some(SyntaxMode::Julia)),
            "set-syntax-objc" => self.set_current_syntax(Some(SyntaxMode::Objc)),
            "set-syntax-plain" => self.set_current_syntax(Some(SyntaxMode::Plain)),
            "set-syntax-auto" => self.set_current_syntax(None),
            "find" => self.find_prompt(),
            "replace" => self.replace_prompt(),
            "toggle-tree" => self.toggle_sidebar(),
            "focus-tree" => self.toggle_tree_focus(),
            "close-tab" => self.close_current_tab(),
            "demo-mode" => self.show_demo_mode(),
            "help" => self.shortcuts_dialog(),
            "quit" => self.confirm_quit(),
            _ => self.message = "Unknown command".to_string(),
        }
    }

    fn render_simple_picker(&self, title: &str, query_line: &str, matches: &[PickerItem], selected: usize, empty: &str, highlight_query: &str) {
        let frame = picker_frame(self.cols, self.rows);
        let rows = frame.list_rows;
        let panel_width = frame.width;
        let panel_height = frame.height;
        let start_col = frame.start_col;
        let start_row = frame.start_row;
        let inner = panel_width.saturating_sub(2);
        let border = ansi_style(Some(BLUE), Some(BG_FLOAT), true, false, false);
        let query_style = ansi_style(Some(FG), Some(BG_HIGHLIGHT), false, false, false);
        let mut out = String::new();
        out.push_str("\x1b[?25l");
        out.push_str(&format!("\x1b[{start_row};{start_col}H{border}╔{}╗\x1b[0m", "═".repeat(inner)));
        out.push_str(&format!("\x1b[{};{start_col}H{border}║\x1b[0m{}{}\x1b[0m{border}║\x1b[0m", start_row + 1, ansi_style(Some(ACCENT), Some(BG_FLOAT), true, false, false), fit_plain(title, inner)));
        out.push_str(&format!("\x1b[{};{start_col}H{border}║\x1b[0m{query_style}{}\x1b[0m{border}║\x1b[0m", start_row + 2, fit_plain(query_line, inner)));
        out.push_str(&format!("\x1b[{};{start_col}H{border}╠{}╣\x1b[0m", start_row + 3, "═".repeat(inner)));
        for i in 0..rows {
            let row = start_row + 4 + i;
            let cell = if let Some(item) = matches.get(i) {
                let prefix = if i == selected { " › " } else { "   " };
                let left_plain = format!("{prefix}{}", item.label);
                let right_plain = if item.detail.is_empty() { String::new() } else { format!("  {}", item.detail) };
                let spaces = inner.saturating_sub(visual_width(&left_plain) + visual_width(&right_plain)).max(1);
                // Bold the query match inside label (fall back to detail).
                let label_ranges = match_bold_ranges(&item.label, highlight_query);
                let (bold_label, bold_detail) = if !label_ranges.is_empty() {
                    (apply_bold_ansi(&item.label, &label_ranges), item.detail.clone())
                } else {
                    let detail_ranges = match_bold_ranges(&item.detail, highlight_query);
                    (item.label.clone(), apply_bold_ansi(&item.detail, &detail_ranges))
                };
                let ansi = format!("{prefix}{bold_label}{}{}", " ".repeat(spaces), if bold_detail.is_empty() { String::new() } else { format!("  {bold_detail}") });
                fit_ansi(&ansi, inner)
            } else if matches.is_empty() && i == 0 {
                fit_plain(&format!("   {empty}"), inner)
            } else {
                fit_plain("", inner)
            };
            let style = if i == selected && matches.get(i).is_some() { ansi_style(Some(BG_DARK), Some(ACCENT), true, false, false) } else { ansi_style(Some(FG), Some(BG_FLOAT), false, false, false) };
            out.push_str(&format!("\x1b[{row};{start_col}H{border}║\x1b[0m{style}{cell}\x1b[0m{border}║\x1b[0m"));
        }
        out.push_str(&format!("\x1b[{};{start_col}H{border}╚{}╝\x1b[0m", start_row + panel_height - 1, "═".repeat(inner)));
        print!("{out}");
        let _ = io::stdout().flush();
    }

    fn project_search_prompt(&mut self) {
        let root = self.root.clone();
        self.project_search_prompt_in(root);
    }

    fn project_search_prompt_in(&mut self, scope: PathBuf) {
        let files = self.collect_files_under(&scope, 3000);
        let mut query = String::new();
        let mut active = String::new();
        let mut matches = Vec::new();
        let mut scanned = 0usize;
        let mut selected = 0usize;
        loop {
            if query != active {
                active = query.clone();
                matches.clear();
                scanned = 0;
            }
            if !query.is_empty() && scanned < files.len() && matches.len() < PROJECT_SEARCH_LIMIT {
                let next = min(files.len(), scanned + 25);
                self.search_file_slice(&files[scanned..next], &query, &mut matches, PROJECT_SEARCH_LIMIT);
                scanned = next;
            }
            selected = min(selected, matches.len().saturating_sub(1));
            let old = self.message.clone();
            self.message = "Find in files".to_string();
            let _ = self.render();
            self.message = old;
            // Strip `%` for highlight so `%Foo` still bolds `Foo` in results.
            let (needle, _) = parse_search_query(&query);
            let highlight = if needle.is_empty() { query.clone() } else { needle };
            let placeholder = if query.is_empty() {
                "type text to search files (%Foo = case-sensitive)"
            } else if scanned < files.len() {
                "searching…"
            } else {
                &query
            };
            self.render_simple_picker(" Find in Files ", placeholder, &matches, selected, "No matches", &highlight);
            let key = match self.read_key() {
                Ok(Some(k)) => k,
                _ => continue,
            };
            let key = match self.picker_mouse(&key, selected, matches.len()) {
                PickerMouse::NotMouse => key,
                PickerMouse::Handled(s) => { selected = s; continue; }
                PickerMouse::Cancel => { self.message = "Search cancelled".to_string(); return; }
                PickerMouse::Activate(i) => { selected = i; "\n".to_string() }
            };
            match key.as_str() {
                "\r" | "\n" => {
                    if let Some(item) = matches.get(selected) {
                        if let Some(path) = &item.path {
                            self.open_file(path.clone(), false);
                            if let Some(line) = item.line { self.go_to_line(line); }
                        }
                    } else { self.message = if query.is_empty() { "Search cancelled" } else { "No match" }.to_string(); }
                    return;
                }
                "\x1b" => { self.message = "Search cancelled".to_string(); return; }
                "\x1b[A" | "\x10" => selected = selected.saturating_sub(1),
                "\x1b[B" | "\x0e" => selected = min(matches.len().saturating_sub(1), selected + 1),
                "\x7f" | "\x08" => { remove_last_char(&mut query); selected = 0; }
                "\x15" => { query.clear(); selected = 0; }
                _ => {
                    if let Some(pasted) = key.strip_prefix("\0AZPASTE:") {
                        query.push_str(&pasted.replace('\r', " ").replace('\n', " "));
                        selected = 0;
                    } else if is_printable(&key) {
                        query.push_str(&key);
                        selected = 0;
                    }
                }
            }
        }
    }

    fn search_file_slice(&self, files: &[PickerItem], query: &str, matches: &mut Vec<PickerItem>, limit: usize) {
        let (needle, ignore_case) = parse_search_query(query);
        if needle.is_empty() {
            return;
        }
        for file in files {
            if matches.len() >= limit {
                break;
            }
            let Some(path) = &file.path else { continue; };
            let Ok(meta) = fs::metadata(path) else { continue; };
            if meta.len() > 5 * 1024 * 1024 {
                continue;
            }
            let Ok(text) = fs::read_to_string(path) else { continue; };
            for (i, line) in text.replace("\r\n", "\n").replace('\r', "\n").lines().enumerate() {
                if project_line_matches(line, &needle, ignore_case) {
                    matches.push(PickerItem {
                        label: format!("{}:{}", relative_path(&self.root, path), i + 1),
                        detail: truncate_plain(line.trim(), 42),
                        path: Some(path.clone()),
                        line: Some(i + 1),
                        action: None,
                    });
                    if matches.len() >= limit {
                        break;
                    }
                }
            }
        }
    }

    fn replace_in_files_prompt(&mut self) {
        self.search_replace_dialog(self.root.clone(), 1);
    }

    fn replace_in_files_prompt_scoped(&mut self, scope: PathBuf) {
        self.search_replace_dialog(scope, 1);
    }

    fn apply_project_replace(&mut self, scope: &Path, query: &str, replacement: &str) -> String {
        let (needle, ignore_case) = parse_search_query(query);
        if needle.is_empty() { return "Empty search".to_string(); }
        let files = if scope.is_file() {
            vec![PickerItem { label: String::new(), detail: String::new(), path: Some(scope.to_path_buf()), line: None, action: None }]
        } else {
            self.collect_files_under(scope, 3000)
        };
        let mut per_file: Vec<(PathBuf, usize)> = Vec::new();
        let mut total = 0usize;
        let mut truncated = false;
        for file in &files {
            let Some(path) = &file.path else { continue; };
            let Ok(meta) = fs::metadata(path) else { continue; };
            if meta.len() > 5 * 1024 * 1024 { continue; }
            let Ok(text) = fs::read_to_string(path) else { continue; };
            let mut count = 0usize;
            for line in text.replace("\r\n", "\n").replace('\r', "\n").lines() {
                count += count_matches_in_line(line, &needle, ignore_case);
                if total + count >= REPLACE_MATCH_LIMIT { truncated = true; break; }
            }
            if count > 0 {
                total += count;
                per_file.push((path.clone(), count));
            }
            if truncated { break; }
        }
        if per_file.is_empty() { return "No matches".to_string(); }
        let mut files_ok = 0usize;
        let mut applied = 0usize;
        let mut skipped = 0usize;
        let mut errors = 0usize;
        for (path, _) in &per_file {
            if self.tabs.iter().any(|t| t.path.as_ref() == Some(path) && t.modified) {
                skipped += 1;
                continue;
            }
            let Ok(text) = fs::read_to_string(path) else { errors += 1; continue; };
            let crlf = text.contains("\r\n");
            let norm = text.replace("\r\n", "\n").replace('\r', "\n");
            let mut out_lines = Vec::new();
            let mut count = 0usize;
            for line in norm.lines() {
                let (replaced, n) = replace_in_line(line, &needle, replacement, ignore_case);
                count += n;
                out_lines.push(replaced);
            }
            let sep = if crlf { "\r\n" } else { "\n" };
            let mut new_text = out_lines.join(sep);
            if norm.ends_with('\n') {
                new_text.push_str(sep);
            }
            if atomic_write_file(path, new_text.as_bytes()).is_err() { errors += 1; continue; }
            files_ok += 1;
            applied += count;
            let mut lf = out_lines.join("\n");
            if norm.ends_with('\n') { lf.push('\n'); }
            self.reload_open_tabs_for_path(path, &lf, crlf);
        }
        let mut msg = format!("Replaced {applied} in {files_ok} file{}", if files_ok == 1 { "" } else { "s" });
        if skipped > 0 {
            msg.push_str(&format!(" ({skipped} skipped: unsaved open buffer{})", if skipped == 1 { "" } else { "s" }));
        }
        if errors > 0 {
            msg.push_str(&format!(" ({errors} failed)"));
        }
        if truncated {
            msg.push_str(" (capped: Narrow your search)");
        }
        msg
    }

    /// Reload open tabs for a file rewritten on disk (project-wide replace).
    /// Clears undo/redo: stored byte positions refer to the old content.
    fn reload_open_tabs_for_path(&mut self, path: &Path, new_text: &str, crlf: bool) {
        for tab in &mut self.tabs {
            if tab.path.as_ref() != Some(&path.to_path_buf()) {
                continue;
            }
            let mut lines: Vec<String> = new_text.split('\n').map(str::to_string).collect();
            if lines.is_empty() {
                lines.push(String::new());
            }
            tab.lines = lines;
            tab.cursor.line = min(tab.cursor.line, tab.lines.len().saturating_sub(1));
            tab.cursor.col = clamp_char_boundary(&tab.lines[tab.cursor.line], min(tab.cursor.col, tab.lines[tab.cursor.line].len()));
            tab.col_offset = 0;
            tab.row_offset = 0;
            tab.undo.clear();
            tab.redo.clear();
            tab.saved_revision = tab.revision;
            tab.saved_hash = buffer_hash(&tab.lines);
            tab.modified = false;
            tab.crlf = crlf;
        }
    }

    fn selected_tree_path(&mut self, prefer_current_file: bool) -> PathBuf {
        self.refresh_tree();
        if !prefer_current_file {
            if let Some(row) = self.tree_rows.get(self.tree_index) { return row.path.clone(); }
        }
        if let Some(path) = &self.tab().path { return path.clone(); }
        self.tree_rows.get(self.tree_index).map(|r| r.path.clone()).unwrap_or_else(|| self.root.clone())
    }

    fn base_dir_for_tree_action(&mut self) -> PathBuf {
        let path = self.selected_tree_path(false);
        if path.is_dir() { path } else { path.parent().unwrap_or(&self.root).to_path_buf() }
    }

    fn new_tree_file_prompt(&mut self) { let base = self.base_dir_for_tree_action(); self.create_file_prompt(Some(base)); }
    fn new_tree_folder_prompt(&mut self) { let base = self.base_dir_for_tree_action(); self.create_folder_prompt(Some(base)); }

    fn create_file_prompt(&mut self, base_dir: Option<PathBuf>) {
        let name = self.prompt("New file: ", "");
        if name.trim().is_empty() { self.message = "New file cancelled".to_string(); return; }
        let base = base_dir.unwrap_or_else(|| self.root.clone());
        let path = absolute_path(Path::new(name.trim()), Some(&base));
        if path.is_dir() { self.message = "That is a folder".to_string(); return; }
        if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
        if !path.exists() && fs::write(&path, b"").is_err() { self.message = "Could not create file".to_string(); return; }
        self.needs_tree_refresh = true;
        self.open_file(path.clone(), false);
        self.message = format!("Created {}", relative_path(&self.root, &path));
    }

    fn create_folder_prompt(&mut self, base_dir: Option<PathBuf>) {
        let name = self.prompt("New folder: ", "");
        if name.trim().is_empty() { self.message = "New folder cancelled".to_string(); return; }
        let base = base_dir.unwrap_or_else(|| self.root.clone());
        let path = absolute_path(Path::new(name.trim()), Some(&base));
        if fs::create_dir_all(&path).is_err() { self.message = "Could not create folder".to_string(); return; }
        if let Some(parent) = path.parent() { self.expanded.insert(parent.to_path_buf()); }
        self.needs_tree_refresh = true;
        self.reveal_path_in_tree(&path);
        self.message = format!("Created folder {}", relative_path(&self.root, &path));
    }

    fn rename_tree_path_prompt(&mut self, prefer_current_file: bool) {
        let path = self.selected_tree_path(prefer_current_file);
        if path == self.root { self.message = "Cannot rename project root".to_string(); return; }
        let default = path.file_name().and_then(OsStr::to_str).unwrap_or("");
        let name = self.prompt("Rename to: ", default);
        if name.trim().is_empty() || name.trim() == default { self.message = "Rename cancelled".to_string(); return; }
        let new_path = absolute_path(Path::new(name.trim()), path.parent());
        if new_path.exists() { self.message = "Target already exists".to_string(); return; }
        if fs::rename(&path, &new_path).is_err() { self.message = "Rename failed".to_string(); return; }
        let was_dir = new_path.is_dir();
        for tab in &mut self.tabs {
            if tab.path.as_ref() == Some(&path) {
                tab.path = Some(new_path.clone());
                tab.name = new_path.file_name().and_then(OsStr::to_str).unwrap_or("Untitled").to_string();
            } else if was_dir {
                if let Some(tp) = tab.path.clone() {
                    if let Ok(rest) = tp.strip_prefix(&path) {
                        tab.path = Some(new_path.join(rest));
                    }
                }
            }
        }
        self.needs_tree_refresh = true;
        self.reveal_path_in_tree(&new_path);
        self.message = "Renamed".to_string();
    }

    fn delete_tree_path_prompt(&mut self, prefer_current_file: bool) {
        let path = self.selected_tree_path(prefer_current_file);
        if path == self.root { self.message = "Cannot delete project root".to_string(); return; }
        let answer = self.prompt(&format!("Delete {}? y/N: ", path.file_name().and_then(OsStr::to_str).unwrap_or("path")), "");
        if answer.to_ascii_lowercase() != "y" { self.message = "Delete cancelled".to_string(); return; }
        let was_dir = path.is_dir();
        let ok = if was_dir { fs::remove_dir_all(&path).is_ok() } else { fs::remove_file(&path).is_ok() };
        if !ok { self.message = "Delete failed".to_string(); return; }
        let mut i = self.tabs.len();
        while i > 0 {
            i -= 1;
            let remove = self.tabs[i].path.as_ref().map(|p| p == &path || (was_dir && p.starts_with(&path))).unwrap_or(false);
            if remove {
                let tab = self.tabs.remove(i);
                self.delete_recovery_for_tab(&tab);
            }
        }
        if self.tabs.is_empty() { self.tabs.push(Tab::empty()); }
        self.tab_index = min(self.tab_index, self.tabs.len() - 1);
        self.needs_tree_refresh = true;
        self.message = "Deleted".to_string();
    }

    fn reveal_path_in_tree(&mut self, path: &Path) {
        let mut cur = path.parent();
        while let Some(p) = cur {
            self.expanded.insert(p.to_path_buf());
            if p == self.root { break; }
            cur = p.parent();
        }
        self.needs_tree_refresh = true;
        self.refresh_tree();
        if let Some(idx) = self.tree_rows.iter().position(|r| r.path == path) {
            self.tree_index = idx;
            self.ensure_tree_visible();
        }
    }

    fn toggle_sidebar(&mut self) {
        self.sidebar_hidden = !self.sidebar_hidden;
        if self.sidebar_hidden && self.focus == Focus::Tree { self.focus = Focus::Editor; }
        self.message = if self.sidebar_hidden { "Tree hidden" } else { "Tree shown" }.to_string();
    }

    fn toggle_tree_focus(&mut self) {
        if self.sidebar_hidden {
            self.sidebar_hidden = false;
            self.focus = Focus::Tree;
            self.message = "Tree shown and focused".to_string();
        } else {
            self.focus = if self.focus == Focus::Tree { Focus::Editor } else { Focus::Tree };
            self.message = if self.focus == Focus::Tree { "Tree focused" } else { "Editor focused" }.to_string();
        }
    }

    fn switch_to_tab_number(&mut self, number: usize) {
        let visible = self.visible_tab_indexes();
        if let Some(idx) = visible.get(number.saturating_sub(1)) {
            self.tab_index = *idx;
            self.clear_selection();
            self.message = format!("Tab {number}");
        } else {
            self.message = format!("No tab {number}");
        }
    }

    fn set_current_syntax(&mut self, syntax: Option<SyntaxMode>) {
        self.tab_mut().syntax_mode = syntax;
        self.message = match syntax { Some(s) => format!("Syntax set to {}", s.label()), None => "Syntax set to AUTO".to_string() };
        self.close_autocomplete();
    }

    fn show_demo_mode(&mut self) {
        let _ = self.render();
        let width = max(28, self.cols / 3);
        let height = max(8, self.rows / 2);
        let quick = vec!["main.rs".to_string(), "src/editor.rs".to_string(), "README.md".to_string(), "app.php  #function run".to_string()];
        let cmd = vec!["Save                   Ctrl+S".to_string(), "Set syntax PHP         force current tab".to_string(), "Project search         find text in files".to_string(), "Shortcuts              Ctrl+K".to_string()];
        let welcome = vec!["   __ _ ____".to_string(), "  / _` |_  /".to_string(), " | (_| |/ / ".to_string(), r"  \__,_/___|".to_string(), "Ctrl+O Quick open".to_string(), "Ctrl+P Command palette".to_string(), "Ctrl+T Tree/editor".to_string()];
        self.render_demo_tile(2, 2, width, height, "Welcome", &welcome, None, &[0,1,2,3]);
        self.render_demo_tile(2, 4 + width, width, height, "Quick Open", &quick, Some(0), &[]);
        self.render_demo_tile(2, 6 + width * 2, self.cols.saturating_sub(7 + width * 2), height, "Command Palette", &cmd, Some(0), &[]);
        let _ = self.read_key_blocking();
        self.message = "Demo closed".to_string();
    }

    fn render_demo_tile(&self, row: usize, col: usize, width: usize, height: usize, title: &str, lines: &[String], selected: Option<usize>, logo_lines: &[usize]) {
        if width < 8 || height < 4 || col > self.cols { return; }
        let inner = width.saturating_sub(2);
        let border = ansi_style(Some(BLUE), Some(BG_FLOAT), true, false, false);
        let mut out = String::new();
        out.push_str(&format!("\x1b[{row};{col}H{border}╔{}╗\x1b[0m", "═".repeat(inner)));
        out.push_str(&format!("\x1b[{};{col}H{border}║\x1b[0m{}{}\x1b[0m{border}║\x1b[0m", row + 1, ansi_style(Some(ACCENT), Some(BG_FLOAT), true, false, false), fit_plain(title, inner)));
        for i in 0..height.saturating_sub(3) {
            let r = row + 2 + i;
            let text = lines.get(i).map(String::as_str).unwrap_or("");
            let style = if selected == Some(i) { ansi_style(Some(BG_DARK), Some(ACCENT), true, false, false) } else if logo_lines.contains(&i) { ansi_style(Some(ORANGE), Some(BG_FLOAT), true, false, false) } else { ansi_style(Some(FG), Some(BG_FLOAT), false, false, false) };
            out.push_str(&format!("\x1b[{r};{col}H{border}║\x1b[0m{style}{}\x1b[0m{border}║\x1b[0m", fit_plain(text, inner)));
        }
        out.push_str(&format!("\x1b[{};{col}H{border}╚{}╝\x1b[0m", row + height - 1, "═".repeat(inner)));
        print!("{out}");
        let _ = io::stdout().flush();
    }

    fn handle_autocomplete_key(&mut self, key: &str) -> bool {
        if key == "\t" {
            if self.autocomplete_visible {
                self.accept_autocomplete();
                return true;
            }
            if self.refresh_autocomplete(true) { return true; }
            self.insert_text("\t");
            return true;
        }
        if !self.autocomplete_visible { return false; }
        match key {
            "\r" | "\n" => { self.accept_autocomplete(); true }
            "\x1b" => { self.close_autocomplete(); self.message = "Autocomplete closed".to_string(); true }
            "\x1b[A" => { self.autocomplete_index = self.autocomplete_index.saturating_sub(1); true }
            "\x1b[B" => { self.autocomplete_index = min(self.autocomplete_items.len().saturating_sub(1), self.autocomplete_index + 1); true }
            _ => false,
        }
    }

    fn refresh_autocomplete(&mut self, explicit: bool) -> bool {
        let Some((mode, prefix, start)) = self.autocomplete_context(explicit) else {
            self.close_autocomplete(); return false;
        };
        let items = self.autocomplete_suggestions(&mode, &prefix);
        if items.is_empty() { self.close_autocomplete(); return false; }
        self.autocomplete_visible = true;
        self.autocomplete_items = items;
        self.autocomplete_index = 0;
        self.autocomplete_line = self.tab().cursor.line;
        self.autocomplete_start_col = start;
        self.autocomplete_prefix = prefix;
        true
    }

    fn close_autocomplete(&mut self) {
        self.autocomplete_visible = false;
        self.autocomplete_items.clear();
        self.autocomplete_index = 0;
        self.autocomplete_prefix.clear();
    }

    fn accept_autocomplete(&mut self) {
        if !self.autocomplete_visible { return; }
        let Some(item) = self.autocomplete_items.get(self.autocomplete_index).cloned() else { self.close_autocomplete(); return; };
        if self.tab().cursor.line != self.autocomplete_line { self.close_autocomplete(); return; }
        let start = Pos { line: self.autocomplete_line, col: self.autocomplete_start_col };
        let end = self.tab().cursor;
        let before = self.tab().cursor;
        let deleted = self.apply_delete_range(start, end);
        let end_pos = self.apply_insert_at(start, &item.insert);
        self.tab_mut().cursor = end_pos;
        self.push_history(HistoryEntry { ops: vec![TextOp::Delete { pos: start, text: deleted }, TextOp::Insert { pos: start, text: item.insert.clone() }], before, after: end_pos });
        self.mark_edited();
        self.clear_selection();
        self.message = format!("Completed {}", item.label);
        self.close_autocomplete();
    }

    fn autocomplete_context(&self, explicit: bool) -> Option<(String, String, usize)> {
        let tab = self.tab();
        let line = &tab.lines[tab.cursor.line];
        let before = &line[..tab.cursor.col];
        plugins::completion_context(tab.syntax(), before, explicit)
    }

    fn autocomplete_suggestions(&self, mode: &str, prefix: &str) -> Vec<CompletionItem> {
        plugins::completion_items(mode, prefix, plugins::CompletionContext { lines: &self.tab().lines, scan_limit: HUGE_SCAN_LIMIT })
    }


    fn render_autocomplete_dropdown(&self, cursor_row: usize, cursor_col: usize) -> String {
        let Some((start_col, start_row, width, count, first)) = self.autocomplete_geometry(cursor_row, cursor_col) else {
            return String::new();
        };
        let mut out = String::new();
        for (screen_i, item) in self.autocomplete_items.iter().enumerate().skip(first).take(count) {
            let row = start_row + screen_i - first;
            if row >= self.status_line { break; }
            let detail_width = min(20, max(10, width * 28 / 100));
            let label_width = width.saturating_sub(detail_width + 4).max(8);
            let label = truncate_plain(&item.label, label_width);
            let detail = truncate_plain(&item.detail, detail_width);
            let spaces = width.saturating_sub(visual_width(&label) + visual_width(&detail) + 2).max(1);
            let text = format!(" {label}{}{detail} ", " ".repeat(spaces));
            let style = if screen_i == self.autocomplete_index { ansi_style(Some(BG_DARK), Some(ACCENT), true, false, false) } else { ansi_style(Some(FG), Some(BG_FLOAT), false, false, false) };
            out.push_str(&format!("\x1b[{row};{start_col}H{style}{}\x1b[0m", fit_plain(&text, width)));
        }
        out
    }

    fn state_dir(&self) -> PathBuf {
        let base = env::var_os("XDG_STATE_HOME").map(PathBuf::from).unwrap_or_else(|| {
            env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| env::temp_dir()).join(".local/state")
        });
        let dir = base.join("az-rust");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn session_file(&self) -> PathBuf { self.state_dir().join(format!("session-{}.txt", simple_hash(self.root.to_string_lossy().as_bytes()))) }
    fn recovery_dir(&self) -> PathBuf { let d = self.state_dir().join("recovery"); let _ = fs::create_dir_all(&d); d }
    fn recovery_file_for(&self, tab: &Tab) -> PathBuf {
        let key = match &tab.path { Some(p) => p.to_string_lossy().to_string(), None => format!("untitled:{}", tab.name) };
        self.recovery_dir().join(format!("{}.rec", simple_hash(format!("{}\0{}", self.root.to_string_lossy(), key).as_bytes())))
    }

    fn write_recovery_for_current_tab(&mut self) {
        if !self.tab().modified {
            let file = self.recovery_file_for(self.tab());
            let _ = fs::remove_file(file);
            return;
        }
        let file = self.recovery_file_for(self.tab());
        let key = file.to_string_lossy().to_string();
        if self.last_recovery_write.get(&key).map(|t| t.elapsed() < Duration::from_millis(250)).unwrap_or(false) { return; }
        self.last_recovery_write.insert(key, Instant::now());
        let root_s = self.root.to_string_lossy().to_string();
        let path_s = self.tab().path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let name = self.tab().name.clone();
        let cursor = self.tab().cursor;
        let revision = self.tab().revision;
        let text = self.tab().text();
        let mut data = String::new();
        data.push_str(&format!("root={}\n", escape_state(&root_s)));
        data.push_str(&format!("path={}\n", escape_state(&path_s)));
        data.push_str(&format!("name={}\n", escape_state(&name)));
        data.push_str(&format!("cursor_line={}\n", cursor.line));
        data.push_str(&format!("cursor_col={}\n", cursor.col));
        data.push_str(&format!("revision={}\n", revision));
        data.push_str("---TEXT---\n");
        data.push_str(&text);
        let _ = atomic_write_file(&file, data.as_bytes());
    }

    fn delete_recovery_for_tab(&self, tab: &Tab) { let _ = fs::remove_file(self.recovery_file_for(tab)); }
    fn delete_recovery_file(&self, path: &Path) {
        let mut temp = Tab::empty();
        temp.path = Some(path.to_path_buf());
        temp.name = path.file_name().and_then(OsStr::to_str).unwrap_or("Untitled").to_string();
        self.delete_recovery_for_tab(&temp);
    }

    fn offer_recovery(&mut self) {
        let Ok(entries) = fs::read_dir(self.recovery_dir()) else { return; };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(OsStr::to_str) != Some("rec") { continue; }
            let Ok(data) = fs::read_to_string(&path) else { continue; };
            let Some((headers, text)) = data.split_once("---TEXT---\n") else { continue; };
            let map = parse_state_headers(headers);
            if map.get("root").map(String::as_str) != Some(self.root.to_string_lossy().as_ref()) { continue; }
            let name = map.get("name").cloned().unwrap_or_else(|| "Untitled".to_string());
            let answer = self.prompt(&format!("Recover unsaved changes for {name}? y/N: "), "");
            if answer.to_ascii_lowercase() != "y" { let _ = fs::remove_file(&path); continue; }
            let path_value = map.get("path").cloned().unwrap_or_default();
            let file_path = if path_value.is_empty() { None } else { Some(PathBuf::from(path_value)) };
            let crlf = text.contains("\r\n");
            let norm = text.replace("\r\n", "\n").replace('\r', "\n");
            let mut lines: Vec<String> = norm.split('\n').map(str::to_string).collect();
            if lines.is_empty() { lines.push(String::new()); }
            let mut tab = Tab::empty();
            tab.path = file_path;
            tab.name = name.clone();
            tab.lines = lines;
            tab.crlf = crlf;
            tab.cursor.line = map.get("cursor_line").and_then(|v| v.parse().ok()).unwrap_or(0);
            tab.cursor.line = min(tab.cursor.line, tab.lines.len().saturating_sub(1));
            tab.cursor.col = map.get("cursor_col").and_then(|v| v.parse().ok()).unwrap_or(0);
            tab.cursor.col = clamp_char_boundary(&tab.lines[tab.cursor.line], min(tab.cursor.col, tab.lines[tab.cursor.line].len()));
            // Differ from the recovered text so the tab stays dirty until saved.
            tab.saved_hash = buffer_hash(&tab.lines).wrapping_add(1);
            tab.modified = true;
            tab.revision = map.get("revision").and_then(|v| v.parse().ok()).unwrap_or(1);
            if self.is_hidden_initial_tab(0, &self.tabs[0]) {
                self.tabs[0] = tab; self.tab_index = 0; self.hide_initial_untitled = false;
            } else { self.tabs.push(tab); self.tab_index = self.tabs.len() - 1; }
            self.message = format!("Recovered {name}");
        }
    }

    fn try_restore_session(&mut self) {
        if self.tabs.len() != 1 || self.tabs[0].path.is_some() { return; }
        let Ok(data) = fs::read_to_string(self.session_file()) else { return; };
        let mut tabs = Vec::new();
        let mut saved_tab_index: usize = 0;
        for line in data.lines() {
            if let Some(rest) = line.strip_prefix("tab=") {
                let parts: Vec<String> = rest.split('\t').map(unescape_state).collect();
                if parts.is_empty() { continue; }
                let path = PathBuf::from(&parts[0]);
                if !path.is_file() { continue; }
                if let Ok(mut tab) = Tab::from_path(path) {
                    tab.cursor.line = parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                    tab.cursor.line = min(tab.cursor.line, tab.lines.len().saturating_sub(1));
                    tab.cursor.col = parts.get(2).and_then(|v| v.parse().ok()).unwrap_or(0);
                    tab.cursor.col = clamp_char_boundary(&tab.lines[tab.cursor.line], min(tab.cursor.col, tab.lines[tab.cursor.line].len()));
                    tab.row_offset = parts.get(3).and_then(|v| v.parse().ok()).unwrap_or(0);
                    tab.syntax_mode = parts.get(4).and_then(|v| SyntaxMode::from_word(v));
                    tabs.push(tab);
                }
            } else if let Some(rest) = line.strip_prefix("expanded=") {
                self.expanded.insert(PathBuf::from(unescape_state(rest)));
            } else if line == "sidebar_hidden=1" {
                self.sidebar_hidden = true;
            } else if let Some(rest) = line.strip_prefix("tab_index=") {
                saved_tab_index = rest.parse().unwrap_or(0);
            }
        }
        if !tabs.is_empty() {
            self.tabs = tabs;
            self.tab_index = min(saved_tab_index, self.tabs.len().saturating_sub(1));
            self.hide_initial_untitled = false;
            self.focus = if self.sidebar_hidden { Focus::Editor } else { Focus::Tree };
            self.message = "Session restored".to_string();
            self.show_welcome = false;
            self.needs_tree_refresh = true;
        }
    }

    fn save_session(&self) {
        let mut out = String::new();
        out.push_str(&format!("root={}\n", escape_state(self.root.to_string_lossy().as_ref())));
        out.push_str(&format!("tab_index={}\n", self.tab_index));
        out.push_str(&format!("sidebar_hidden={}\n", if self.sidebar_hidden { 1 } else { 0 }));
        for e in &self.expanded { out.push_str(&format!("expanded={}\n", escape_state(e.to_string_lossy().as_ref()))); }
        for tab in &self.tabs {
            if let Some(path) = &tab.path {
                out.push_str("tab=");
                out.push_str(&escape_state(path.to_string_lossy().as_ref()));
                out.push('\t'); out.push_str(&tab.cursor.line.to_string());
                out.push('\t'); out.push_str(&tab.cursor.col.to_string());
                out.push('\t'); out.push_str(&tab.row_offset.to_string());
                out.push('\t'); out.push_str(tab.syntax_mode.map(|s| s.label()).unwrap_or(""));
                out.push('\n');
            }
        }
        let _ = atomic_write_file(&self.session_file(), out.as_bytes());
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Segment { pub(crate) start: usize, pub(crate) end: usize, pub(crate) color: &'static str }

fn highlight_segments(line: &str, syntax: SyntaxMode) -> Vec<Segment> {
    plugins::highlight_segments(line, syntax)
}

fn color_at<'a>(segments: &'a [Segment], pos: usize) -> Option<&'static str> {
    for seg in segments.iter().rev() { if pos >= seg.start && pos < seg.end { return Some(seg.color); } }
    None
}

fn current_minute() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() / 60 }

fn time_date_text() -> String {
    if let Ok(out) = Command::new("date").arg("+%I:%M %p  %d/%m/%Y").output() {
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    } else { String::new() }
}

fn absolute_path(path: &Path, base: Option<&Path>) -> PathBuf {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        // Resolve relative paths against explicit base, or CWD if base is
        // relative/missing. This ensures session/recovery keys and tree roots
        // are stable regardless of how az was invoked.
        let b = base.unwrap_or(&cwd);
        let abs_base = if b.is_absolute() {
            b.to_path_buf()
        } else {
            cwd.join(b)
        };
        abs_base.join(path)
    };
    // Lexically normalise `.` and `..` without touching the filesystem
    // (so non-existent new files still resolve).
    let mut out = PathBuf::new();
    for comp in joined.components() {
        use std::path::Component;
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn parse_cli_path(raw: &str) -> (String, Option<usize>) {
    // Returns (path, line). Supports `file:line`, `:line`, and plain paths.
    // Windows drive `C:\...` is not specially handled (Linux-first editor).
    let t = raw.trim();
    if let Some(rest) = t.strip_prefix(':') {
        if let Ok(n) = rest.trim().parse::<usize>() {
            return (".".to_string(), Some(n));
        }
        return (t.to_string(), None);
    }
    if let Some((left, right)) = t.rsplit_once(':') {
        if !left.is_empty() {
            if let Ok(n) = right.trim().parse::<usize>() {
                // Avoid splitting `dir:` with empty line part handled above.
                return (left.to_string(), Some(n));
            }
        }
    }
    (t.to_string(), None)
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string()
}

fn atomic_write_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let tmp = path.with_file_name(format!(".{}.aztmp.{}", path.file_name().and_then(OsStr::to_str).unwrap_or("file"), std::process::id()));
    fs::write(&tmp, bytes)?;
    if let Ok(meta) = fs::metadata(path) { let _ = fs::set_permissions(&tmp, meta.permissions()); }
    if let Err(err) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(err);
    }
    Ok(())
}

fn current_user_is_root() -> bool {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn geteuid() -> u32;
        }
        unsafe { geteuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Write `bytes` to `path` as root. The password is sent on sudo's stdin
/// (`-S`) and is never placed in argv. Content is staged in a temp file so
/// a wrong password cannot swallow the document as a second password attempt.
fn write_file_with_sudo(path: &Path, bytes: &[u8], password: &str) -> Result<(), String> {
    let tmp = env::temp_dir().join(format!("az-save-{}.tmp", std::process::id()));
    if let Err(err) = fs::write(&tmp, bytes) {
        return Err(format!("could not stage save ({err})"));
    }
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let spawn = Command::new("sudo")
        .args([
            "-k",
            "-S",
            "-p",
            "",
            "--",
            "sh",
            "-c",
            "mkdir -p -- \"$1\" && cp -f -- \"$2\" \"$3\"",
            "az-save",
        ])
        .arg(parent)
        .arg(&tmp)
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawn {
        Ok(child) => child,
        Err(err) => {
            let _ = fs::remove_file(&tmp);
            return Err(format!("sudo: {err}"));
        }
    };
    let wrote = child.stdin.as_mut().map(|stdin| {
        stdin.write_all(password.as_bytes()).is_ok() && stdin.write_all(b"\n").is_ok()
    }).unwrap_or(false);
    drop(child.stdin.take());
    let output = child.wait_with_output();
    let _ = fs::remove_file(&tmp);
    if !wrote {
        return Err("could not send password to sudo".to_string());
    }
    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(sudo_failure_message(&out.stderr)),
        Err(err) => Err(err.to_string()),
    }
}

fn sudo_failure_message(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let line = text.lines().map(str::trim).find(|line| {
        let low = line.to_ascii_lowercase();
        !line.is_empty() && !low.contains("password")
    });
    match line {
        Some(line) => line.chars().take(80).collect(),
        None => "wrong password".to_string(),
    }
}

fn ansi_fg(hex: &str) -> String { let (r, g, b) = rgb(hex); format!("\x1b[38;2;{r};{g};{b}m") }
fn ansi_bg(hex: &str) -> String { let (r, g, b) = rgb(hex); format!("\x1b[48;2;{r};{g};{b}m") }

/// Welcome logo: `az` block letters with the ttfx `highlight` final gradient
/// (Tokyo Night stops). Generated offline with ttfx — do not hand-edit.
/// Each row is (styled text, plain visual width); rows are ragged, pad per row.
const TTFX_LOGO_WIDTH: usize = 15;

fn ttfx_logo() -> Vec<(String, usize)> {
    vec![
        ("\x1b[38;2;255;255;255m  ████   ██████".to_string(), 15),
        ("\x1b[38;2;221;243;255m █    █       █".to_string(), 15),
        ("\x1b[38;2;173;225;255m ██████      █".to_string(), 14),
        ("\x1b[38;2;125;207;255m      █     █".to_string(), 13),
        ("\x1b[38;2;122;187;252m █    █    █".to_string(), 12),
        ("\x1b[38;2;122;172;249m  ████   ██████".to_string(), 15),
    ]
}
fn ansi_style(fg: Option<&str>, bg: Option<&str>, bold: bool, dim: bool, underline: bool) -> String {
    let mut s = String::new();
    if let Some(f) = fg { s.push_str(&ansi_fg(f)); }
    if let Some(b) = bg { s.push_str(&ansi_bg(b)); }
    if bold { s.push_str("\x1b[1m"); }
    if dim { s.push_str("\x1b[2m"); }
    if underline { s.push_str("\x1b[4m"); }
    s
}
fn reset_fg_bg() -> &'static str { "\x1b[0m" }
fn rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    if h.len() >= 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(255);
        (r, g, b)
    } else { (255, 255, 255) }
}

fn visual_width(text: &str) -> usize {
    text.chars().map(|c| if c == '\t' { 4 } else if is_wide(c) { 2 } else if c.is_control() { 2 } else { 1 }).sum()
}

fn fit_plain(text: &str, width: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for c in text.chars() {
        let w = if c == '\t' { 4 } else if is_wide(c) { 2 } else if c.is_control() { 2 } else { 1 };
        if used + w > width { break; }
        if c.is_control() && c != '\t' { out.push('^'); out.push(((c as u8) + 64) as char); }
        else if c == '\t' { out.push_str("    "); }
        else { out.push(c); }
        used += w;
    }
    if used < width { out.push_str(&" ".repeat(width - used)); }
    out
}

fn fit_ansi(text: &str, width: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i] != b'm' { i += 1; }
            if i < bytes.len() { i += 1; }
            out.push_str(&text[start..i]);
            continue;
        }
        let ch = next_char(text, i);
        let w = visual_width(ch);
        if used + w > width { break; }
        out.push_str(ch);
        used += w;
        i += ch.len();
    }
    if used < width { out.push_str(&" ".repeat(width - used)); }
    out
}

fn truncate_plain(text: &str, width: usize) -> String {
    let mut s = fit_plain(text, width);
    while s.ends_with(' ') { s.pop(); }
    s
}

fn escape_control(text: &str) -> String {
    text.chars().map(|c| if c.is_control() { ' ' } else { c }).collect()
}

fn word_count(lines: &[String]) -> usize { lines.iter().flat_map(|l| l.split_whitespace()).count() }

fn utf8_sequence_len(b: u8) -> usize {
    if b & 0b1111_1000 == 0b1111_0000 { 4 } else if b & 0b1111_0000 == 0b1110_0000 { 3 } else if b & 0b1110_0000 == 0b1100_0000 { 2 } else { 1 }
}

fn clamp_char_boundary(s: &str, mut idx: usize) -> usize {
    idx = min(idx, s.len());
    while idx > 0 && !s.is_char_boundary(idx) { idx -= 1; }
    idx
}
fn next_char_boundary(s: &str, idx: usize) -> usize { let i = clamp_char_boundary(s, idx); if i >= s.len() { i } else { i + next_char(s, i).len() } }
fn prev_char_boundary(s: &str, idx: usize) -> usize { let mut i = clamp_char_boundary(s, idx); if i == 0 { return 0; } i -= 1; while i > 0 && !s.is_char_boundary(i) { i -= 1; } i }
fn next_char(s: &str, idx: usize) -> &str { let i = clamp_char_boundary(s, idx); let end = next_char_boundary_raw(s, i); &s[i..end] }
fn next_char_boundary_raw(s: &str, idx: usize) -> usize { s[idx..].chars().next().map(|c| idx + c.len_utf8()).unwrap_or(idx) }
fn prev_char(s: &str, idx: usize) -> &str { let start = prev_char_boundary(s, idx); &s[start..idx] }

fn is_wide(c: char) -> bool {
    let cp = c as u32;
    matches!(cp, 0x1100..=0x115F | 0x2329..=0x232A | 0x2E80..=0xA4CF | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF | 0xFE10..=0xFE19 | 0xFE30..=0xFE6F | 0xFF00..=0xFF60 | 0xFFE0..=0xFFE6)
}

fn is_printable(key: &str) -> bool {
    if key == "\t" { return true; }
    if key.is_empty() || key.starts_with('\x1b') { return false; }
    !key.chars().any(|c| c.is_control())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MouseEvent {
    button: u32,
    x: usize,
    y: usize,
    is_release: bool,
}

impl MouseEvent {
    fn is_scroll(&self) -> bool {
        self.button & 64 != 0
    }
    fn scroll_up(&self) -> bool {
        self.is_scroll() && self.button & 1 == 0
    }
}

/// Legacy X10 mouse report (`ESC [ M Cb Cx Cy`, all bytes +32) for terminals
/// without SGR-1006 support. Release is Cb==3, wheel is 64/65, drag is 32+btn.
fn parse_legacy_mouse(key: &str) -> Option<MouseEvent> {
    let b = key.as_bytes();
    if b.len() != 6 || b[0] != 0x1b || b[1] != b'[' || b[2] != b'M' {
        return None;
    }
    let button = b[3].wrapping_sub(32) as u32;
    let col = b[4].wrapping_sub(32) as usize;
    let row = b[5].wrapping_sub(32) as usize;
    if col == 0 || row == 0 {
        return None;
    }
    Some(MouseEvent { button, x: col, y: row, is_release: button == 3 })
}
/// Split one input chunk into individual mouse events.
/// `read_key` may glue several SGR (`ESC [ < Cb ; Cx ; Cy M/m`) or legacy
/// (`ESC [ M Cb Cx Cy`) reports together when the terminal flushes a burst
/// (fast wheel scrolling, touchpad gestures). Parsing the chunk as a single
/// event fails, so the whole burst would be silently dropped.
fn parse_mouse_events(key: &str) -> Vec<MouseEvent> {
    const SGR_PREFIX: &str = "\x1b[<";
    const LEGACY_PREFIX: &str = "\x1b[M";
    let mut out = Vec::new();
    let mut i = 0;
    while i < key.len() {
        let rest = &key[i..];
        if let Some(tail) = rest.strip_prefix(SGR_PREFIX) {
            match tail.find(['M', 'm']) {
                Some(end) => {
                    if let Some(ev) = parse_sgr_mouse(&rest[..SGR_PREFIX.len() + end + 1]) {
                        out.push(ev);
                    }
                    i += SGR_PREFIX.len() + end + 1;
                }
                None => break, // trailing partial report; ignore
            }
        } else if rest.starts_with(LEGACY_PREFIX) {
            if rest.len() < 6 {
                break; // trailing partial report; ignore
            }
            if let Some(ev) = parse_legacy_mouse(&rest[..6]) {
                out.push(ev);
            }
            i += 6;
        } else {
            // Stray bytes between reports; skip one char and keep scanning.
            i += rest.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        }
    }
    out
}
/// `M` = press/drag/scroll, `m` = release. Returns 1-indexed `x`/`y`.
/// Parse SGR mouse sequences: `ESC [ < Cb ; Cx ; Cy M/m`.
fn parse_sgr_mouse(key: &str) -> Option<MouseEvent> {
    let body = key.strip_prefix("\x1b[<")?;
    let is_release = body.ends_with('m');
    if !body.ends_with('M') && !is_release {
        return None;
    }
    let inner = &body[..body.len().saturating_sub(1)];
    let mut parts = inner.split(';');
    let button: u32 = parts.next()?.parse().ok()?;
    let x: usize = parts.next()?.parse().ok()?;
    let y: usize = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if x == 0 || y == 0 {
        return None;
    }
    Some(MouseEvent { button, x, y, is_release })
}

fn is_ctrl_k(k: &str) -> bool { k == "\x0b" || k == "\x1b[75;5u" || k == "\x1b[107;5u" }
fn is_ctrl_shift_o(k: &str) -> bool { k == "\x1b[79;6u" || k == "\x1b[111;6u" }
fn is_ctrl_shift_h(k: &str) -> bool { k == "\x1b[72;6u" || k == "\x1b[104;6u" }
/// Shared `%` convention: `%Foo` = case-sensitive, otherwise case-insensitive.
/// Used by both in-file find and Find in Files so behaviour stays in sync.
fn parse_search_query(query: &str) -> (String, bool) {
    if let Some(rest) = query.strip_prefix('%') {
        (rest.to_string(), false)
    } else {
        (query.to_string(), true)
    }
}
fn project_line_matches(line: &str, needle: &str, ignore_case: bool) -> bool {
    if needle.is_empty() {
        return false;
    }
    if ignore_case {
        line.to_ascii_lowercase().contains(&needle.to_ascii_lowercase())
    } else {
        line.contains(needle)
    }
}

/// Count non-overlapping occurrences of `needle` in one line.
/// Byte-safe: ASCII case-folding never changes byte length or boundaries.
fn count_matches_in_line(line: &str, needle: &str, ignore_case: bool) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0usize;
    let mut from = 0usize;
    if ignore_case {
        let lower = line.to_ascii_lowercase();
        let nl = needle.to_ascii_lowercase();
        while let Some(p) = lower[from..].find(&nl).map(|p| p + from) {
            count += 1;
            from = p + needle.len();
        }
    } else {
        while let Some(p) = line[from..].find(needle).map(|p| p + from) {
            count += 1;
            from = p + needle.len();
        }
    }
    count
}

/// Replace all non-overlapping occurrences of `needle` in one line,
/// returning the new line plus the replacement count.
fn replace_in_line(line: &str, needle: &str, replacement: &str, ignore_case: bool) -> (String, usize) {
    if needle.is_empty() {
        return (line.to_string(), 0);
    }
    let mut out = String::with_capacity(line.len());
    let mut count = 0usize;
    let mut last = 0usize;
    let mut from = 0usize;
    if ignore_case {
        let lower = line.to_ascii_lowercase();
        let nl = needle.to_ascii_lowercase();
        while let Some(p) = lower[from..].find(&nl).map(|p| p + from) {
            out.push_str(&line[last..p]);
            out.push_str(replacement);
            count += 1;
            last = p + needle.len();
            from = last;
        }
    } else {
        while let Some(p) = line[from..].find(needle).map(|p| p + from) {
            out.push_str(&line[last..p]);
            out.push_str(replacement);
            count += 1;
            last = p + needle.len();
            from = last;
        }
    }
    out.push_str(&line[last..]);
    (out, count)
}
fn is_ctrl_shift_z(k: &str) -> bool { k == "\x1b[90;6u" || k == "\x1b[122;6u" }
fn is_ctrl_backspace(k: &str) -> bool { k == "\x17" || k == "\x1b[127;5u" || k == "\x1b[8;5u" }
fn is_ctrl_left(k: &str) -> bool { matches!(k, "\x1b[1;5D" | "\x1b[5D" | "\x1bO5D" | "\x1bOd" | "\x1b[1;3D" | "\x1b[3D") }
fn is_ctrl_right(k: &str) -> bool { matches!(k, "\x1b[1;5C" | "\x1b[5C" | "\x1bO5C" | "\x1bOc" | "\x1b[1;3C" | "\x1b[3C") }
fn is_ctrl_shift_left(k: &str) -> bool { k == "\x1b[1;6D" || k == "\x1b[68;6u" }
fn is_ctrl_shift_right(k: &str) -> bool { k == "\x1b[1;6C" || k == "\x1b[67;6u" }
fn is_ctrl_home(k: &str) -> bool { matches!(k, "\x1b[1;5H" | "\x1b[7;5~" | "\x1bO5H") }
fn is_ctrl_end(k: &str) -> bool { matches!(k, "\x1b[1;5F" | "\x1b[8;5~" | "\x1bO5F") }
fn is_ctrl_shift_home(k: &str) -> bool { k == "\x1b[1;6H" || k == "\x1b[7;6~" }
fn is_ctrl_shift_end(k: &str) -> bool { k == "\x1b[1;6F" || k == "\x1b[8;6~" }
fn is_alt_up(k: &str) -> bool { matches!(k, "\x1b[1;3A" | "\x1b\x1b[A") }
fn is_alt_down(k: &str) -> bool { matches!(k, "\x1b[1;3B" | "\x1b\x1b[B") }
fn is_alt_shift_up(k: &str) -> bool { matches!(k, "\x1b[1;4A" | "\x1b\x1b[1;2A") }
fn is_alt_shift_down(k: &str) -> bool { matches!(k, "\x1b[1;4B" | "\x1b\x1b[1;2B") }
fn is_ctrl_shift_e(k: &str) -> bool { k == "\x1b[69;6u" || k == "\x1b[101;6u" }
fn tab_number(k: &str) -> Option<usize> {
    if k.len() == 2 && k.as_bytes()[0] == 0x1b && (b'1'..=b'9').contains(&k.as_bytes()[1]) { return Some((k.as_bytes()[1] - b'0') as usize); }
    None
}

/// Map a click column to a byte index in `line`, starting the walk at
/// `start_byte` (the current `col_offset`). `visual_target` is the 0-based
/// cell offset from the first visible cell. Tab/wide/control chars use the
/// same widths as rendering (`display_cell` + `visual_width`).
fn editor_click_col(line: &str, start_byte: usize, visual_target: usize) -> usize {
    let mut byte_i = clamp_char_boundary(line, min(start_byte, line.len()));
    let mut used = 0usize;
    while byte_i < line.len() {
        let ch = next_char(line, byte_i);
        let w = visual_width(&display_cell(ch));
        if used + w > visual_target {
            break;
        }
        used += w;
        byte_i += ch.len();
    }
    byte_i
}

/// Byte range of the word at `col` (`is_word_char` run), if any.
/// Only the end of line falls back to the word before the caret.
fn word_range_at(line: &str, col: usize) -> Option<(usize, usize)> {
    let col = clamp_char_boundary(line, min(col, line.len()));
    let mut start = if col < line.len() && is_word_char(next_char(line, col)) {
        col
    } else if col == line.len() && col > 0 && is_word_char(prev_char(line, col)) {
        prev_char_boundary(line, col)
    } else {
        return None;
    };
    while start > 0 && is_word_char(prev_char(line, start)) {
        start = prev_char_boundary(line, start);
    }
    let mut end = start;
    while end < line.len() && is_word_char(next_char(line, end)) {
        end = next_char_boundary(line, end);
    }
    if start < end { Some((start, end)) } else { None }
}

/// All shortcuts shown in the Ctrl+K dialog: (keys, action).
fn shortcut_defs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Ctrl+S", "Save"),
        ("Ctrl+O", "Quick open file / symbol / file:line / :line"),
        ("Ctrl+P", "Command palette"),
        ("Ctrl+K", "This shortcuts dialog (searchable)"),
        ("Ctrl+F", "Search & replace dialog (%term = case-sensitive)"),
        ("Ctrl+L", "Find next (uses last pattern)"),
        ("Ctrl+Shift+O", "Find in files (separate modal)"),
        ("Ctrl+R", "Search & replace dialog, replace field"),
        ("Ctrl+Shift+H", "Search & replace in the project"),
        ("Ctrl+G", "Go to line"),
        ("Ctrl+E", "Go to end of line"),
        ("Home / End", "Go to start / end of line"),
        ("Ctrl+Home / Alt+Up", "Go to start of file (Shift = select)"),
        ("Ctrl+End / Alt+Down", "Go to end of file (Shift = select)"),
        ("Ctrl+T", "Focus tree/editor"),
        ("Ctrl+H", "Hide/show tree (tree focus only)"),
        ("+ / -", "Tree width (tree focus only)"),
        ("Ctrl+N", "New empty tab"),
        ("+ on the tab bar", "New empty tab"),
        ("Ctrl+D", "Close tab (asks if modified)"),
        ("Ctrl+Q", "Quit (asks if any tab modified)"),
        ("Ctrl+Z / Ctrl+Y", "Undo / redo"),
        ("Ctrl+C / Ctrl+X", "Copy / cut selection or line"),
        ("Ctrl+V", "Paste"),
        ("Ctrl+A", "Select all"),
        ("Ctrl+W", "Delete current line"),
        ("Alt+1-9", "Switch a visible tab"),
        ("Ctrl+Tab / Ctrl+Shift+Tab", "Next / previous tab"),
        ("Tab", "Accept autocomplete"),
        ("Click", "Move cursor / open file / expand folder / switch tab"),
        ("Drag", "Select text"),
        ("Double-click", "Select word (editor) / rename (sidebar file)"),
        ("Triple-click", "Select line"),
        ("Right-click", "Context menu (tab / sidebar / editor)"),
        ("Middle-click tab", "Close tab"),
        ("Wheel", "Pan editor, move sidebar, cycle tabs"),
        ("Click outside a dialog", "Close it"),
    ]
}

/// Map a 1-based topbar click column to a position in `tab_widths`.
/// `prefix_w` is the plain-cell width of the bar prefix (title area).
fn tab_hit_index(prefix_w: usize, tab_widths: &[usize], click_col: usize) -> Option<usize> {
    if click_col == 0 || click_col <= prefix_w {
        return None;
    }
    let mut x = prefix_w + 1;
    for (i, w) in tab_widths.iter().enumerate() {
        if click_col >= x && click_col < x + w {
            return Some(i);
        }
        x += w;
    }
    None
}

/// Clamp a context menu into the visible area (1-based terminal cells).
/// Returns (start_col, start_row, width, height); the status line stays clear.
fn context_menu_geometry(item_count: usize, max_item_w: usize, col: usize, row: usize, cols: usize, rows: usize) -> (usize, usize, usize, usize) {
    let width = min(max_item_w + 6, cols.saturating_sub(2)).max(12);
    let height = min(item_count + 2, rows.saturating_sub(1)).max(3);
    let start_col = min(col, (cols + 1).saturating_sub(width)).max(1);
    let start_row = min(row, rows.saturating_sub(height)).max(1);
    (start_col, start_row, width, height)
}

const PASTE_START: &[u8] = b"\x1b[200~";
const PASTE_END: &[u8] = b"\x1b[201~";

/// Split a buffer that begins with a bracketed-paste start marker.
/// `None` when the end marker is not in the buffer yet.
fn take_bracketed_paste(bytes: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    if !bytes.starts_with(PASTE_START) {
        return None;
    }
    let body_and_end = &bytes[PASTE_START.len()..];
    let pos = find_bytes(body_and_end, PASTE_END)?;
    let body = body_and_end[..pos].to_vec();
    let rest = body_and_end[pos + PASTE_END.len()..].to_vec();
    Some((body, rest))
}

fn is_fast_paste_byte(byte: u8) -> bool {
    byte == b'\t' || byte == b'\n' || byte == b'\r' || (byte >= 0x20 && byte != 0x7f)
}

fn pending_is_paste_burst(pending: &VecDeque<u8>) -> bool {
    if pending.len() >= 16 {
        return true;
    }
    // A typed character followed by Enter must stay a real newline (auto-indent).
    let newlines = pending.iter().filter(|byte| **byte == b'\n' || **byte == b'\r').count();
    newlines >= 1 && pending.len() >= 8
}

fn buffer_hash(lines: &[String]) -> u64 {
    let mut hasher = DefaultHasher::new();
    lines.len().hash(&mut hasher);
    for line in lines {
        line.hash(&mut hasher);
    }
    hasher.finish()
}

fn chip_row_width(chips: &[(String, &str, &str, bool)]) -> usize {
    let mut width = 0usize;
    for (i, (text, _, _, _)) in chips.iter().enumerate() {
        if i > 0 {
            width += 1;
        }
        width += visual_width(text);
    }
    width
}

fn window_indexes(len: usize, current: usize, max_visible: usize) -> std::ops::Range<usize> {
    if max_visible == 0 || len <= max_visible {
        return 0..len;
    }
    let current = current.min(len - 1);
    let start = current.saturating_sub(max_visible / 2).min(len - max_visible);
    start..start + max_visible
}

fn visual_at_byte(line: &str, byte_col: usize) -> usize {
    let byte_col = clamp_char_boundary(line, min(byte_col, line.len()));
    visual_width(&line[..byte_col])
}

fn byte_at_visual(line: &str, visual_col: usize) -> usize {
    let mut byte_i = 0usize;
    let mut used = 0usize;
    while byte_i < line.len() {
        if used >= visual_col {
            break;
        }
        let ch = next_char(line, byte_i);
        let w = visual_width(&display_cell(ch));
        if used + w > visual_col {
            break;
        }
        used += w;
        byte_i += ch.len();
    }
    byte_i
}

/// Visual column where the viewport should start so `cursor_byte` is on screen.
fn fit_visual_offset(line: &str, cursor_byte: usize, width: usize) -> usize {
    let cursor_vis = visual_at_byte(line, cursor_byte);
    if width == 0 {
        return cursor_vis;
    }
    let mut offset = cursor_vis.saturating_sub(width - 1);
    loop {
        let start_byte = byte_at_visual(line, offset);
        let start_vis = visual_at_byte(line, start_byte);
        if cursor_vis < start_vis + width || start_byte >= cursor_byte {
            return start_vis;
        }
        let next = next_char_boundary(line, start_byte);
        if next <= start_byte {
            return start_vis;
        }
        offset = visual_at_byte(line, next);
    }
}

fn shift_visual(text: &str, cols: usize) -> String {
    if cols == 0 {
        return text.to_string();
    }
    let mut used = 0usize;
    for (i, ch) in text.char_indices() {
        if used >= cols {
            return text[i..].to_string();
        }
        let w = visual_width(&text[i..i + ch.len_utf8()]);
        if used + w > cols {
            return text[i + ch.len_utf8()..].to_string();
        }
        used += w;
    }
    String::new()
}

fn line_as_clipboard(lines: &[String], line: usize) -> String {
    let text = lines.get(line).cloned().unwrap_or_default();
    if line + 1 < lines.len() {
        format!("{text}\n")
    } else {
        text
    }
}

fn count_in_lines(lines: &[String], needle: &str, ignore_case: bool) -> usize {
    lines.iter().map(|line| count_matches_in_line(line, needle, ignore_case)).sum()
}

fn apply_line_edit(value: &mut String, cursor: &mut usize, key: &str) -> bool {
    let mut chars: Vec<char> = value.chars().collect();
    let cur = (*cursor).min(chars.len());
    match key {
        "\x7f" | "\x08" => {
            if cur == 0 {
                return true;
            }
            chars.remove(cur - 1);
            *cursor = cur - 1;
        }
        "\x15" => {
            chars.clear();
            *cursor = 0;
        }
        "\x1b[D" => *cursor = cur.saturating_sub(1),
        "\x1b[C" => *cursor = min(chars.len(), cur + 1),
        "\x1b[H" | "\x1bOH" | "\x1b[1~" => *cursor = 0,
        "\x1b[F" | "\x1bOF" | "\x1b[4~" => *cursor = chars.len(),
        _ => {
            if let Some(pasted) = key.strip_prefix("\0AZPASTE:") {
                let extra: Vec<char> = pasted.replace(['\r', '\n'], " ").chars().collect();
                let n = extra.len();
                chars.splice(cur..cur, extra);
                *cursor = cur + n;
            } else if is_printable(key) {
                let extra: Vec<char> = key.chars().collect();
                let n = extra.len();
                chars.splice(cur..cur, extra);
                *cursor = cur + n;
            } else {
                return false;
            }
        }
    }
    *value = chars.into_iter().collect();
    true
}

fn field_window(value: &str, cursor: usize, width: usize) -> (String, usize) {
    let chars: Vec<char> = value.chars().collect();
    let cursor = cursor.min(chars.len());
    if width == 0 {
        return (String::new(), 0);
    }
    if chars.len() <= width {
        return (chars.into_iter().collect(), cursor);
    }
    let start = cursor.saturating_sub(width / 2).min(chars.len().saturating_sub(width));
    let end = min(chars.len(), start + width);
    let shown: String = chars[start..end].iter().collect();
    (shown, cursor - start)
}

fn scrollbar_thumb(track: usize, view: usize, content: usize, offset: usize) -> (usize, usize) {
    if track == 0 {
        return (0, 0);
    }
    if content <= view || view == 0 {
        return (0, track);
    }
    let thumb = max(1, track * view / content).min(track);
    let max_off = content - view;
    let travel = track - thumb;
    let start = if max_off == 0 || travel == 0 { 0 } else { travel * offset.min(max_off) / max_off };
    (start.min(travel), thumb)
}

fn scrollbar_offset(track: usize, view: usize, content: usize, click: usize) -> usize {
    if content <= view || track == 0 {
        return 0;
    }
    let thumb = max(1, track * view / content).min(track);
    let travel = track.saturating_sub(thumb);
    let max_off = content - view;
    if travel == 0 {
        return 0;
    }
    let pos = click.saturating_sub(thumb / 2).min(travel);
    max_off * pos / travel
}

struct PickerFrame {
    start_col: usize,
    start_row: usize,
    width: usize,
    height: usize,
    list_rows: usize,
}

fn picker_frame(cols: usize, rows: usize) -> PickerFrame {
    let list_rows = min(12, max(1, rows.saturating_sub(8)));
    let desired = max(50, cols * 62 / 100);
    let max_allowed = cols.saturating_sub(2).max(20);
    let min_allowed = max_allowed.min(30);
    let width = min(max_allowed, desired).max(min_allowed).max(20);
    let height = list_rows + 4;
    let start_col = max(1, (cols.saturating_sub(width)) / 2 + 1);
    let start_row = max(2, (rows.saturating_sub(height)) / 2 + 1);
    PickerFrame { start_col, start_row, width, height, list_rows }
}

enum PickerMouse {
    NotMouse,
    Handled(usize),
    Cancel,
    Activate(usize),
}

enum ReplaceTarget {
    Buffer,
    File(PathBuf),
    Dir(PathBuf),
}

struct ReplaceCount {
    key: String,
    files: Vec<PathBuf>,
    index: usize,
    count: usize,
    done: bool,
    capped: bool,
}

impl ReplaceCount {
    fn idle() -> Self {
        Self { key: String::new(), files: Vec::new(), index: 0, count: 0, done: false, capped: false }
    }
    fn invalidate(&mut self) {
        self.key.clear();
        self.done = false;
    }
}

struct ReplaceGeom {
    col: usize,
    row: usize,
    width: usize,
    height: usize,
    replace_btn: (usize, usize, usize),
    cancel_btn: (usize, usize, usize),
    field_rows: [usize; 3],
}

impl ReplaceGeom {
    fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.col && x < self.col + self.width && y >= self.row && y < self.row + self.height
    }
    fn hits(&self, x: usize, y: usize, btn: (usize, usize, usize)) -> bool {
        y == btn.1 && x >= btn.0 && x < btn.0 + btn.2
    }
    fn field_at(&self, y: usize) -> Option<usize> {
        self.field_rows.iter().position(|row| *row == y)
    }
}

fn is_ctrl_tab(k: &str) -> bool {
    matches!(k, "\x1b[1;5I" | "\x1b[27;5;9~" | "\x1b[9;5u")
}
fn is_ctrl_shift_tab(k: &str) -> bool {
    matches!(k, "\x1b[1;6I" | "\x1b[27;6;9~" | "\x1b[9;6u")
}

fn escape_sequence_done(bytes: &[u8]) -> bool {
    if bytes.starts_with(b"\x1b[M") && bytes.len() >= 6 {
        return true;
    }
    if bytes.len() < 2 {
        return false;
    }
    matches!(bytes[bytes.len() - 1], b'~' | b'u' | b'A' | b'B' | b'C' | b'D' | b'H' | b'F' | b'M' | b'm')
}

#[cfg(unix)]
fn stdin_pending() -> bool {
    #[repr(C)]
    struct PollFd {
        fd: c_int,
        events: i16,
        revents: i16,
    }
    unsafe extern "C" {
        fn poll(fds: *mut PollFd, nfds: c_ulong, timeout: c_int) -> c_int;
    }
    let mut fd = PollFd { fd: io::stdin().as_raw_fd(), events: 1, revents: 0 };
    unsafe { poll(&mut fd, 1, 0) > 0 }
}

#[cfg(not(unix))]
fn stdin_pending() -> bool { false }

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|window| window == needle)
}
fn remove_last_char(s: &mut String) { if let Some((idx, _)) = s.char_indices().last() { s.truncate(idx); } }

fn pos_gt(a: Pos, b: Pos) -> bool { a.line > b.line || (a.line == b.line && a.col > b.col) }
fn range_overlaps_selection(line: usize, start_col: usize, end_col: usize, a: Pos, b: Pos) -> bool {
    let start = Pos { line, col: start_col };
    let end = Pos { line, col: end_col };
    pos_gt(end, a) && pos_gt(b, start)
}

fn display_cell(ch: &str) -> String {
    if ch == "\t" { return "    ".to_string(); }
    let mut chars = ch.chars();
    if let Some(c) = chars.next() {
        if c.is_control() {
            let code = c as u32;
            if code < 128 { return format!("^{}", ((code as u8) + 64) as char); }
            return " ".to_string();
        }
    }
    ch.to_string()
}

fn text_between(lines: &[String], start: Pos, end: Pos) -> String {
    if start.line == end.line { return lines[start.line][start.col..end.col].to_string(); }
    let mut out = String::new();
    out.push_str(&lines[start.line][start.col..]);
    out.push('\n');
    for line in start.line + 1..end.line { out.push_str(&lines[line]); out.push('\n'); }
    out.push_str(&lines[end.line][..end.col]);
    out
}

fn end_pos_for_text(start: Pos, text: &str) -> Pos {
    let parts: Vec<&str> = text.split('\n').collect();
    if parts.len() == 1 { Pos { line: start.line, col: start.col + text.len() } } else { Pos { line: start.line + parts.len() - 1, col: parts.last().unwrap().len() } }
}

fn indent_for_newline(before: &str, after: &str) -> String {
    let mut indent: String = before.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
    let trimmed = before.trim_end();
    if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') || trimmed.ends_with(':') { indent.push_str("    "); }
    let after_trim = after.trim_start();
    if (after_trim.starts_with('}') || after_trim.starts_with(']') || after_trim.starts_with(')')) && indent.len() >= 4 { indent.truncate(indent.len() - 4); }
    indent
}

fn find_in_line(line: &str, needle: &str, offset: usize, ignore_case: bool) -> Option<usize> {
    let offset = clamp_char_boundary(line, min(offset, line.len()));
    if ignore_case {
        let hay = line[offset..].to_ascii_lowercase();
        let n = needle.to_ascii_lowercase();
        hay.find(&n).map(|p| offset + p)
    } else { line[offset..].find(needle).map(|p| offset + p) }
}

fn parse_quick_open_query(query: &str) -> (bool, String, Option<usize>) {
    let q = query.trim();
    if let Some(rest) = q.strip_prefix(':') {
        return (true, String::new(), rest.parse().ok());
    }
    if let Some((left, right)) = q.rsplit_once(':') {
        if let Ok(n) = right.parse::<usize>() { return (false, left.to_string(), Some(n)); }
    }
    (false, q.to_string(), None)
}

fn quick_score(label: &str, query: &str) -> Option<i32> {
    let label_l = label.to_ascii_lowercase();
    let query_l = query.to_ascii_lowercase();
    if query_l.is_empty() { return Some(0); }
    if let Some(pos) = label_l.find(&query_l) { return Some(pos as i32); }
    let mut score = 0i32;
    let mut last = 0usize;
    for ch in query_l.chars() {
        if let Some(pos) = label_l[last..].find(ch) {
            score += pos as i32 + 2;
            last += pos + ch.len_utf8();
        } else { return None; }
    }
    Some(score + 50)
}

/// Byte ranges in `text` to render bold for `query`.
/// Mirrors `quick_score()`: contiguous substring wins, else per-char fuzzy.
fn match_bold_ranges(text: &str, query: &str) -> Vec<(usize, usize)> {
    let q = query.trim();
    if q.is_empty() || text.is_empty() {
        return Vec::new();
    }
    let text_l = text.to_ascii_lowercase();
    let q_l = q.to_ascii_lowercase();
    if let Some(byte_pos) = text_l.find(&q_l) {
        // Map back to original byte range (ASCII-lowercase keeps byte len).
        let end = (byte_pos + q.len()).min(text.len());
        let end = clamp_char_boundary(text, end);
        return vec![(byte_pos, end)];
    }
    // Fuzzy: bold each query char at its first in-order occurrence.
    let mut out = Vec::new();
    let mut search_from = 0usize;
    for ch in q_l.chars() {
        if search_from >= text_l.len() {
            break;
        }
        if let Some(rel) = text_l[search_from..].find(ch) {
            let abs = search_from + rel;
            let end = (abs + ch.len_utf8()).min(text.len());
            out.push((abs, end));
            search_from = end;
        } else {
            break;
        }
    }
    out
}

/// Wrap byte ranges in bold on/off. Caller must size with `fit_ansi()`.
fn apply_bold_ansi(text: &str, ranges: &[(usize, usize)]) -> String {
    if ranges.is_empty() {
        return text.to_string();
    }
    let mut out = String::new();
    let mut cursor = 0usize;
    for (a, b) in ranges {
        let a = (*a).min(text.len());
        let b = (*b).min(text.len()).max(a);
        if a > cursor {
            out.push_str(&text[cursor..a]);
        }
        out.push_str("\x1b[1m");
        out.push_str(&text[a..b]);
        out.push_str("\x1b[22m");
        cursor = b;
    }
    if cursor < text.len() {
        out.push_str(&text[cursor..]);
    }
    out
}

fn extract_symbols(text: &str, syntax: SyntaxMode) -> Vec<(String, usize)> {
    plugins::extract_symbols(text, syntax)
}

fn is_word_char(s: &str) -> bool { s.chars().next().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) }
fn is_space_char(s: &str) -> bool { s.chars().next().map(|c| c.is_whitespace()).unwrap_or(false) }

fn stty_command() -> Command {
    let mut cmd = Command::new("stty");
    if let Ok(tty) = File::open("/dev/tty") {
        cmd.stdin(tty);
    }
    cmd
}

fn stty_output<const N: usize>(args: [&str; N]) -> io::Result<Vec<u8>> {
    let out = stty_command().args(args).output()?;
    if out.status.success() { Ok(out.stdout) } else { Err(io::Error::new(io::ErrorKind::Other, "stty failed")) }
}

fn stty_status<const N: usize>(args: [&str; N]) -> io::Result<()> {
    let status = stty_command().args(args).status()?;
    if status.success() { Ok(()) } else { Err(io::Error::new(io::ErrorKind::Other, "stty failed")) }
}

#[cfg(unix)]
#[repr(C)]
struct WinSize {
    ws_row: c_ushort,
    ws_col: c_ushort,
    ws_xpixel: c_ushort,
    ws_ypixel: c_ushort,
}

#[cfg(unix)]
unsafe extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

#[cfg(unix)]
fn terminal_size_from_ioctl() -> Option<(usize, usize)> {
    const TIOCGWINSZ: c_ulong = 0x5413;
    let tty = File::open("/dev/tty").ok();
    let fd = tty.as_ref().map(|f| f.as_raw_fd()).unwrap_or(0);
    let mut size = WinSize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
    let ok = unsafe { ioctl(fd, TIOCGWINSZ, &mut size) } == 0;
    if ok && size.ws_row > 0 && size.ws_col > 0 {
        Some((size.ws_row as usize, size.ws_col as usize))
    } else {
        None
    }
}

#[cfg(not(unix))]
fn terminal_size_from_ioctl() -> Option<(usize, usize)> { None }

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0b11) << 4) | (b1 >> 4)) as usize] as char);
        if i + 1 < data.len() { out.push(TABLE[(((b1 & 0b1111) << 2) | (b2 >> 6)) as usize] as char); } else { out.push('='); }
        if i + 2 < data.len() { out.push(TABLE[(b2 & 0b11_1111) as usize] as char); } else { out.push('='); }
        i += 3;
    }
    out
}

/// Update check: compare our version against `Cargo.toml` on GitHub main.
/// Runs once at launch (short curl timeout, silent on any failure). Set
/// `AZ_NO_UPDATE_CHECK=1` to skip.
const UPDATE_CHECK_URL: &str = "https://raw.githubusercontent.com/arazgholami/az/refs/heads/main/Cargo.toml";

/// Reads the `[package] version` from Cargo.toml text.
fn parse_remote_version(text: &str) -> Option<String> {
    let mut in_package = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_package = t == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some(rest) = t.strip_prefix("version") else { continue; };
        let Some(rest) = rest.trim_start().strip_prefix('=') else { continue; };
        let Some(quoted) = rest.trim_start().strip_prefix('"') else { continue; };
        let Some(end) = quoted.find('"') else { continue; };
        return Some(quoted[..end].to_string());
    }
    None
}

fn version_part_num(s: &str) -> u64 {
    s.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0)
}

/// True when `remote` is a strictly newer dotted version than `local`.
fn is_newer_version(remote: &str, local: &str) -> bool {
    let r: Vec<&str> = remote.trim().split('.').collect();
    let l: Vec<&str> = local.trim().split('.').collect();
    for i in 0..max(r.len(), l.len()) {
        let a = r.get(i).map(|s| version_part_num(s)).unwrap_or(0);
        let b = l.get(i).map(|s| version_part_num(s)).unwrap_or(0);
        if a != b {
            return a > b;
        }
    }
    false
}

fn fetch_remote_version() -> Option<String> {
    if env::var("AZ_NO_UPDATE_CHECK").is_ok() {
        return None;
    }
    let out = Command::new("curl")
        .args(["-fsSL", "--max-time", "5", UPDATE_CHECK_URL])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_remote_version(&String::from_utf8_lossy(&out.stdout))
}

/// Returns the newer remote version, if any. Never blocks long, never errors.
fn check_for_updates() -> Option<String> {
    let remote = fetch_remote_version()?;
    if is_newer_version(&remote, env!("CARGO_PKG_VERSION")) {
        Some(remote)
    } else {
        None
    }
}

fn osc52_sequence(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", base64_encode(text.as_bytes()))
}

/// Pipe `text` to a clipboard tool's stdin. True only on exit success.
fn pipe_to_clipboard_tool(prog: &str, args: &[&str], text: &str) -> bool {
    let mut child = match Command::new(prog)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    let wrote = child
        .stdin
        .as_mut()
        .map(|s| s.write_all(text.as_bytes()).is_ok())
        .unwrap_or(false);
    // Close stdin so the tool sees EOF before we wait: clipboard tools
    // (wl-copy/xclip/xsel/pbcopy) read stdin to end before exiting, so
    // waiting with the pipe still open deadlocks the editor.
    drop(child.stdin.take());
    child.wait().map(|s| s.success()).unwrap_or(false) && wrote
}

/// Best-effort verified clipboard write via external tools:
/// Wayland (`wl-copy`), X11 (`xclip`/`xsel`), macOS (`pbcopy`).
fn try_clipboard_tool(text: &str) -> bool {
    if env::var("WAYLAND_DISPLAY").is_ok() && pipe_to_clipboard_tool("wl-copy", &[], text) {
        return true;
    }
    if env::var("DISPLAY").is_ok() {
        if pipe_to_clipboard_tool("xclip", &["-selection", "clipboard"], text) {
            return true;
        }
        if pipe_to_clipboard_tool("xsel", &["--clipboard", "--input"], text) {
            return true;
        }
    }
    pipe_to_clipboard_tool("pbcopy", &[], text)
}

/// Read the OS clipboard. `Some("")` means a tool answered and it was empty.
/// `None` means no tool could be used; the caller tries the Wayland protocol.
fn read_clipboard_command(prog: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(prog)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn read_system_clipboard() -> Option<String> {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        if let Some(text) = read_clipboard_command("wl-paste", &["-n"]) {
            return Some(text);
        }
    }
    if env::var("DISPLAY").is_ok() {
        if let Some(text) = read_clipboard_command("xclip", &["-selection", "clipboard", "-o"]) {
            return Some(text);
        }
        if let Some(text) = read_clipboard_command("xsel", &["--clipboard", "--output"]) {
            return Some(text);
        }
    }
    if let Some(text) = read_clipboard_command("pbpaste", &[]) {
        return Some(text);
    }
    wayland_clip::paste()
}

fn simple_hash(data: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in data { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); }
    format!("{h:016x}")
}

fn escape_state(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '=' => out.push_str("\\e"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape_state(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('e') => out.push('='),
                Some('\\') => out.push('\\'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else { out.push(c); }
    }
    out
}

fn parse_state_headers(headers: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in headers.lines() {
        if let Some((k, v)) = line.split_once('=') { map.insert(k.to_string(), unescape_state(v)); }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_path_file_line() {
        assert_eq!(parse_cli_path("main.rs:20"), ("main.rs".to_string(), Some(20)));
        assert_eq!(parse_cli_path("a/b.php:1"), ("a/b.php".to_string(), Some(1)));
    }

    #[test]
    fn cli_path_bare_line() {
        assert_eq!(parse_cli_path(":20"), (".".to_string(), Some(20)));
    }

    #[test]
    fn cli_path_plain() {
        assert_eq!(parse_cli_path("src/main.rs"), ("src/main.rs".to_string(), None));
        assert_eq!(parse_cli_path("."), (".".to_string(), None));
    }

    #[test]
    fn absolute_is_absolute() {
        let p = absolute_path(Path::new("src/main.rs"), None);
        assert!(p.is_absolute(), "expected absolute, got {p:?}");
    }

    #[test]
    fn absolute_normalises_dotdot() {
        let cwd = env::current_dir().unwrap();
        let p = absolute_path(Path::new("a/../b"), Some(&cwd));
        assert!(!p.to_string_lossy().contains(".."));
        assert!(p.to_string_lossy().ends_with("b"));
    }

    #[test]
    fn quick_open_query_parsing() {
        assert_eq!(
            parse_quick_open_query(":20"),
            (true, String::new(), Some(20))
        );
        let (bare, q, line) = parse_quick_open_query("main.rs:20");
        assert!(!bare);
        assert_eq!(q, "main.rs");
        assert_eq!(line, Some(20));
    }

    #[test]
    fn html_auto_close_recovers_tag_after_gt() {
        assert_eq!(
            crate::plugins::html::last_unclosed_tag("<div>"),
            Some("div".to_string())
        );
        assert_eq!(crate::plugins::html::last_unclosed_tag("<br/>"), None);
        assert_eq!(
            crate::plugins::html::last_unclosed_tag("</div>"),
            None
        );
    }

    #[test]
    fn find_case_insensitive() {
        assert_eq!(find_in_line("Hello World", "world", 0, true), Some(6));
        assert_eq!(find_in_line("Hello World", "world", 0, false), None);
    }

    #[test]
    fn search_query_percent_is_case_sensitive() {
        assert_eq!(parse_search_query("%Foo"), ("Foo".to_string(), false));
        assert_eq!(parse_search_query("Foo"), ("Foo".to_string(), true));
        assert_eq!(parse_search_query("%"), (String::new(), false));
        assert_eq!(parse_search_query(""), (String::new(), true));
    }

    #[test]
    fn project_line_matches_respects_case() {
        assert!(project_line_matches("Hello World", "world", true));
        assert!(!project_line_matches("Hello World", "world", false));
        assert!(project_line_matches("Hello World", "World", false));
        assert!(!project_line_matches("anything", "", true));
        assert!(!project_line_matches("anything", "", false));
    }

    #[test]
    fn ctrl_navigation_bindings() {
        assert!(is_ctrl_home("\x1b[1;5H"));
        assert!(is_ctrl_home("\x1b[7;5~"));
        assert!(!is_ctrl_home("\x1b[H"));
        assert!(is_ctrl_end("\x1b[1;5F"));
        assert!(is_ctrl_end("\x1b[8;5~"));
        assert!(!is_ctrl_end("\x1b[F"));
        assert!(is_alt_up("\x1b[1;3A"));
        assert!(is_alt_up("\x1b\x1b[A"));
        assert!(!is_alt_up("\x1b[A"));
        assert!(is_alt_down("\x1b[1;3B"));
        assert!(is_alt_down("\x1b\x1b[B"));
        assert!(!is_alt_down("\x1b[B"));
        assert!(is_alt_shift_up("\x1b[1;4A"));
        assert!(is_alt_shift_up("\x1b\x1b[1;2A"));
        assert!(is_alt_shift_down("\x1b[1;4B"));
        assert!(is_alt_shift_down("\x1b\x1b[1;2B"));
        assert!(is_ctrl_shift_home("\x1b[1;6H"));
        assert!(is_ctrl_shift_end("\x1b[1;6F"));
        assert!(is_ctrl_shift_e("\x1b[69;6u"));
        assert!(is_ctrl_shift_e("\x1b[101;6u"));
        assert!(!is_ctrl_shift_e("\x1b[70;6u"));
    }

    #[test]
    fn ctrl_shift_find_bindings() {
        assert!(is_ctrl_shift_o("\x1b[79;6u"));
        assert!(is_ctrl_shift_o("\x1b[111;6u"));
        assert!(!is_ctrl_shift_o("\x1b[70;6u"));
    }

    #[test]
    fn ctrl_shift_h_binding() {
        assert!(is_ctrl_shift_h("\x1b[72;6u"));
        assert!(is_ctrl_shift_h("\x1b[104;6u"));
        assert!(!is_ctrl_shift_h("\x1b[79;6u"));
        assert!(!is_ctrl_shift_h("\x1b[H"));
    }

    #[test]
    fn replace_in_line_cases() {
        assert_eq!(replace_in_line("foo bar foo", "foo", "baz", false), ("baz bar baz".to_string(), 2));
        assert_eq!(replace_in_line("Foo BAR foo", "foo", "baz", true), ("baz BAR baz".to_string(), 2));
        assert_eq!(replace_in_line("Foo BAR", "foo", "baz", false), ("Foo BAR".to_string(), 0));
        assert_eq!(replace_in_line("aaa", "aa", "b", false), ("ba".to_string(), 1));
        assert_eq!(replace_in_line("hello", "", "x", false), ("hello".to_string(), 0));
        assert_eq!(replace_in_line("hello", "l", "", false), ("heo".to_string(), 2));
        assert_eq!(replace_in_line("héllo Héllo", "héllo", "hi", true), ("hi hi".to_string(), 2));
        assert_eq!(count_matches_in_line("foo Foo", "foo", true), 2);
        assert_eq!(count_matches_in_line("foo Foo", "foo", false), 1);
        assert_eq!(count_matches_in_line("anything", "", true), 0);
    }

    #[test]
    fn context_menu_geometry_clamps() {
        // Fits as requested.
        assert_eq!(context_menu_geometry(6, 20, 10, 5, 80, 24), (10, 5, 26, 8));
        // Bottom-right corner: shifted up/left, status line stays clear.
        let (c, r, w, h) = context_menu_geometry(9, 20, 79, 23, 80, 24);
        assert_eq!((w, h), (26, 11));
        assert!(c + w - 1 <= 80);
        assert!(r + h - 1 <= 23);
        // Narrow terminal: width clamped, still on screen.
        let (c2, r2, w2, h2) = context_menu_geometry(9, 40, 30, 9, 30, 10);
        assert!(w2 <= 28 && c2 >= 1 && c2 + w2 - 1 <= 30);
        assert!(r2 >= 1 && r2 + h2 - 1 <= 9);
    }

    #[test]
    fn ctrl_k_binding() {
        assert!(is_ctrl_k("\x0b"));
        assert!(is_ctrl_k("\x1b[75;5u"));
        assert!(is_ctrl_k("\x1b[107;5u"));
        assert!(!is_ctrl_k("\x1f"));
        assert!(!is_ctrl_k("k"));
        // Shortcuts dialog has full coverage including itself.
        for want in [
            "Ctrl+S", "Ctrl+O", "Ctrl+P", "Ctrl+K", "Ctrl+F", "Ctrl+L",
            "Ctrl+Shift+O", "Ctrl+R", "Ctrl+Shift+H", "Ctrl+G",
        ] {
            assert!(shortcut_defs().iter().any(|(l, _)| *l == want), "missing {want}");
        }
    }

    #[test]
    fn osc52_sequence_bytes() {
        assert_eq!(osc52_sequence("hi"), "\x1b]52;c;aGk=\x07");
        assert_eq!(osc52_sequence(""), "\x1b]52;c;\x07");
        // Missing tool fails fast without hanging.
        assert!(!pipe_to_clipboard_tool("az-definitely-not-a-tool", &[], "x"));
    }

    #[test]
    fn escape_roundtrip() {
        let s = "a=b\tc\nd\\e";
        assert_eq!(unescape_state(&escape_state(s)), s);
    }

    fn has_comment(segs: &[crate::Segment]) -> bool {
        segs.iter().any(|s| s.color == crate::COMMENT)
    }

    #[test]
    fn blade_css_id_is_not_php_comment() {
        // Real Blade <style> case: #header is ID selector, not PHP `#` comment.
        let segs = crate::plugins::highlight_segments("#header { background: #fff; }", SyntaxMode::Blade);
        assert!(!has_comment(&segs), "CSS ID line flagged as comment: {segs:?}");
        // ID selector should highlight (yellow), hex should highlight (orange via css).
        let id_hit = segs.iter().any(|s| s.start == 0 && s.end == 7);
        assert!(id_hit, "expected #header range, got {segs:?}");
    }

    #[test]
    fn blade_hex_color_is_not_comment() {
        let segs = crate::plugins::highlight_segments("  color: #fff;", SyntaxMode::Blade);
        assert!(!has_comment(&segs), "hex color flagged as comment: {segs:?}");
    }

    #[test]
    fn blade_href_hash_is_not_comment() {
        let segs = crate::plugins::highlight_segments("<a href=\"#section\">", SyntaxMode::Blade);
        assert!(!has_comment(&segs), "href #section flagged as comment: {segs:?}");
    }

    #[test]
    fn blade_https_is_not_comment() {
        let segs = crate::plugins::highlight_segments("<a href=\"https://example.com\">", SyntaxMode::Blade);
        assert!(!has_comment(&segs), "https URL flagged as comment: {segs:?}");
        let segs_php = crate::plugins::highlight_segments("$u = \"https://example.com\";", SyntaxMode::Php);
        assert!(!has_comment(&segs_php), "PHP string URL flagged as comment: {segs_php:?}");
    }

    #[test]
    fn php_real_comments_still_work() {
        let segs = crate::plugins::highlight_segments("# real comment", SyntaxMode::Php);
        assert!(has_comment(&segs));
        let segs2 = crate::plugins::highlight_segments("$x = 1; // trailing", SyntaxMode::Php);
        assert!(has_comment(&segs2));
    }

    #[test]
    fn blade_directive_vs_email() {
        let segs = crate::plugins::highlight_segments("  @if($x)", SyntaxMode::Blade);
        assert!(segs.iter().any(|s| s.color == crate::PURPLE), "expected @if purple: {segs:?}");
        let segs2 = crate::plugins::highlight_segments("contact user@example.com here", SyntaxMode::Blade);
        // @example in email must NOT be marked as Blade directive.
        let bad = segs2.iter().any(|s| s.color == crate::PURPLE);
        assert!(!bad, "email flagged as Blade: {segs2:?}");
    }

    #[test]
    fn new_modes_from_word_and_path() {
        use std::path::Path;
        let cases = [
            ("md", SyntaxMode::Markdown),
            ("json", SyntaxMode::Json),
            ("toml", SyntaxMode::Toml),
            ("yaml", SyntaxMode::Yaml),
            ("sh", SyntaxMode::Bash),
            ("env", SyntaxMode::Dotenv),
            ("ini", SyntaxMode::Ini),
            ("log", SyntaxMode::Log),
            ("rs", SyntaxMode::Rust),
            ("nginx", SyntaxMode::Nginx),
            ("apache", SyntaxMode::Apache),
            ("dockerfile", SyntaxMode::Dockerfile),
            ("sql", SyntaxMode::Sql),
            ("python", SyntaxMode::Python),
            ("java", SyntaxMode::Java),
            ("csharp", SyntaxMode::Csharp),
            ("cpp", SyntaxMode::Cpp),
            ("c", SyntaxMode::C),
            ("go", SyntaxMode::Go),
            ("kotlin", SyntaxMode::Kotlin),
            ("swift", SyntaxMode::Swift),
            ("ruby", SyntaxMode::Ruby),
            ("dart", SyntaxMode::Dart),
            ("scala", SyntaxMode::Scala),
            ("r", SyntaxMode::R),
            ("lua", SyntaxMode::Lua),
            ("perl", SyntaxMode::Perl),
            ("haskell", SyntaxMode::Haskell),
            ("elixir", SyntaxMode::Elixir),
            ("clojure", SyntaxMode::Clojure),
            ("zig", SyntaxMode::Zig),
            ("julia", SyntaxMode::Julia),
            ("objc", SyntaxMode::Objc),
            ("ts", SyntaxMode::TypeScript),
            ("xml", SyntaxMode::Xml),
        ];
        for (word, mode) in cases {
            assert_eq!(SyntaxMode::from_word(word), Some(mode), "word {word}");
        }
        assert_eq!(SyntaxMode::from_path(Some(Path::new("x.md"))), SyntaxMode::Markdown);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("x.json"))), SyntaxMode::Json);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("Cargo.toml"))), SyntaxMode::Toml);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.yaml"))), SyntaxMode::Yaml);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("run.sh"))), SyntaxMode::Bash);
        assert_eq!(SyntaxMode::from_path(Some(Path::new(".env"))), SyntaxMode::Dotenv);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("app.ini"))), SyntaxMode::Ini);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("out.log"))), SyntaxMode::Log);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("main.rs"))), SyntaxMode::Rust);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("Dockerfile"))), SyntaxMode::Dockerfile);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("q.sql"))), SyntaxMode::Sql);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.ts"))), SyntaxMode::TypeScript);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.xml"))), SyntaxMode::Xml);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("nginx.conf"))), SyntaxMode::Nginx);
        assert_eq!(SyntaxMode::from_path(Some(Path::new(".htaccess"))), SyntaxMode::Apache);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.service"))), SyntaxMode::Systemd);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.py"))), SyntaxMode::Python);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("A.java"))), SyntaxMode::Java);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.cs"))), SyntaxMode::Csharp);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.cpp"))), SyntaxMode::Cpp);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.c"))), SyntaxMode::C);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.go"))), SyntaxMode::Go);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.kt"))), SyntaxMode::Kotlin);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.swift"))), SyntaxMode::Swift);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.rb"))), SyntaxMode::Ruby);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("Gemfile"))), SyntaxMode::Ruby);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.dart"))), SyntaxMode::Dart);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.scala"))), SyntaxMode::Scala);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.r"))), SyntaxMode::R);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.lua"))), SyntaxMode::Lua);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.pl"))), SyntaxMode::Perl);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.hs"))), SyntaxMode::Haskell);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.ex"))), SyntaxMode::Elixir);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.clj"))), SyntaxMode::Clojure);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.zig"))), SyntaxMode::Zig);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.jl"))), SyntaxMode::Julia);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("a.m"))), SyntaxMode::Objc);
    }

    #[test]
    fn new_modes_highlight_smoke() {
        let cases = [
            ("# Title", SyntaxMode::Markdown),
            ("{\"a\": 1}", SyntaxMode::Json),
            ("[pkg] # c", SyntaxMode::Toml),
            ("key: 1 # c", SyntaxMode::Yaml),
            ("#!/bin/sh # c", SyntaxMode::Bash),
            ("KEY=1 # c", SyntaxMode::Dotenv),
            ("[s] ; c", SyntaxMode::Ini),
            ("2026-09-30 ERROR boom", SyntaxMode::Log),
            ("fn main() // c", SyntaxMode::Rust),
            ("server { # c", SyntaxMode::Nginx),
            ("<VirtualHost # c", SyntaxMode::Apache),
            ("FROM rust # c", SyntaxMode::Dockerfile),
            ("[Unit] # c", SyntaxMode::Systemd),
            ("SELECT 1 -- c", SyntaxMode::Sql),
            ("const x: number = 1", SyntaxMode::TypeScript),
            ("<tag attr=\"v\">", SyntaxMode::Xml),
            ("def hello(): # c", SyntaxMode::Python),
            ("public class A // c", SyntaxMode::Java),
            ("public class A // c", SyntaxMode::Csharp),
            ("int main() // c", SyntaxMode::Cpp),
            ("int main() // c", SyntaxMode::C),
            ("func main() // c", SyntaxMode::Go),
            ("fun main() // c", SyntaxMode::Kotlin),
            ("func view() // c", SyntaxMode::Swift),
            ("def hello # c", SyntaxMode::Ruby),
            ("class App // c", SyntaxMode::Dart),
            ("def hello // c", SyntaxMode::Scala),
            ("x <- 1 # c", SyntaxMode::R),
            ("local x = 1 -- c", SyntaxMode::Lua),
            ("my $x = 1; # c", SyntaxMode::Perl),
            ("main = putStrLn x -- c", SyntaxMode::Haskell),
            ("def hello do # c", SyntaxMode::Elixir),
            ("(defn hello [] ; c", SyntaxMode::Clojure),
            ("fn main() // c", SyntaxMode::Zig),
            ("function f() # c", SyntaxMode::Julia),
            ("@interface App // c", SyntaxMode::Objc),
        ];
        for (line, mode) in cases {
            let segs = crate::plugins::highlight_segments(line, mode);
            assert!(!segs.is_empty(), "no segments for {line:?} in {mode:?}");
        }
    }

    #[test]
    fn remote_version_parsing() {
        let toml = "[package]\nname = \"az\"\nversion = \"2.6.0\"\nedition = \"2021\"\n";
        assert_eq!(parse_remote_version(toml), Some("2.6.0".to_string()));
        assert_eq!(parse_remote_version("nothing here"), None);
        // A version outside [package] must not match.
        assert_eq!(parse_remote_version("[dependencies]\nfoo = \"1\"\n"), None);
        // Malformed entries are skipped, not fatal.
        assert_eq!(parse_remote_version("[package]\nname = \"az\"\n"), None);
    }

    #[test]
    fn newer_version_comparison() {
        assert!(is_newer_version("2.6.0", "2.5.0"));
        assert!(is_newer_version("2.6", "2.5.0"));
        assert!(is_newer_version("3.0.0", "2.9.9"));
        assert!(is_newer_version("2.5.1", "2.5.0"));
        assert!(!is_newer_version("2.5.0", "2.5.0"));
        assert!(!is_newer_version("2.4.9", "2.5.0"));
        assert!(!is_newer_version("2.5.0", "2.6.0"));
        assert!(!is_newer_version("2.5.0", "2.5"));
    }

    #[test]
    fn picker_bold_ranges_substring() {
        assert_eq!(match_bold_ranges("main.rs", "main"), vec![(0, 4)]);
        assert_eq!(match_bold_ranges("Save as", "save"), vec![(0, 4)]);
        assert!(match_bold_ranges("main.rs", "").is_empty());
    }

    #[test]
    fn picker_bold_ranges_fuzzy() {
        // Subsequence fallback bolds each char in order.
        let ranges = match_bold_ranges("command palette", "cp");
        assert_eq!(ranges.len(), 2);
        // No match at all -> empty.
        assert!(match_bold_ranges("abc", "xyz").is_empty());
    }

    #[test]
    fn picker_bold_ansi_wraps() {
        let out = apply_bold_ansi("main.rs", &[(0, 4)]);
        assert_eq!(out, "\x1b[1mmain\x1b[22m.rs");
        assert_eq!(apply_bold_ansi("abc", &[]), "abc");
    }

    #[test]
    fn sgr_mouse_click_and_scroll() {
        let click = parse_sgr_mouse("\x1b[<0;30;10M").unwrap();
        assert_eq!(click.button, 0);
        assert_eq!((click.x, click.y), (30, 10));
        assert!(!click.is_release);
        assert!(!click.is_scroll());

        let release = parse_sgr_mouse("\x1b[<0;30;10m").unwrap();
        assert!(release.is_release);

        let up = parse_sgr_mouse("\x1b[<64;10;5M").unwrap();
        assert!(up.is_scroll());
        assert!(up.scroll_up());

        let down = parse_sgr_mouse("\x1b[<65;10;5M").unwrap();
        assert!(down.is_scroll());
        assert!(!down.scroll_up());

        assert!(parse_sgr_mouse("\x1b[A").is_none());
        assert!(parse_sgr_mouse("\x1b[<0;0;0M").is_none());
    }

    #[test]
    fn legacy_mouse_parsing() {
        // ESC [ M Cb Cx Cy, all +32: ' '(0), '>'(30), '*'(10).
        let click = parse_legacy_mouse("\x1b[M >*").unwrap();
        assert_eq!(click.button, 0);
        assert_eq!((click.x, click.y), (30, 10));
        assert!(!click.is_release);
        assert!(!click.is_scroll());
        // Release is Cb==3 ('#'), wheel up/down are 64/65 ('`'/'a').
        assert!(parse_legacy_mouse("\x1b[M#**").unwrap().is_release);
        let up = parse_legacy_mouse("\x1b[M`*+").unwrap();
        assert!(up.is_scroll() && up.scroll_up());
        let down = parse_legacy_mouse("\x1b[Ma*+").unwrap();
        assert!(down.is_scroll() && !down.scroll_up());
        assert_eq!(parse_legacy_mouse("\x1b[<0;30;10M"), None);
        assert_eq!(parse_legacy_mouse("\x1b[Mab"), None);
        assert_eq!(parse_legacy_mouse("hello"), None);
    }

    #[test]
    fn word_range_at_selects_words() {
        assert_eq!(word_range_at("hello world", 0), Some((0, 5)));
        assert_eq!(word_range_at("hello world", 4), Some((0, 5)));
        assert_eq!(word_range_at("hello world", 6), Some((6, 11)));
        assert_eq!(word_range_at("hello world", 5), None);
        assert_eq!(word_range_at("a  b", 2), None);
        // Caret just past a word still counts as inside it.
        assert_eq!(word_range_at("hello world", 11), Some((6, 11)));
        assert_eq!(word_range_at("foo_bar", 3), Some((0, 7)));
        assert_eq!(word_range_at("a+b", 1), None);
        assert_eq!(word_range_at("a+b", 2), Some((2, 3)));
        assert_eq!(word_range_at("", 0), None);
        assert_eq!(word_range_at("héllo", 1), Some((0, 6)));
    }

    #[test]
    fn editor_click_col_maps_visual_to_bytes() {        // "a\tb": `a` is 1 cell, tab is 4 cells.
        assert_eq!(editor_click_col("a\tb", 0, 0), 0);
        assert_eq!(editor_click_col("a\tb", 0, 1), 1);
        // Inside the tab (visual 1..5) stays at tab start.
        assert_eq!(editor_click_col("a\tb", 0, 2), 1);
        assert_eq!(editor_click_col("a\tb", 0, 5), 2);
        // Past EOL clamps to line length.
        assert_eq!(editor_click_col("a\tb", 0, 100), 3);
        // Multi-byte: é is 2 bytes, 1 cell; wide char is 2 cells.
        assert_eq!(editor_click_col("héllo", 0, 2), 3);
        assert_eq!(editor_click_col("あx", 0, 1), 0);
        assert_eq!(editor_click_col("あx", 0, 2), 3);
        // Topbar: prefix occupies cols 1..=prefix_w, tabs follow back to back.
        assert_eq!(tab_hit_index(10, &[5, 7], 10), None);
        assert_eq!(tab_hit_index(10, &[5, 7], 11), Some(0));
        assert_eq!(tab_hit_index(10, &[5, 7], 15), Some(0));
        assert_eq!(tab_hit_index(10, &[5, 7], 16), Some(1));
        assert_eq!(tab_hit_index(10, &[5, 7], 22), Some(1));
        assert_eq!(tab_hit_index(10, &[5, 7], 23), None);
        assert_eq!(tab_hit_index(10, &[], 11), None);
    }

    #[test]
    fn mouse_burst_splits_into_events() {
        // Fast scrolling glues several SGR reports into one stdin read.
        let evs = parse_mouse_events("\x1b[<65;60;10M\x1b[<65;60;10M\x1b[<64;60;10M");
        assert_eq!(evs.len(), 3);
        assert!(evs[0].is_scroll() && !evs[0].scroll_up());
        assert_eq!((evs[1].x, evs[1].y), (60, 10));
        assert!(evs[2].scroll_up());
        // Single reports still yield exactly one event.
        assert_eq!(parse_mouse_events("\x1b[<0;30;10M").len(), 1);
        // Trailing partial report is ignored, complete ones are kept.
        assert_eq!(parse_mouse_events("\x1b[<65;60;10M\x1b[<65;").len(), 1);
        // Legacy bursts split into 6-byte reports too.
        assert_eq!(parse_mouse_events("\x1b[Ma*+\x1b[M`*+").len(), 2);
        // Non-mouse input yields nothing (and never loops forever).
        assert!(parse_mouse_events("hello").is_empty());
    }

    #[test]
    fn wheel_events_scroll_editor_and_tree() {
        let dir = std::env::temp_dir().join("az-wheel-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for i in 0..10 {
            std::fs::write(dir.join(format!("f{i}.txt")), "x\n").unwrap();
        }
        let big = dir.join("big.txt");
        std::fs::write(&big, (0..100).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n")).unwrap();
        let mut ed = Editor::new(vec!["az".to_string(), big.to_str().unwrap().to_string()]);
        ed.rows = 24;
        ed.cols = 80;
        ed.content_height = 19;
        ed.refresh_tree();
        ed.selection_anchor = Some(Pos { line: 0, col: 0 });
        // One wheel notch pans the viewport. The caret and the selection stay.
        ed.handle_key("\x1b[<65;60;10M".to_string());
        assert_eq!(ed.tab().cursor.line, 0);
        assert_eq!(ed.tab().row_offset, 1);
        assert!(ed.selection_anchor.is_some());
        // A burst still applies every event (previously the whole burst was dropped).
        ed.handle_key("\x1b[<65;60;10M\x1b[<65;60;10M\x1b[<65;60;10M".to_string());
        assert_eq!(ed.tab().cursor.line, 0);
        assert_eq!(ed.tab().row_offset, 4);
        // Wheel-up scrolls back one line.
        ed.handle_key("\x1b[<64;60;10M".to_string());
        assert_eq!(ed.tab().row_offset, 3);
        assert_eq!(ed.tab().cursor.line, 0);
        // Wheel over the sidebar moves the tree selection by one row.
        ed.tree_index = 0;
        ed.handle_key("\x1b[<65;5;10M".to_string());
        assert_eq!(ed.tree_index, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clipboard_pipe_closes_stdin_before_wait() {
        // `cat` only exits after stdin EOF: proves stdin is closed before
        // wait() instead of deadlocking (wl-copy/xclip/xsel/pbcopy read
        // stdin to end the same way).
        assert!(pipe_to_clipboard_tool("cat", &[], "hello"));
        assert!(!pipe_to_clipboard_tool("az-definitely-missing-tool", &[], "hello"));
    }

    #[test]
    fn bracketed_paste_keeps_body_and_trailing_input() {
        let buf = b"\x1b[200~line1\nline2\x1b[201~\x1b[A";
        let (body, rest) = take_bracketed_paste(buf).unwrap();
        assert_eq!(body, b"line1\nline2");
        assert_eq!(rest, b"\x1b[A");
        assert!(take_bracketed_paste(b"\x1b[200~partial").is_none());
        assert!(take_bracketed_paste(b"\x1b[A").is_none());
    }

    #[test]
    fn fast_paste_burst_stops_at_escape() {
        let pending: VecDeque<u8> = b"hello\nworld\x1b[A".iter().copied().collect();
        assert!(pending_is_paste_burst(&pending));
        let mut raw = Vec::new();
        let mut rest = pending;
        while rest.front().copied().is_some_and(is_fast_paste_byte) {
            raw.push(rest.pop_front().unwrap());
        }
        assert_eq!(raw, b"hello\nworld");
        assert_eq!(rest.into_iter().collect::<Vec<_>>(), b"\x1b[A");
    }

    #[test]
    fn bulk_paste_is_one_undo_step() {
        let dir = std::env::temp_dir().join("az-paste-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("p.txt");
        std::fs::write(&file, "").unwrap();
        let mut ed = Editor::new(vec!["az".into(), file.to_str().unwrap().to_string()]);
        ed.handle_key("\0AZPASTE:line1\nline2\nline3".into());
        assert_eq!(ed.tab().lines, vec!["line1", "line2", "line3"]);
        assert_eq!(ed.tab().undo.len(), 1);
        ed.undo();
        assert_eq!(ed.tab().lines, vec![""]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sudo_failure_hides_password_prompt() {
        assert_eq!(sudo_failure_message(b"[sudo] password for araz: \n"), "wrong password");
        assert_eq!(sudo_failure_message(b"sorry, try again\n"), "sorry, try again");
    }

    #[test]
    fn readonly_file_is_permission_denied() {
        let path = std::env::temp_dir().join(format!("az-perm-{}", std::process::id()));
        std::fs::write(&path, b"x").unwrap();
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&path, perms).unwrap();
        let err = std::fs::write(&path, b"y").unwrap_err();
        let _ = std::fs::set_permissions(&path, {
            let mut p = std::fs::metadata(&path).unwrap().permissions();
            p.set_readonly(false);
            p
        });
        let _ = std::fs::remove_file(&path);
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn shell_rc_and_shebang_use_bash() {
        use std::path::{Path, PathBuf};
        assert_eq!(SyntaxMode::from_path(Some(Path::new("/home/x/.bashrc"))), SyntaxMode::Bash);
        assert_eq!(SyntaxMode::from_path(Some(Path::new(".bash_profile"))), SyntaxMode::Bash);
        assert_eq!(SyntaxMode::from_path(Some(Path::new(".zshrc"))), SyntaxMode::Bash);
        assert_eq!(SyntaxMode::from_path(Some(Path::new("profile"))), SyntaxMode::Bash);
        assert_eq!(plugins::tree_color(Path::new(".bashrc"), false), RED);
        assert_eq!(plugins::syntax_from_shebang("#!/bin/bash"), Some(SyntaxMode::Bash));
        assert_eq!(plugins::syntax_from_shebang("#!/usr/bin/env sh"), Some(SyntaxMode::Bash));
        assert_eq!(plugins::syntax_from_shebang("#!/usr/bin/python3"), None);
        let mut tab = Tab::empty();
        tab.lines = vec!["#!/usr/bin/env bash".to_string()];
        assert_eq!(tab.syntax(), SyntaxMode::Bash);
        tab.path = Some(PathBuf::from("app.py"));
        tab.lines = vec!["#!/bin/bash".to_string()];
        assert_eq!(tab.syntax(), SyntaxMode::Python);
        tab.syntax_mode = Some(SyntaxMode::Python);
        tab.path = Some(PathBuf::from(".bashrc"));
        assert_eq!(tab.syntax(), SyntaxMode::Python);
    }

    #[test]
    fn typed_enter_is_not_a_paste() {
        let short: VecDeque<u8> = b"x\n".iter().copied().collect();
        assert!(!pending_is_paste_burst(&short));
        let enough: VecDeque<u8> = b"abcdefg\n".iter().copied().collect();
        assert!(pending_is_paste_burst(&enough));
    }

    #[test]
    fn tab_window_keeps_current_visible() {
        assert_eq!(window_indexes(5, 2, 9).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
        assert_eq!(window_indexes(12, 0, 9).collect::<Vec<_>>(), (0..9).collect::<Vec<_>>());
        assert_eq!(window_indexes(12, 11, 9).collect::<Vec<_>>(), (3..12).collect::<Vec<_>>());
        let mid = window_indexes(12, 6, 9).collect::<Vec<_>>();
        assert_eq!(mid.len(), 9);
        assert!(mid.contains(&6));
    }

    #[test]
    fn visual_scroll_keeps_tab_on_screen() {
        let line = "\t\t\tx";
        assert_eq!(visual_at_byte(line, 3), 12);
        assert_eq!(fit_visual_offset(line, 3, 10), 4);
        let start = fit_visual_offset(line, 3, 10);
        assert!(visual_at_byte(line, 3) >= start);
        assert!(visual_at_byte(line, 3) < start + 10);
        assert_eq!(line_as_clipboard(&["a".into(), "b".into()], 0), "a\n");
        assert_eq!(line_as_clipboard(&["a".into(), "b".into()], 1), "b");
    }

    #[test]
    fn titlebar_hit_matches_label() {
        let mode = "editor";
        let prefix = format!(" az   {mode} ");
        assert_eq!(prefix.len(), 7 + mode.len());
        let x = 4 + 1 + mode.len() + 2 + 1;
        assert_eq!(x, prefix.len() + 1);
        let label = " Open ";
        assert_eq!((x + 1)..=(x + label.len()), (prefix.len() + 2)..=(prefix.len() + 1 + label.len()));
    }

    #[test]
    fn picker_click_outside_cancels() {
        let dir = std::env::temp_dir().join(format!("az-picker-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("p.txt");
        std::fs::write(&file, "x").unwrap();
        let mut ed = Editor::new(vec!["az".into(), file.to_str().unwrap().into()]);
        ed.rows = 24;
        ed.cols = 80;
        assert!(matches!(ed.picker_mouse("\x1b[<0;1;1M", 0, 3), PickerMouse::Cancel));
        let frame = picker_frame(80, 24);
        let key = format!("\x1b[<0;{};{}M", frame.start_col + 2, frame.start_row + 4);
        assert!(matches!(ed.picker_mouse(&key, 0, 3), PickerMouse::Activate(0)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn new_tab_shortcut_opens_untitled() {
        let dir = std::env::temp_dir().join(format!("az-newtab-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("n.txt");
        std::fs::write(&file, "keep").unwrap();
        let mut ed = Editor::new(vec!["az".into(), file.to_str().unwrap().into()]);
        ed.handle_key("\x0e".into());
        assert_eq!(ed.tabs.len(), 2);
        assert!(ed.tab().path.is_none());
        assert_eq!(ed.tab().lines, vec!["".to_string()]);
        assert_eq!(ed.message, "New file. Ctrl+S to choose a filename");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrap_and_undo_restores_text() {
        let dir = std::env::temp_dir().join(format!("az-wrap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("w.txt");
        std::fs::write(&file, "hello").unwrap();
        let mut ed = Editor::new(vec!["az".into(), file.to_str().unwrap().into()]);
        ed.handle_key("\x01".into());
        ed.handle_key("(".into());
        assert_eq!(ed.tab().lines, vec!["(hello)".to_string()]);
        ed.undo();
        assert_eq!(ed.tab().lines, vec!["hello".to_string()]);
        ed.handle_key("\x01".into());
        ed.handle_key("\"".into());
        assert_eq!(ed.tab().lines, vec!["\"hello\"".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_then_undo_clears_dirty_when_text_matches() {
        let dir = std::env::temp_dir().join(format!("az-dirty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("u.txt");
        std::fs::write(&file, "hello").unwrap();
        let mut ed = Editor::new(vec!["az".into(), file.to_str().unwrap().into()]);
        ed.handle_key("!".into());
        assert!(ed.tab().modified);
        ed.undo();
        assert_eq!(ed.tab().lines, vec!["hello".to_string()]);
        assert!(!ed.tab().modified);
        assert!(ed.tab().revision > ed.tab().saved_revision);
        ed.handle_key("!".into());
        ed.handle_key("\x13".into());
        assert!(!ed.tab().modified);
        ed.handle_key("?".into());
        assert!(ed.tab().modified);
        ed.undo();
        assert_eq!(ed.tab().lines, vec!["!hello".to_string()]);
        assert!(!ed.tab().modified);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn crlf_file_roundtrips() {
        let dir = std::env::temp_dir().join(format!("az-crlf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("c.txt");
        std::fs::write(&file, b"a\r\nb\r\n").unwrap();
        let tab = Tab::from_path(file).unwrap();
        assert!(tab.crlf);
        assert_eq!(tab.lines, vec!["a".to_string(), "b".to_string(), "".to_string()]);
        assert_eq!(tab.text(), "a\r\nb\r\n");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
