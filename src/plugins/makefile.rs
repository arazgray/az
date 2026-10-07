use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, RED, Segment, CompletionItem};
use super::{comp, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = hash_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in variable_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: RED });
        }
    }
    // Target `foo:` at line start (not inside strings).
    let t = line.trim_start();
    if !t.is_empty() && !t.starts_with(['#', '.', '\t']) {
        if let Some(colon) = t.find(':') {
            let head = &t[..colon];
            if !head.is_empty()
                && !head.contains([' ', '\t', '=', '?'])
                && !head.starts_with('$')
            {
                let indent = line.len() - t.len();
                if !pos_in_ranges(indent, &strings) {
                    s.push(Segment { start: indent, end: indent + colon, color: BLUE });
                }
            }
        }
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if directives().contains(&w) {
            s.push(Segment { start: a, end: b, color: PURPLE });
        } else if functions().contains(&w) {
            s.push(Segment { start: a, end: b, color: CYAN });
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

fn variable_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() && (bytes[i + 1] == b'(' || bytes[i + 1] == b'{') {
            let open = bytes[i + 1];
            let close = if open == b'(' { b')' } else { b'}' };
            let start = i;
            i += 2;
            while i < bytes.len() && bytes[i] != close {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            out.push((start, i));
        } else if bytes[i] == b'$' && i + 1 < bytes.len() && (bytes[i + 1] == b'@' || bytes[i + 1] == b'<' || bytes[i + 1] == b'^' || bytes[i + 1] == b'*' || bytes[i + 1] == b'?' || bytes[i + 1] == b'%') {
            out.push((i, i + 2));
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("makefile:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "makefile:word" => {
            let mut all = Vec::new();
            for k in directives() {
                all.push(comp(k, k, "directive"));
            }
            for f in functions() {
                all.push(comp(f, f, "function"));
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
    if t.is_empty() || t.starts_with(['#', '.', '\t']) {
        return Vec::new();
    }
    if let Some(colon) = t.find(':') {
        let head = t[..colon].trim();
        if !head.is_empty() && !head.contains([' ', '\t', '=']) && !head.starts_with('$') {
            let name: String = head.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.' || *c == '%').collect();
            if !name.is_empty() {
                return vec![format!("target {name}")];
            }
        }
    }
    Vec::new()
}

fn directives() -> Vec<&'static str> {
    vec!["ifeq", "ifneq", "ifdef", "ifndef", "else", "endif", "include", "export", "override", "define", "endef", "undefine", "unexport", "vpath"]
}

fn functions() -> Vec<&'static str> {
    vec!["abspath", "addprefix", "addsuffix", "and", "basename", "call", "dir", "error", "eval", "file", "filter", "filter-out", "findstring", "firstword", "flavor", "foreach", "guile", "info", "join", "lastword", "notdir", "or", "origin", "patsubst", "realpath", "shell", "sort", "strip", "subst", "suffix", "value", "warning", "wildcard", "word", "wordlist", "words"]
}
