use crate::{COMMENT, GREEN2, PURPLE, Segment, CompletionItem};
use super::{comp, find_between, pos_in_ranges, string_ranges, suffix_token, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    for (start, end) in directive_ranges(line) {
        s.push(Segment { start, end, color: PURPLE });
    }
    for (start, end) in find_between(line, "{{", "}}") { s.push(Segment { start, end, color: GREEN2 }); }
    for (start, end) in find_between(line, "{{--", "--}}") { s.push(Segment { start, end, color: COMMENT }); }
    s
}

/// `@directive` only at word boundaries and outside strings.
/// Skips emails (`user@example.com`) and `@@` escapes.
fn directive_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let strings = string_ranges(line);
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            if pos_in_ranges(i, &strings) {
                i += 1;
                continue;
            }
            // `@@if` is escaped Blade — skip both.
            if i + 1 < bytes.len() && bytes[i + 1] == b'@' {
                i += 2;
                continue;
            }
            // Must be at start or after whitespace / ( [ { > "' `:  — not inside email/word.
            let ok_before = if i == 0 {
                true
            } else {
                let p = bytes[i - 1];
                p.is_ascii_whitespace()
                    || matches!(p, b'(' | b'[' | b'{' | b'>' | b'"' | b'\'' | b'`' | b':' | b';' | b',')
            };
            if !ok_before {
                i += 1;
                continue;
            }
            let start = i;
            i += 1;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-')
            {
                i += 1;
            }
            if i > start + 1 {
                out.push((start, i));
            }
            continue;
        }
        i += 1;
    }
    out
}

pub(crate) fn completion_context(before: &str, _explicit: bool) -> Option<(String, String, usize)> {
    suffix_token(before, '@').map(|(prefix, start)| ("blade:directive".to_string(), prefix, start))
}

pub(crate) fn completion_items(kind: &str, _ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "blade:directive" => directives().iter().map(|s| comp(s, s, "Blade")).collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn symbols(line: &str) -> Vec<String> {
    let trimmed = line.trim_start();
    for dir in ["@section", "@if", "@foreach", "@forelse", "@while", "@switch", "@component"] {
        if trimmed.starts_with(dir) { return vec![trimmed.to_string()]; }
    }
    Vec::new()
}

fn directives() -> Vec<&'static str> { vec!["@auth","@aware","@break","@can","@cannot","@case","@choice","@class","@component","@continue","@csrf","@dd","@disabled","@each","@else","@elseif","@empty","@endauth","@endcan","@endcannot","@endcase","@endcomponent","@endempty","@endenv","@enderror","@endforeach","@endif","@endisset","@endonce","@endproduction","@endpush","@endsection","@endswitch","@endunless","@endwhile","@env","@error","@extends","@foreach","@forelse","@if","@include","@isset","@json","@lang","@method","@once","@php","@production","@props","@push","@section","@selected","@stack","@style","@switch","@unless","@vite","@while","@yield"] }
