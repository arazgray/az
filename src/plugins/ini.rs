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
    // inline ; comment (outside strings)
    if let Some(pos) = inline_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    // [section]
    if t.starts_with('[') {
        if let Some(end) = t.find(']') {
            s.push(Segment { start: indent, end: indent + end + 1, color: CYAN });
            return s;
        }
    }
    // key = / key :
    if let Some(sep) = ["=", ":"].iter().filter_map(|p| line.find(p)).min() {
        if !pos_in_ranges(sep, &strings) {
            let key_part = line[..sep].trim();
            if !key_part.is_empty() && !key_part.contains(' ') {
                if let Some(ks) = line[..sep].rfind(key_part) {
                    s.push(Segment { start: ks, end: ks + key_part.len(), color: BLUE });
                }
            }
        }
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    for w in ["true", "false", "yes", "no", "on", "off"] {
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

fn inline_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    for (i, c) in line.char_indices() {
        if (c == ';' || c == '#') && !pos_in_ranges(i, strings) {
            // only treat as comment if preceded by whitespace or at start
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
