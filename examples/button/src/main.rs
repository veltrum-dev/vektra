#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, WindowBounds,
    WindowOptions, actions, div, px, relative, size,
};

actions!(vektra_button_example, [Tab, TabPrev]);
use gpui_platform::application;
use vektra::{
    Button, ButtonSize, ButtonVariant, IconName, IconSource, ThemeMode, current_theme,
    resolved_theme_mode, set_theme_mode, theme_mode,
};

struct ButtonExample {
    clicks: usize,
    last_clicked: SharedString,
    selected: bool,
    loading: bool,
    progress: f32,
    focus_handle: FocusHandle,
}

impl ButtonExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            clicks: 0,
            last_clicked: "暂无".into(),
            selected: false,
            loading: false,
            progress: 0.25,
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

    fn switch_theme(&mut self, mode: ThemeMode, window: &mut Window, cx: &mut Context<Self>) {
        set_theme_mode(mode, cx);
        self.last_clicked = format!(
            "主题已切换：{}",
            resolved_theme_mode_label(resolved_theme_mode(window, cx))
        )
        .into();
        cx.notify();
    }

    fn toggle_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = !self.selected;
        self.record_click("切换 selected".into(), window, cx);
    }

    fn toggle_loading(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.loading = !self.loading;
        self.record_click("切换 loading".into(), window, cx);
    }

    fn advance_progress(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.progress = if self.progress >= 1. {
            0.
        } else {
            (self.progress + 0.25).min(1.)
        };
        self.record_click("推进 progress".into(), window, cx);
    }
}

impl Render for ButtonExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);
        div()
            .id("vektra-button-example")
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
                    .max_w(px(980.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .child("Vektra 按钮")
                            .text_size(px(24.))
                            .child(format!(
                                "点击次数：{}  上次点击：{}  主题设置：{}  当前主题：{}",
                                self.clicks,
                                self.last_clicked,
                                theme_mode_label(theme_mode(cx)),
                                resolved_theme_mode_label(resolved_theme_mode(window, cx))
                            )),
                    )
                    .child(
                        self.section("主题模式").child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .flex_wrap()
                                .child(
                                    Button::new("theme-system")
                                        .label("跟随系统")
                                        .variant(ButtonVariant::Outline)
                                        .on_click_in(cx, |this, _, window, cx| {
                                            this.switch_theme(ThemeMode::System, window, cx);
                                        }),
                                )
                                .child(Button::new("theme-light").label("浅色").on_click_in(
                                    cx,
                                    |this, _, window, cx| {
                                        this.switch_theme(ThemeMode::Light, window, cx);
                                    },
                                ))
                                .child(
                                    Button::new("theme-dark")
                                        .label("深色")
                                        .variant(ButtonVariant::Secondary)
                                        .on_click_in(cx, |this, _, window, cx| {
                                            this.switch_theme(ThemeMode::Dark, window, cx);
                                        }),
                                ),
                        ),
                    )
                    .child(
                        self.section("主题令牌").child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.))
                                .text_size(px(13.))
                                .child(format!("主色（primary）：{:?}", theme.semantic.primary))
                                .child(format!(
                                    "背景色（background）：{:?}",
                                    theme.semantic.background
                                ))
                                .child(format!("边框色（border）：{:?}", theme.semantic.border))
                                .child(format!("焦点环（ring）：{:?}", theme.semantic.ring)),
                        ),
                    )
                    .child(self.section("变体 × 尺寸").child(
                        div().flex().flex_col().gap(px(10.)).children([
                            self.variant_row("primary", "主要按钮", ButtonVariant::Primary, cx),
                            self.variant_row("outline", "描边按钮", ButtonVariant::Outline, cx),
                            self.variant_row("ghost", "幽灵按钮", ButtonVariant::Ghost, cx),
                            self.variant_row(
                                "destructive",
                                "危险按钮",
                                ButtonVariant::Destructive,
                                cx,
                            ),
                            self.variant_row("secondary", "次要按钮", ButtonVariant::Secondary, cx),
                            self.variant_row("link", "链接按钮", ButtonVariant::Link, cx),
                        ]),
                    ))
                    .child(
                        self.section("带图标").child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(10.))
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.))
                                        .flex_wrap()
                                        .child(
                                            self.click_button("icon-start", "设置", cx)
                                                .start_icon(IconName::Settings),
                                        )
                                        .child(
                                            self.click_button("icon-end", "下一步", cx)
                                                .end_icon(IconName::Settings),
                                        )
                                        .child(
                                            self.click_button("icon-both", "两端图标", cx)
                                                .start_icon(IconName::Settings)
                                                .end_icon(IconSource::asset("icons/settings.svg")),
                                        )
                                        .child(
                                            self.click_button("icon-fixed", "固定宽度", cx)
                                                .start_icon(IconName::Settings)
                                                .width(px(112.)),
                                        )
                                        .child(
                                            Button::new("icon-disabled")
                                                .label("禁用")
                                                .start_icon(IconName::Settings)
                                                .disabled(true),
                                        )
                                        .child(
                                            self.click_button("icon-link", "链接图标", cx)
                                                .variant(ButtonVariant::Link)
                                                .start_icon(IconName::Settings),
                                        ),
                                )
                                .child(
                                    div().w(px(360.)).max_w(relative(1.)).child(
                                        self.click_button("icon-full", "填满父容器", cx)
                                            .start_icon(IconName::Settings)
                                            .end_icon(IconName::Settings)
                                            .full_width(),
                                    ),
                                ),
                        ),
                    )
                    .child(
                        self.section("交互状态").child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(10.))
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.))
                                        .flex_wrap()
                                        .child(self.click_button("state-normal", "正常", cx))
                                        .child(
                                            Button::new("state-selected")
                                                .label(if self.selected {
                                                    "已选中（点击切换）"
                                                } else {
                                                    "未选中（点击切换）"
                                                })
                                                .selected(self.selected)
                                                .on_click_in(cx, |this, _, window, cx| {
                                                    this.toggle_selected(window, cx);
                                                }),
                                        )
                                        .child(
                                            Button::new("state-loading")
                                                .label("受控 loading")
                                                .start_icon(IconName::Settings)
                                                .loading(self.loading),
                                        )
                                        .child(
                                            Button::new("state-progress")
                                                .label(format!(
                                                    "受控 progress {:.0}%",
                                                    self.progress * 100.
                                                ))
                                                .start_icon(IconName::Settings)
                                                .end_icon(IconName::Settings)
                                                .progress(self.progress)
                                                .width(px(220.)),
                                        )
                                        .child(
                                            Button::new("state-disabled-combined")
                                                .label("禁用 + selected + progress")
                                                .selected(true)
                                                .progress(0.65)
                                                .disabled(true),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.))
                                        .flex_wrap()
                                        .child(
                                            Button::new("state-toggle-loading")
                                                .label(if self.loading {
                                                    "停止 loading"
                                                } else {
                                                    "开始 loading"
                                                })
                                                .variant(ButtonVariant::Outline)
                                                .on_click_in(cx, |this, _, window, cx| {
                                                    this.toggle_loading(window, cx);
                                                }),
                                        )
                                        .child(
                                            Button::new("state-advance-progress")
                                                .label("推进 progress")
                                                .variant(ButtonVariant::Outline)
                                                .on_click_in(cx, |this, _, window, cx| {
                                                    this.advance_progress(window, cx);
                                                }),
                                        ),
                                ),
                        ),
                    )
                    .child(
                        self.section("中文自动空格").child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .flex_wrap()
                                .child(self.click_button("cn-default", "保存", cx))
                                .child(
                                    self.click_button("cn-enabled", "确定", cx)
                                        .auto_insert_space(true),
                                )
                                .child(
                                    self.click_button("cn-disabled", "取消", cx)
                                        .auto_insert_space(false),
                                )
                                .child(self.click_button("cn-long", "保存设置", cx))
                                .child(self.click_button("cn-mixed", "保存1", cx)),
                        ),
                    )
                    .child(
                        self.section("宽度").child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(10.))
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.))
                                        .flex_wrap()
                                        .child(self.click_button("width-auto", "自动宽度", cx))
                                        .child(
                                            self.click_button("width-fixed", "固定宽度 200px", cx)
                                                .width(px(200.)),
                                        )
                                        .child(
                                            self.click_button("width-narrow", "较长文本", cx)
                                                .width(px(72.)),
                                        )
                                        .child(
                                            self.click_button("width-cn", "保存", cx)
                                                .width(px(88.)),
                                        ),
                                )
                                .child(
                                    div()
                                        .w(px(360.))
                                        .max_w(relative(1.))
                                        .flex()
                                        .flex_col()
                                        .gap(px(8.))
                                        .child(
                                            self.click_button("width-full", "填满父容器", cx)
                                                .full_width(),
                                        )
                                        .child(
                                            self.click_button(
                                                "width-disabled-full",
                                                "禁用并填满",
                                                cx,
                                            )
                                            .full_width()
                                            .disabled(true),
                                        )
                                        .child(
                                            self.click_button(
                                                "width-fixed-then-full",
                                                "先固定后填满",
                                                cx,
                                            )
                                            .width(px(120.))
                                            .full_width(),
                                        )
                                        .child(
                                            self.click_button(
                                                "width-full-then-fixed",
                                                "先填满后固定",
                                                cx,
                                            )
                                            .full_width()
                                            .width(px(180.)),
                                        ),
                                ),
                        ),
                    ),
            )
    }
}

impl ButtonExample {
    fn section(&self, title: &'static str) -> gpui::Div {
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(div().text_size(px(16.)).child(title))
    }

    fn variant_row(
        &self,
        id_prefix: &'static str,
        label: &'static str,
        variant: ButtonVariant,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        div()
            .flex()
            .gap(px(8.))
            .flex_wrap()
            .items_center()
            .child(div().w(px(92.)).text_size(px(13.)).child(label))
            .child(
                self.click_button(format!("{id_prefix}-xs"), "超小", cx)
                    .variant(variant)
                    .size(ButtonSize::Xs),
            )
            .child(
                self.click_button(format!("{id_prefix}-sm"), "小", cx)
                    .variant(variant)
                    .size(ButtonSize::Sm),
            )
            .child(
                self.click_button(format!("{id_prefix}-md"), "中", cx)
                    .variant(variant)
                    .size(ButtonSize::Md),
            )
            .child(
                self.click_button(format!("{id_prefix}-lg"), "大", cx)
                    .variant(variant)
                    .size(ButtonSize::Lg),
            )
            .child(
                Button::new(format!("{id_prefix}-disabled"))
                    .label("禁用")
                    .variant(variant)
                    .size(ButtonSize::Md)
                    .disabled(true),
            )
    }

    fn click_button(
        &self,
        id: impl Into<gpui::ElementId>,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> Button {
        let clicked = SharedString::new_static(label);
        Button::new(id)
            .label(label)
            .tooltip(label)
            .aria_description(label)
            .on_click_in(cx, move |this, _, window, cx| {
                this.record_click(clicked.clone(), window, cx);
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
            let bounds = Bounds::centered(None, size(px(920.), px(760.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| ButtonExample::new(window, cx)),
            )
            .expect("Button 示例窗口应能成功打开");
            cx.activate(true);
        });
}

fn theme_mode_label(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::System => "跟随系统",
        ThemeMode::Light => "浅色",
        ThemeMode::Dark => "深色",
    }
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
