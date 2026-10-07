use crate::{BLUE, COMMENT, CYAN, GREEN, MAGENTA, ORANGE, PURPLE, Segment, CompletionItem};
use super::{comp, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in directive_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: MAGENTA });
        }
    }
    for (a, b) in register_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: CYAN });
        }
    }
    // Labels `name:` at line start.
    let t = line.trim_start();
    if !t.is_empty() && !t.starts_with([';', '#', '.', '"', '\'']) {
        if let Some(colon) = t.find(':') {
            let head = &t[..colon];
            if !head.is_empty() && !head.contains([' ', '\t']) {
                let indent = line.len() - t.len();
                if !pos_in_ranges(indent, &strings) {
                    s.push(Segment { start: indent, end: indent + colon, color: BLUE });
                }
            }
        }
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if instructions().iter().any(|i| i.eq_ignore_ascii_case(w)) {
            s.push(Segment { start: a, end: b, color: PURPLE });
        }
    }
    for (a, b) in number_ranges(line) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    s
}

fn comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    for (i, c) in line.char_indices() {
        if (c == ';' || c == '#') && !pos_in_ranges(i, strings) {
            return Some(i);
        }
        if c == '/' && line[i..].starts_with("//") && !pos_in_ranges(i, strings) {
            return Some(i);
        }
    }
    None
}

fn directive_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
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

fn register_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' || bytes[i] == b'$' {
            let start = i;
            i += 1;
            let mut end = i;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric()) {
                end += 1;
            }
            if end > start + 1 {
                out.push((start, end));
                i = end;
                continue;
            }
        }
        i += 1;
    }
    // Bare x86 registers without prefix.
    for (a, b) in find_words(line) {
        let w = &line[a..b].to_ascii_lowercase();
        if ["eax", "ebx", "ecx", "edx", "esi", "edi", "esp", "ebp", "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rsp", "rbp", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"].contains(&w.as_str()) {
            out.push((a, b));
        }
    }
    out
}

fn number_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = scan_ranges(line, |c| c.is_ascii_digit());
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'0' && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X') {
            let start = i;
            i += 2;
            while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                i += 1;
            }
            out.push((start, i));
            continue;
        }
        i += 1;
    }
    out
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("asm:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "asm:word" => {
            let mut all = Vec::new();
            for ins in instructions() {
                all.push(comp(ins, ins, "instruction"));
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
    if t.is_empty() || t.starts_with([';', '#', '.', '"', '\'']) {
        return Vec::new();
    }
    if let Some(colon) = t.find(':') {
        let head = t[..colon].trim();
        if !head.is_empty() && !head.contains([' ', '\t']) {
            return vec![format!("label {head}")];
        }
    }
    Vec::new()
}

fn instructions() -> Vec<&'static str> {
    vec!["mov", "add", "sub", "mul", "imul", "div", "idiv", "and", "or", "xor", "not", "neg", "shl", "shr", "sal", "sar", "rol", "ror", "cmp", "test", "jmp", "je", "jne", "jg", "jge", "jl", "jle", "ja", "jb", "call", "ret", "push", "pop", "lea", "nop", "hlt", "int", "syscall", "leave", "enter", "loop"]
}
