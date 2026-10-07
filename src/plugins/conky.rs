use crate::{COMMENT, CYAN, GREEN, ORANGE, PURPLE, Segment, CompletionItem};
use super::{comp, find_words, pos_in_ranges, scan_ranges, string_ranges, word_suffix, CompletionContext};

pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut s = Vec::new();
    let strings = string_ranges(line);
    for (a, b) in &strings {
        s.push(Segment { start: *a, end: *b, color: GREEN });
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
        } else if types().contains(&w) {
            s.push(Segment { start: a, end: b, color: CYAN });
        } else if builtins().contains(&w) {
            s.push(Segment { start: a, end: b, color: CYAN });
        } else if constants().contains(&w) {
            s.push(Segment { start: a, end: b, color: ORANGE });
        } else if constants().contains(&w) {
            s.push(Segment { start: a, end: b, color: ORANGE });
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
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        for m in ["#"].iter() {
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
            return Some(("conky:word".to_string(), prefix, start));
        }
    }
    None
}

pub(crate) fn completion_items(kind: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    match kind {
        "conky:word" => {
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
    vec!["above", "below", "bottom_left", "bottom_right", "bottom_middle", "desktop", "dock", "normal", "override", "skip_pager", "skip_taskbar", "sticky", "top_left", "top_right", "top_middle", "middle_left", "middle_right", "middle_middle", "undecorated"]
}

fn types() -> Vec<&'static str> {
    vec!["alignment", "append_file", "background", "border_inner_margin", "border_outer_margin", "border_width", "color0", "color1", "color2", "color3", "color4", "color5", "color6", "color7", "color8", "color9", "colorN", "cpu_avg_samples", "default_bar_height", "default_bar_width", "default_color", "default_gauge_height", "default_gauge_width", "default_graph_height", "default_graph_width", "default_outline_color", "default_shade_color", "diskio_avg_samples", "display", "double_buffer", "draw_borders", "draw_graph_borders", "draw_outline", "draw_shades", "extra_newline", "font", "format_human_readable", "gap_x", "gap_y", "http_refresh"]
}

fn builtins() -> Vec<&'static str> {
    vec!["acpiacadapter", "acpifan", "acpitemp", "addr", "addrs", "alignc", "alignr", "apcupsd", "apcupsd_cable", "apcupsd_charge", "apcupsd_lastxfer", "apcupsd_linev", "apcupsd_load", "apcupsd_loadbar", "apcupsd_loadgauge", "apcupsd_loadgraph", "apcupsd_model", "apcupsd_name", "apcupsd_status", "apcupsd_temp", "apcupsd_timeleft", "apcupsd_upsmode", "apm_adapter", "apm_battery_life", "apm_battery_time", "audacious_bar", "audacious_bitrate", "audacious_channels", "audacious_filename", "audacious_frequency", "audacious_length", "audacious_length_seconds", "audacious_main_volume", "audacious_playlist_length", "audacious_playlist_position", "audacious_position", "audacious_position_seconds", "audacious_status", "audacious_title", "battery", "battery_bar", "battery_percent", "battery_short", "battery_time", "blink", "bmpx_album", "bmpx_artist", "bmpx_bitrate", "bmpx_title", "bmpx_track", "bmpx_uri", "buffers", "cached", "cmdline_to_pid", "color"]
}

fn constants() -> Vec<&'static str> {
    vec!["no", "none", "yes"]
}
