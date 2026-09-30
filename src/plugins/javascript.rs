use crate::{BLUE, COMMENT, CYAN, GREEN, MAGENTA, ORANGE, PURPLE, YELLOW, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_prefixed_words, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let templates = template_ranges(line);
    let strings = string_ranges(line);
    for (a, b) in &templates { s.push(Segment { start: *a, end: *b, color: GREEN }); }
    for (a, b) in &strings { s.push(Segment { start: *a, end: *b, color: GREEN }); }
    for (a, b) in regex_ranges(line) { s.push(Segment { start: a, end: b, color: ORANGE }); }
    for (a, b) in find_prefixed_words(line, '@') { s.push(Segment { start: a, end: b, color: MAGENTA }); }
    if let Some(pos) = line_comment_start(line, &strings, &templates) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    if let Some((a, b)) = block_comment_range(line) { s.push(Segment { start: a, end: b, color: COMMENT }); }
    for (a, b) in find_words(line) {
        let w = &line[a..b];
        let color = if keywords().contains(&w) { PURPLE }
            else if builtins().contains(&w) { CYAN }
            else if constants().contains(&w) { ORANGE }
            else if after_nonspace_is(line, b, '(') { BLUE }
            else if w.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) { YELLOW }
            else { continue; };
        s.push(Segment { start: a, end: b, color });
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) { s.push(Segment { start: a, end: b, color: ORANGE }); }
    s
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if before.ends_with('.') { return Some(("javascript:member".to_string(), String::new(), before.len())); }
    if let Some(pos) = before.rfind('.') {
        let after = &before[pos + 1..];
        if !after.is_empty() && after.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
            return Some(("javascript:member".to_string(), after.to_string(), pos + 1));
        }
    }
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 { return Some(("javascript:word".to_string(), prefix, start)); }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "javascript:member" => members().iter().map(|s| comp(s, s, "member")).collect(),
        "javascript:word" => {
            let mut all = Vec::new();
            for s in keywords() { all.push(comp(s, s, "keyword")); }
            for s in builtins() { all.push(comp(s, s, "builtin")); }
            for s in snippets() { all.push(comp(s.0, s.1, "snippet")); }
            for s in document_symbols(ctx.lines, ctx.scan_limit) { all.push(comp(&s, &s, "symbol")); }
            all
        }
        _ => Vec::new(),
    }
}

pub(crate) fn document_symbols(lines: &[String], limit: usize) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines.iter().take(limit) { out.extend(symbols(line)); }
    out.sort();
    out.dedup();
    out
}

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let trimmed = line.trim_start();
    let mut out = Vec::new();
    for marker in ["function ", "class "] {
        if let Some(pos) = trimmed.find(marker) {
            let after = &trimmed[pos + marker.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$').collect();
            if !name.is_empty() { out.push(format!("{} {}", marker.trim(), name)); }
        }
    }
    for marker in ["const ", "let ", "var "] {
        if let Some(pos) = trimmed.find(marker) {
            let after = &trimmed[pos + marker.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$').collect();
            if !name.is_empty() && (trimmed.contains("=>") || trimmed.contains("function")) {
                out.push(format!("{}()", name));
            }
        }
    }
    out
}

pub(crate) fn looks_like_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("import ") || t.starts_with("export ") || t.starts_with("const ") || t.starts_with("let ") ||
    t.starts_with("var ") || t.starts_with("async ") || t.starts_with("await ") || t.starts_with("function ") ||
    t.contains("=>") || t.contains("console.") || t.contains("document.") || t.contains("window.") ||
    t.contains("addEventListener")
}

pub(crate) fn looks_like_context(before: &str) -> bool {
    let t = before.trim_start();
    before.contains("<script") || t.starts_with("import ") || t.starts_with("export ") || t.starts_with("const ") ||
    t.starts_with("let ") || t.starts_with("var ") || before.contains("=>") || before.contains("console.") ||
    before.contains("document.") || before.contains("window.") || before.ends_with(",") || before.ends_with(".")
}

fn block_comment_range(line: &str) -> Option<(usize, usize)> {
    let start = line.find("/*")?;
    let end = line[start + 2..].find("*/").map(|p| start + 2 + p + 2).unwrap_or(line.len());
    Some((start, end))
}

fn template_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' { i += 2; continue; }
                if bytes[i] == b'`' { i += 1; break; }
                i += 1;
            }
            out.push((start, i.min(bytes.len())));
        } else { i += 1; }
    }
    out
}

fn regex_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] != b'/' && bytes[i + 1] != b'*' && likely_regex_start(line, i) {
            let start = i;
            i += 1;
            let mut in_class = false;
            while i < bytes.len() {
                if bytes[i] == b'\\' { i += 2; continue; }
                if bytes[i] == b'[' { in_class = true; }
                if bytes[i] == b']' { in_class = false; }
                if bytes[i] == b'/' && !in_class {
                    i += 1;
                    while i < bytes.len() && bytes[i].is_ascii_alphabetic() { i += 1; }
                    out.push((start, i));
                    break;
                }
                i += 1;
            }
        } else { i += 1; }
    }
    out
}

fn likely_regex_start(line: &str, slash: usize) -> bool {
    let before = line[..slash].trim_end();
    before.is_empty() || before.ends_with('(') || before.ends_with('=') || before.ends_with(':') || before.ends_with(',') || before.ends_with("return")
}

/// First `//` outside strings/templates, skipping `://` protocols.
fn line_comment_start(line: &str, strings: &[(usize, usize)], templates: &[(usize, usize)]) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'/' {
            if pos_in_ranges(i, strings) || pos_in_ranges(i, templates) {
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

fn keywords() -> Vec<&'static str> { vec!["as","async","await","break","case","catch","class","const","continue","debugger","default","delete","do","else","export","extends","finally","for","from","function","get","if","import","in","instanceof","let","new","of","return","set","static","super","switch","this","throw","try","typeof","var","void","while","with","yield"] }
fn builtins() -> Vec<&'static str> { vec!["Array","Boolean","Date","Error","JSON","Map","Math","Number","Object","Promise","Reflect","RegExp","Set","String","Symbol","WeakMap","WeakSet","console","document","fetch","globalThis","localStorage","navigator","performance","sessionStorage","window"] }
fn constants() -> Vec<&'static str> { vec!["true","false","null","undefined","NaN","Infinity"] }
fn members() -> Vec<&'static str> { vec!["add","addEventListener","append","appendChild","catch","classList","closest","dataset","filter","find","forEach","getAttribute","includes","join","length","map","match","preventDefault","push","querySelector","querySelectorAll","reduce","remove","removeAttribute","removeEventListener","replace","setAttribute","slice","split","then","toLowerCase","toString","toUpperCase","trim","value"] }
fn snippets() -> Vec<(&'static str, &'static str)> { vec![("cl", "console.log()"), ("fn", "function ()"), ("afn", "async function ()"), ("forof", "for (const item of items)"), ("qs", "document.querySelector()"), ("qsa", "document.querySelectorAll()")] }
