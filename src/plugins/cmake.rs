use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, RED, Segment, CompletionItem};
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
    for (a, b) in variable_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: RED });
        }
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        let lw = w.to_ascii_lowercase();
        if commands().iter().any(|c| *c == lw.as_str()) {
            s.push(Segment { start: a, end: b, color: PURPLE });
        } else if types().contains(&w) {
            s.push(Segment { start: a, end: b, color: CYAN });
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

fn variable_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            let start = i;
            i += 2;
            while i < bytes.len() && bytes[i] != b'}' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            out.push((start, i));
        } else {
            i += 1;
        }
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("cmake:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "cmake:word" => {
            let mut all = Vec::new();
            for k in commands() {
                all.push(comp(k, k, "command"));
            }
            for t in types() {
                all.push(comp(t, t, "constant"));
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
    let t = line.trim_start().to_ascii_lowercase();
    for marker in ["function(", "macro(", "project("] {
        if let Some(pos) = t.find(marker) {
            let after = &t[pos + marker.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return vec![format!("{} {}", marker.trim_end_matches('('), name)];
            }
        }
    }
    Vec::new()
}

fn commands() -> Vec<&'static str> {
    vec!["project", "cmake_minimum_required", "add_executable", "add_library", "target_link_libraries", "target_include_directories", "include_directories", "find_package", "set", "option", "if", "elseif", "else", "endif", "foreach", "endforeach", "while", "endwhile", "function", "endfunction", "macro", "endmacro", "return", "message", "install", "enable_testing", "add_test", "add_subdirectory", "include", "list", "string", "file", "execute_process", "configure_file"]
}

fn types() -> Vec<&'static str> {
    vec!["ON", "OFF", "TRUE", "FALSE", "CACHE", "PARENT_SCOPE", "REQUIRED", "QUIET", "PUBLIC", "PRIVATE", "INTERFACE"]
}
