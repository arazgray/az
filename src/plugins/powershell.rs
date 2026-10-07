use crate::{BLUE, COMMENT, CYAN, GREEN, MAGENTA, ORANGE, PURPLE, RED, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_between, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = hash_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in find_between(line, "<#", "#>") {
        s.push(Segment { start: a, end: b, color: COMMENT });
    }
    for (a, b) in variable_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: RED });
        }
    }
    for (a, b) in cmdlet_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: BLUE });
        }
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
    // Attributes like [CmdletBinding()]
    for (a, b) in attr_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: MAGENTA });
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
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b':') {
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

fn cmdlet_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (a, b) in find_words(line) {
        let w = &line[a..b];
        if let Some(dash) = w.find('-') {
            let (verb, noun) = (&w[..dash], &w[dash + 1..]);
            if !verb.is_empty() && !noun.is_empty() && verb.chars().all(|c| c.is_ascii_alphabetic()) && noun.chars().all(|c| c.is_ascii_alphanumeric()) {
                out.push((a, b));
            }
        }
    }
    out
}

fn attr_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if let Some(end) = line[i..].find(']') {
                out.push((i, i + end + 1));
                i += end + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("powershell:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "powershell:word" => {
            let mut all = Vec::new();
            for k in keywords() {
                all.push(comp(k, k, "keyword"));
            }
            for b in builtins() {
                all.push(comp(b, b, "cmdlet"));
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
    if let Some(pos) = t.find("function ") {
        let after = &t[pos + "function ".len()..];
        let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();
        if !name.is_empty() {
            return vec![format!("function {name}")];
        }
    }
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec!["function", "if", "else", "elseif", "foreach", "for", "while", "do", "until", "switch", "try", "catch", "finally", "throw", "return", "break", "continue", "param", "begin", "process", "end", "class", "enum", "using", "namespace", "in", "filter", "trap", "data", "from", "dynamicparam"]
}

fn builtins() -> Vec<&'static str> {
    vec!["Get-ChildItem", "Set-Location", "Get-Content", "Set-Content", "Write-Host", "Write-Output", "Get-Item", "New-Item", "Remove-Item", "Copy-Item", "Move-Item", "Invoke-Expression", "Invoke-Command", "Start-Process", "Get-Process", "Stop-Process", "Get-Service", "Import-Module", "Export-ModuleMember", "Test-Path", "Join-Path", "Split-Path", "Select-Object", "Where-Object", "ForEach-Object", "Sort-Object", "Measure-Object"]
}

fn constants() -> Vec<&'static str> {
    vec!["true", "false", "null"]
}
