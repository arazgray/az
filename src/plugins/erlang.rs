use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, YELLOW, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = percent_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
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

fn percent_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    for (i, c) in line.char_indices() {
        if c == '%' && !pos_in_ranges(i, strings) {
            return Some(i);
        }
    }
    None
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("erlang:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "erlang:word" => {
            let mut all = Vec::new();
            for k in keywords() {
                all.push(comp(k, k, "keyword"));
            }
            for b in builtins() {
                all.push(comp(b, b, "bif"));
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
    if t.starts_with("-module") || t.starts_with("-export") || t.starts_with("-record") {
        return vec![t.trim_end_matches('.').to_string()];
    }
    // name(...) -> clauses look like functions.
    if let Some(paren) = t.find('(') {
        let head = t[..paren].trim();
        if !head.is_empty() && head.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) && head.chars().all(|c| c.is_alphanumeric() || c == '_') {
            let name: String = head.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() && !keywords().contains(&name.as_str()) {
                return vec![name];
            }
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["module", "export", "record", "case", "of", "end", "fun", "if", "receive", "after", "try", "catch", "begin", "query", "when", "and", "or", "not", "xor", "div", "rem", "band", "bor", "bxor", "bsl", "bsr"]
}

fn builtins() -> Vec<&'static str> {
    vec!["spawn", "send", "self", "whereis", "register", "length", "hd", "tl", "list_to_atom", "atom_to_list", "integer_to_list", "list_to_integer", "tuple_to_list", "list_to_tuple", "apply", "error", "exit", "throw", "make_ref", "node", "nodes", "monitor", "link", "unlink"]
}

fn constants() -> Vec<&'static str> {
    vec!["true", "false", "undefined", "ok", "error"]
}
