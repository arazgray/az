use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, YELLOW, Segment, CompletionItem};
use super::{comp, find_between, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    for (a, b) in find_between(line, "<!--", "-->") {
        s.push(Segment { start: a, end: b, color: COMMENT });
    }
    if let Some(pos) = line_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in block_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: YELLOW });
        }
    }
    for (a, b) in tag_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if keywords().contains(&w) {
            s.push(Segment { start: a, end: b, color: PURPLE });
        } else if directives().iter().any(|d| *d == w) {
            s.push(Segment { start: a, end: b, color: CYAN });
        } else if is_attr(line, a, b) {
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
            let t = line.trim_start();
            if t.starts_with("//") || line.to_ascii_lowercase().contains("<script") {
                return Some(i);
            }
            return None;
        }
        i += 1;
    }
    None
}

fn block_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for open in ["{#", "{@", "{:", "{/"] {
        let mut pos = 0;
        while let Some(a) = line[pos..].find(open) {
            let start = pos + a;
            if let Some(b) = line[start + 2..].find('}') {
                let end = start + 2 + b + 1;
                out.push((start, end));
                pos = end;
            } else {
                out.push((start, line.len()));
                break;
            }
        }
    }
    let mut pos = 0;
    while let Some(a) = line[pos..].find('{') {
        let start = pos + a;
        if out.iter().any(|(s, e)| start >= *s && start < *e) {
            pos = start + 1;
            continue;
        }
        if let Some(b) = line[start + 1..].find('}') {
            let end = start + 1 + b + 1;
            out.push((start, end));
            pos = end;
        } else {
            break;
        }
    }
    out
}

fn tag_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' && i + 1 < bytes.len() && (bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'/') {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i] != b'>' {
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

fn is_attr(line: &str, _a: usize, b: usize) -> bool {
    let after = line[b..].chars().find(|c| !c.is_whitespace());
    matches!(after, Some('='))
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("svelte:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "svelte:word" => {
            let mut all = Vec::new();
            for k in keywords() {
                all.push(comp(k, k, "keyword"));
            }
            for d in directives() {
                all.push(comp(d, d, "directive"));
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
    for marker in ["function ", "let ", "export "] {
        if let Some(pos) = t.find(marker) {
            let after = &t[pos + marker.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return vec![format!("{} {}", marker.trim(), name)];
            }
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["script", "style", "export", "let", "function", "import", "from", "if", "else", "each", "await", "then", "catch", "key", "svelte", "props"]
}

fn directives() -> Vec<&'static str> {
    vec!["on:click", "on:input", "bind:value", "bind:checked", "use:action", "transition:fade", "in:fly", "out:fade", "animate:flip", "class:active"]
}
