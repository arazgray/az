use crate::{COMMENT, GREEN, ORANGE, PURPLE, Segment, CompletionItem};
use super::{comp, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    let t = line.trim_start();
    let indent = line.len() - t.len();
    if t.starts_with('.') {
        let rest = &t[1..];
        let mut end = 0;
        for (i, ch) in rest.char_indices() {
            if ch.is_ascii_alphabetic() { end = i + ch.len_utf8(); } else { break; }
        }
        if end > 0 {
            s.push(Segment { start: indent, end: indent + 1 + end, color: PURPLE });
        }
    }
    if t.starts_with(".\"") {
        s.push(Segment { start: indent, end: line.len(), color: COMMENT });
        return s;
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    s
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("groff:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "groff:word" => {
            let mut all = Vec::new();
            for k in keywords() {
                all.push(comp(k, k, "keyword"));
            }
            for t in types() {
                all.push(comp(t, t, "type"));
            }
            for b in builtins() {
                all.push(comp(b, b, "builtin"));
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
    for prefix in [".TH ", ".th "] {
        if let Some(rest) = t.strip_prefix(prefix) {
            let name: String = rest.trim_start().trim_start_matches('"').chars().take_while(|c| !c.is_whitespace() && *c != '"').collect();
            if !name.is_empty() {
                return vec![name];
            }
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec![]
}

fn types() -> Vec<&'static str> {
    vec![]
}

fn builtins() -> Vec<&'static str> {
    vec![]
}
