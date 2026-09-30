use crate::{COMMENT, GREEN, PURPLE, RED, Segment, CompletionItem};
use super::{comp, pos_in_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = hash_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
        return s;
    }
    let t = line.trim_start();
    if let Some(first) = t.split_whitespace().next() {
        if instructions().contains(&first) {
            let indent = line.len() - t.len();
            s.push(Segment { start: indent, end: indent + first.len(), color: PURPLE });
        }
    }
    for (a, b) in variable_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: RED });
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
        if bytes[i] == b'$' {
            let start = i;
            i += 1;
            if i < bytes.len() && bytes[i] == b'{' {
                while i < bytes.len() && bytes[i] != b'}' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                out.push((start, i));
            } else {
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                if i > start + 1 {
                    out.push((start, i));
                }
            }
        } else {
            i += 1;
        }
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    let t = before.trim_start();
    if t.is_empty() || !t.contains(' ') {
        if let Some((prefix, start)) = word_suffix(before) {
            if explicit || !prefix.is_empty() {
                let up = prefix.to_ascii_uppercase();
                if instructions().iter().any(|i| i.starts_with(up.as_str())) {
                    return Some(("dockerfile:instr".to_string(), prefix, start));
                }
            }
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, _ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "dockerfile:instr" => instructions().iter().map(|i| comp(i, i, "instruction")).collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn symbols(_line: &str) -> Vec<String> {
    Vec::new()
}

fn instructions() -> Vec<&'static str> {
    vec!["FROM", "RUN", "CMD", "LABEL", "EXPOSE", "ENV", "ADD", "COPY", "ENTRYPOINT", "VOLUME", "USER", "WORKDIR", "ARG", "ONBUILD", "STOPSIGNAL", "HEALTHCHECK", "SHELL"]
}
