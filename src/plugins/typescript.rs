use crate::{Segment, CompletionItem};
use super::{comp, CompletionContext};

/// TypeScript reuses JavaScript highlighting plus extra keywords.
/// Keeps one code path for both JS-family languages.
pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = super::javascript::segments(line);
    for (a, b) in super::find_words(line) {
        let w = &line[a..b];
        if extra_keywords().contains(&w) {
            s.push(Segment { start: a, end: b, color: crate::PURPLE });
        } else if extra_types().contains(&w) {
            s.push(Segment { start: a, end: b, color: crate::CYAN });
        }
    }
    s
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    // Delegate member/word detection to JavaScript, relabel kind to typescript.
    if let Some((kind, prefix, start)) = super::javascript::completion_context(before, explicit) {
        let ts_kind = kind.replace("javascript", "typescript");
        return Some((ts_kind, prefix, start));
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "typescript:member" => super::javascript::completion_items("javascript:member", ctx),
        "typescript:word" => {
            let mut all = super::javascript::completion_items("javascript:word", ctx);
            for k in extra_keywords() {
                all.push(comp(k, k, "keyword"));
            }
            for t in extra_types() {
                all.push(comp(t, t, "type"));
            }
            all
        }
        _ => Vec::new(),
    }
}

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let mut out = super::javascript::symbols(line);
    let t = line.trim_start();
    for marker in ["interface ", "type ", "enum ", "namespace "] {
        if let Some(pos) = t.find(marker) {
            let after = &t[pos + marker.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                out.push(format!("{} {}", marker.trim(), name));
            }
        }
    }
    out
}

fn extra_keywords() -> Vec<&'static str> {
    vec!["interface", "type", "enum", "namespace", "readonly", "abstract", "implements", "declare", "keyof", "infer", "is", "asserts", "satisfies", "override"]
}

fn extra_types() -> Vec<&'static str> {
    vec!["string", "number", "boolean", "void", "never", "unknown", "any", "Record", "Partial", "Pick", "Omit", "Readonly", "Promise"]
}
