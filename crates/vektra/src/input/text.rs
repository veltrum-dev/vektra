//! Input 的单行文本替换、索引转换与 Unicode 边界工具。

use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation as _;

pub(super) fn replace_text(value: &str, range: Range<usize>, replacement: &str) -> String {
    let mut next =
        String::with_capacity(value.len() - (range.end - range.start) + replacement.len());
    next.push_str(&value[..range.start]);
    next.push_str(replacement);
    next.push_str(&value[range.end..]);
    next
}

pub(super) fn normalize_single_line(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                output.push(' ');
            }
            '\n' => output.push(' '),
            _ => output.push(ch),
        }
    }
    output
}

pub(super) fn utf16_to_utf8(text: &str, offset: usize) -> usize {
    let mut utf16 = 0;
    for (byte, ch) in text.char_indices() {
        if utf16 >= offset {
            return byte;
        }
        let next = utf16 + ch.len_utf16();
        if offset < next {
            return byte;
        }
        utf16 = next;
    }
    text.len()
}

pub(super) fn utf8_to_utf16(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    text.char_indices()
        .take_while(|(byte, _)| *byte < offset)
        .map(|(_, ch)| ch.len_utf16())
        .sum()
}

pub(super) fn range_from_utf16(text: &str, range: Range<usize>) -> Range<usize> {
    normalize_selection(
        text,
        utf16_to_utf8(text, range.start)..utf16_to_utf8(text, range.end),
    )
}

pub(super) fn range_to_utf16(text: &str, range: Range<usize>) -> Range<usize> {
    utf8_to_utf16(text, range.start)..utf8_to_utf16(text, range.end)
}

pub(super) fn previous_grapheme_boundary(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .take_while(|index| *index < offset)
        .last()
        .unwrap_or(0)
}

pub(super) fn next_grapheme_boundary(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .find(|index| *index > offset)
        .unwrap_or(text.len())
}

pub(super) fn nearest_paired_boundary(offset: usize, from: &[usize], to: &[usize]) -> usize {
    debug_assert_eq!(from.len(), to.len());
    debug_assert!(!from.is_empty());
    let index = match from.binary_search(&offset) {
        Ok(index) => index,
        Err(0) => 0,
        Err(index) if index == from.len() => from.len() - 1,
        Err(index) => {
            let before = index - 1;
            if offset - from[before] <= from[index] - offset {
                before
            } else {
                index
            }
        }
    };
    to[index]
}

pub(super) fn previous_word_boundary(text: &str, offset: usize) -> usize {
    text.split_word_bound_indices()
        .filter(|(_, segment)| segment.unicode_words().next().is_some())
        .map(|(start, _)| start)
        .take_while(|start| *start < offset)
        .last()
        .unwrap_or(0)
}

pub(super) fn next_word_boundary(text: &str, offset: usize) -> usize {
    text.split_word_bound_indices()
        .filter(|(_, segment)| segment.unicode_words().next().is_some())
        .map(|(start, segment)| start + segment.len())
        .find(|end| *end > offset)
        .unwrap_or(text.len())
}

pub(super) fn nearest_grapheme_boundary(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    if offset == text.len()
        || text
            .grapheme_indices(true)
            .any(|(index, _)| index == offset)
    {
        return offset;
    }
    let before = previous_grapheme_boundary(text, offset + 1);
    let after = next_grapheme_boundary(text, before);
    if offset - before <= after.saturating_sub(offset) {
        before
    } else {
        after
    }
}

pub(super) fn normalize_selection(text: &str, range: Range<usize>) -> Range<usize> {
    let start = nearest_grapheme_boundary(text, range.start.min(text.len()));
    let end = nearest_grapheme_boundary(text, range.end.min(text.len()));
    start.min(end)..start.max(end)
}

pub(super) fn word_range_at(text: &str, offset: usize) -> Range<usize> {
    if text.is_empty() {
        return 0..0;
    }
    let offset = offset.min(text.len().saturating_sub(1));
    if let Some((start, word)) = text
        .unicode_word_indices()
        .find(|(start, word)| offset >= *start && offset < *start + word.len())
    {
        return start..start + word.len();
    }
    let start = nearest_grapheme_boundary(text, offset);
    start..next_grapheme_boundary(text, start)
}
