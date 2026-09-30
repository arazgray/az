//! Language plugin facade for Az.
//!
//! Each language lives in its own file in this folder. The editor calls only
//! the small functions in this module, then this module delegates to the right
//! plugin. To add a new built in plugin:
//!
//! 1. Create `src/plugins/my_language.rs`.
//! 2. Add `pub(crate) mod my_language;` below.
//! 3. Add its extension and command word to `from_path` and `from_word`.
//! 4. Add it to `highlight_segments`, `completion_context`, `completion_items`,
//!    and `extract_symbols` if the language supports those features.
//!
//! See `example.rs` for a tiny documented skeleton.

use std::ffi::OsStr;
use std::path::Path;

use crate::{BLUE, CYAN, FG_DARK, GREEN, MAGENTA, ORANGE, PURPLE, RED, Segment, SyntaxMode, YELLOW, CompletionItem};

pub(crate) mod php;
pub(crate) mod html;
pub(crate) mod css;
pub(crate) mod javascript;
pub(crate) mod blade;
pub(crate) mod example;

#[derive(Clone, Copy)]
pub(crate) struct CompletionContext<'a> {
    pub(crate) lines: &'a [String],
    pub(crate) scan_limit: usize,
}

pub(crate) fn mode_label(mode: SyntaxMode) -> &'static str {
    match mode {
        SyntaxMode::Php => "PHP",
        SyntaxMode::Blade => "BLADE",
        SyntaxMode::Html => "HTML",
        SyntaxMode::Css => "CSS",
        SyntaxMode::JavaScript => "JS",
        SyntaxMode::Plain => "PLAIN",
    }
}

pub(crate) fn from_word(word: &str) -> Option<SyntaxMode> {
    match word.trim().to_ascii_lowercase().as_str() {
        "php" => Some(SyntaxMode::Php),
        "blade" => Some(SyntaxMode::Blade),
        "html" => Some(SyntaxMode::Html),
        "css" => Some(SyntaxMode::Css),
        "js" | "javascript" | "mjs" | "cjs" | "jsx" => Some(SyntaxMode::JavaScript),
        "plain" | "text" | "txt" => Some(SyntaxMode::Plain),
        _ => None,
    }
}

pub(crate) fn from_path(path: Option<&Path>) -> SyntaxMode {
    let Some(path) = path else { return SyntaxMode::Plain; };
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or("").to_ascii_lowercase();
    if name.ends_with(".blade.php") { return SyntaxMode::Blade; }
    match path.extension().and_then(OsStr::to_str).unwrap_or("").to_ascii_lowercase().as_str() {
        "php" | "phtml" => SyntaxMode::Php,
        "html" | "htm" | "xml" | "svg" => SyntaxMode::Html,
        "css" => SyntaxMode::Css,
        "js" | "mjs" | "cjs" | "jsx" => SyntaxMode::JavaScript,
        _ => SyntaxMode::Plain,
    }
}

pub(crate) fn is_programming_mode(mode: SyntaxMode) -> bool {
    !matches!(mode, SyntaxMode::Plain)
}

pub(crate) fn tree_color(path: &Path, is_dir: bool) -> &'static str {
    if is_dir { return BLUE; }
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or("").to_ascii_lowercase();
    if name.ends_with(".blade.php") { return MAGENTA; }
    match path.extension().and_then(OsStr::to_str).unwrap_or("").to_ascii_lowercase().as_str() {
        "php" | "phtml" => PURPLE,
        "html" | "htm" | "xml" | "svg" => ORANGE,
        "css" | "scss" | "sass" | "less" => BLUE,
        "js" | "mjs" | "cjs" | "jsx" => YELLOW,
        "ts" | "tsx" => CYAN,
        "rs" => ORANGE,
        "md" | "txt" => GREEN,
        "json" | "toml" | "yaml" | "yml" => CYAN,
        "sh" | "bash" | "zsh" => RED,
        _ => FG_DARK,
    }
}

pub(crate) fn highlight_segments(line: &str, syntax: SyntaxMode) -> Vec<Segment> {
    match syntax {
        SyntaxMode::Php => mixed_segments(line, true, false, false),
        SyntaxMode::Blade => mixed_segments(line, true, true, true),
        SyntaxMode::Html => mixed_segments(line, line_contains_php(line), false, true),
        SyntaxMode::Css => css::segments(line),
        SyntaxMode::JavaScript => javascript::segments(line),
        SyntaxMode::Plain => Vec::new(),
    }
}

fn mixed_segments(line: &str, php_is_primary: bool, include_blade: bool, html_is_primary: bool) -> Vec<Segment> {
    let mut out = Vec::new();

    if php_is_primary || html_is_primary || line.contains('<') || line.contains('>') {
        out.extend(html::segments(line));
    }

    if css::looks_like_line(line) || html::has_inline_style(line) {
        out.extend(css::segments(line));
    }

    if javascript::looks_like_line(line) || line.to_ascii_lowercase().contains("<script") {
        out.extend(javascript::segments(line));
    }

    if php_is_primary || line_contains_php(line) {
        out.extend(php::segments(line));
    }

    if include_blade {
        out.extend(blade::segments(line));
    }

    out
}

pub(crate) fn completion_context(syntax: SyntaxMode, before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if syntax == SyntaxMode::Plain { return None; }

    if syntax == SyntaxMode::Blade {
        if let Some(ctx) = blade::completion_context(before, explicit) { return Some(ctx); }
    }

    if matches!(syntax, SyntaxMode::Html | SyntaxMode::Blade | SyntaxMode::Php) || before.contains('<') {
        if let Some(ctx) = html::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::Css || css::looks_like_context(before) || html::has_open_inline_style(before) {
        if let Some(ctx) = css::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::JavaScript || javascript::looks_like_context(before) {
        if let Some(ctx) = javascript::completion_context(before, explicit) { return Some(ctx); }
    }

    if matches!(syntax, SyntaxMode::Php | SyntaxMode::Blade) || line_contains_php(before) || before.contains('$') {
        if let Some(ctx) = php::completion_context(before, explicit) { return Some(ctx); }
    }

    None
}

pub(crate) fn completion_items(kind: &str, prefix: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    let mut items = match kind.split_once(':').map(|(plugin, _)| plugin).unwrap_or(kind) {
        "blade" => blade::completion_items(kind, ctx),
        "html" => html::completion_items(kind, ctx),
        "css" => css::completion_items(kind, ctx),
        "javascript" => javascript::completion_items(kind, ctx),
        "php" => php::completion_items(kind, ctx),
        _ => Vec::new(),
    };
    let p = prefix.to_ascii_lowercase();
    items.retain(|i| i.label.to_ascii_lowercase().starts_with(&p));
    items.sort_by_key(|i| (i.label.len(), i.label.to_ascii_lowercase()));
    items.dedup_by(|a, b| a.label == b.label);
    items.truncate(30);
    items
}

pub(crate) fn extract_symbols(text: &str, syntax: SyntaxMode) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let no = idx + 1;
        match syntax {
            SyntaxMode::Php => {
                out.extend(php::symbols(line).into_iter().map(|s| (s, no)));
                out.extend(html::symbols(line).into_iter().map(|s| (s, no)));
                if css::looks_like_line(line) { out.extend(css::symbols(line).into_iter().map(|s| (s, no))); }
                if javascript::looks_like_line(line) { out.extend(javascript::symbols(line).into_iter().map(|s| (s, no))); }
            }
            SyntaxMode::Blade => {
                out.extend(php::symbols(line).into_iter().map(|s| (s, no)));
                out.extend(blade::symbols(line).into_iter().map(|s| (s, no)));
                out.extend(html::symbols(line).into_iter().map(|s| (s, no)));
                if css::looks_like_line(line) { out.extend(css::symbols(line).into_iter().map(|s| (s, no))); }
                if javascript::looks_like_line(line) { out.extend(javascript::symbols(line).into_iter().map(|s| (s, no))); }
            }
            SyntaxMode::Html => {
                if line_contains_php(line) { out.extend(php::symbols(line).into_iter().map(|s| (s, no))); }
                out.extend(html::symbols(line).into_iter().map(|s| (s, no)));
                if css::looks_like_line(line) { out.extend(css::symbols(line).into_iter().map(|s| (s, no))); }
                if javascript::looks_like_line(line) { out.extend(javascript::symbols(line).into_iter().map(|s| (s, no))); }
            }
            SyntaxMode::Css => out.extend(css::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::JavaScript => out.extend(javascript::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Plain => {}
        }
    }
    out
}

pub(crate) fn comp(label: &str, insert: &str, detail: &str) -> CompletionItem {
    CompletionItem { label: label.to_string(), insert: insert.to_string(), detail: detail.to_string() }
}

pub(crate) fn line_contains_php(line: &str) -> bool {
    line.contains("<?") || line.contains("?>") || line.contains("$") || line.contains("->") || line.contains("::")
}

pub(crate) fn find_between(line: &str, start_pat: &str, end_pat: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(a) = line[pos..].find(start_pat) {
        let start = pos + a;
        let after = start + start_pat.len();
        if let Some(b) = line[after..].find(end_pat) {
            let end = after + b + end_pat.len();
            out.push((start, end));
            pos = end;
        } else {
            out.push((start, line.len()));
            break;
        }
    }
    out
}

pub(crate) fn string_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let quote = bytes[i];
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' { i += 2; continue; }
                if bytes[i] == quote { i += 1; break; }
                i += 1;
            }
            out.push((start, i.min(bytes.len())));
        } else { i += 1; }
    }
    out
}

pub(crate) fn find_words(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in line.char_indices() {
        if c.is_alphanumeric() || c == '_' {
            if start.is_none() { start = Some(i); }
        } else if let Some(s) = start.take() {
            out.push((s, i));
        }
    }
    if let Some(s) = start { out.push((s, line.len())); }
    out
}

pub(crate) fn scan_ranges<F: Fn(char) -> bool>(line: &str, pred: F) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in line.char_indices() {
        if pred(c) {
            if start.is_none() { start = Some(i); }
        } else if let Some(s) = start.take() {
            out.push((s, i));
        }
    }
    if let Some(s) = start { out.push((s, line.len())); }
    out
}

pub(crate) fn find_literals(line: &str, pats: &[&str]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for pat in pats {
        let mut pos = 0;
        while let Some(idx) = line[pos..].find(pat) {
            let s = pos + idx;
            out.push((s, s + pat.len()));
            pos = s + pat.len();
        }
    }
    out
}

pub(crate) fn find_prefixed_words(line: &str, prefix: char) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let p = prefix as u8;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == p {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-') { i += 1; }
            if i > start + 1 { out.push((start, i)); }
        } else { i += 1; }
    }
    out
}

pub(crate) fn after_nonspace_is(line: &str, from: usize, ch: char) -> bool {
    line[from..].chars().find(|c| !c.is_whitespace()) == Some(ch)
}

pub(crate) fn pos_in_ranges(pos: usize, ranges: &[(usize, usize)]) -> bool {
    ranges.iter().any(|(a, b)| pos >= *a && pos < *b)
}

pub(crate) fn is_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':' | b'.')
}

pub(crate) fn word_suffix(before: &str) -> Option<(String, usize)> {
    let mut start = before.len();
    for (i, c) in before.char_indices().rev() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            start = i;
        } else { break; }
    }
    if start < before.len() { Some((before[start..].to_string(), start)) } else { None }
}

pub(crate) fn suffix_token(before: &str, marker: char) -> Option<(String, usize)> {
    let idx = before.rfind(marker)?;
    if before[idx + marker.len_utf8()..].chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        Some((before[idx..].to_string(), idx))
    } else { None }
}
