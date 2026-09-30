use crate::{BLUE, COMMENT, GREEN, ORANGE, PURPLE, Segment, CompletionItem};
use super::{pos_in_ranges, scan_ranges, string_ranges, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    let t = line.trim_start();
    if t.starts_with('#') {
        s.push(Segment { start: 0, end: line.len(), color: COMMENT });
        return s;
    }
    if t.starts_with("export ") {
        s.push(Segment { start: line.len() - t.len(), end: line.len() - t.len() + 6, color: PURPLE });
    }
    if let Some(eq) = line.find('=') {
        if !pos_in_ranges(eq, &strings) {
            let key_part = line[..eq].trim();
            // strip leading "export "
            let key_clean = key_part.strip_prefix("export ").unwrap_or(key_part).trim();
            if !key_clean.is_empty() {
                if let Some(ks) = line[..eq].rfind(key_clean) {
                    s.push(Segment { start: ks, end: ks + key_clean.len(), color: BLUE });
                }
            }
            // value bools/numbers
            let val = line[eq + 1..].trim();
            if matches!(val, "true" | "false") {
                if let Some(vs) = line[eq + 1..].find(val) {
                    s.push(Segment { start: eq + 1 + vs, end: eq + 1 + vs + val.len(), color: PURPLE });
                }
            }
        }
    } else if let Some(pos) = hash_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    s
}

fn hash_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    for (i, c) in line.char_indices() {
        if c == '#' && !pos_in_ranges(i, strings) {
            return Some(i);
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
