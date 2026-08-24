//! Input 的 AccessKit 文本 run 与字素索引映射。

use super::MAX_CHARS_PER_TEXT_RUN;
use gpui::{A11ySubtreeBuilder, accesskit};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation as _;

fn for_each_a11y_character_unit(text: &str, mut visit: impl FnMut(Range<usize>)) {
    for (grapheme_start, grapheme) in text.grapheme_indices(true) {
        if grapheme.len() <= usize::from(u8::MAX) {
            visit(grapheme_start..grapheme_start + grapheme.len());
            continue;
        }

        let mut unit_start = grapheme_start;
        let mut unit_len = 0;
        for (relative_start, ch) in grapheme.char_indices() {
            let char_len = ch.len_utf8();
            if unit_len > 0 && unit_len + char_len > usize::from(u8::MAX) {
                let unit_end = grapheme_start + relative_start;
                visit(unit_start..unit_end);
                unit_start = unit_end;
                unit_len = 0;
            }
            unit_len += char_len;
        }
        visit(unit_start..grapheme_start + grapheme.len());
    }
}

fn a11y_character_count(text: &str) -> usize {
    let mut count = 0;
    for_each_a11y_character_unit(text, |_| count += 1);
    count
}

fn a11y_unit_index_for_byte(text: &str, byte_offset: usize) -> usize {
    let byte_offset = byte_offset.min(text.len());
    let mut unit_count = 0;
    let mut result = None;
    for_each_a11y_character_unit(text, |unit| {
        if result.is_none() {
            if byte_offset <= unit.start {
                result = Some(unit_count);
            } else if byte_offset < unit.end {
                result = Some(if byte_offset - unit.start <= unit.end - byte_offset {
                    unit_count
                } else {
                    unit_count + 1
                });
            } else if byte_offset == unit.end {
                result = Some(unit_count + 1);
            }
        }
        unit_count += 1;
    });
    result.unwrap_or(unit_count)
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
    let total_characters = a11y_character_count(text);
    let num_chunks = total_characters.div_ceil(MAX_CHARS_PER_TEXT_RUN).max(1);
    let mut word_starts = text
        .split_word_bound_indices()
        .filter(|(_, segment)| segment.unicode_words().next().is_some())
        .map(|(start, _)| start)
        .peekable();

    let mut runs = Vec::with_capacity(num_chunks);
    let mut character_lengths = Vec::with_capacity(MAX_CHARS_PER_TEXT_RUN);
    let mut chunk_word_starts = Vec::new();
    let mut byte_start = 0;
    let mut byte_end = 0;
    let mut character_index = 0;
    let mut last_word_character = None;

    let mut push_run = |character_lengths: &mut Vec<u8>,
                        chunk_word_starts: &mut Vec<u8>,
                        byte_start: usize,
                        byte_end: usize| {
        let chunk_index = runs.len();
        let mut node = accesskit::Node::new(accesskit::Role::TextRun);
        node.set_text_direction(accesskit::TextDirection::LeftToRight);
        node.set_value(&text[byte_start..byte_end]);
        node.set_character_lengths(std::mem::replace(
            character_lengths,
            Vec::with_capacity(MAX_CHARS_PER_TEXT_RUN),
        ));
        node.set_word_starts(std::mem::take(chunk_word_starts));
        runs.push((synthetic_node_id(chunk_index as u64), node));
    };

    for (grapheme_start, grapheme) in text.grapheme_indices(true) {
        let grapheme_end = grapheme_start + grapheme.len();
        while word_starts
            .peek()
            .is_some_and(|word_start| *word_start < grapheme_end)
        {
            let word_start = word_starts.next().expect("peek 已确认存在 word start");
            if word_start >= grapheme_start && last_word_character != Some(character_index) {
                chunk_word_starts.push(checked_a11y_length(character_lengths.len()));
                last_word_character = Some(character_index);
            }
        }

        let mut push_unit = |unit: Range<usize>| {
            if character_lengths.is_empty() {
                byte_start = unit.start;
            }
            byte_end = unit.end;
            character_lengths.push(checked_a11y_length(unit.end - unit.start));
            character_index += 1;
            if character_lengths.len() == MAX_CHARS_PER_TEXT_RUN {
                push_run(
                    &mut character_lengths,
                    &mut chunk_word_starts,
                    byte_start,
                    byte_end,
                );
            }
        };

        if grapheme.len() <= usize::from(u8::MAX) {
            push_unit(grapheme_start..grapheme_end);
        } else {
            let mut unit_start = grapheme_start;
            let mut unit_len = 0;
            for (relative_start, ch) in grapheme.char_indices() {
                let char_len = ch.len_utf8();
                if unit_len > 0 && unit_len + char_len > usize::from(u8::MAX) {
                    let unit_end = grapheme_start + relative_start;
                    push_unit(unit_start..unit_end);
                    unit_start = unit_end;
                    unit_len = 0;
                }
                unit_len += char_len;
            }
            push_unit(unit_start..grapheme_end);
        }
    }

    if !character_lengths.is_empty() || total_characters == 0 {
        push_run(
            &mut character_lengths,
            &mut chunk_word_starts,
            byte_start,
            byte_end,
        );
    }

    let run_count = runs.len();
    for (chunk_index, (_, node)) in runs.iter_mut().enumerate() {
        if chunk_index > 0 {
            node.set_previous_on_line(synthetic_node_id(chunk_index as u64 - 1));
        }
        if chunk_index + 1 < run_count {
            node.set_next_on_line(synthetic_node_id(chunk_index as u64 + 1));
        }
    }
    let anchor = a11y_text_position(
        a11y_unit_index_for_byte(text, selection_tail),
        &synthetic_node_id,
    );
    let focus = a11y_text_position(
        a11y_unit_index_for_byte(text, selection_head),
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
