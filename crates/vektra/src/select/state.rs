//! Select 的窗口私有交互状态、索引导航与 typeahead。

use super::SelectDataSource;
use crate::VirtualListState;
use gpui::{Bounds, Context, ElementId, Pixels, Subscription, Task, Window};
use std::{cell::Cell, hash::Hash, rc::Rc, time::Duration};
use unicode_segmentation::UnicodeSegmentation as _;

const TYPEAHEAD_TIMEOUT: Duration = Duration::from_millis(500);

pub(super) struct SelectInteractionState {
    pub(super) open: bool,
    ready: bool,
    pub(super) active_index: Option<usize>,
    active_key: Option<ElementId>,
    preserve_active_on_open: bool,
    source_revision: u64,
    source_count: usize,
    pub(super) virtual_list: VirtualListState,
    typeahead_buffer: String,
    typeahead_generation: u64,
    typeahead_task: Option<Task<()>>,
    pub(super) trigger_bounds: Rc<Cell<Bounds<Pixels>>>,
    _activation_subscription: Subscription,
}

impl SelectInteractionState {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let activation_subscription = cx.observe_window_activation(window, |this, window, cx| {
            if !window.is_window_active() && this.open {
                this.open = false;
                cx.notify();
            }
        });
        Self {
            open: false,
            ready: false,
            active_index: None,
            active_key: None,
            preserve_active_on_open: false,
            source_revision: 0,
            source_count: 0,
            virtual_list: VirtualListState::new(),
            typeahead_buffer: String::new(),
            typeahead_generation: 0,
            typeahead_task: None,
            trigger_bounds: Rc::new(Cell::new(Bounds::default())),
            _activation_subscription: activation_subscription,
        }
    }

    pub(super) fn reconcile<T>(
        &mut self,
        source: &dyn SelectDataSource<T>,
        ready: bool,
        enabled: bool,
        preferred: Option<usize>,
        cx: &mut Context<Self>,
    ) where
        T: Clone + Eq + Hash + 'static,
    {
        self.ready = ready;
        let source_changed =
            self.source_revision != source.revision() || self.source_count != source.item_count();
        self.source_revision = source.revision();
        self.source_count = source.item_count();
        self.virtual_list
            .reconcile(self.source_count, self.source_revision);

        if !enabled {
            let changed = self.open || self.active_index.take().is_some();
            self.open = false;
            self.active_key = None;
            self.preserve_active_on_open = false;
            self.clear_typeahead();
            if changed {
                cx.notify();
            }
            return;
        }
        if !ready {
            let changed = self.active_index.take().is_some();
            self.active_key = None;
            self.preserve_active_on_open = false;
            self.clear_typeahead();
            if changed {
                cx.notify();
            }
            return;
        }

        let previous = self.active_index;
        if source_changed {
            let preserved = self
                .active_key
                .as_ref()
                .and_then(|key| source.index_of_key(key))
                .filter(|index| source.is_enabled(*index));
            if self.active_key.is_some() && preserved.is_none() {
                self.preserve_active_on_open = false;
            }
            self.active_index = preserved.or_else(|| reconciled_active_index(source, previous));
            self.active_key = self.active_index.map(|index| source.key(index));
        } else if self
            .active_index
            .is_some_and(|index| !source.is_enabled(index))
        {
            self.active_index = reconciled_active_index(source, previous);
            self.active_key = self.active_index.map(|index| source.key(index));
        }

        if self.open && self.active_index.is_none() {
            self.active_index = preferred
                .filter(|index| source.is_enabled(*index))
                .or_else(|| source.first_enabled());
            self.active_key = self.active_index.map(|index| source.key(index));
        }
        if self.active_index != previous {
            self.reveal_active();
            cx.notify();
        }
    }

    pub(super) fn open_with<T>(
        &mut self,
        source: &dyn SelectDataSource<T>,
        preferred: Option<usize>,
        from_end: bool,
        scroll_to_active: bool,
        cx: &mut Context<Self>,
    ) where
        T: Clone + Eq + Hash + 'static,
    {
        self.open = true;
        if !self.ready {
            self.active_index = None;
            self.active_key = None;
            cx.notify();
            return;
        }
        self.active_index = preferred
            .filter(|index| source.is_enabled(*index))
            .or_else(|| {
                self.preserve_active_on_open
                    .then_some(self.active_index)
                    .flatten()
                    .filter(|index| source.is_enabled(*index))
            })
            .or_else(|| {
                if from_end {
                    source.last_enabled()
                } else {
                    source.first_enabled()
                }
            });
        self.active_key = self.active_index.map(|index| source.key(index));
        self.preserve_active_on_open = self.active_index.is_some();
        if scroll_to_active {
            self.reveal_active();
        }
        cx.notify();
    }

    pub(super) fn close(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.open {
            return false;
        }
        self.open = false;
        cx.notify();
        true
    }

    pub(super) fn move_active<T>(
        &mut self,
        source: &dyn SelectDataSource<T>,
        movement: ActiveMovement,
        cx: &mut Context<Self>,
    ) -> bool
    where
        T: Clone + Eq + Hash + 'static,
    {
        if !self.ready || source.first_enabled().is_none() {
            return false;
        }
        let current = self.active_index;
        let next = match movement {
            ActiveMovement::Previous => current
                .and_then(|index| source.next_enabled(index, false, false))
                .or(current)
                .or_else(|| source.first_enabled()),
            ActiveMovement::Next => current
                .and_then(|index| source.next_enabled(index, true, false))
                .or(current)
                .or_else(|| source.first_enabled()),
            ActiveMovement::First => source.first_enabled(),
            ActiveMovement::Last => source.last_enabled(),
            ActiveMovement::PagePrevious => self.page_target(source, false),
            ActiveMovement::PageNext => self.page_target(source, true),
        };
        let Some(next) = next else {
            return false;
        };
        if self.active_index == Some(next) {
            return true;
        }
        self.active_index = Some(next);
        self.active_key = Some(source.key(next));
        self.preserve_active_on_open = true;
        self.reveal_active();
        cx.notify();
        true
    }

    pub(super) fn set_hovered<T>(
        &mut self,
        source: &dyn SelectDataSource<T>,
        index: usize,
        cx: &mut Context<Self>,
    ) where
        T: Clone + Eq + Hash + 'static,
    {
        if !self.ready || !source.is_enabled(index) || self.active_index == Some(index) {
            return;
        }
        self.active_index = Some(index);
        self.active_key = Some(source.key(index));
        self.preserve_active_on_open = true;
        cx.notify();
    }

    pub(super) fn submittable_active_index<T>(
        &self,
        source: &dyn SelectDataSource<T>,
    ) -> Option<usize>
    where
        T: Clone + Eq + Hash + 'static,
    {
        let active = self.active_index?;
        (self.ready && source.is_enabled(active)).then_some(active)
    }

    pub(super) fn can_submit<T>(&self, source: &dyn SelectDataSource<T>, index: usize) -> bool
    where
        T: Clone + Eq + Hash + 'static,
    {
        self.ready && source.is_enabled(index)
    }

    pub(super) fn typeahead<T>(
        &mut self,
        source: &dyn SelectDataSource<T>,
        text: &str,
        cx: &mut Context<Self>,
    ) -> bool
    where
        T: Clone + Eq + Hash + 'static,
    {
        if !self.ready {
            return false;
        }
        let input = text.to_lowercase();
        if input.is_empty() {
            return false;
        }
        let repeating = !self.typeahead_buffer.is_empty()
            && self
                .typeahead_buffer
                .graphemes(true)
                .all(|grapheme| grapheme.to_lowercase() == input);
        self.typeahead_buffer.push_str(&input);
        let query = if repeating {
            input
        } else {
            self.typeahead_buffer.clone()
        };
        self.restart_typeahead_timer(cx);

        let Some(next) = source.search_prefix(&query, self.active_index) else {
            return false;
        };
        self.open = true;
        self.active_index = Some(next);
        self.active_key = Some(source.key(next));
        self.preserve_active_on_open = true;
        self.reveal_active();
        cx.notify();
        true
    }

    fn page_target<T>(&self, source: &dyn SelectDataSource<T>, forward: bool) -> Option<usize>
    where
        T: Clone + Eq + Hash + 'static,
    {
        let mut target = self.active_index.or_else(|| {
            if forward {
                source.first_enabled()
            } else {
                source.last_enabled()
            }
        })?;
        let row_steps = self
            .virtual_list
            .visible_range()
            .len()
            .saturating_sub(2)
            .max(1);
        let boundary = if forward {
            target
                .saturating_add(row_steps)
                .min(self.source_count.saturating_sub(1))
        } else {
            target.saturating_sub(row_steps)
        };
        while let Some(next) = source.next_enabled(target, forward, false) {
            if (forward && next > boundary) || (!forward && next < boundary) {
                break;
            }
            target = next;
        }
        Some(target)
    }

    fn reveal_active(&self) {
        if let Some(index) = self.active_index {
            self.virtual_list.reveal_index(index);
        }
    }

    fn restart_typeahead_timer(&mut self, cx: &mut Context<Self>) {
        self.typeahead_generation = self.typeahead_generation.wrapping_add(1);
        let generation = self.typeahead_generation;
        self.typeahead_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(TYPEAHEAD_TIMEOUT).await;
            let _ = this.update(cx, |this, _| {
                if this.typeahead_generation == generation {
                    this.typeahead_buffer.clear();
                    this.typeahead_task = None;
                }
            });
        }));
    }

    fn clear_typeahead(&mut self) {
        self.typeahead_generation = self.typeahead_generation.wrapping_add(1);
        self.typeahead_buffer.clear();
        self.typeahead_task = None;
    }
}

pub(super) fn reconciled_active_index<T>(
    source: &dyn SelectDataSource<T>,
    previous: Option<usize>,
) -> Option<usize>
where
    T: Clone + Eq + Hash + 'static,
{
    let previous = previous?;
    source
        .next_enabled(previous.saturating_sub(1), true, false)
        .or_else(|| source.next_enabled(previous.saturating_add(1), false, false))
        .or_else(|| source.first_enabled())
}

#[derive(Clone, Copy)]
pub(super) enum ActiveMovement {
    Previous,
    Next,
    First,
    Last,
    PagePrevious,
    PageNext,
}
