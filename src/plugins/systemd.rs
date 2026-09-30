use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, Segment, CompletionItem};
use super::{pos_in_ranges, scan_ranges, string_ranges, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    let t = line.trim_start();
    let indent = line.len() - t.len();
    if t.starts_with('#') || t.starts_with(';') {
        s.push(Segment { start: indent, end: line.len(), color: COMMENT });
        return s;
    }
    if let Some(pos) = inline_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    if t.starts_with('[') {
        if let Some(end) = t.find(']') {
            s.push(Segment { start: indent, end: indent + end + 1, color: CYAN });
            return s;
        }
    }
    if let Some(eq) = line.find('=') {
        if !pos_in_ranges(eq, &strings) {
            let key_part = line[..eq].trim();
            if !key_part.is_empty() && !key_part.contains(' ') {
                if let Some(ks) = line[..eq].rfind(key_part) {
                    s.push(Segment { start: ks, end: ks + key_part.len(), color: BLUE });
                }
            }
            let val = line[eq + 1..].trim();
            if matches!(val, "true" | "false" | "yes" | "no") {
                if let Some(vs) = line[eq + 1..].find(val) {
                    s.push(Segment { start: eq + 1 + vs, end: eq + 1 + vs + val.len(), color: PURPLE });
                }
            }
        }
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    s
}

fn inline_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    for (i, c) in line.char_indices() {
        if (c == '#' || c == ';') && !pos_in_ranges(i, strings) {
            if i == 0 || line.as_bytes()[i - 1].is_ascii_whitespace() {
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

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let t = line.trim();
    if t.starts_with('[') && t.ends_with(']') {
        return vec![t.to_string()];
    }
    Vec::new()
}
