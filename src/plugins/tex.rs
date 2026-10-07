use crate::{COMMENT, CYAN, GREEN, MAGENTA, ORANGE, PURPLE, Segment, CompletionItem};
use super::{comp, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = percent_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in command_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: PURPLE });
        }
    }
    for (a, b) in brace_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: CYAN });
        }
    }
    for (a, b) in math_ranges(line) {
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

fn percent_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    for (i, c) in line.char_indices() {
        if c == '%' && !pos_in_ranges(i, strings) {
            if i == 0 || line.as_bytes()[i - 1] != b'\\' {
                return Some(i);
            }
        }
    }
    None
}

fn command_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            let start = i;
            i += 1;
            if i < bytes.len() && !bytes[i].is_ascii_alphabetic() {
                i += 1;
                out.push((start, i));
                continue;
            }
            while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
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

fn brace_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let start = i;
            let mut depth = 1;
            i += 1;
            while i < bytes.len() && depth > 0 {
                if bytes[i] == b'{' {
                    depth += 1;
                } else if bytes[i] == b'}' {
                    depth -= 1;
                }
                i += 1;
            }
            out.push((start, i));
        } else {
            i += 1;
        }
    }
    out
}

fn math_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i;
            i += 1;
            if i < bytes.len() && bytes[i] == b'$' {
                i += 1;
            }
            while i < bytes.len() && bytes[i] != b'$' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
                if i < bytes.len() && bytes[i - 2] == b'$' && bytes[i - 1] == b'$' {
                    // already consumed
                }
                out.push((start, i));
            } else {
                out.push((start, i));
            }
        } else {
            i += 1;
        }
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if before.rfind('\\').is_some() {
        if let Some((prefix, start)) = word_suffix(before) {
            if explicit || !prefix.is_empty() {
                return Some(("tex:cmd".to_string(), prefix, start));
            }
        }
    }
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("tex:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "tex:cmd" | "tex:word" => {
            let mut all = Vec::new();
            for k in commands() {
                all.push(comp(k, k, "command"));
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
    for cmd in ["\\section", "\\subsection", "\\subsubsection", "\\chapter", "\\part", "\\paragraph"] {
        if let Some(pos) = t.find(cmd) {
            let after = &t[pos + cmd.len()..];
            let name = after.trim().trim_start_matches('{').chars().take_while(|c| *c != '}').collect::<String>();
            if !name.is_empty() {
                return vec![format!("{} {}", cmd.trim_start_matches('\\'), name)];
            }
        }
    }
    if let Some(pos) = t.find("\\label{") {
        let after = &t[pos + "\\label{".len()..];
        let name: String = after.chars().take_while(|c| *c != '}').collect();
        if !name.is_empty() {
            return vec![format!("label {name}")];
        }
    }
    let _ = find_words(line);
    Vec::new()
}

fn commands() -> Vec<&'static str> {
    vec!["section", "subsection", "subsubsection", "chapter", "part", "paragraph", "label", "ref", "cite", "usepackage", "documentclass", "begin", "end", "item", "textbf", "textit", "texttt", "emph", "frac", "sqrt", "sum", "int", "figure", "table", "caption", "includegraphics"]
}
