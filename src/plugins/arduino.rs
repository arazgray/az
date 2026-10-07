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
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: COMMENT });
        }
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

fn line_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        for m in ["//"].iter() {
            let mb = m.as_bytes();
            if i + mb.len() <= bytes.len() && &bytes[i..i + mb.len()] == mb && !pos_in_ranges(i, strings) {
                if *m == "//" && i > 0 && bytes[i - 1] == b':' {
                    break;
                }
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("arduino:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "arduino:word" => {
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
    if t.starts_with('#') || t.starts_with("//") {
        return Vec::new();
    }
    let Some(paren) = t.find('(') else { return Vec::new(); };
    if !t[paren..].contains(')') {
        return Vec::new();
    }
    let head = t[..paren].trim_end();
    if head.contains(['=', ';']) {
        return Vec::new();
    }
    if let Some(last) = head.split_whitespace().last() {
        if last.chars().all(|c| c.is_alphanumeric() || c == '_')
            && last.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false)
            && !matches!(last, "if" | "for" | "while" | "switch" | "return" | "catch" | "foreach" | "using" | "lock" | "sizeof" | "typeof")
        {
            return vec![format!("fn {last}")];
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["case", "class", "default", "do", "double", "else", "for", "if", "new", "private", "protected", "public", "short", "signed", "static", "String", "switch", "this", "throw", "try", "unsigned", "void", "while", "goto", "continue", "break", "return"]
}

fn types() -> Vec<&'static str> {
    vec!["boolean", "byte", "char", "float", "int", "long", "word"]
}

fn builtins() -> Vec<&'static str> {
    vec!["HIGH", "LOW", "INPUT", "OUTPUT", "DEC", "BIN", "HEX", "OCT", "BYTE", "PI", "HALF_PI", "TWO_PI", "LSBFIRST", "MSBFIRST", "CHANGE", "FALLING", "RISING", "DEFAULT", "EXTERNAL", "INTERNAL", "INTERNAL1V1", "INTERNAL2V56", "abs", "acos", "asin", "atan", "atan2", "ceil", "constrain", "cos", "degrees", "exp", "floor", "log", "map", "max", "min", "radians", "random", "randomSeed", "round", "sin", "sq", "sqrt", "tan", "bitRead", "bitWrite", "bitSet", "bitClear", "bit", "highByte", "lowByte", "analogReference", "analogRead", "analogWrite", "attachInterrupt", "detachInterrupt", "delay", "delayMicroseconds", "millis"]
}

fn constants() -> Vec<&'static str> {
    vec!["false", "null", "true"]
}
