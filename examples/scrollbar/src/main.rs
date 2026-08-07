#![cfg_attr(target_family = "wasm", no_main)]

#[path = "../../shared.rs"]
mod shared;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    Orientation, ParentElement, Render, Styled, Window, WindowBounds, WindowOptions, actions, div,
    px, size,
};
use gpui_platform::application;
use vektra::{
    Radio, RadioGroup, ScrollAxis, ScrollGutter, ScrollVisibility, ScrollableExt, ScrollbarConfig,
    current_theme,
};

actions!(vektra_scrollbar_example, [Tab, TabPrev]);

struct ScrollbarExample {
    focus_handle: FocusHandle,
    axis: ScrollAxis,
    visibility: ScrollVisibility,
    gutter: ScrollGutter,
}

impl ScrollbarExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            focus_handle,
            axis: ScrollAxis::Both,
            visibility: ScrollVisibility::Always,
            gutter: ScrollGutter::Overlay,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}

impl Render for ScrollbarExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);
        let axis_control = RadioGroup::new("example-scrollbar-axis")
            .selected_value(Some(self.axis))
            .orientation(Orientation::Horizontal)
            .aria_label("滚动轴")
            .on_change_in(cx, |this, axis, _, cx| {
                this.axis = axis;
                cx.notify();
            })
            .child(Radio::new("axis-vertical", ScrollAxis::Vertical).label("Vertical"))
            .child(Radio::new("axis-horizontal", ScrollAxis::Horizontal).label("Horizontal"))
            .child(Radio::new("axis-both", ScrollAxis::Both).label("Both"));
        let visibility_control = RadioGroup::new("example-scrollbar-visibility")
            .selected_value(Some(self.visibility))
            .orientation(Orientation::Horizontal)
            .aria_label("显隐")
            .on_change_in(cx, |this, visibility, _, cx| {
                this.visibility = visibility;
                cx.notify();
            })
            .child(Radio::new("visibility-auto", ScrollVisibility::Auto).label("Auto"))
            .child(Radio::new("visibility-always", ScrollVisibility::Always).label("Always"))
            .child(Radio::new("visibility-never", ScrollVisibility::Never).label("Never"));
        let gutter_control = RadioGroup::new("example-scrollbar-gutter")
            .selected_value(Some(self.gutter))
            .orientation(Orientation::Horizontal)
            .aria_label("布局")
            .on_change_in(cx, |this, gutter, _, cx| {
                this.gutter = gutter;
                cx.notify();
            })
            .child(Radio::new("gutter-overlay", ScrollGutter::Overlay).label("Overlay"))
            .child(Radio::new("gutter-stable", ScrollGutter::Stable).label("Stable"));

        div()
            .id("vektra-scrollbar-example")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .w(px(920.))
                    .child(div().text_size(px(24.)).child("Vektra Scrollbar"))
                    .child("默认展示 Both + Always：X/Y 两轴都可见。悬停 Track 会显现轨道，悬停 Thumb 会高亮并加宽。")
                    .child(shared::theme_selector("scrollbar-example", window, cx))
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .justify_between()
                            .gap(px(20.))
                            .children([
                                control_group("Axis", axis_control),
                                control_group("Visibility", visibility_control),
                                gutter_control_group(
                                    "Gutter",
                                    gutter_control,
                                    self.gutter,
                                    theme.semantic.muted,
                                    theme.semantic.primary,
                                ),
                            ]),
                    )
                    .child(
                        div()
                            .h(px(360.))
                            .w_full()
                            .border_1()
                            .border_color(theme.semantic.border)
                            .bg(theme.semantic.surface)
                            .child(large_content(
                                theme.semantic.muted,
                                theme.semantic.primary,
                            ))
                            .scrollbar_with(ScrollbarConfig {
                                axis: self.axis,
                                visibility: self.visibility,
                                gutter: self.gutter,
                            })
                            .scrollbar_id("interactive-scrollbar")
                            .scrollbar_aria_label("双轴内容"),
                    ),
            )
    }
}

fn control_group(label: &'static str, control: impl IntoElement) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(div().text_size(px(12.)).child(label))
        .child(control)
}

fn gutter_control_group(
    label: &'static str,
    control: impl IntoElement,
    gutter: ScrollGutter,
    content_color: gpui::Hsla,
    gutter_color: gpui::Hsla,
) -> gpui::Div {
    let (probe, hint) = match gutter {
        ScrollGutter::Overlay => (
            div()
                .relative()
                .h(px(14.))
                .w(px(112.))
                .rounded(px(3.))
                .bg(content_color)
                .child(
                    div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_end()
                        .pr(px(2.))
                        .text_size(px(8.))
                        .child("CONTENT EDGE →"),
                )
                .child(
                    div()
                        .absolute()
                        .right_0()
                        .top_0()
                        .bottom_0()
                        .w(px(14.))
                        .rounded(px(3.))
                        .bg(gutter_color.opacity(0.7)),
                ),
            "0px · Track 覆盖内容",
        ),
        ScrollGutter::Stable => (
            div()
                .h(px(14.))
                .w(px(112.))
                .flex()
                .rounded(px(3.))
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_end()
                        .pr(px(2.))
                        .text_size(px(8.))
                        .bg(content_color)
                        .child("CONTENT EDGE →"),
                )
                .child(div().h_full().w(px(14.)).bg(gutter_color.opacity(0.7))),
            "14px · 独立 Gutter",
        ),
    };

    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(div().text_size(px(12.)).child(label))
        .child(control)
        .child(probe)
        .child(div().text_size(px(11.)).child(hint))
}

fn large_content(background: gpui::Hsla, accent: gpui::Hsla) -> gpui::Div {
    div()
        .flex_none()
        .w(px(1320.))
        .flex()
        .flex_col()
        .child(
            div()
                .flex_none()
                .h(px(40.))
                .flex()
                .items_center()
                .px(px(16.))
                .bg(accent.opacity(0.16))
                .child("← 横向内容 1320px：拖动底部 Thumb 查看 →"),
        )
        .child(
            div()
                .grid()
                .grid_cols(6)
                .gap(px(12.))
                .p(px(16.))
                .children((1..=42).map(|index| {
                    div()
                        .h(px(72.))
                        .rounded(px(6.))
                        .bg(background)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(format!("Item {index}"))
                })),
        )
}

fn run_example() {
    application()
        .with_assets(vektra::assets::Assets)
        .run(|cx: &mut App| {
            cx.bind_keys([
                KeyBinding::new("tab", Tab, None),
                KeyBinding::new("shift-tab", TabPrev, None),
            ]);
            let bounds = Bounds::centered(None, size(px(1040.), px(720.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| ScrollbarExample::new(window, cx)),
            )
            .expect("Scrollbar 示例窗口应能成功打开");
            cx.activate(true);
        });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}
