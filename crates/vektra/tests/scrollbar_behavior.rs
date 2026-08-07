use gpui::{
    Context, InteractiveElement, IntoElement, KeyDownEvent, Keystroke, Modifiers, ParentElement,
    Pixels, Render, ScrollDelta, ScrollHandle, ScrollWheelEvent, Styled, TestAppContext,
    TouchPhase, Window, div, point, px,
};
use vektra::{
    ScrollArea, ScrollAxis, ScrollGutter, ScrollVisibility, ScrollableExt, ScrollbarConfig,
    ThemeMode, set_theme_mode,
};

#[test]
fn public_builders_cover_default_shortcuts_config_and_external_handle() {
    let handle = ScrollHandle::new();
    let _: ScrollArea = div().scrollbar();
    let _: ScrollArea = div().vertical_scrollbar();
    let _: ScrollArea = div().horizontal_scrollbar();
    let _: ScrollArea = div().scrollbar_for(&handle);
    let _: ScrollArea = div().vertical_scrollbar_for(&handle);
    let _: ScrollArea = div().horizontal_scrollbar_for(&handle);
    let _: ScrollArea = div().scrollbar_with(ScrollbarConfig {
        axis: ScrollAxis::Both,
        visibility: ScrollVisibility::Always,
        gutter: ScrollGutter::Stable,
    });
    let _: ScrollArea = div()
        .scrollbar_with_axis(ScrollAxis::Vertical)
        .scrollbar_visibility(ScrollVisibility::Always)
        .scrollbar_gutter(ScrollGutter::Stable)
        .scrollbar_id("scroll-area")
        .scrollbar_aria_label("滚动内容");
    let _: ScrollArea = div().id("existing-id").scrollbar();
}

struct ScrollbarView {
    handle: ScrollHandle,
}

impl Render for ScrollbarView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("viewport")
            .debug_selector(|| "scrollbar-test-viewport".into())
            .w(px(200.))
            .h(px(160.))
            .child(div().w(px(600.)).h(px(640.)).child("overflow"))
            .scrollbar_for(&self.handle)
            .scrollbar_visibility(ScrollVisibility::Always)
            .scrollbar_aria_label("测试滚动区域")
    }
}

struct TinyScrollbarView;

impl Render for TinyScrollbarView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(28.))
            .h(px(20.))
            .child(div().w(px(120.)).h(px(100.)))
            .scrollbar_with(ScrollbarConfig {
                axis: ScrollAxis::Both,
                visibility: ScrollVisibility::Always,
                gutter: ScrollGutter::Stable,
            })
    }
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

#[gpui::test]
fn both_axes_scroll_and_external_handle_stays_authoritative(cx: &mut TestAppContext) {
    let handle = ScrollHandle::new();
    let (_, cx) = cx.add_window_view(|_, _| ScrollbarView {
        handle: handle.clone(),
    });
    draw(cx);

    assert_eq!(handle.max_offset(), point(px(400.), px(480.)));
    let viewport = cx.debug_bounds("scrollbar-test-viewport").unwrap();
    cx.simulate_event(ScrollWheelEvent {
        position: viewport.center(),
        delta: ScrollDelta::Pixels(point(px(-90.), px(0.))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    cx.simulate_event(ScrollWheelEvent {
        position: viewport.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-120.))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    draw(cx);

    assert!(handle.offset().x < Pixels::ZERO);
    assert!(handle.offset().y < Pixels::ZERO);
}

#[gpui::test]
fn keyboard_and_thumb_drag_update_the_same_scroll_handle(cx: &mut TestAppContext) {
    let handle = ScrollHandle::new();
    let (_, cx) = cx.add_window_view(|_, _| ScrollbarView {
        handle: handle.clone(),
    });
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse("pagedown").unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
    assert!(handle.offset().y < px(-100.));

    let viewport = cx.debug_bounds("scrollbar-test-viewport").unwrap();
    let track_x = viewport.right() - px(7.);
    let thumb_y = viewport.top() + px(40.);
    cx.simulate_mouse_down(
        point(track_x, thumb_y),
        gpui::MouseButton::Left,
        Modifiers::none(),
    );
    cx.simulate_mouse_move(
        point(track_x, viewport.bottom() - px(28.)),
        Some(gpui::MouseButton::Left),
        Modifiers::none(),
    );
    cx.simulate_mouse_up(
        point(track_x, viewport.bottom() - px(28.)),
        gpui::MouseButton::Left,
        Modifiers::none(),
    );
    draw(cx);

    assert!(handle.offset().y < px(-300.));
}

#[gpui::test]
fn renders_in_all_theme_modes_and_tiny_bounds(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| TinyScrollbarView);

    for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
        cx.update(|_, cx| set_theme_mode(mode, cx));
        draw(cx);
    }
}
