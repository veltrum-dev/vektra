use gpui::{
    App, Context, ElementId, IntoElement, KeyDownEvent, Keystroke, ParentElement, Render, Styled,
    TestAppContext, Window, div, px,
};
use std::{cell::Cell, ops::Range, rc::Rc};
use vektra::{LazyDataSource, Select, SelectDataSource, SelectEntry, SelectOption};

struct GeneratedSelectSource {
    count: usize,
    item_reads: Cell<usize>,
    range_requests: Cell<usize>,
}

impl GeneratedSelectSource {
    fn new(count: usize) -> Self {
        Self {
            count,
            item_reads: Cell::new(0),
            range_requests: Cell::new(0),
        }
    }

    fn enabled(index: usize) -> bool {
        !index.is_multiple_of(10)
    }
}

impl LazyDataSource for GeneratedSelectSource {
    type Item = SelectEntry<usize>;
    type Key = ElementId;

    fn item_count(&self) -> usize {
        self.count
    }

    fn revision(&self) -> u64 {
        1
    }

    fn key(&self, index: usize) -> Self::Key {
        ElementId::named_usize("million-option", index)
    }

    fn item(&self, index: usize) -> Option<Self::Item> {
        if index >= self.count {
            return None;
        }
        self.item_reads.set(self.item_reads.get() + 1);
        Some(SelectEntry::Option(
            SelectOption::new(self.key(index), index, format!("大数据选项 {index:07}"))
                .disabled(!Self::enabled(index)),
        ))
    }

    fn request_range(&self, _range: Range<usize>, _window: &mut Window, _cx: &mut App) {
        self.range_requests.set(self.range_requests.get() + 1);
    }
}

impl SelectDataSource<usize> for GeneratedSelectSource {
    fn index_of_key(&self, key: &ElementId) -> Option<usize> {
        match key {
            ElementId::NamedInteger(name, index) if name.as_ref() == "million-option" => {
                usize::try_from(*index)
                    .ok()
                    .filter(|index| *index < self.count)
            }
            _ => None,
        }
    }

    fn index_of_value(&self, value: &usize) -> Option<usize> {
        (*value < self.count).then_some(*value)
    }

    fn first_enabled(&self) -> Option<usize> {
        (0..self.count.min(10)).find(|index| Self::enabled(*index))
    }

    fn is_enabled(&self, index: usize) -> bool {
        index < self.count && Self::enabled(index)
    }

    fn last_enabled(&self) -> Option<usize> {
        let mut index = self.count.checked_sub(1)?;
        while !Self::enabled(index) {
            index = index.checked_sub(1)?;
        }
        Some(index)
    }

    fn next_enabled(&self, index: usize, forward: bool, wrap: bool) -> Option<usize> {
        let mut candidate = index;
        for _ in 0..=10 {
            candidate = if forward {
                match candidate
                    .checked_add(1)
                    .filter(|candidate| *candidate < self.count)
                {
                    Some(candidate) => candidate,
                    None if wrap => 0,
                    None => return None,
                }
            } else {
                match candidate.checked_sub(1) {
                    Some(candidate) => candidate,
                    None if wrap => self.count.checked_sub(1)?,
                    None => return None,
                }
            };
            if Self::enabled(candidate) {
                return Some(candidate);
            }
        }
        None
    }

    fn search_prefix(&self, query: &str, _after: Option<usize>) -> Option<usize> {
        query
            .strip_prefix("大数据选项 ")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|index| self.is_enabled(*index))
    }

    fn option_count(&self) -> usize {
        self.count
    }

    fn option_position(&self, index: usize) -> Option<usize> {
        (index < self.count).then_some(index)
    }
}

struct MillionSelectView {
    source: Rc<GeneratedSelectSource>,
    selected: Option<usize>,
}

impl Render for MillionSelectView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let source: Rc<dyn SelectDataSource<usize>> = self.source.clone();
        div().w(px(360.)).h(px(320.)).child(
            Select::new("million-select")
                .aria_label("百万项大数据 Select")
                .selected_value(self.selected)
                .data_source(source)
                .on_change_in(cx, |this, value, _, cx| {
                    this.selected = Some(value);
                    cx.notify();
                }),
        )
    }
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn key_down(key: &str, cx: &mut gpui::VisualTestContext) {
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse(key).unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
}

#[gpui::test]
fn million_item_select_reads_only_selected_and_visible_rows(cx: &mut TestAppContext) {
    let source = Rc::new(GeneratedSelectSource::new(1_000_000));
    let (_, cx) = cx.add_window_view(|_, _| MillionSelectView {
        source: source.clone(),
        selected: Some(500_001),
    });
    draw(cx);
    assert!(source.item_reads.get() < 10);

    cx.update(|window, cx| window.focus_next(cx));
    key_down("down", cx);
    draw(cx);
    assert!(source.item_reads.get() < 100);
    assert!(source.range_requests.get() > 0);

    key_down("end", cx);
    draw(cx);
    draw(cx);
    assert!(
        cx.debug_bounds("vektra-select-option-million-option-999999")
            .is_some()
    );
    assert!(source.item_reads.get() < 200);
}
