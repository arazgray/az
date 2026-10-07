use crate::{COMMENT, GREEN, ORANGE, PURPLE, Segment, CompletionItem};
use super::{comp, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    let t = line.trim_start();
    let indent = line.len() - t.len();
    let tl = t.to_ascii_lowercase();
    if tl == "rem" || tl.starts_with("rem ") || tl.starts_with("rem\t") || t.starts_with("::") {
        s.push(Segment { start: indent, end: line.len(), color: COMMENT });
        return s;
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if keywords().contains(&w) {
            s.push(Segment { start: a, end: b, color: PURPLE });
        }
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
            return Some(("batch:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "batch:word" => {
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
    if t.starts_with(':') && !t.starts_with("::") {
        let name: String = t[1..].chars().take_while(|c| c.is_alphanumeric() || matches!(*c, '_' | '-' | '.')).collect();
        if !name.is_empty() {
            return vec![format!("label {name}")];
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["echo", "set", "if", "else", "for", "goto", "call", "exit", "shift", "pause", "rem"]
}

fn types() -> Vec<&'static str> {
    vec![]
}

fn builtins() -> Vec<&'static str> {
    vec![]
}
