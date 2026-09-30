use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, YELLOW, Segment, CompletionItem};
use super::{comp, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = semicolon_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in keyword_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: YELLOW });
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
        } else if w.starts_with("def") || w.contains('-') {
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

fn semicolon_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    for (i, c) in line.char_indices() {
        if c == ';' && !pos_in_ranges(i, strings) {
            return Some(i);
        }
    }
    None
}

fn keyword_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b':' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'_' | b'-' | b'?' | b'!')) {
                i += 1;
            }
            if i > start + 1 {
                out.push((start, i));
                continue;
            }
        }
        i += 1;
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("clojure:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "clojure:word" => {
            let mut all = Vec::new();
            for k in keywords() {
                all.push(comp(k, k, "keyword"));
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
    let t = line.trim();
    if t.starts_with("(ns ") {
        let name: String = t[4..].trim().chars().take_while(|c| c.is_alphanumeric() || matches!(*c, '_' | '-' | '.')).collect();
        if !name.is_empty() {
            return vec![format!("ns {name}")];
        }
    }
    for marker in ["(defn ", "(defn- ", "(defmacro ", "(def ", "(defonce ", "(deftest "] {
        if let Some(pos) = t.find(marker) {
            let after = &t[pos + marker.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || matches!(*c, '_' | '-' | '?' | '!')).collect();
            if !name.is_empty() {
                return vec![format!("{} {}", marker.trim_matches(['(', ' ']), name)];
            }
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["def", "defn", "defn-", "defmacro", "defonce", "deftest", "ns", "fn", "let", "if", "when", "when-not", "cond", "case", "do", "loop", "recur", "throw", "try", "catch", "finally", "quote", "var", "doseq", "dotimes", "for", "and", "or", "not"]
}

fn builtins() -> Vec<&'static str> {
    vec!["map", "filter", "reduce", "apply", "partial", "comp", "first", "rest", "cons", "conj", "assoc", "dissoc", "get", "contains?", "str", "prn", "println", "vec", "list", "set", "atom", "swap!", "reset!", "defprotocol", "deftype", "defrecord"]
}

fn constants() -> Vec<&'static str> {
    vec!["true", "false", "nil"]
}
