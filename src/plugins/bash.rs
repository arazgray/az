use crate::{BLUE, COMMENT, CYAN, GREEN, MAGENTA, ORANGE, PURPLE, RED, Segment, CompletionItem};
use super::{comp, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if line.trim_start().starts_with("#!") {
        s.push(Segment { start: 0, end: line.len(), color: MAGENTA });
        return s;
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
        } else if w.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) && w.len() > 1 {
            // ENV_VAR style
            s.push(Segment { start: a, end: b, color: BLUE });
        }
    }
    for (a, b) in variable_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: RED });
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

pub(crate) fn variable_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i;
            i += 1;
            if i < bytes.len() && bytes[i] == b'{' {
                i += 1;
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
    if let Some(idx) = before.rfind('$') {
        let after = &before[idx + 1..];
        let clean = after.trim_start_matches('{');
        if clean.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Some(("bash:var".to_string(), before[idx..].to_string(), idx));
        }
    }
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("bash:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "bash:var" => {
            let mut vars: Vec<String> = vec!["$HOME", "$PATH", "$USER", "$PWD", "$OLDPWD", "$SHELL", "$EDITOR", "$1", "$2", "$@"].into_iter().map(str::to_string).collect();
            for line in ctx.lines.iter().take(ctx.scan_limit) {
                for (a, b) in variable_ranges(line) {
                    vars.push(line[a..b].to_string());
                }
            }
            let mut out: Vec<CompletionItem> = vars.into_iter().map(|v| comp(&v, &v, "variable")).collect();
            out.sort_by(|a, b| a.label.cmp(&b.label));
            out.dedup_by(|a, b| a.label == b.label);
            out
        }
        "bash:word" => {
            let mut all = Vec::new();
            for k in keywords() {
                all.push(comp(k, k, "keyword"));
            }
            for b in builtins() {
                all.push(comp(b, b, "builtin"));
            }
            all
        }
        _ => Vec::new(),
    }
}

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let t = line.trim_start();
    // func foo() {  /  foo() {
    for pat in ["function "] {
        if let Some(pos) = t.find(pat) {
            let after = &t[pos + pat.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return vec![format!("fn {}", name)];
            }
        }
    }
    if let Some(pos) = t.find("()") {
        let before = t[..pos].trim_end();
        if let Some(name) = before.split_whitespace().last() {
            if !name.is_empty() {
                return vec![format!("fn {}", name)];
            }
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done", "case", "esac", "in", "function", "select", "time", "return", "exit", "break", "continue", "export", "local", "readonly", "declare", "set", "unset", "shift", "trap", "exec", "eval"]
}

fn builtins() -> Vec<&'static str> {
    vec!["echo", "printf", "cd", "pwd", "ls", "cp", "mv", "rm", "mkdir", "cat", "grep", "sed", "awk", "cut", "sort", "uniq", "head", "tail", "find", "xargs", "chmod", "chown", "tar", "curl", "wget", "ssh", "scp", "git", "cargo", "sudo", "test"]
}
