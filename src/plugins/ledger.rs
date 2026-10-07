use crate::{COMMENT, GREEN, ORANGE, Segment, CompletionItem};
use super::{comp, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    let t = line.trim_start();
    let indent = line.len() - t.len();
    if t.starts_with(";") {
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
            return Some(("ledger:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "ledger:word" => {
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
    if t.len() < 11 {
        return Vec::new();
    }
    let tb = t.as_bytes();
    let date_ok = tb[0..4].iter().all(|c| c.is_ascii_digit())
        && (tb[4] == b'-' || tb[4] == b'/')
        && tb[5..7].iter().all(|c| c.is_ascii_digit())
        && (tb[7] == b'-' || tb[7] == b'/')
        && tb[8..10].iter().all(|c| c.is_ascii_digit())
        && (tb.len() == 10 || tb[10] == b' ' || tb[10] == b'\t');
    if date_ok {
        let payee = t[10..].trim().split(';').next().unwrap_or("").trim();
        if !payee.is_empty() {
            return vec![payee.chars().take(50).collect()];
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
