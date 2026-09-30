use crate::{COMMENT, GREEN, MAGENTA, ORANGE, YELLOW, Segment, CompletionItem};
use super::{comp, find_between, is_name_byte, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    for (a, b) in find_between(line, "<!--", "-->") { s.push(Segment { start: a, end: b, color: COMMENT }); }
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let start = i;
            i += 1;
            if i < bytes.len() && bytes[i] == b'/' { i += 1; }
            while i < bytes.len() && is_name_byte(bytes[i]) { i += 1; }
            if i > start + 1 { s.push(Segment { start, end: i, color: MAGENTA }); }
        } else { i += 1; }
    }
    for (a, b) in string_ranges(line) { s.push(Segment { start: a, end: b, color: GREEN }); }
    for (a, b) in attr_ranges(line) { s.push(Segment { start: a, end: b, color: YELLOW }); }
    for (a, b) in entity_ranges(line) { s.push(Segment { start: a, end: b, color: ORANGE }); }
    s
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = tag_prefix(before) {
        if explicit || !prefix.is_empty() || before.ends_with('<') || before.ends_with("</") {
            return Some(("html:tag".to_string(), prefix, start));
        }
    }
    if is_inside_tag(before) {
        if let Some((prefix, start)) = word_suffix(before) {
            if explicit || !prefix.is_empty() { return Some(("html:attr".to_string(), prefix, start)); }
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, _ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "html:tag" => tags().iter().map(|s| comp(s, s, "tag")).collect(),
        "html:attr" => attrs().iter().map(|s| comp(s, s, "attr")).collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = line.to_ascii_lowercase();
    for tag in ["h1", "h2", "h3", "section", "article", "main"] {
        let open = format!("<{tag}");
        if lower.contains(&open) { out.push(format!("<{tag}>")); }
    }
    out
}

pub(crate) fn last_unclosed_tag(before: &str) -> Option<String> {
    // Called right after `>` is typed, so `before` usually ends with `>`.
    // Strip that single just-typed `>` to recover the tag (e.g. `<div>` -> `<div`).
    let inner = if before.ends_with('>') {
        &before[..before.len().saturating_sub(1)]
    } else {
        before
    };
    let lt = inner.rfind('<')?;
    // If there is another `>` after `<`, this tag was already closed earlier.
    if inner[lt..].contains('>') {
        return None;
    }
    // Self-closing `<br/>` or closing `</div` must not auto-close.
    if inner[lt..].contains('/') {
        return None;
    }
    let tag: String = inner[lt + 1..]
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == ':')
        .collect();
    if tag.is_empty() { None } else { Some(tag) }
}

pub(crate) fn is_void_tag(tag: &str) -> bool {
    void_tags().contains(&tag.to_ascii_lowercase().as_str())
}

pub(crate) fn has_inline_style(line: &str) -> bool {
    line.to_ascii_lowercase().contains("style=")
}

pub(crate) fn has_open_inline_style(before: &str) -> bool {
    let lower = before.to_ascii_lowercase();
    if let Some(pos) = lower.rfind("style=") {
        let rest = &before[pos..];
        let quote = rest.chars().find(|c| *c == '\'' || *c == '"');
        if let Some(q) = quote {
            let after_quote = rest.find(q).map(|i| &rest[i + 1..]).unwrap_or("");
            return !after_quote.contains(q);
        }
    }
    false
}

fn attr_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b':' || bytes[i] == b'@' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'-' | b'_' | b':' | b'.' | b'@')) { i += 1; }
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() { j += 1; }
            if j < bytes.len() && bytes[j] == b'=' { out.push((start, i)); }
        } else { i += 1; }
    }
    out
}

fn entity_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'#') { i += 1; }
            if i < bytes.len() && bytes[i] == b';' { out.push((start, i + 1)); }
        } else { i += 1; }
    }
    out
}

fn tag_prefix(before: &str) -> Option<(String, usize)> {
    let lt = before.rfind('<')?;
    if before[lt..].contains('>') { return None; }
    let start = if before[lt..].starts_with("</") { lt + 2 } else { lt + 1 };
    if before[start..].chars().all(|c| c.is_alphanumeric() || c == '-' || c == ':') {
        Some((before[start..].to_string(), start))
    } else { None }
}

fn is_inside_tag(before: &str) -> bool {
    before.rfind('<').map(|lt| !before[lt..].contains('>')).unwrap_or(false)
}

fn tags() -> Vec<&'static str> { vec!["a","abbr","address","area","article","aside","audio","b","base","bdi","bdo","blockquote","body","br","button","canvas","caption","cite","code","col","colgroup","data","datalist","dd","del","details","dfn","dialog","div","dl","dt","em","embed","fieldset","figcaption","figure","footer","form","h1","h2","h3","h4","h5","h6","head","header","hr","html","i","iframe","img","input","ins","kbd","label","legend","li","link","main","map","mark","meta","meter","nav","noscript","object","ol","optgroup","option","output","p","picture","pre","progress","q","rp","rt","ruby","s","samp","script","section","select","slot","small","source","span","strong","style","sub","summary","sup","svg","table","tbody","td","template","textarea","tfoot","th","thead","time","title","tr","track","u","ul","var","video","wbr"] }
fn void_tags() -> Vec<&'static str> { vec!["area","base","br","col","embed","hr","img","input","link","meta","param","source","track","wbr"] }
fn attrs() -> Vec<&'static str> { vec!["accept","accept-charset","accesskey","action","allow","alt","aria-","aria-label","aria-hidden","aria-expanded","aria-controls","async","autocomplete","autofocus","autoplay","charset","checked","cite","class","cols","colspan","content","contenteditable","controls","crossorigin","data-","datetime","defer","dir","disabled","download","draggable","enctype","for","form","height","hidden","href","hreflang","id","integrity","lang","loading","loop","max","maxlength","media","method","min","minlength","multiple","muted","name","novalidate","pattern","placeholder","poster","preload","readonly","rel","required","role","rows","rowspan","sandbox","scope","selected","sizes","slot","spellcheck","src","srcset","style","tabindex","target","title","type","value","width","wrap"] }
