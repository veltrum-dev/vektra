#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, WindowBounds,
    WindowOptions, actions, div, px, size,
};

actions!(vektra_icon_button_example, [Tab, TabPrev]);
use gpui_platform::application;
use vektra::{
    ComponentSize, Icon, IconButton, IconButtonVariant, IconName, IconSource, ThemeMode,
    current_theme, resolved_theme_mode, set_theme_mode,
};

struct IconButtonExample {
    clicks: usize,
    last_clicked: SharedString,
    focus_handle: FocusHandle,
}

impl IconButtonExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            clicks: 0,
            last_clicked: "暂无".into(),
            focus_handle,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }

    fn record_click(&mut self, label: SharedString, _: &mut Window, cx: &mut Context<Self>) {
        self.clicks += 1;
        self.last_clicked = label;
        cx.notify();
    }

    fn record_focus(&mut self, label: &'static str, focused: bool, cx: &mut Context<Self>) {
        self.last_clicked = if focused {
            format!("已聚焦：{label}")
        } else {
            format!("已失焦：{label}")
        }
        .into();
        cx.notify();
    }

    fn switch_theme(&mut self, mode: ThemeMode, window: &mut Window, cx: &mut Context<Self>) {
        set_theme_mode(mode, cx);
        self.last_clicked = format!(
            "主题已切换：{}",
            resolved_theme_mode_label(resolved_theme_mode(window, cx))
        )
        .into();
        cx.notify();
    }
}

impl Render for IconButtonExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);
        div()
            .id("vektra-icon-button-example")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .size_full()
            .overflow_y_scroll()
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(18.))
                    .p(px(20.))
                    .max_w(px(880.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .child("Vektra 图标按钮")
                            .text_size(px(24.))
                            .child(format!(
                                "点击次数：{}  上次点击：{}  当前主题：{}",
                                self.clicks,
                                self.last_clicked,
                                resolved_theme_mode_label(resolved_theme_mode(window, cx))
                            )),
                    )
                    .child(
                        self.section("纯图标").child(
                            div()
                                .flex()
                                .gap(px(12.))
                                .items_center()
                                .child(Icon::new(IconName::Settings))
                                .child(Icon::new(IconSource::asset("icons/settings.svg"))),
                        ),
                    )
                    .child(
                        self.section("主题模式").child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .flex_wrap()
                                .child(self.theme_button(
                                    "theme-system",
                                    "跟随系统",
                                    ThemeMode::System,
                                    cx,
                                ))
                                .child(self.theme_button(
                                    "theme-light",
                                    "浅色",
                                    ThemeMode::Light,
                                    cx,
                                ))
                                .child(self.theme_button(
                                    "theme-dark",
                                    "深色",
                                    ThemeMode::Dark,
                                    cx,
                                )),
                        ),
                    )
                    .child(self.section("变体").child(
                        div().flex().gap(px(8.)).flex_wrap().children([
                            self.icon_button(
                                "variant-primary",
                                "主要",
                                IconButtonVariant::Primary,
                                ComponentSize::Md,
                                cx,
                            ),
                            self.icon_button(
                                "variant-outline",
                                "描边",
                                IconButtonVariant::Outline,
                                ComponentSize::Md,
                                cx,
                            ),
                            self.icon_button(
                                "variant-ghost",
                                "幽灵",
                                IconButtonVariant::Ghost,
                                ComponentSize::Md,
                                cx,
                            ),
                            self.icon_button(
                                "variant-destructive",
                                "危险",
                                IconButtonVariant::Destructive,
                                ComponentSize::Md,
                                cx,
                            ),
                            self.icon_button(
                                "variant-secondary",
                                "次要",
                                IconButtonVariant::Secondary,
                                ComponentSize::Md,
                                cx,
                            ),
                        ]),
                    ))
                    .child(self.section("尺寸").child(
                        div().flex().gap(px(8.)).flex_wrap().children([
                            self.icon_button(
                                "size-xs",
                                "超小",
                                IconButtonVariant::Outline,
                                ComponentSize::Xs,
                                cx,
                            ),
                            self.icon_button(
                                "size-sm",
                                "小",
                                IconButtonVariant::Outline,
                                ComponentSize::Sm,
                                cx,
                            ),
                            self.icon_button(
                                "size-md",
                                "中",
                                IconButtonVariant::Outline,
                                ComponentSize::Md,
                                cx,
                            ),
                            self.icon_button(
                                "size-lg",
                                "大",
                                IconButtonVariant::Outline,
                                ComponentSize::Lg,
                                cx,
                            ),
                        ]),
                    ))
                    .child(
                        self.section("状态").child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .flex_wrap()
                                .child(self.default_icon_button("default", "默认", cx))
                                .child(
                                    self.default_icon_button("aria", "有 aria_label", cx)
                                        .aria_label("设置"),
                                )
                                .child(
                                    IconButton::new(
                                        "path",
                                        IconSource::asset("icons/settings.svg"),
                                    )
                                    .aria_label("自定义路径")
                                    .variant(IconButtonVariant::Secondary)
                                    .on_click_in(
                                        cx,
                                        |this, _, window, cx| {
                                            this.record_click("自定义路径".into(), window, cx);
                                        },
                                    ),
                                )
                                .child(
                                    IconButton::new("disabled", IconName::Settings)
                                        .aria_label("禁用设置")
                                        .disabled(true),
                                )
                                .child(
                                    self.default_icon_button("custom-color", "自定义图标色", cx)
                                        .variant(IconButtonVariant::Outline)
                                        .icon_color(theme.semantic.destructive),
                                ),
                        ),
                    ),
            )
    }
}

impl IconButtonExample {
    fn section(&self, title: &'static str) -> gpui::Div {
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(div().text_size(px(16.)).child(title))
    }

    fn icon_button(
        &self,
        id: &'static str,
        label: &'static str,
        variant: IconButtonVariant,
        size: ComponentSize,
        cx: &mut Context<Self>,
    ) -> IconButton {
        self.default_icon_button(id, label, cx)
            .variant(variant)
            .size(size)
    }

    fn default_icon_button(
        &self,
        id: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> IconButton {
        let clicked = SharedString::new_static(label);
        IconButton::new(id, IconName::Settings)
            .aria_label(label)
            .aria_description(label)
            .tooltip(label)
            .on_click_in(cx, move |this, _, window, cx| {
                this.record_click(clicked.clone(), window, cx);
            })
            .on_focus_in(cx, move |this, _, cx| {
                this.record_focus(label, true, cx);
            })
            .on_blur_in(cx, move |this, _, cx| {
                this.record_focus(label, false, cx);
            })
    }

    fn theme_button(
        &self,
        id: &'static str,
        label: &'static str,
        mode: ThemeMode,
        cx: &mut Context<Self>,
    ) -> IconButton {
        IconButton::new(id, IconSource::asset("icons/settings.svg"))
            .aria_label(label)
            .tooltip(label)
            .variant(IconButtonVariant::Outline)
            .on_click_in(cx, move |this, _, window, cx| {
                this.switch_theme(mode, window, cx);
            })
    }
}

fn run_example() {
    application()
        .with_assets(vektra::assets::Assets)
        .run(|cx: &mut App| {
            cx.bind_keys([
                KeyBinding::new("tab", Tab, None),
                KeyBinding::new("shift-tab", TabPrev, None),
            ]);
            let bounds = Bounds::centered(None, size(px(760.), px(620.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| IconButtonExample::new(window, cx)),
            )
            .expect("IconButton 示例窗口应能成功打开");
            cx.activate(true);
        });
}

fn resolved_theme_mode_label(mode: vektra::ResolvedThemeMode) -> &'static str {
    match mode {
        vektra::ResolvedThemeMode::Light => "浅色",
        vektra::ResolvedThemeMode::Dark => "深色",
    }
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
