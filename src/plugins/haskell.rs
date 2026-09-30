use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_between, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = dash_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in find_between(line, "{-", "-}") {
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

fn dash_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'-' {
            if pos_in_ranges(i, strings) {
                i += 2;
                continue;
            }
            return Some(i);
        }
        i += 1;
    }
    None
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("haskell:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "haskell:word" => {
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
    let t = line.trim();
    if t.is_empty() || t.starts_with("--") {
        return Vec::new();
    }
    // `name :: Type` signature
    if let Some(pos) = t.find("::") {
        let name = t[..pos].trim().split_whitespace().next().unwrap_or("");
        let clean: String = name.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '\'').collect();
        if !clean.is_empty() {
            return vec![clean];
        }
    }
    // `name args = ...` definition
    if let Some(pos) = t.find('=') {
        if !t.contains("==") && !t.contains("/=") && !t.contains("=>") {
            let lhs = t[..pos].trim();
            if let Some(first) = lhs.split_whitespace().next() {
                if first.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
                    let clean: String = first.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '\'').collect();
                    if !clean.is_empty() && !keywords().contains(&clean.as_str()) {
                        return vec![clean];
                    }
                }
            }
        }
    }
    // module / data / newtype
    for marker in ["module ", "data ", "newtype ", "type ", "class ", "instance "] {
        if let Some(pos) = t.find(marker) {
            let after = &t[pos + marker.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '\'').collect();
            if !name.is_empty() {
                return vec![format!("{} {}", marker.trim(), name)];
            }
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["case", "class", "data", "default", "deriving", "do", "else", "foreign", "if", "import", "in", "infix", "infixl", "infixr", "instance", "let", "module", "newtype", "of", "then", "type", "where", "qualified", "as", "hiding", "mdo", "family", "role", "stock", "anyclass"]
}

fn types() -> Vec<&'static str> {
    vec!["Int", "Integer", "Float", "Double", "Char", "Bool", "String", "Maybe", "Either", "IO", "FilePath", "Ordering", "Show", "Eq", "Ord", "Functor", "Applicative", "Monad", "Num"]
}

fn constants() -> Vec<&'static str> {
    vec!["True", "False", "Nothing", "Just", "Left", "Right"]
}
