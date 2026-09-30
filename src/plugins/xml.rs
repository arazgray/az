use crate::{COMMENT, GREEN, MAGENTA, ORANGE, YELLOW, Segment, CompletionItem};
use super::{find_between, is_name_byte, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    for (a, b) in find_between(line, "<!--", "-->") {
        s.push(Segment { start: a, end: b, color: COMMENT });
    }
    // <?xml ... ?> prolog
    for (a, b) in find_between(line, "<?", "?>") {
        s.push(Segment { start: a, end: b, color: MAGENTA });
    }
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let start = i;
            i += 1;
            if i < bytes.len() && (bytes[i] == b'/' || bytes[i] == b'?' || bytes[i] == b'!') {
                i += 1;
            }
            while i < bytes.len() && is_name_byte(bytes[i]) {
                i += 1;
            }
            if i > start + 1 {
                s.push(Segment { start, end: i, color: MAGENTA });
            }
        } else {
            i += 1;
        }
    }
    for (a, b) in string_ranges(line) {
        s.push(Segment { start: a, end: b, color: GREEN });
    }
    for (a, b) in attr_ranges(line) {
        s.push(Segment { start: a, end: b, color: YELLOW });
    }
    for (a, b) in entity_ranges(line) {
        s.push(Segment { start: a, end: b, color: ORANGE });
    }
    s
}

fn attr_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b':' || bytes[i] == b'_' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'-' | b'_' | b':' | b'.')) {
                i += 1;
            }
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'=' {
                out.push((start, i));
            }
        } else {
            i += 1;
        }
    }
    out
}

fn entity_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'#') {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b';' {
                out.push((start, i + 1));
            }
        } else {
            i += 1;
        }
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    // Reuse HTML tag/attr logic shape, relabeled to xml.
    if let Some(ctx) = super::html::completion_context(before, explicit) {
        return Some((ctx.0.replace("html", "xml"), ctx.1, ctx.2));
    }
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || !prefix.is_empty() {
            return Some(("xml:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "xml:tag" => super::html::completion_items("html:tag", ctx),
        "xml:attr" => super::html::completion_items("html:attr", ctx),
        "xml:word" => Vec::new(),
        _ => Vec::new(),
    }
}

pub(crate) fn symbols(_line: &str) -> Vec<String> {
    Vec::new()
}
