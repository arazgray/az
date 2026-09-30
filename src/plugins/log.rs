use crate::{COMMENT, CYAN, GREEN, RED, YELLOW, Segment, CompletionItem};
use super::CompletionContext;

/// Log highlighting: timestamps dim-cyan, levels colored.
/// Line-local only; no completion or symbols (logs are read-mostly).
pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    for (a, b) in timestamp_ranges(line) {
        s.push(Segment { start: a, end: b, color: CYAN });
    }
    let upper = line.to_ascii_uppercase();
    for (pat, color) in [
        ("FATAL", RED),
        ("CRITICAL", RED),
        ("ERROR", RED),
        ("FAIL", RED),
        ("FAILED", RED),
        ("PANIC", RED),
        ("WARN", YELLOW),
        ("WARNING", YELLOW),
        ("INFO", GREEN),
        ("DEBUG", COMMENT),
        ("TRACE", COMMENT),
    ] {
        let mut pos = 0;
        while let Some(idx) = upper[pos..].find(pat) {
            let a = pos + idx;
            let b = a + pat.len();
            if boundary_ok(line, a, b) {
                s.push(Segment { start: a, end: b, color });
            }
            pos = b;
        }
    }
    s
}

fn boundary_ok(line: &str, a: usize, b: usize) -> bool {
    let bytes = line.as_bytes();
    let before_ok = a == 0 || !bytes[a - 1].is_ascii_alphanumeric();
    let after_ok = b >= bytes.len() || !bytes[b].is_ascii_alphanumeric();
    before_ok && after_ok
}

fn timestamp_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    // [2026-09-30 10:00:00] or 2026-09-30T10:00:00 or 10:00:00
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if let Some(end) = line[i..].find(']') {
                let inner = &line[i + 1..i + end];
                if inner.chars().any(|c| c.is_ascii_digit()) && inner.contains(['-', ':', '/']) {
                    out.push((i, i + end + 1));
                    i += end + 1;
                    continue;
                }
            }
        }
        // YYYY-MM-DD
        if i + 10 <= bytes.len()
            && bytes[i].is_ascii_digit()
            && bytes[i + 1].is_ascii_digit()
            && bytes[i + 2].is_ascii_digit()
            && bytes[i + 3].is_ascii_digit()
            && bytes[i + 4] == b'-'
        {
            out.push((i, (i + 10).min(bytes.len())));
            i += 10;
            continue;
        }
        // HH:MM:SS
        if i + 8 <= bytes.len()
            && bytes[i].is_ascii_digit()
            && bytes[i + 1].is_ascii_digit()
            && bytes[i + 2] == b':'
            && bytes[i + 3].is_ascii_digit()
            && bytes[i + 4].is_ascii_digit()
            && bytes[i + 5] == b':'
        {
            out.push((i, (i + 8).min(bytes.len())));
            i += 8;
            continue;
        }
        i += 1;
    }
    out
}

pub(crate) fn completion_context(_before: &str, _explicit: bool) -> Option<(String, String, usize)> {
    None
}

pub(crate) fn completion_items(_kind: &str, _ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    Vec::new()
}

pub(crate) fn symbols(_line: &str) -> Vec<String> {
    Vec::new()
}
