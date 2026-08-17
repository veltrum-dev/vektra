//! Select 的窗口私有交互状态、导航、测量分页与 typeahead。

use gpui::{
    Bounds, Context, ElementId, Pixels, ScrollHandle, SharedString, Subscription, Task, Window,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use unicode_segmentation::UnicodeSegmentation as _;

const TYPEAHEAD_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Clone, PartialEq, Eq)]
pub(super) struct OptionSnapshot {
    pub(super) id: ElementId,
    pub(super) disabled: bool,
    pub(super) accessible_name: SharedString,
}

pub(super) struct SelectInteractionState {
    pub(super) open: bool,
    ready: bool,
    pub(super) active_id: Option<ElementId>,
    previous: Vec<OptionSnapshot>,
    option_bounds: Vec<(ElementId, Bounds<Pixels>)>,
    pub(super) scroll_handle: ScrollHandle,
    pending_scroll: bool,
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
            active_id: None,
            previous: Vec::new(),
            option_bounds: Vec::new(),
            scroll_handle: ScrollHandle::new(),
            pending_scroll: false,
            typeahead_buffer: String::new(),
            typeahead_generation: 0,
            typeahead_task: None,
            trigger_bounds: Rc::new(Cell::new(Bounds::default())),
            _activation_subscription: activation_subscription,
        }
    }

    pub(super) fn reconcile(
        &mut self,
        next: Vec<OptionSnapshot>,
        ready: bool,
        enabled: bool,
        preferred: Option<&ElementId>,
        cx: &mut Context<Self>,
    ) {
        self.ready = ready;
        self.option_bounds.retain(|(id, _)| {
            next.iter()
                .any(|option| option.id == *id && !option.disabled)
        });
        if !enabled {
            let changed = self.open || self.active_id.take().is_some();
            self.open = false;
            self.previous = next;
            self.pending_scroll = false;
            self.clear_typeahead();
            if changed {
                cx.notify();
            }
            return;
        }
        if !ready {
            let changed = self.active_id.take().is_some();
            self.previous = next;
            self.pending_scroll = false;
            self.clear_typeahead();
            if changed {
                cx.notify();
            }
            return;
        }

        let previous_active = self.active_id.clone();
        let options_changed = self.previous != next;
        if let Some(active) = self.active_id.as_ref()
            && next
                .iter()
                .any(|option| option.id == *active && !option.disabled)
        {
            self.previous = next;
            if options_changed {
                self.pending_scroll = true;
            }
            return;
        }

        self.active_id = reconciled_active_id(&self.previous, &next, self.active_id.as_ref());
        if self.open && self.active_id.is_none() {
            self.active_id = preferred
                .filter(|id| {
                    next.iter()
                        .any(|option| option.id == **id && !option.disabled)
                })
                .cloned()
                .or_else(|| {
                    next.iter()
                        .find(|option| !option.disabled)
                        .map(|option| option.id.clone())
                });
        }
        self.previous = next;
        if self.active_id != previous_active {
            self.pending_scroll = true;
            cx.notify();
        }
    }

    pub(super) fn open_with(
        &mut self,
        preferred: Option<ElementId>,
        from_end: bool,
        scroll_to_active: bool,
        cx: &mut Context<Self>,
    ) {
        self.open = true;
        if !self.ready {
            self.active_id = None;
            self.pending_scroll = false;
            cx.notify();
            return;
        }
        self.active_id = preferred
            .filter(|id| {
                self.previous
                    .iter()
                    .any(|option| option.id == *id && !option.disabled)
            })
            .or_else(|| {
                if from_end {
                    self.previous.iter().rev().find(|option| !option.disabled)
                } else {
                    self.previous.iter().find(|option| !option.disabled)
                }
                .map(|option| option.id.clone())
            });
        self.pending_scroll = scroll_to_active;
        cx.notify();
    }

    pub(super) fn close(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.open {
            return false;
        }
        self.open = false;
        self.pending_scroll = false;
        cx.notify();
        true
    }

    pub(super) fn move_active(&mut self, movement: ActiveMovement, cx: &mut Context<Self>) -> bool {
        if !self.ready {
            return false;
        }
        let enabled = self
            .previous
            .iter()
            .filter(|option| !option.disabled)
            .collect::<Vec<_>>();
        if enabled.is_empty() {
            return false;
        }
        let current = self
            .active_id
            .as_ref()
            .and_then(|id| enabled.iter().position(|option| option.id == *id));
        let target = match movement {
            ActiveMovement::Previous => current.unwrap_or(0).saturating_sub(1),
            ActiveMovement::Next => current.map_or(0, |index| (index + 1).min(enabled.len() - 1)),
            ActiveMovement::First => 0,
            ActiveMovement::Last => enabled.len() - 1,
            ActiveMovement::PagePrevious => self.page_target(&enabled, current, false),
            ActiveMovement::PageNext => self.page_target(&enabled, current, true),
        };
        let next = enabled[target].id.clone();
        if self.active_id.as_ref() == Some(&next) {
            return true;
        }
        self.active_id = Some(next);
        self.pending_scroll = true;
        cx.notify();
        true
    }

    pub(super) fn set_hovered(&mut self, id: ElementId, cx: &mut Context<Self>) {
        if !self.ready
            || !self
                .previous
                .iter()
                .any(|option| option.id == id && !option.disabled)
        {
            return;
        }
        self.pending_scroll = false;
        if self.active_id.as_ref() != Some(&id) {
            self.active_id = Some(id);
            cx.notify();
        }
    }

    pub(super) fn take_scroll_request(&mut self, id: &ElementId) -> bool {
        if self.ready && self.pending_scroll && self.active_id.as_ref() == Some(id) {
            self.pending_scroll = false;
            true
        } else {
            false
        }
    }

    pub(super) fn submittable_active_id(&self) -> Option<ElementId> {
        let active = self.active_id.as_ref()?;
        (self.ready
            && self
                .previous
                .iter()
                .any(|option| option.id == *active && !option.disabled))
        .then(|| active.clone())
    }

    pub(super) fn can_submit(&self, id: &ElementId) -> bool {
        self.ready
            && self
                .previous
                .iter()
                .any(|option| option.id == *id && !option.disabled)
    }

    pub(super) fn update_option_bounds(&mut self, id: ElementId, bounds: Bounds<Pixels>) {
        if let Some((_, previous)) = self
            .option_bounds
            .iter_mut()
            .find(|(option_id, _)| *option_id == id)
        {
            *previous = bounds;
        } else {
            self.option_bounds.push((id, bounds));
        }
    }

    fn page_target(
        &self,
        enabled: &[&OptionSnapshot],
        current: Option<usize>,
        forward: bool,
    ) -> usize {
        let current = current.unwrap_or(if forward { 0 } else { enabled.len() - 1 });
        let viewport = self.scroll_handle.bounds();
        if viewport.size.height <= Pixels::ZERO {
            return current;
        }
        let Some(current_bounds) = self.option_bounds_for(&enabled[current].id) else {
            return if forward {
                (current + 1).min(enabled.len() - 1)
            } else {
                current.saturating_sub(1)
            };
        };

        if forward {
            let boundary = if current_bounds.bottom() < viewport.bottom() {
                viewport.bottom()
            } else {
                current_bounds.bottom() + viewport.size.height
            };
            enabled
                .iter()
                .enumerate()
                .skip(current + 1)
                .take_while(|(_, option)| {
                    self.option_bounds_for(&option.id)
                        .is_some_and(|bounds| bounds.bottom() <= boundary)
                })
                .map(|(index, _)| index)
                .last()
                .unwrap_or(current)
        } else {
            let boundary = if current_bounds.top() > viewport.top() {
                viewport.top()
            } else {
                current_bounds.top() - viewport.size.height
            };
            enabled
                .iter()
                .enumerate()
                .take(current)
                .rev()
                .take_while(|(_, option)| {
                    self.option_bounds_for(&option.id)
                        .is_some_and(|bounds| bounds.top() >= boundary)
                })
                .map(|(index, _)| index)
                .last()
                .unwrap_or(current)
        }
    }

    fn option_bounds_for(&self, id: &ElementId) -> Option<Bounds<Pixels>> {
        self.option_bounds
            .iter()
            .find(|(option_id, _)| option_id == id)
            .map(|(_, bounds)| *bounds)
    }

    pub(super) fn typeahead(&mut self, text: &str, cx: &mut Context<Self>) -> bool {
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

        let start = self
            .active_id
            .as_ref()
            .and_then(|active| self.previous.iter().position(|option| option.id == *active))
            .unwrap_or_else(|| self.previous.len().saturating_sub(1));
        let next = (1..=self.previous.len()).find_map(|offset| {
            let option = &self.previous[(start + offset) % self.previous.len()];
            (!option.disabled && option.accessible_name.to_lowercase().starts_with(&query))
                .then(|| option.id.clone())
        });
        let Some(next) = next else {
            return false;
        };
        self.open = true;
        self.pending_scroll = true;
        if self.active_id.as_ref() != Some(&next) {
            self.active_id = Some(next);
        }
        cx.notify();
        true
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

pub(super) fn reconciled_active_id(
    previous: &[OptionSnapshot],
    next: &[OptionSnapshot],
    active_id: Option<&ElementId>,
) -> Option<ElementId> {
    let old_position =
        active_id.and_then(|active| previous.iter().position(|option| option.id == *active))?;
    next.iter()
        .skip(old_position.min(next.len()))
        .find(|option| !option.disabled)
        .or_else(|| {
            next.iter()
                .take(old_position.min(next.len()))
                .rev()
                .find(|option| !option.disabled)
        })
        .map(|option| option.id.clone())
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
