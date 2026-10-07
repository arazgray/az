use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, YELLOW, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_between, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = line_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in find_between(line, "/*", "*/") {
        s.push(Segment { start: a, end: b, color: COMMENT });
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if keywords().contains(&w) {
            s.push(Segment { start: a, end: b, color: PURPLE });
        } else if types().contains(&w) {
            s.push(Segment { start: a, end: b, color: CYAN });
        } else if modifiers().contains(&w) {
            s.push(Segment { start: a, end: b, color: YELLOW });
        } else if after_nonspace_is(line, b, '(') {
            s.push(Segment { start: a, end: b, color: BLUE });
        }
    }
    for (a, b) in number_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    s
}

fn line_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'/' {
            if pos_in_ranges(i, strings) {
                i += 2;
                continue;
            }
            if i > 0 && bytes[i - 1] == b':' {
                i += 2;
                continue;
            }
            return Some(i);
        }
        i += 1;
    }
    None
}

fn number_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = scan_ranges(line, |c| c.is_ascii_digit());
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'0' && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X') {
            let start = i;
            i += 2;
            while i < bytes.len() && (bytes[i].is_ascii_hexdigit() || bytes[i] == b'_') {
                i += 1;
            }
            out.push((start, i));
            continue;
        }
        i += 1;
    }
    // ether units like 1 ether / 100 wei
    for w in ["wei", "gwei", "szabo", "finney", "ether"] {
        let mut pos = 0;
        while let Some(idx) = line[pos..].find(w) {
            let a = pos + idx;
            let b = a + w.len();
            let before_ok = a == 0 || !line.as_bytes()[a - 1].is_ascii_alphanumeric();
            let after_ok = b >= line.len() || !line.as_bytes()[b].is_ascii_alphanumeric();
            if before_ok && after_ok {
                out.push((a, b));
            }
            pos = b;
        }
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("solidity:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "solidity:word" => {
            let mut all = Vec::new();
            for k in keywords() {
                all.push(comp(k, k, "keyword"));
            }
            for t in types() {
                all.push(comp(t, t, "type"));
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
    for marker in ["contract ", "library ", "interface ", "function ", "modifier ", "event ", "struct ", "enum "] {
        if let Some(pos) = t.find(marker) {
            let after = &t[pos + marker.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return vec![format!("{} {}", marker.trim(), name)];
            }
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["pragma", "contract", "library", "interface", "function", "modifier", "event", "struct", "enum", "mapping", "constructor", "if", "else", "for", "while", "do", "break", "continue", "return", "returns", "emit", "require", "assert", "revert", "new", "delete", "import", "as", "from", "using", "for", "assembly", "unchecked", "try", "catch", "virtual", "override", "abstract"]
}

fn types() -> Vec<&'static str> {
    vec!["bool", "string", "bytes", "address", "uint", "uint8", "uint16", "uint32", "uint64", "uint128", "uint256", "int", "int8", "int256", "bytes32", "memory", "storage", "calldata", "payable"]
}

fn modifiers() -> Vec<&'static str> {
    vec!["public", "private", "external", "internal", "pure", "view", "payable", "constant", "immutable", "indexed", "anonymous", "virtual", "override"]
}
