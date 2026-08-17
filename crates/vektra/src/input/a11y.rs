//! Input 的 AccessKit 文本 run 与字素索引映射。

use super::MAX_CHARS_PER_TEXT_RUN;
use gpui::{A11ySubtreeBuilder, accesskit};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation as _;

pub(super) fn a11y_character_units(text: &str) -> Vec<Range<usize>> {
    let mut units = Vec::new();
    for (grapheme_start, grapheme) in text.grapheme_indices(true) {
        if grapheme.len() <= usize::from(u8::MAX) {
            units.push(grapheme_start..grapheme_start + grapheme.len());
            continue;
        }

        let mut unit_start = grapheme_start;
        let mut unit_len = 0;
        for (relative_start, ch) in grapheme.char_indices() {
            let char_len = ch.len_utf8();
            if unit_len > 0 && unit_len + char_len > usize::from(u8::MAX) {
                let unit_end = grapheme_start + relative_start;
                units.push(unit_start..unit_end);
                unit_start = unit_end;
                unit_len = 0;
            }
            unit_len += char_len;
        }
        units.push(unit_start..grapheme_start + grapheme.len());
    }
    units
}

fn a11y_unit_index_for_byte(units: &[Range<usize>], byte_offset: usize, text_len: usize) -> usize {
    let byte_offset = byte_offset.min(text_len);
    for (index, unit) in units.iter().enumerate() {
        if byte_offset <= unit.start {
            return index;
        }
        if byte_offset < unit.end {
            return if byte_offset - unit.start <= unit.end - byte_offset {
                index
            } else {
                index + 1
            };
        }
        if byte_offset == unit.end {
            return index + 1;
        }
    }
    units.len()
}

fn grapheme_boundary_at_or_before(text: &str, byte_offset: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .take_while(|index| *index <= byte_offset.min(text.len()))
        .last()
        .unwrap_or(0)
}

fn checked_a11y_length(length: usize) -> u8 {
    let Ok(length) = u8::try_from(length) else {
        unreachable!("AccessKit 文本单元已按 u8 上限安全分段")
    };
    length
}

fn a11y_text_position(
    character_index: usize,
    synthetic_node_id: impl Fn(u64) -> accesskit::NodeId,
) -> accesskit::TextPosition {
    let chunk_index =
        if character_index > 0 && character_index.is_multiple_of(MAX_CHARS_PER_TEXT_RUN) {
            character_index / MAX_CHARS_PER_TEXT_RUN - 1
        } else {
            character_index / MAX_CHARS_PER_TEXT_RUN
        };
    accesskit::TextPosition {
        node: synthetic_node_id(chunk_index as u64),
        character_index: character_index - chunk_index * MAX_CHARS_PER_TEXT_RUN,
    }
}

pub(super) fn build_a11y_text_runs(
    text: &str,
    selection_tail: usize,
    selection_head: usize,
    synthetic_node_id: impl Fn(u64) -> accesskit::NodeId,
) -> (
    Vec<(accesskit::NodeId, accesskit::Node)>,
    accesskit::TextSelection,
) {
    let units = a11y_character_units(text);
    let total_characters = units.len();
    let num_chunks = total_characters.div_ceil(MAX_CHARS_PER_TEXT_RUN).max(1);
    let word_starts = text
        .split_word_bound_indices()
        .filter(|(_, segment)| segment.unicode_words().next().is_some())
        .map(|(start, _)| grapheme_boundary_at_or_before(text, start))
        .map(|start| a11y_unit_index_for_byte(&units, start, text.len()))
        .fold(Vec::new(), |mut starts, start| {
            if starts.last() != Some(&start) {
                starts.push(start);
            }
            starts
        });

    let mut runs = Vec::with_capacity(num_chunks);
    for chunk_index in 0..num_chunks {
        let character_start = chunk_index * MAX_CHARS_PER_TEXT_RUN;
        let character_end = (character_start + MAX_CHARS_PER_TEXT_RUN).min(total_characters);
        let byte_start = units
            .get(character_start)
            .map_or(text.len(), |unit| unit.start);
        let byte_end = units
            .get(character_end.saturating_sub(1))
            .map_or(byte_start, |unit| unit.end);
        let mut node = accesskit::Node::new(accesskit::Role::TextRun);
        node.set_text_direction(accesskit::TextDirection::LeftToRight);
        node.set_value(&text[byte_start..byte_end]);
        node.set_character_lengths(
            units[character_start..character_end]
                .iter()
                .map(|unit| checked_a11y_length(unit.end - unit.start))
                .collect::<Vec<_>>(),
        );
        node.set_word_starts(
            word_starts
                .iter()
                .filter(|start| **start >= character_start && **start < character_end)
                .map(|start| checked_a11y_length(*start - character_start))
                .collect::<Vec<_>>(),
        );
        if chunk_index > 0 {
            node.set_previous_on_line(synthetic_node_id(chunk_index as u64 - 1));
        }
        if chunk_index + 1 < num_chunks {
            node.set_next_on_line(synthetic_node_id(chunk_index as u64 + 1));
        }
        runs.push((synthetic_node_id(chunk_index as u64), node));
    }
    let anchor = a11y_text_position(
        a11y_unit_index_for_byte(&units, selection_tail, text.len()),
        &synthetic_node_id,
    );
    let focus = a11y_text_position(
        a11y_unit_index_for_byte(&units, selection_head, text.len()),
        &synthetic_node_id,
    );
    (runs, accesskit::TextSelection { anchor, focus })
}

pub(super) fn push_a11y_text_runs(
    builder: &mut A11ySubtreeBuilder,
    text: &str,
    selection_tail: usize,
    selection_head: usize,
) {
    let (runs, selection) = build_a11y_text_runs(text, selection_tail, selection_head, |chunk| {
        builder.synthetic_node_id(chunk)
    });
    for (id, node) in runs {
        builder.push_child(id, node);
    }
    builder.parent_node().set_text_selection(selection);
}
