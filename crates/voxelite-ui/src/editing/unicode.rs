use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

pub(super) fn floor(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(i, _)| i)
        .chain([text.len()])
        .take_while(|i| *i <= offset)
        .last()
        .unwrap_or(0)
}
pub(super) fn ceil(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(i, _)| i)
        .chain([text.len()])
        .find(|i| *i >= offset)
        .unwrap_or(text.len())
}
pub(super) fn scalar_floor(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}
pub(super) fn scalar_ceil(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset += 1;
    }
    offset
}
pub(super) fn adjacent(text: &str, head: usize, forward: bool) -> usize {
    if forward {
        text.grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|i| *i > head)
            .unwrap_or(text.len())
    } else {
        text.grapheme_indices(true)
            .map(|(i, _)| i)
            .take_while(|i| *i < head)
            .last()
            .unwrap_or(0)
    }
}
fn units(text: &str) -> Vec<Range<usize>> {
    let mut result: Vec<Range<usize>> = Vec::new();
    let mut punctuation = false;
    for (i, word) in text.split_word_bound_indices() {
        // Unicode segmentation handles words, CJK and extended emoji. Merge
        // adjacent punctuation (but never emoji) into useful editing units.
        let p = word.chars().all(|c| {
            c.is_ascii_punctuation() || matches!(c, '—' | '–' | '…' | '。' | '、' | '！' | '？')
        });
        if p && punctuation {
            result.last_mut().unwrap().end = i + word.len();
        } else {
            result.push(i..i + word.len());
        }
        punctuation = p;
    }
    result
}
pub(super) fn word_at(text: &str, offset: usize) -> Range<usize> {
    let offset = floor(text, offset.min(text.len()));
    let units = units(text);
    units
        .iter()
        .find(|r| r.contains(&offset))
        .cloned()
        .or_else(|| units.last().cloned())
        .unwrap_or(0..0)
}
pub(super) fn word_boundary(text: &str, head: usize, forward: bool) -> usize {
    let units = units(text);
    if forward {
        units
            .iter()
            .find(|r| r.end > head && !text[(*r).clone()].chars().all(char::is_whitespace))
            .map_or(text.len(), |r| r.end)
    } else {
        units
            .iter()
            .rev()
            .find(|r| r.start < head && !text[(*r).clone()].chars().all(char::is_whitespace))
            .map_or(0, |r| r.start)
    }
}
/// Normalize CRLF, line separators and tabs to spaces; discard other controls.
pub fn single_line(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                result.push(' ');
            }
            '\n' | '\t' | '\u{2028}' | '\u{2029}' => result.push(' '),
            c if !c.is_control() => result.push(c),
            _ => {}
        }
    }
    result
}
/// Convert native UTF-16 offsets to safe UTF-8 scalar boundaries. Out-of-range
/// and surrogate-interior offsets clamp; editing operations snap to graphemes.
pub fn from_utf16(text: &str, range: Range<usize>) -> Range<usize> {
    fn byte(text: &str, offset: usize) -> usize {
        let mut utf16 = 0;
        for (i, c) in text.char_indices() {
            if utf16 >= offset {
                return i;
            }
            utf16 += c.len_utf16();
        }
        text.len()
    }
    byte(text, range.start.min(range.end))..byte(text, range.start.max(range.end))
}
/// Convert valid UTF-8 boundaries to native UTF-16 offsets.
pub fn to_utf16(text: &str, range: Range<usize>) -> Range<usize> {
    text[..range.start].encode_utf16().count()..text[..range.end].encode_utf16().count()
}
