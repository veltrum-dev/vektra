use gpui::{
    Context, ElementId, InteractiveElement, IntoElement, KeyDownEvent, Keystroke, Modifiers,
    MouseButton, ParentElement, Render, ScrollDelta, ScrollWheelEvent, SharedString, Styled,
    TestAppContext, TouchPhase, Window, div, point, px,
};
use vektra::{
    LazyDataSource, OwnedDataSource, ScrollGutter, ScrollVisibility, ScrollbarConfig, VirtualList,
    VirtualListState,
};

#[test]
fn owned_vec_and_array_use_the_same_lazy_protocol() {
    let from_vec = OwnedDataSource::from_vec(vec![10, 20, 30], |index, _| index);
    let from_array = OwnedDataSource::from_array([10, 20, 30], |index, _| index);

    assert_eq!(from_vec.item_count(), 3);
    assert_eq!(from_array.item_count(), 3);
    assert_eq!(from_vec.item(1), Some(20));
    assert_eq!(from_array.key(2), 2);
}

struct PrecisionMillionRowView {
    state: VirtualListState,
}

impl Render for PrecisionMillionRowView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let list = VirtualList::new(
            "precision-million-row-list",
            self.state.clone(),
            1_000_000,
            px(48.),
            |index| ElementId::from(SharedString::from(format!("precision-row-{index}"))),
            |index, _, _| {
                div()
                    .id(("precision-row-content", index))
                    .debug_selector(move || format!("precision-row-{index}"))
                    .w_full()
                    .h_full()
                    .child(format!("第 {index} 行"))
            },
        )
        .scrollbar(
            ScrollbarConfig::new()
                .visibility(ScrollVisibility::Always)
                .gutter(ScrollGutter::Stable),
        );
        div().w(px(320.)).h(px(240.)).child(list)
    }
}

struct MillionRowView {
    state: VirtualListState,
    item_count: usize,
}

impl Render for MillionRowView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let list = VirtualList::new(
            "million-row-list",
            self.state.clone(),
            self.item_count,
            px(24.),
            |index| ElementId::from(SharedString::from(format!("row-{index}"))),
            |index, _, _| div().child(format!("第 {index} 行")),
        )
        .aria_label("百万项虚拟列表")
        .scrollbar(
            ScrollbarConfig::new()
                .visibility(ScrollVisibility::Always)
                .gutter(ScrollGutter::Stable),
        );
        div().w(px(320.)).h(px(240.)).child(list)
    }
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

#[gpui::test]
fn million_rows_materialize_only_the_viewport_and_support_large_jumps(cx: &mut TestAppContext) {
    let state = VirtualListState::new();
    let (_, cx) = cx.add_window_view(|_, _| MillionRowView {
        state: state.clone(),
        item_count: 1_000_000,
    });

    draw(cx);
    let first = state.metrics();
    assert!(first.materialized_rows > 0);
    assert!(
        first.materialized_rows <= 16,
        "240px 视口与 24px 行高不应物化超过 16 行，实际为 {}",
        first.materialized_rows
    );
    assert_eq!(first.cached_rows, 0);
    assert_eq!(first.max_cached_rows, 0);

    let viewport = state.scroll_handle().bounds();
    cx.simulate_event(ScrollWheelEvent {
        position: viewport.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-120.))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    draw(cx);
    assert!(state.scroll_handle().offset().y < px(0.));

    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse("pagedown").unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
    draw(cx);
    assert!(state.scroll_handle().offset().y < px(-120.));

    state.scroll_to_index(900_000);
    draw(cx);
    let jumped = state.metrics();
    assert!(jumped.visible_range.contains(&900_000));
    assert!(jumped.materialized_rows <= 16);

    state.scroll_to_start();
    draw(cx);
    assert!(state.visible_range().contains(&0));
}

#[gpui::test]
fn million_rows_preserve_one_pixel_scrolls_at_high_indices(cx: &mut TestAppContext) {
    let state = VirtualListState::new();
    let (_, cx) = cx.add_window_view(|_, _| PrecisionMillionRowView {
        state: state.clone(),
    });
    draw(cx);

    for scale_factor in [1., 1.25, 1.5, 2.] {
        cx.update(|window, _| window.set_scale_factor(scale_factor));
        state.scroll_to_index(900_000);
        draw(cx);
        let selector = "precision-row-900000";
        let first = cx.debug_bounds(selector).unwrap();
        let viewport = state.scroll_handle().bounds();
        let mut previous_top = first.top();

        for _ in 0..2 {
            cx.simulate_event(ScrollWheelEvent {
                position: viewport.center(),
                delta: ScrollDelta::Pixels(point(px(0.), px(-1.))),
                modifiers: Modifiers::none(),
                touch_phase: TouchPhase::Moved,
            });
            draw(cx);

            let moved = cx.debug_bounds(selector).unwrap();
            let physical_step = (previous_top - moved.top()).as_f32() * scale_factor;
            assert!(
                (0.5..=2.5).contains(&physical_step),
                "{scale_factor} 倍缩放下的高位 1px 滚动被吞掉或跳动：{physical_step} 物理像素"
            );
            previous_top = moved.top();
            assert!(state.visible_range().contains(&900_000));
            assert!(state.metrics().materialized_rows <= 8);
        }
    }
}

#[gpui::test]
fn million_row_scrollbar_drag_updates_the_logical_visible_range(cx: &mut TestAppContext) {
    let state = VirtualListState::new();
    let (_, cx) = cx.add_window_view(|_, _| PrecisionMillionRowView {
        state: state.clone(),
    });
    draw(cx);

    let viewport = state.scroll_handle().bounds();
    let track_x = viewport.right() - px(7.);
    let target_y = viewport.top() + viewport.size.height * 0.9;
    cx.simulate_mouse_down(
        point(track_x, target_y),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.simulate_mouse_up(
        point(track_x, target_y),
        MouseButton::Left,
        Modifiers::none(),
    );
    draw(cx);

    let visible = state.visible_range();
    assert!(visible.start > 800_000, "拖动后实际范围为 {visible:?}");
    assert!(visible.end <= 1_000_000);
    assert!(state.metrics().materialized_rows <= 8);
}
