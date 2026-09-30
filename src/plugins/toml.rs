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
    // [sections] and [[arrays]]
    let t = line.trim();
    if (t.starts_with('[') && t.contains(']')) && !pos_in_ranges(line.find('[').unwrap_or(usize::MAX), &strings) {
        if let Some(a) = line.find('[') {
            if let Some(b) = line.rfind(']') {
                if b > a {
                    s.push(Segment { start: a, end: b + 1, color: CYAN });
                }
            }
        }
    }
    // key = (before =)
    if let Some(eq) = line.find('=') {
        if !pos_in_ranges(eq, &strings) {
            let key_part = &line[..eq];
            let start = key_part.len() - key_part.trim_start().len();
            // trim trailing spaces of key
            let key_trimmed = key_part.trim();
            if !key_trimmed.is_empty() {
                let key_start = line[..eq].rfind(key_trimmed).unwrap_or(start);
                if !pos_in_ranges(key_start, &strings) {
                    s.push(Segment { start: key_start, end: key_start + key_trimmed.len(), color: BLUE });
                }
            }
        }
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    // bools
    for w in ["true", "false"] {
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

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let t = line.trim();
    if t.starts_with('[') && t.ends_with(']') {
        return vec![t.to_string()];
    }
    Vec::new()
}
