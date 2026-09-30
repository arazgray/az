use crate::{BLUE, COMMENT, CYAN, GREEN, MAGENTA, ORANGE, PURPLE, YELLOW, Segment, CompletionItem};
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
    // preprocessor #import / #define and directives @interface / @end
    let t = line.trim_start();
    if t.starts_with('#') {
        let off = line.len() - t.len();
        s.push(Segment { start: off, end: line.len(), color: MAGENTA });
        return s;
    }
    if t.starts_with('@') {
        let off = line.len() - t.len();
        let mut end = off + 1;
        for (i, c) in t[1..].char_indices() {
            if c.is_alphanumeric() || c == '_' {
                end = off + 1 + i + c.len_utf8();
            } else {
                break;
            }
        }
        s.push(Segment { start: off, end, color: MAGENTA });
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
        } else if after_nonspace_is(line, b, '(') || after_nonspace_is(line, b, ':') {
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

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("objc:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "objc:word" => {
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
    for marker in ["@interface ", "@implementation ", "@protocol "] {
        if t.starts_with(marker) {
            let name: String = t[marker.len()..].trim().chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return vec![format!("{} {}", marker.trim(), name)];
            }
        }
    }
    // `- (void)doThing` / `+ (id)shared`
    if t.starts_with("- ") || t.starts_with('+') || t.starts_with("-(") || t.starts_with("+(") {
        if let Some(pos) = t.find(')') {
            let after = t[pos + 1..].trim();
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':').collect();
            if !name.is_empty() {
                return vec![name];
            }
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["auto", "break", "case", "const", "continue", "default", "do", "else", "enum", "extern", "for", "goto", "if", "inline", "register", "restrict", "return", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "in", "out", "inout", "bycopy", "byref", "oneway", "self", "super", "id", "Class", "SEL", "IMP", "BOOL", "nil", "Nil", "YES", "NO", "atomic", "nonatomic", "strong", "weak", "copy", "readonly", "readwrite", "interface", "implementation", "protocol", "end", "property", "synthesize", "dynamic", "autoreleasepool", "try", "catch", "finally", "throw", "synchronized"]
}

fn types() -> Vec<&'static str> {
    vec!["int", "long", "short", "char", "float", "double", "void", "unsigned", "signed", "size_t", "bool", "NSString", "NSArray", "NSDictionary", "NSObject", "NSNumber", "NSData", "NSDate", "NSError", "UIView", "id"]
}

fn constants() -> Vec<&'static str> {
    vec!["nil", "Nil", "YES", "NO", "true", "false", "NULL", "self", "super"]
}
