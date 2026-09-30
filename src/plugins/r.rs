use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = hash_comment_start(line, &strings) {
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

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("r:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "r:word" => {
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
    // `name <- function(...)` or `name = function(...)`
    let t = line.trim();
    if t.contains("function") {
        let name_part = t.split("function").next().unwrap_or("");
        let name: String = name_part
            .trim_end_matches(|c: char| c == '-' || c == '<' || c == '=' || c.is_whitespace())
            .split_whitespace()
            .last()
            .unwrap_or("")
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
            .collect();
        if !name.is_empty() {
            return vec![name];
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["if", "else", "for", "in", "while", "repeat", "function", "return", "next", "break", "TRUE", "FALSE", "NULL", "NA", "Inf", "NaN", "library", "require"]
}

fn builtins() -> Vec<&'static str> {
    vec!["c", "list", "data.frame", "matrix", "vector", "factor", "library", "require", "source", "print", "cat", "paste", "paste0", "sprintf", "lapply", "sapply", "apply", "tapply", "mapply", "mean", "sum", "min", "max", "length", "nrow", "ncol", "names", "colnames", "rownames", "head", "tail", "str", "summary", "plot", "ggplot", "lm", "glm", "read.csv", "write.csv"]
}

fn constants() -> Vec<&'static str> {
    vec!["TRUE", "FALSE", "NULL", "NA", "NA_integer_", "NA_real_", "NA_character_", "Inf", "NaN"]
}
