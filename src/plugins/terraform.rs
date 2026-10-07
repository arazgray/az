use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, RED, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_between, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in find_between(line, "/*", "*/") {
        s.push(Segment { start: a, end: b, color: COMMENT });
    }
    for (a, b) in interp_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: RED });
        }
    }
    // Block labels: resource "aws_x" "name" { -> block type purple, labels cyan.
    let t = line.trim_start();
    let indent = line.len() - t.len();
    if let Some(first) = t.split_whitespace().next() {
        if blocks().contains(&first) && !pos_in_ranges(indent, &strings) {
            s.push(Segment { start: indent, end: indent + first.len(), color: PURPLE });
        }
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if blocks().contains(&w) {
            s.push(Segment { start: a, end: b, color: PURPLE });
        } else if keywords().contains(&w) {
            s.push(Segment { start: a, end: b, color: CYAN });
        } else if after_nonspace_is(line, b, '(') || after_nonspace_is(line, b, '=') {
            s.push(Segment { start: a, end: b, color: BLUE });
        }
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    s
}

fn comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' && !pos_in_ranges(i, strings) {
            return Some(i);
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' && !pos_in_ranges(i, strings) {
            if i == 0 || bytes[i - 1] != b':' {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn interp_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(a) = line[pos..].find("${") {
        let start = pos + a;
        if let Some(b) = line[start + 2..].find('}') {
            let end = start + 2 + b + 1;
            out.push((start, end));
            pos = end;
        } else {
            out.push((start, line.len()));
            break;
        }
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("terraform:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "terraform:word" => {
            let mut all = Vec::new();
            for k in blocks() {
                all.push(comp(k, k, "block"));
            }
            for k in keywords() {
                all.push(comp(k, k, "keyword"));
            }
            for sym in document_symbols(ctx.lines, ctx.scan_limit) {
                all.push(comp(&sym, &sym, "symbol"));
            }
            all
        }
        _ => Vec::new(),
    }
}

pub(crate) fn document_symbols(lines: &[String], limit: usize) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines.iter().take(limit) {
        out.extend(symbols(line));
    }
    out.sort();
    out.dedup();
    out
}

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let t = line.trim_start();
    for marker in ["resource ", "variable ", "output ", "module ", "provider ", "data "] {
        if t.starts_with(marker) {
            let rest = t[marker.len()..].trim().trim_matches('"').to_string();
            let name: String = rest.chars().take_while(|c| *c != '{').collect::<String>().trim().trim_matches('"').to_string();
            if !name.is_empty() {
                return vec![format!("{} {}", marker.trim(), name)];
            }
        }
    }
    Vec::new()
}

fn blocks() -> Vec<&'static str> {
    vec!["resource", "variable", "output", "module", "provider", "terraform", "data", "locals", "moved", "import", "removed", "check"]
}

fn keywords() -> Vec<&'static str> {
    vec!["source", "version", "count", "for_each", "depends_on", "lifecycle", "backend", "required_providers", "required_version", "default", "type", "description", "sensitive", "nullable", "validation", "precondition", "postcondition"]
}
