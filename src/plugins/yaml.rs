use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, Segment, CompletionItem};
use super::{pos_in_ranges, scan_ranges, string_ranges, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = hash_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    // list marker "- "
    let t = line.trim_start();
    if t.starts_with("- ") || t == "-" {
        let indent = line.len() - t.len();
        s.push(Segment { start: indent, end: indent + 1, color: CYAN });
    }
    // key: (before first colon outside strings)
    if let Some(colon) = colon_outside_strings(line, &strings) {
        let key_part = line[..colon].trim();
        if !key_part.is_empty() && !key_part.contains(' ') || line[..colon].contains(':') == false {
            let key_start = line[..colon].rfind(key_part).unwrap_or(0);
            if !pos_in_ranges(key_start, &strings) {
                s.push(Segment { start: key_start, end: key_start + key_part.len(), color: BLUE });
            }
        }
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    for w in ["true", "false", "null", "yes", "no", "on", "off"] {
        let mut pos = 0;
        while let Some(idx) = line[pos..].find(w) {
            let a = pos + idx;
            let b = a + w.len();
            let before_ok = a == 0 || !line.as_bytes()[a - 1].is_ascii_alphanumeric();
            let after_ok = b >= line.len() || !line.as_bytes()[b].is_ascii_alphanumeric();
            if before_ok && after_ok && !pos_in_ranges(a, &strings) {
                s.push(Segment { start: a, end: b, color: PURPLE });
            }
            pos = b;
        }
    }
    s
}

fn hash_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    for (i, c) in line.char_indices() {
        if c == '#' && !pos_in_ranges(i, strings) {
            // YAML requires space before # for comments, except at line start.
            // Keep it simple: any # outside strings starts a comment.
            return Some(i);
        }
    }
    None
}

fn colon_outside_strings(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    for (i, c) in line.char_indices() {
        if c == ':' && !pos_in_ranges(i, strings) {
            let after = &line[i + 1..];
            if after.is_empty() || after.starts_with([' ', '\t']) {
                return Some(i);
            }
        }
    }
    None
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
