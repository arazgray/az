use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    let t = line.trim_start();
    let indent = line.len() - t.len();
    if t.starts_with("#") {
        s.push(Segment { start: indent, end: line.len(), color: COMMENT });
        return s;
    }
    if let Some(colon) = t.find(':') {
        let head = t[..colon].trim();
        if !head.is_empty() && !head.contains('=') {
            if let Some(first) = head.split_whitespace().next() {
                if first.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false) {
                    let a = indent + t.find(first).unwrap_or(0);
                    s.push(Segment { start: a, end: a + first.len(), color: BLUE });
                }
            }
        }
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if keywords().contains(&w) {
            s.push(Segment { start: a, end: b, color: PURPLE });
        } else if builtins().contains(&w) {
            s.push(Segment { start: a, end: b, color: CYAN });
        } else if constants().contains(&w) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        } else if constants().contains(&w) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        } else if after_nonspace_is(line, b, '(') {
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

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("salt:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "salt:word" => {
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
    if t.is_empty() || t.starts_with('#') {
        return Vec::new();
    }
    if let Some(colon) = t.find(':') {
        let head = t[..colon].trim();
        if !head.is_empty() && !head.contains('=') {
            let first = head.split_whitespace().next().unwrap_or("");
            if !first.is_empty() && first.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false) {
                return vec![first.to_string()];
            }
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["if", "elif", "else", "or", "not", "and", "endif", "end"]
}

fn types() -> Vec<&'static str> {
    vec![]
}

fn builtins() -> Vec<&'static str> {
    vec!["grain", "grains", "compound", "pcre", "grain_pcre", "list", "pillar"]
}

fn constants() -> Vec<&'static str> {
    vec!["True", "False"]
}
