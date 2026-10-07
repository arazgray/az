use crate::{BLUE, COMMENT, CYAN, GREEN, ORANGE, YELLOW, Segment, CompletionItem};
use super::{after_nonspace_is, comp, find_between, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
    }
    if let Some(pos) = percent_comment_start(line, &strings) {
        s.push(Segment { start: pos, end: line.len(), color: COMMENT });
    }
    for (a, b) in find_between(line, "%{", "%}") {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: COMMENT });
        }
    }
    for (a, b) in find_words(line) {
        if pos_in_ranges(a, &strings) {
            continue;
        }
        let w = &line[a..b];
        if builtins().contains(&w) {
            s.push(Segment { start: a, end: b, color: CYAN });
        } else if after_nonspace_is(line, b, '(') {
            s.push(Segment { start: a, end: b, color: BLUE });
        } else if w.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
            s.push(Segment { start: a, end: b, color: YELLOW });
        }
    }
    for (a, b) in scan_ranges(line, |c| c.is_ascii_digit()) {
        if !pos_in_ranges(a, &strings) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        }
    }
    s
}

fn percent_comment_start(line: &str, strings: &[(usize, usize)]) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        for m in ["%"].iter() {
            let mb = m.as_bytes();
            if i + mb.len() <= bytes.len() && &bytes[i..i + mb.len()] == mb && !pos_in_ranges(i, strings) {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if let Some((prefix, start)) = word_suffix(before) {
        if explicit || prefix.len() >= 2 {
            return Some(("lilypond:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "lilypond:word" => {
            let mut all = Vec::new();
            for k in keywords() {
                all.push(comp(k, k, "keyword"));
            }
            for t in types() {
                all.push(comp(t, t, "type"));
            }
            for b in builtins() {
                all.push(comp(b, b, "builtin"));
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
    let _ = line;
    Vec::new()
}

fn keywords() -> Vec<&'static str> {
    vec![]
}

fn types() -> Vec<&'static str> {
    vec![]
}

fn builtins() -> Vec<&'static str> {
    vec!["staff", "spacing", "signature", "routine", "notes", "handler", "corrected", "beams", "arpeggios", "Volta_engraver", "Voice", "Vertical_align_engraver", "Vaticana_ligature_engraver", "VaticanaVoice", "VaticanaStaff", "Tweak_engraver", "Tuplet_engraver", "Trill_spanner_engraver", "Timing_translator", "Time_signature_performer", "Time_signature_engraver", "Tie_performer", "Tie_engraver", "Text_spanner_engraver", "Text_engraver", "Tempo_performer", "Tab_tie_follow_engraver", "Tab_staff_symbol_engraver", "Tab_note_heads_engraver", "TabVoice", "TabStaff", "Stem_engraver", "Stanza_number_engraver", "Stanza_number_align_engraver", "Staff_symbol_engraver", "Staff_performer", "Staff_collecting_engraver", "StaffGroup", "Staff", "Spanner_break_forbid_engraver", "Span_bar_stub_engraver", "Span_bar_engraver", "Span_arpeggio_engraver", "Spacing_engraver", "Slur_performer", "Slur_engraver", "Slash_repeat_engraver", "Separating_line_group_engraver", "Script_row_engraver", "Script_engraver", "Script_column_engraver", "Score", "Rhythmic_column_engraver", "RhythmicStaff", "Rest_engraver", "Rest_collision_engraver", "Repeat_tie_engraver", "Repeat_acknowledge_engraver", "Pure_from_neighbor_engraver", "Pitched_trill_engraver"]
}
