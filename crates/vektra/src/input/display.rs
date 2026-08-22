//! Input 真实文本与可见文本之间的映射。

use super::{PASSWORD_MASK, nearest_paired_boundary};
use gpui::SharedString;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation as _;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct DisplayText {
    pub(super) text: SharedString,
    pub(super) masked: bool,
    pub(super) real_boundaries: Vec<usize>,
    pub(super) display_boundaries: Vec<usize>,
}

impl DisplayText {
    pub(super) fn new(value: impl Into<SharedString>, password_hidden: bool) -> Self {
        let value = value.into();
        if !password_hidden {
            return Self {
                text: value.clone(),
                masked: false,
                real_boundaries: Vec::new(),
                display_boundaries: Vec::new(),
            };
        }

        let real_boundaries = value
            .grapheme_indices(true)
            .map(|(offset, _)| offset)
            .chain(std::iter::once(value.len()))
            .collect::<Vec<_>>();

        let grapheme_count = real_boundaries.len().saturating_sub(1);
        let text = std::iter::repeat_n(PASSWORD_MASK, grapheme_count).collect::<String>();
        let mask_len = PASSWORD_MASK.len_utf8();
        let display_boundaries = (0..=grapheme_count).map(|index| index * mask_len).collect();
        Self {
            text: text.into(),
            masked: true,
            real_boundaries,
            display_boundaries,
        }
    }

    pub(super) fn display_offset(&self, real_offset: usize) -> usize {
        if !self.masked {
            return real_offset.min(self.text.len());
        }
        nearest_paired_boundary(real_offset, &self.real_boundaries, &self.display_boundaries)
    }

    pub(super) fn real_offset(&self, display_offset: usize) -> usize {
        if !self.masked {
            return display_offset.min(self.text.len());
        }
        nearest_paired_boundary(
            display_offset,
            &self.display_boundaries,
            &self.real_boundaries,
        )
    }

    pub(super) fn display_range(&self, range: Range<usize>) -> Range<usize> {
        self.display_offset(range.start)..self.display_offset(range.end)
    }
}
