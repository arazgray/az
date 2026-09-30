use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, YELLOW, Segment, CompletionItem};
use super::{find_between, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let t = line.trim_start();
    // Headings: # .. ###### at line start
    if let Some(h) = heading_level(t) {
        let indent = line.len() - t.len();
        s.push(Segment { start: indent, end: line.len(), color: if h <= 2 { YELLOW } else { ORANGE } });
        return s;
    }
    // Blockquote / hr / list markers
    if t.starts_with('>') {
        s.push(Segment { start: 0, end: line.len(), color: COMMENT });
        return s;
    }
    if t == "---" || t == "***" || t == "___" {
        s.push(Segment { start: 0, end: line.len(), color: COMMENT });
        return s;
    }
    let trimmed = t;
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        let indent = line.len() - t.len();
        s.push(Segment { start: indent, end: indent + 1, color: CYAN });
    }
    // Code spans `code`
    for (a, b) in find_between(line, "`", "`") {
        // skip empty `` markers edge
        if b - a >= 2 {
            s.push(Segment { start: a, end: b, color: GREEN });
        }
    }
    // Bold **x** / __x__
    for (a, b) in find_between(line, "**", "**") {
        if b - a > 4 {
            s.push(Segment { start: a, end: b, color: PURPLE });
        }
    }
    for (a, b) in find_between(line, "__", "__") {
        if b - a > 4 {
            s.push(Segment { start: a, end: b, color: PURPLE });
        }
    }
    // Links [text](url)
    for (a, b) in link_ranges(line) {
        s.push(Segment { start: a, end: b, color: BLUE });
    }
    s
}

fn heading_level(t: &str) -> Option<usize> {
    let mut n = 0;
    for c in t.chars() {
        if c == '#' && n < 6 {
            n += 1;
        } else {
            break;
        }
    }
    if n > 0 && t[n..].starts_with([' ', '\t']) {
        Some(n)
    } else {
        None
    }
}

fn link_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if let Some(close) = line[i..].find(']') {
                let after = i + close + 1;
                if after < bytes.len() && bytes[after] == b'(' {
                    if let Some(end) = line[after..].find(')') {
                        out.push((i, after + end + 1));
                        i = after + end + 1;
                        continue;
                    }
                }
            }
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

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let t = line.trim_start();
    if heading_level(t).is_some() {
        return vec![t.to_string()];
    }
    Vec::new()
}
