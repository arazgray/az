use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, PURPLE, RED, YELLOW, Segment, CompletionItem};
use super::{comp, find_between, find_prefixed_words, scan_ranges, string_ranges, suffix_token, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    for (a, b) in find_between(line, "/*", "*/") { s.push(Segment { start: a, end: b, color: COMMENT }); }
    for (a, b) in string_ranges(line) { s.push(Segment { start: a, end: b, color: GREEN }); }
    for (a, b) in find_prefixed_words(line, '@') { s.push(Segment { start: a, end: b, color: PURPLE }); }
    for (a, b) in property_ranges(line) { s.push(Segment { start: a, end: b, color: BLUE }); }
    for (a, b) in hex_color_ranges(line) { s.push(Segment { start: a, end: b, color: ORANGE }); }
    for (a, b) in id_selector_ranges(line) { s.push(Segment { start: a, end: b, color: YELLOW }); }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) { s.push(Segment { start: a, end: b, color: ORANGE }); }
    if let Some(pos) = line.find("!important") { s.push(Segment { start: pos, end: pos + 10, color: RED }); }
    if !line.contains(':') && line.contains('{') { if let Some(pos) = line.find('{') { s.push(Segment { start: 0, end: pos, color: CYAN }); } }
    s
}

pub(crate) fn completion_context(before: &str, _explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = suffix_token(before, '@') { return Some(("css:at".to_string(), prefix, start)); }
    if before.trim_end().ends_with(':') || before.rsplit_once(':').map(|(_, r)| !r.contains(';') && !r.contains('{')).unwrap_or(false) {
        if let Some((prefix, start)) = value_suffix(before) { return Some(("css:value".to_string(), prefix, start)); }
    }
    if let Some((prefix, start)) = word_suffix(before) { return Some(("css:property".to_string(), prefix, start)); }
    None
}

pub(crate) fn completion_items(kind: &str, _ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "css:at" => at_rules().iter().map(|s| comp(s, s, "at-rule")).collect(),
        "css:value" => values().iter().chain(functions().iter()).map(|s| comp(s, s, "value")).collect(),
        "css:property" => props().iter().map(|s| comp(s, s, "property")).collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn symbols(line: &str) -> Vec<String> {
    if let Some(pos) = line.find('{') {
        let sel = line[..pos].trim();
        if !sel.is_empty() { return vec![sel.to_string()]; }
    }
    Vec::new()
}

pub(crate) fn looks_like_context(before: &str) -> bool {
    let t = before.trim_start();
    before.contains("style=\"") || before.contains("style='") || t.starts_with('@') || before.contains('{') || before.contains(':')
}

pub(crate) fn looks_like_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('@') || t.contains("{") || t.contains("}") || property_ranges(line).len() > 0 || line.contains("!important")
}

fn property_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'-' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'-') { i += 1; }
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() { j += 1; }
            if j < bytes.len() && bytes[j] == b':' { out.push((start, i)); }
        } else { i += 1; }
    }
    out
}

fn hex_color_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_hexdigit() { i += 1; }
            let len = i - start - 1;
            if matches!(len, 3 | 4 | 6 | 8) { out.push((start, i)); }
        } else { i += 1; }
    }
    out
}

/// CSS ID selectors: `#header`, `#app-1`. Skips hex colors (handled above)
/// and `#` inside strings is already green — this only runs on raw text but
/// hex ranges take precedence via ordering in `mixed_segments`/`color_at`.
fn id_selector_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let strings = string_ranges(line);
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' {
            if super::pos_in_ranges(i, &strings) {
                i += 1;
                continue;
            }
            let start = i;
            i += 1;
            // CSS id: # followed by letter/_/-, then alnum/_/-. Pure digits = not id.
            if i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' || bytes[i] == b'-') {
                i += 1;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-')
                {
                    i += 1;
                }
                // Don't double-mark hex colors as ids (e.g. #fff handled as hex).
                let hex_len = i - start - 1;
                let all_hex = line[start + 1..i].bytes().all(|b| b.is_ascii_hexdigit());
                if all_hex && matches!(hex_len, 3 | 4 | 6 | 8) {
                    // hex — leave to hex_color_ranges
                } else {
                    out.push((start, i));
                }
                continue;
            }
        } else {
            i += 1;
        }
    }
    out
}

fn word_suffix(before: &str) -> Option<(String, usize)> {
    let mut start = before.len();
    for (i, c) in before.char_indices().rev() {
        if c.is_alphanumeric() || c == '-' { start = i; } else { break; }
    }
    if start < before.len() { Some((before[start..].to_string(), start)) } else { None }
}

fn value_suffix(before: &str) -> Option<(String, usize)> {
    word_suffix(before).or_else(|| Some((String::new(), before.len())))
}

fn props() -> Vec<&'static str> { vec!["align-content","align-items","align-self","animation","animation-delay","animation-duration","animation-name","appearance","aspect-ratio","backdrop-filter","background","background-attachment","background-color","background-image","background-position","background-repeat","background-size","border","border-bottom","border-bottom-color","border-bottom-left-radius","border-bottom-right-radius","border-box","border-collapse","border-color","border-left","border-radius","border-right","border-spacing","border-style","border-top","border-width","bottom","box-shadow","box-sizing","caption-side","clear","clip-path","color","column-gap","columns","contain","content","cursor","display","filter","flex","flex-basis","flex-direction","flex-flow","flex-grow","flex-shrink","flex-wrap","float","font","font-family","font-size","font-style","font-weight","gap","grid","grid-area","grid-auto-columns","grid-auto-flow","grid-auto-rows","grid-column","grid-template","grid-template-areas","grid-template-columns","grid-template-rows","height","inset","justify-content","justify-items","justify-self","left","letter-spacing","line-height","list-style","margin","margin-bottom","margin-left","margin-right","margin-top","max-height","max-width","min-height","min-width","object-fit","opacity","order","outline","overflow","overflow-x","overflow-y","padding","padding-bottom","padding-left","padding-right","padding-top","place-content","place-items","pointer-events","position","right","row-gap","scroll-behavior","text-align","text-decoration","text-overflow","text-transform","top","transform","transition","transition-duration","transition-property","translate","user-select","vertical-align","visibility","white-space","width","word-break","z-index"] }
fn values() -> Vec<&'static str> { vec!["absolute","auto","baseline","block","bold","border-box","both","center","column","contents","cover","fixed","flex","grid","hidden","inherit","initial","inline","inline-block","inline-flex","inline-grid","italic","left","none","normal","nowrap","pointer","relative","repeat","right","row","scroll","solid","space-around","space-between","space-evenly","sticky","transparent","underline","unset","visible","wrap","start","end","stretch","min-content","max-content","fit-content","currentColor","var()","calc()","clamp()","min()","max()","linear-gradient()","rgb()","rgba()","hsl()","hsla()"] }
fn functions() -> Vec<&'static str> { vec!["var()","calc()","clamp()","min()","max()","repeat()","minmax()","fit-content()","url()","linear-gradient()","radial-gradient()","rgb()","rgba()","hsl()","hsla()","translate()","translateX()","translateY()","scale()","rotate()","skew()"] }
fn at_rules() -> Vec<&'static str> { vec!["@charset","@container","@font-face","@import","@keyframes","@layer","@media","@namespace","@page","@property","@scope","@supports"] }
