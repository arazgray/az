use crate::{BLUE, COMMENT, CYAN, GREEN, MAGENTA, ORANGE, PURPLE, YELLOW, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_between, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = hash_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in find_between(line, "=pod", "=cut") {
        s.push(Segment { start: a, end: b, color: COMMENT });
    }
    for (a, b) in sigil_ranges(line) {
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
        } else if after_nonspace_is(line, b, '(') {
            s.push(Segment { start: a, end: b, color: BLUE });
        }
    }
    // $ @ % sigils
    for (a, b) in find_prefixed(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: MAGENTA });
        }
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
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

fn sigil_ranges(line: &str) -> Vec<(usize, usize)> {
    // $var @var %var
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if matches!(bytes[i], b'$' | b'@' | b'%') {
            let start = i;
            i += 1;
            if i < bytes.len() && bytes[i] == b'#' {
                i += 1;
            }
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
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

fn find_prefixed(line: &str) -> Vec<(usize, usize)> {
    sigil_ranges(line)
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("perl:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "perl:word" => {
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
    let t = line.trim_start();
    if let Some(pos) = t.find("sub ") {
        let after = &t[pos + 4..];
        let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':').collect();
        if !name.is_empty() {
            return vec![format!("sub {name}")];
        }
    }
    if t.starts_with("package ") {
        let name: String = t[8..].trim().chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':').collect();
        if !name.is_empty() {
            return vec![format!("package {name}")];
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["break", "continue", "do", "else", "elsif", "for", "foreach", "if", "last", "my", "our", "local", "next", "package", "redo", "require", "return", "sub", "unless", "until", "use", "while", "no", "default", "given", "when", "state", "say", "die"]
}

fn builtins() -> Vec<&'static str> {
    vec!["print", "say", "printf", "sprintf", "push", "pop", "shift", "unshift", "split", "join", "map", "grep", "sort", "keys", "values", "each", "defined", "undef", "ref", "bless", "die", "warn", "open", "close", "chomp", "chop", "length", "substr", "index", "open"]
}
