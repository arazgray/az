use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, YELLOW, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_between, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    for (a, b) in find_between(line, "{#", "#}") {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: COMMENT });
        }
    }
    for (a, b) in find_between(line, "{%", "%}") {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: YELLOW });
        }
    }
    for (a, b) in find_between(line, "{{", "}}") {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: YELLOW });
        }
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if types().contains(&w) {
            s.push(Segment { start: a, end: b, color: CYAN });
        } else if builtins().contains(&w) {
            s.push(Segment { start: a, end: b, color: CYAN });
        } else if constants().contains(&w) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        } else if constants().contains(&w) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        } else if after_nonspace_is(line, b, '(') {
            s.push(Segment { start: a, end: b, color: BLUE });
        } else if w.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
            s.push(Segment { start: a, end: b, color: YELLOW });
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
            return Some(("twig:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "twig:word" => {
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
    if let Some(pos) = line.find("{%") {
        let words: Vec<&str> = line[pos + 2..].split_whitespace().collect();
        if words.len() >= 2 && matches!(words[0], "block" | "macro" | "embed") {
            let name: String = words[1].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return vec![format!("{} {}", words[0], name)];
            }
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec![]
}

fn types() -> Vec<&'static str> {
    vec!["and", "as", "constant", "defined", "divisibleby", "empty", "even", "in", "is", "iterable", "not", "odd", "or", "with"]
}

fn builtins() -> Vec<&'static str> {
    vec!["abs", "batch", "capitalize", "convert", "encoding", "default", "escape", "first", "format", "join", "json_encode", "keys", "last", "length", "lower", "merge", "nl2br", "number_format", "raw", "replace", "reverse", "round", "slice", "sort", "split", "striptags", "title", "trim", "upper", "url_encode", "attribute", "block", "cycle", "date", "dump", "include", "max", "min", "parent", "random", "range", "source", "template_from_string"]
}

fn constants() -> Vec<&'static str> {
    vec!["false", "null", "true"]
}
