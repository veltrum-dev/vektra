use allocation_counter::AllocationInfo;
use gpui::{
    AnyElement, AppContext, Context, ElementId, Entity, FocusHandle, InteractiveElement,
    IntoElement, KeyDownEvent, Keystroke, ParentElement, Render, ScrollHandle, SharedString,
    Styled, TestAppContext, VisualTestContext, Window, div, point, px,
};
use std::rc::Rc;
use std::{cell::Cell, hint::black_box, time::Duration};
use vektra::{
    Button, Checkbox, ComponentSize, IconButton, IconSource, Input, InputState, LazyDataSource,
    Radio, RadioGroup, ScrollAxis, ScrollGutter, ScrollVisibility, ScrollableExt, ScrollbarConfig,
    Select, SelectDataSource, SelectEntry, SelectOption, Switch, VirtualList, VirtualListState,
};

pub const SELECT_DATA_SIZES: &[usize] = &[1, 10, 100, 1_000, 10_000, 100_000];
pub const SELECT_RENDER_SIZES: &[usize] = &[1, 10, 100, 1_000, 10_000];
pub const INPUT_BYTE_SIZES: &[usize] = &[0, 16, 1_024, 64 * 1_024, 1_024 * 1_024];
pub const WALL_BUILD_SIZES: &[usize] = &[100, 1_000, 10_000, 100_000];
pub const WALL_RENDER_SIZES: &[usize] = &[100, 1_000, 10_000];
pub const SCROLLBAR_SIZES: &[usize] = &[1_000, 10_000, 100_000];
pub const VIRTUAL_LIST_SIZES: &[usize] = &[100, 1_000, 10_000, 100_000];

#[derive(Default)]
pub struct AllocationRecorder {
    samples: Cell<u64>,
    count_total: Cell<u128>,
    count_current: Cell<i128>,
    count_max: Cell<u128>,
    bytes_total: Cell<u128>,
    bytes_current: Cell<i128>,
    bytes_max: Cell<u128>,
}

impl AllocationRecorder {
    pub fn measure<T>(&self, operation: impl FnOnce() -> T) -> T {
        let mut output = None;
        let info = allocation_counter::measure(|| output = Some(operation()));
        self.record(info);
        output.expect("allocation measurement must run its operation")
    }

    pub fn report(&self, benchmark: &str) {
        let samples = self.samples.get();
        if samples == 0 {
            return;
        }
        let divisor = samples as f64;
        eprintln!(
            "VEKTRA_ALLOCATION benchmark={benchmark} samples={samples} allocations/op={:.2} \
             allocated_bytes/op={:.2} net_allocations/op={:.2} net_bytes/op={:.2} \
             peak_allocations/op={:.2} peak_bytes/op={:.2}",
            self.count_total.get() as f64 / divisor,
            self.bytes_total.get() as f64 / divisor,
            self.count_current.get() as f64 / divisor,
            self.bytes_current.get() as f64 / divisor,
            self.count_max.get() as f64 / divisor,
            self.bytes_max.get() as f64 / divisor,
        );
    }

    fn record(&self, info: AllocationInfo) {
        self.samples.set(self.samples.get() + 1);
        self.count_total
            .set(self.count_total.get() + u128::from(info.count_total));
        self.count_current
            .set(self.count_current.get() + i128::from(info.count_current));
        self.count_max
            .set(self.count_max.get() + u128::from(info.count_max));
        self.bytes_total
            .set(self.bytes_total.get() + u128::from(info.bytes_total));
        self.bytes_current
            .set(self.bytes_current.get() + i128::from(info.bytes_current));
        self.bytes_max
            .set(self.bytes_max.get() + u128::from(info.bytes_max));
    }
}

pub fn mixed_text(target_bytes: usize, alternate: bool) -> SharedString {
    if target_bytes == 0 {
        return "".into();
    }
    let pattern = if alternate {
        "B界o\u{308}ب🚀"
    } else {
        "A中e\u{301}א🙂"
    };
    let mut text = String::with_capacity(target_bytes);
    while text.len() < target_bytes {
        let start = text.len();
        for character in pattern.chars() {
            if text.len() + character.len_utf8() > target_bytes {
                break;
            }
            text.push(character);
        }
        if text.len() == start {
            text.push('x');
        }
    }
    text.into()
}

pub fn select_tree(count: usize, disabled_percent: usize, generation: usize) -> Select<usize> {
    let mut select = Select::new("benchmark-select")
        .selected_value(Some(enabled_position(count, disabled_percent, count / 2)))
        .aria_label("可扩展性选择器");
    for index in 0..count {
        select = select.option(
            SelectOption::new(
                element_id("benchmark-option", index),
                index,
                option_label(index, generation),
            )
            .disabled(is_disabled(index, disabled_percent)),
        );
    }
    select
}

pub fn component_wall(count: usize, kind: WallKind, changed_percent: usize) -> Vec<AnyElement> {
    (0..count)
        .map(|index| wall_component(index, kind, is_changed(index, changed_percent)))
        .collect()
}

pub fn tooltip_wall(count: usize) -> Vec<AnyElement> {
    (0..count)
        .map(|index| {
            Button::new(element_id("tooltip-wall", index))
                .label(format!("Trigger {index}"))
                .tooltip(format!("Tooltip {index}"))
                .into_any_element()
        })
        .collect()
}

pub fn icon_wall(count: usize, unique_paths: bool) -> Vec<AnyElement> {
    (0..count)
        .map(|index| {
            let path = if unique_paths {
                format!("benchmark/icons/icon-{index}.svg")
            } else {
                "components/input/invalid.svg".to_owned()
            };
            vektra::Icon::new(IconSource::asset(path)).into_any_element()
        })
        .collect()
}

pub fn scrollbar_tree(count: usize, axis: ScrollAxis, gutter: ScrollGutter) -> impl IntoElement {
    let mut content = div();
    for index in 0..count {
        content = content.child(
            div()
                .id(element_id("scroll-child", index))
                .w(px(640.))
                .h(px(20.))
                .child(format!("row {index}")),
        );
    }
    div()
        .w(px(320.))
        .h(px(240.))
        .child(content)
        .scrollbar_with(ScrollbarConfig {
            axis,
            visibility: ScrollVisibility::Always,
            gutter,
        })
}

#[derive(Clone, Copy, Debug)]
pub enum WallKind {
    Button,
    Checkbox,
    Switch,
    Radio,
    IconButton,
    Mixed,
}

impl WallKind {
    pub const ALL: [Self; 6] = [
        Self::Button,
        Self::Checkbox,
        Self::Switch,
        Self::Radio,
        Self::IconButton,
        Self::Mixed,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Checkbox => "checkbox",
            Self::Switch => "switch",
            Self::Radio => "radio",
            Self::IconButton => "icon_button",
            Self::Mixed => "mixed",
        }
    }
}

pub struct SelectFixture {
    pub view: Entity<SelectBenchView>,
    pub cx: VisualTestContext,
}

impl SelectFixture {
    pub fn new(
        count: usize,
        disabled_percent: usize,
        active_position: usize,
        draw_once: bool,
    ) -> Self {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|window, cx| {
            SelectBenchView::new(count, disabled_percent, active_position, window, cx)
        });
        let mut fixture = Self {
            view,
            cx: cx.clone(),
        };
        if draw_once {
            fixture.draw();
        }
        fixture
    }

    pub fn draw(&mut self) {
        draw(&mut self.cx);
    }

    pub fn focus_trigger(&mut self) {
        let focus = self
            .view
            .read_with(&self.cx, |view, _| view.root_focus.clone());
        self.cx.update(|window, cx| {
            cx.activate(true);
            window.activate_window();
            window.focus(&focus, cx);
        });
        self.draw();
        self.cx.update(|window, cx| window.focus_next(cx));
        self.draw();
    }

    pub fn open(&mut self) {
        self.focus_trigger();
        self.key("down");
        self.draw();
    }

    pub fn key(&mut self, key: &str) {
        self.cx.simulate_event(KeyDownEvent {
            keystroke: Keystroke::parse(key).expect("benchmark keystroke must be valid"),
            is_held: false,
            prefer_character_input: false,
        });
    }

    pub fn advance_typeahead_timeout(&mut self) {
        self.cx.executor().advance_clock(Duration::from_millis(500));
        self.cx.run_until_parked();
    }

    pub fn update_options(&mut self) {
        self.view.update(&mut self.cx, |view, cx| {
            view.generation ^= 1;
            cx.notify();
        });
        self.draw();
    }
}

impl Drop for SelectFixture {
    fn drop(&mut self) {
        let _ = self.cx.simulate_close();
        self.cx.quit();
    }
}

pub struct GeneratedSelectSource {
    count: usize,
}

impl GeneratedSelectSource {
    pub fn new(count: usize) -> Self {
        Self { count }
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
        ElementId::named_usize("benchmark-lazy-option", index)
    }

    fn item(&self, index: usize) -> Option<Self::Item> {
        (index < self.count).then(|| {
            SelectEntry::Option(SelectOption::new(
                self.key(index),
                index,
                format!("Lazy option {index:07}"),
            ))
        })
    }
}

impl SelectDataSource<usize> for GeneratedSelectSource {
    fn index_of_key(&self, key: &ElementId) -> Option<usize> {
        match key {
            ElementId::NamedInteger(name, index) if name.as_ref() == "benchmark-lazy-option" => {
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
        (self.count > 0).then_some(0)
    }

    fn is_enabled(&self, index: usize) -> bool {
        index < self.count
    }

    fn last_enabled(&self) -> Option<usize> {
        self.count.checked_sub(1)
    }

    fn next_enabled(&self, index: usize, forward: bool, wrap: bool) -> Option<usize> {
        if forward {
            index
                .checked_add(1)
                .filter(|index| *index < self.count)
                .or_else(|| (wrap && self.count > 0).then_some(0))
        } else {
            index
                .checked_sub(1)
                .or_else(|| wrap.then(|| self.count.saturating_sub(1)))
        }
    }

    fn search_prefix(&self, query: &str, _: Option<usize>) -> Option<usize> {
        query
            .strip_prefix("lazy option ")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|index| *index < self.count)
    }

    fn option_count(&self) -> usize {
        self.count
    }

    fn option_position(&self, index: usize) -> Option<usize> {
        (index < self.count).then_some(index)
    }
}

pub struct LazySelectFixture {
    pub cx: VisualTestContext,
}

impl LazySelectFixture {
    pub fn new(count: usize, draw_once: bool) -> Self {
        let source: Rc<dyn SelectDataSource<usize>> = Rc::new(GeneratedSelectSource::new(count));
        let mut app = TestAppContext::single();
        let (_, cx) = app
            .add_window_view(move |window, cx| LazySelectBenchView::new(count, source, window, cx));
        let mut fixture = Self { cx: cx.clone() };
        if draw_once {
            fixture.draw();
        }
        fixture
    }

    pub fn draw(&mut self) {
        draw(&mut self.cx);
    }

    pub fn open(&mut self) {
        self.cx.update(|window, cx| window.focus_next(cx));
        self.cx.simulate_event(KeyDownEvent {
            keystroke: Keystroke::parse("down").expect("benchmark key must parse"),
            is_held: false,
            prefer_character_input: false,
        });
        self.draw();
    }

    pub fn end_and_draw(&mut self) {
        self.cx.simulate_event(KeyDownEvent {
            keystroke: Keystroke::parse("end").expect("benchmark key must parse"),
            is_held: false,
            prefer_character_input: false,
        });
        self.draw();
    }
}

impl Drop for LazySelectFixture {
    fn drop(&mut self) {
        let _ = self.cx.simulate_close();
        self.cx.quit();
    }
}

struct LazySelectBenchView {
    count: usize,
    source: Rc<dyn SelectDataSource<usize>>,
    root_focus: FocusHandle,
}

impl LazySelectBenchView {
    fn new(
        count: usize,
        source: Rc<dyn SelectDataSource<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            count,
            source,
            root_focus,
        }
    }
}

impl Render for LazySelectBenchView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.root_focus)
            .w(px(320.))
            .h(px(360.))
            .child(
                Select::new("benchmark-lazy-select")
                    .selected_value((self.count > 0).then_some(self.count / 2))
                    .aria_label("惰性 Select benchmark")
                    .data_source(self.source.clone()),
            )
    }
}

pub struct VirtualListFixture {
    pub state: VirtualListState,
    pub cx: VisualTestContext,
}

pub struct TooltipFixture {
    pub cx: VisualTestContext,
}

impl TooltipFixture {
    pub fn new(count: usize, draw_once: bool) -> Self {
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(move |_, _| TooltipBenchView { count });
        let mut fixture = Self { cx: cx.clone() };
        if draw_once {
            fixture.draw();
        }
        fixture
    }

    pub fn draw(&mut self) {
        draw(&mut self.cx);
    }

    pub fn focus_delay_and_draw(&mut self) {
        self.cx.update(|window, cx| {
            cx.activate(true);
            window.activate_window();
            window.focus_next(cx);
        });
        self.cx.executor().advance_clock(Duration::from_millis(800));
        self.cx.run_until_parked();
        self.draw();
    }
}

impl Drop for TooltipFixture {
    fn drop(&mut self) {
        let _ = self.cx.simulate_close();
        self.cx.quit();
    }
}

struct TooltipBenchView {
    count: usize,
}

impl Render for TooltipBenchView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_wrap().children(tooltip_wall(self.count))
    }
}

impl VirtualListFixture {
    pub fn new(count: usize, draw_once: bool) -> Self {
        let state = VirtualListState::new();
        let view_state = state.clone();
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(move |_, _| VirtualListBenchView {
            count,
            state: view_state,
        });
        let mut fixture = Self {
            state,
            cx: cx.clone(),
        };
        if draw_once {
            fixture.draw();
        }
        fixture
    }

    pub fn draw(&mut self) {
        draw(&mut self.cx);
    }

    pub fn jump_and_draw(&mut self, index: usize) {
        self.state.scroll_to_index(index);
        self.draw();
    }
}

impl Drop for VirtualListFixture {
    fn drop(&mut self) {
        let _ = self.cx.simulate_close();
        self.cx.quit();
    }
}

struct VirtualListBenchView {
    count: usize,
    state: VirtualListState,
}

impl Render for VirtualListBenchView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(320.)).h(px(240.)).child(
            VirtualList::new(
                "benchmark-virtual-list",
                self.state.clone(),
                self.count,
                px(24.),
                |index| ElementId::named_usize("benchmark-virtual-row", index),
                |index, _, _| div().child(format!("row {index}")),
            )
            .scrollbar(ScrollbarConfig::new().visibility(ScrollVisibility::Always)),
        )
    }
}

pub struct SelectBenchView {
    count: usize,
    disabled_percent: usize,
    active_position: usize,
    generation: usize,
    root_focus: FocusHandle,
}

impl SelectBenchView {
    fn new(
        count: usize,
        disabled_percent: usize,
        active_position: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.activate(true);
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            count,
            disabled_percent,
            active_position: enabled_position(count, disabled_percent, active_position),
            generation: 0,
            root_focus,
        }
    }
}

impl Render for SelectBenchView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.root_focus)
            .w(px(320.))
            .h(px(360.))
            .child(
                select_tree(self.count, self.disabled_percent, self.generation)
                    .selected_value(Some(self.active_position)),
            )
    }
}

pub struct InputFixture {
    pub view: Entity<InputBenchView>,
    pub cx: VisualTestContext,
}

impl InputFixture {
    pub fn new(value: SharedString, draw_once: bool) -> Self {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| InputBenchView::new(value, cx));
        let mut fixture = Self {
            view,
            cx: cx.clone(),
        };
        if draw_once {
            fixture.draw();
        }
        fixture
    }

    pub fn draw(&mut self) {
        draw(&mut self.cx);
    }

    pub fn focus(&mut self) {
        let state = self.state();
        let focus = state.read_with(&self.cx, |state, _| state.focus_handle().clone());
        self.cx.update(|window, cx| {
            cx.activate(true);
            window.activate_window();
            window.focus(&focus, cx);
            window.draw(cx).clear(cx);
        });
    }

    pub fn state(&self) -> Entity<InputState> {
        self.view.read_with(&self.cx, |view, _| view.state.clone())
    }

    pub fn set_value(&mut self, value: SharedString) {
        self.state()
            .update(&mut self.cx, |state, cx| state.set_value(value, cx));
    }

    pub fn key_and_draw(&mut self, key: &str) {
        self.cx.simulate_keystrokes(key);
        self.draw();
    }

    pub fn input_and_draw(&mut self, text: &str) {
        self.cx.simulate_input(text);
        self.draw();
    }
}

impl Drop for InputFixture {
    fn drop(&mut self) {
        let _ = self.cx.simulate_close();
        self.cx.quit();
    }
}

pub struct InputBenchView {
    state: Entity<InputState>,
}

impl InputBenchView {
    fn new(value: SharedString, cx: &mut Context<Self>) -> Self {
        Self {
            state: cx.new(|cx| InputState::new(value, cx)),
        }
    }
}

impl Render for InputBenchView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(640.)).child(
            Input::new("benchmark-input", self.state.clone())
                .aria_label("混合 Unicode 基准输入")
                .placeholder("benchmark"),
        )
    }
}

pub struct WallFixture {
    pub view: Entity<WallBenchView>,
    pub cx: VisualTestContext,
}

impl WallFixture {
    pub fn new(count: usize, kind: WallKind, draw_once: bool) -> Self {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, _| WallBenchView {
            count,
            kind,
            changed_percent: 0,
        });
        let mut fixture = Self {
            view,
            cx: cx.clone(),
        };
        if draw_once {
            fixture.draw();
        }
        fixture
    }

    pub fn draw(&mut self) {
        draw(&mut self.cx);
    }

    pub fn update_and_draw(&mut self, changed_percent: usize) {
        self.view.update(&mut self.cx, |view, cx| {
            view.changed_percent = changed_percent;
            cx.notify();
        });
        self.draw();
    }
}

impl Drop for WallFixture {
    fn drop(&mut self) {
        let _ = self.cx.simulate_close();
        self.cx.quit();
    }
}

pub struct WallBenchView {
    count: usize,
    kind: WallKind,
    changed_percent: usize,
}

impl Render for WallBenchView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_wrap().children(component_wall(
            self.count,
            self.kind,
            self.changed_percent,
        ))
    }
}

pub struct ScrollbarFixture {
    pub cx: VisualTestContext,
    pub handle: ScrollHandle,
}

impl ScrollbarFixture {
    pub fn new(count: usize, axis: ScrollAxis, gutter: ScrollGutter, draw_once: bool) -> Self {
        let handle = ScrollHandle::new();
        let fixture_handle = handle.clone();
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(move |_, _| ScrollbarBenchView {
            count,
            axis,
            gutter,
            handle: fixture_handle,
        });
        let mut fixture = Self {
            cx: cx.clone(),
            handle,
        };
        if draw_once {
            fixture.draw();
        }
        fixture
    }

    pub fn draw(&mut self) {
        draw(&mut self.cx);
    }

    pub fn scroll_fraction_and_draw(&mut self, fraction: f32) {
        let max = self.handle.max_offset();
        self.handle
            .set_offset(point(max.x * fraction, max.y * fraction));
        self.draw();
    }
}

impl Drop for ScrollbarFixture {
    fn drop(&mut self) {
        let _ = self.cx.simulate_close();
        self.cx.quit();
    }
}

struct ScrollbarBenchView {
    count: usize,
    axis: ScrollAxis,
    gutter: ScrollGutter,
    handle: ScrollHandle,
}

impl Render for ScrollbarBenchView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut content = div();
        for index in 0..self.count {
            content = content.child(
                div()
                    .id(element_id("full-scroll-child", index))
                    .w(px(640.))
                    .h(px(20.))
                    .child(format!("row {index}")),
            );
        }
        div()
            .w(px(320.))
            .h(px(240.))
            .child(content)
            .scrollbar_for(&self.handle)
            .scrollbar_axis(self.axis)
            .scrollbar_visibility(ScrollVisibility::Always)
            .scrollbar_gutter(self.gutter)
    }
}

pub fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn wall_component(index: usize, kind: WallKind, changed: bool) -> AnyElement {
    let actual_kind = match kind {
        WallKind::Mixed => WallKind::ALL[index % (WallKind::ALL.len() - 1)],
        other => other,
    };
    let id = element_id("wall", index);
    match actual_kind {
        WallKind::Button => Button::new(id)
            .label(format!("Action {index}"))
            .selected(changed)
            .into_any_element(),
        WallKind::Checkbox => Checkbox::new(id)
            .label(format!("Check {index}"))
            .checked(changed)
            .into_any_element(),
        WallKind::Switch => Switch::new(id)
            .label(format!("Switch {index}"))
            .checked(changed)
            .into_any_element(),
        WallKind::Radio => RadioGroup::new(id)
            .selected_value(changed.then_some(index))
            .size(ComponentSize::Sm)
            .child(
                Radio::new(element_id("wall-radio", index), index).label(format!("Radio {index}")),
            )
            .into_any_element(),
        WallKind::IconButton => {
            IconButton::new(id, IconSource::asset("components/input/invalid.svg"))
                .aria_label(format!("Icon action {index}"))
                .selected(changed)
                .into_any_element()
        }
        WallKind::Mixed => unreachable!("mixed is resolved before component construction"),
    }
}

fn element_id(prefix: &str, index: usize) -> ElementId {
    SharedString::from(format!("{prefix}-{index}")).into()
}

fn option_label(index: usize, generation: usize) -> SharedString {
    format!("Benchmark {generation} option {index:06}").into()
}

fn is_disabled(index: usize, disabled_percent: usize) -> bool {
    index % 100 < disabled_percent
}

fn is_changed(index: usize, changed_percent: usize) -> bool {
    index % 100 < changed_percent
}

fn enabled_position(count: usize, disabled_percent: usize, preferred: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (preferred.min(count - 1)..count)
        .chain(0..preferred.min(count))
        .find(|index| !is_disabled(*index, disabled_percent))
        .unwrap_or(0)
}

pub fn consume<T>(value: T) {
    black_box(value);
}
