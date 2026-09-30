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
    // blocks: server { / location / foo {
    let t = line.trim();
    if t.ends_with('{') {
        let name = t.trim_end_matches('{').trim();
        if !name.is_empty() {
            if let Some(a) = line.find(name) {
                s.push(Segment { start: a, end: a + name.len(), color: CYAN });
            }
        }
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if directives().contains(&w) {
            s.push(Segment { start: a, end: b, color: BLUE });
        } else if matches!(w, "on" | "off") {
            s.push(Segment { start: a, end: b, color: PURPLE });
        }
    }
    for (a, b) in variable_ranges(line) {
        s.push(Segment { start: a, end: b, color: RED });
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

pub(crate) fn variable_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            if i > start + 1 {
                out.push((start, i));
            }
        } else {
            i += 1;
        }
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("nginx:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, _ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "nginx:word" => directives().iter().map(|d| comp(d, d, "directive")).collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let t = line.trim();
    if t.ends_with('{') {
        let name = t.trim_end_matches('{').trim();
        if !name.is_empty() {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}

fn directives() -> Vec<&'static str> {
    vec!["server", "location", "listen", "server_name", "root", "index", "try_files", "proxy_pass", "proxy_set_header", "add_header", "ssl_certificate", "ssl_certificate_key", "return", "rewrite", "allow", "deny", "gzip", "gzip_types", "expires", "access_log", "error_log", "include", "user", "worker_processes", "events", "http", "upstream", "keepalive_timeout", "client_max_body_size", "auth_basic"]
}
