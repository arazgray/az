use crate::{BLUE, COMMENT, CYAN, GREEN, MAGENTA, ORANGE, Segment, CompletionItem};
use super::{comp, find_between, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = hash_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    // <VirtualHost>, <Directory>, </..>
    for (a, b) in section_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: MAGENTA });
        }
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if directives().contains(&w) {
            s.push(Segment { start: a, end: b, color: BLUE });
        } else if matches!(w, "On" | "Off") {
            s.push(Segment { start: a, end: b, color: CYAN });
        }
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    let _ = find_between;
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

fn section_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let start = i;
            while i < bytes.len() && bytes[i] != b'>' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
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
            return Some(("apache:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, _ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "apache:word" => directives().iter().map(|d| comp(d, d, "directive")).collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let t = line.trim();
    if t.starts_with('<') && t.ends_with('>') && !t.starts_with("</") {
        return vec![t.to_string()];
    }
    Vec::new()
}

fn directives() -> Vec<&'static str> {
    vec!["VirtualHost", "Directory", "DocumentRoot", "ServerName", "ServerAlias", "DirectoryIndex", "AllowOverride", "Require", "Options", "Allow", "Deny", "Order", "RewriteEngine", "RewriteRule", "RewriteCond", "ProxyPass", "ProxyPassReverse", "SSLEngine", "SSLCertificateFile", "SSLCertificateKeyFile", "ErrorLog", "CustomLog", "LogLevel", "AddType", "LoadModule"]
}
