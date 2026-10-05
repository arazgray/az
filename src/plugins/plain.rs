use crate::{ORANGE, Segment};

/// Plain text (`.txt` and the fallback mode).
///
/// Letters, digits, and whitespace stay the normal text color. ASCII
/// punctuation (`~!@#$%^&*()[]` and the rest of that set) is marked so a
/// dense line is easier to scan. Row striping lives in the editor, not here.
pub(crate) fn segments(line: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (i, ch) in line.char_indices() {
        if ch.is_ascii_punctuation() {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(from) = start.take() {
            out.push(Segment { start: from, end: i, color: ORANGE });
        }
    }
    if let Some(from) = start {
        out.push(Segment { start: from, end: line.len(), color: ORANGE });
    }
    out
}
