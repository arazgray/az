use crate::{BLUE, MAGENTA, ORANGE, YELLOW, Segment};

/// Plain text (`.txt` and the fallback mode).
///
/// Letters and whitespace stay the normal text color. Structure pops in
/// calm colors — `()` blue, `[]` yellow, `{}` magenta — while digits and
/// the rest of the ASCII punctuation (`~!@#$%^&*` and friends) share orange
/// so a dense line is easier to scan. Row striping lives in the editor.
pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let mut active: &'static str = "";
    for (i, ch) in line.char_indices() {
        match plain_color(ch) {
            Some(color) if start.is_some() && color == active => {}
            Some(color) => {
                if let Some(from) = start.take() {
                    out.push(Segment { start: from, end: i, color: active });
                }
                start = Some(i);
                active = color;
            }
            None => {
                if let Some(from) = start.take() {
                    out.push(Segment { start: from, end: i, color: active });
                }
            }
        }
    }
    if let Some(from) = start {
        out.push(Segment { start: from, end: line.len(), color: active });
    }
    out
}

fn plain_color(ch: char) -> Option<&'static str> {
    match ch {
        '(' | ')' => Some(BLUE),
        '[' | ']' => Some(YELLOW),
        '{' | '}' => Some(MAGENTA),
        _ if ch.is_ascii_digit() => Some(ORANGE),
        _ if ch.is_ascii_punctuation() => Some(ORANGE),
        _ => None,
    }
}
