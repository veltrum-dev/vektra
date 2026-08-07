#![cfg_attr(target_family = "wasm", no_main)]

#[path = "../../shared.rs"]
mod shared;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, Styled, Window, WindowBounds, WindowOptions, actions, div, px, rgb,
    size,
};
use gpui_platform::application;
use vektra::{
    Button, ButtonVariant, IconButton, IconName, Tooltip, TooltipPlacement, current_theme,
};

actions!(vektra_tooltip_example, [Tab, TabPrev]);

struct TooltipExample {
    focus_handle: FocusHandle,
}

impl TooltipExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self { focus_handle }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}

impl Render for TooltipExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);
        div()
            .id("vektra-tooltip-example")
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
                    .gap(px(14.))
                    .max_w(px(680.))
                    .child(div().text_size(px(24.)).child("Vektra Tooltip"))
                    .child("悬停或使用 Tab 聚焦 500ms 后显示；Escape 关闭且保留焦点。")
                    .child(shared::theme_selector("tooltip-example", window, cx))
                    .child(
                        Button::new("controlled-tooltip")
                            .label("常驻、自定义颜色、无箭头、无动画")
                            .variant(ButtonVariant::Outline)
                            .tooltip(
                                Tooltip::new("这里点击保存")
                                    .open(true)
                                    .arrow(false)
                                    .color(rgb(0xffffff))
                                    .bg_color(rgb(0x222222))
                                    .animated(false),
                            ),
                    )
                    .child(placement_row(
                        "Top",
                        [
                            ("Start", TooltipPlacement::TopStart),
                            ("Center", TooltipPlacement::Top),
                            ("End", TooltipPlacement::TopEnd),
                        ],
                    ))
                    .child(placement_row(
                        "Right",
                        [
                            ("Start", TooltipPlacement::RightStart),
                            ("Center", TooltipPlacement::Right),
                            ("End", TooltipPlacement::RightEnd),
                        ],
                    ))
                    .child(placement_row(
                        "Bottom",
                        [
                            ("Start", TooltipPlacement::BottomStart),
                            ("Center", TooltipPlacement::Bottom),
                            ("End", TooltipPlacement::BottomEnd),
                        ],
                    ))
                    .child(placement_row(
                        "Left",
                        [
                            ("Start", TooltipPlacement::LeftStart),
                            ("Center", TooltipPlacement::Left),
                            ("End", TooltipPlacement::LeftEnd),
                        ],
                    ))
                    .child(
                        div()
                            .flex()
                            .gap(px(12.))
                            .items_center()
                            .child(
                                Button::new("save")
                                    .label("保存")
                                    .tooltip("保存当前修改")
                                    .tooltip_placement(TooltipPlacement::TopStart)
                                    .aria_description("保存当前修改"),
                            )
                            .child(
                                IconButton::new("settings", IconName::Settings)
                                    .aria_label("设置")
                                    .aria_description("打开应用设置")
                                    .tooltip("设置")
                                    .tooltip_placement(TooltipPlacement::Right),
                            )
                            .child(
                                Button::new("disabled")
                                    .label("不可用")
                                    .variant(ButtonVariant::Outline)
                                    .disabled(true)
                                    .tooltip("当前操作不可用")
                                    .tooltip_placement(TooltipPlacement::Left),
                            ),
                    )
                    .child(
                        Button::new("long")
                            .label("长文本")
                            .variant(ButtonVariant::Ghost)
                            .tooltip("这是一段用于验证窄窗口与长中文文本自动换行、flip 和 shift 的补充说明。")
                            .tooltip_placement(TooltipPlacement::BottomEnd),
                    ),
            )
    }
}

fn placement_row(
    side: &'static str,
    placements: [(&'static str, TooltipPlacement); 3],
) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(div().w(px(56.)).child(side))
        .children(placements.map(|(alignment, placement)| {
            Button::new(format!("tooltip-{side}-{alignment}"))
                .label(alignment)
                .width(px(80.))
                .variant(ButtonVariant::Ghost)
                .tooltip(side)
                .tooltip_placement(placement)
        }))
}

fn run_example() {
    application()
        .with_assets(vektra::assets::Assets)
        .run(|cx: &mut App| {
            cx.bind_keys([
                KeyBinding::new("tab", Tab, None),
                KeyBinding::new("shift-tab", TabPrev, None),
            ]);
            let bounds = Bounds::centered(None, size(px(900.), px(760.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| TooltipExample::new(window, cx)),
            )
            .expect("Tooltip 示例窗口应能成功打开");
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
