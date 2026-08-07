#[path = "../../shared.rs"]
mod shared;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions,
    actions, div, px, size,
};
use gpui_platform::application;
use vektra::{
    Button, ButtonVariant, ComponentSize, Icon, IconButton, IconButtonVariant, IconName, Input,
    InputClear, InputState, InputVariant, Tooltip, TooltipPlacement, current_theme,
};

actions!(vektra_input_example, [Tab, TabPrev]);

struct InputExample {
    search: gpui::Entity<InputState>,
    search_icon: gpui::Entity<InputState>,
    search_text: gpui::Entity<InputState>,
    search_icon_text: gpui::Entity<InputState>,
    search_attached_icon_text: gpui::Entity<InputState>,
    search_attached_icon: gpui::Entity<InputState>,
    variants: [gpui::Entity<InputState>; 4],
    sizes: [gpui::Entity<InputState>; 4],
    invalid: gpui::Entity<InputState>,
    read_only: gpui::Entity<InputState>,
    disabled: gpui::Entity<InputState>,
    narrow: gpui::Entity<InputState>,
    status: gpui::SharedString,
    focus_handle: FocusHandle,
}

impl InputExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            search: cx.new(|cx| InputState::new("双击 English/中文选词，三击全选 👩🏽‍💻 e\u{301}", cx)),
            search_icon: cx.new(|cx| InputState::new("Vektra", cx)),
            search_text: cx.new(|cx| InputState::new("GPUI", cx)),
            search_icon_text: cx.new(|cx| InputState::new("组件库", cx)),
            search_attached_icon_text: cx.new(|cx| InputState::new("通用能力", cx)),
            search_attached_icon: cx.new(|cx| InputState::new("边缘操作", cx)),
            variants: [
                cx.new(|cx| InputState::new("Outline", cx)),
                cx.new(|cx| InputState::new("Filled", cx)),
                cx.new(|cx| InputState::new("Borderless", cx)),
                cx.new(|cx| InputState::new("Underline", cx)),
            ],
            sizes: [
                cx.new(|cx| InputState::new("XS", cx)),
                cx.new(|cx| InputState::new("SM", cx)),
                cx.new(|cx| InputState::new("MD", cx)),
                cx.new(|cx| InputState::new("LG", cx)),
            ],
            invalid: cx.new(|cx| InputState::new("格式不正确", cx)),
            read_only: cx.new(|cx| InputState::new("只读内容可选择和复制", cx)),
            disabled: cx.new(|cx| InputState::new("禁用内容", cx)),
            narrow: cx
                .new(|cx| InputState::new("窄宽度长文本：双击选词，三击全选，光标保持可见", cx)),
            status: "尚无用户事件".into(),
            focus_handle,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}

impl Render for InputExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);
        let search = self.search.clone();
        let search_icon = self.search_icon.clone();
        let search_text = self.search_text.clone();
        let search_icon_text = self.search_icon_text.clone();
        let search_attached_icon_text = self.search_attached_icon_text.clone();
        let search_attached_icon = self.search_attached_icon.clone();

        div()
            .id("vektra-input-example")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .size_full()
            .overflow_y_scroll()
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .p(px(20.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .max_w(px(760.))
                    .child(div().text_size(px(24.)).child("Vektra Input"))
                    .child(
                        "纯 GPUI 单行输入；观察动画光标，并尝试中文 IME、双击、三击、撤销和清除。",
                    )
                    .child(self.status.clone())
                    .child(shared::theme_selector("input-example", window, cx))
                    .child(
                        Input::new("search", search.clone())
                            .size(ComponentSize::Md)
                            .placeholder("输入搜索内容")
                            .caret_color(theme.semantic.destructive)
                            .aria_label("搜索")
                            .aria_description("支持中文输入法")
                            .prefix(Icon::new(IconName::Search))
                            .clearable(
                                InputClear::new("清空搜索内容")
                                    .tooltip(Tooltip::new("清空"))
                                    .tooltip_placement(TooltipPlacement::Top),
                            )
                            .attached_suffix(
                                Button::new("submit-search")
                                    .label("搜索")
                                    .variant(ButtonVariant::Ghost)
                                    .size(ComponentSize::Md)
                                    .on_click_in(cx, move |this, _, _, cx| {
                                        let value = search.read(cx).value().to_owned();
                                        this.status = format!("搜索：{value}").into();
                                        cx.notify();
                                    }),
                            )
                            .on_change_in(cx, |this, value, _, cx| {
                                this.status = format!("Changed: {value}").into();
                                cx.notify();
                            })
                            .on_submit_in(cx, |this, value, _, cx| {
                                this.status = format!("搜索：{value}").into();
                                cx.notify();
                            }),
                    )
                    .child(div().text_size(px(18.)).child("Five Search actions"))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(
                                Input::new("search-icon-only", self.search_icon.clone())
                                    .placeholder("搜索")
                                    .aria_label("纯图标搜索输入")
                                    .clearable(InputClear::new("清空纯图标搜索"))
                                    .suffix(
                                        IconButton::new("submit-search-icon", IconName::Search)
                                            .variant(IconButtonVariant::Ghost)
                                            .size(ComponentSize::Xs)
                                            .aria_label("搜索")
                                            .tooltip("搜索")
                                            .on_click_in(cx, move |this, _, _, cx| {
                                                let value = search_icon.read(cx).value().to_owned();
                                                this.status = format!("纯图标搜索：{value}").into();
                                                cx.notify();
                                            }),
                                    )
                                    .on_submit_in(cx, |this, value, _, cx| {
                                        this.status = format!("纯图标搜索：{value}").into();
                                        cx.notify();
                                    }),
                            )
                            .child(
                                Input::new("search-text-only", self.search_text.clone())
                                    .size(ComponentSize::Md)
                                    .placeholder("搜索")
                                    .aria_label("纯文字搜索输入")
                                    .clearable(InputClear::new("清空纯文字搜索"))
                                    .attached_suffix(
                                        Button::new("submit-search-text")
                                            .label("搜索")
                                            .variant(ButtonVariant::Ghost)
                                            .size(ComponentSize::Md)
                                            .on_click_in(cx, move |this, _, _, cx| {
                                                let value = search_text.read(cx).value().to_owned();
                                                this.status = format!("纯文字搜索：{value}").into();
                                                cx.notify();
                                            }),
                                    )
                                    .on_submit_in(cx, |this, value, _, cx| {
                                        this.status = format!("纯文字搜索：{value}").into();
                                        cx.notify();
                                    }),
                            )
                            .child(
                                Input::new("search-icon-text", self.search_icon_text.clone())
                                    .placeholder("搜索")
                                    .aria_label("图标加文字搜索输入")
                                    .clearable(InputClear::new("清空图标加文字搜索"))
                                    .suffix(
                                        Button::new("submit-search-icon-text")
                                            .label("搜索")
                                            .start_icon(IconName::Search)
                                            .variant(ButtonVariant::Ghost)
                                            .size(ComponentSize::Xs)
                                            .on_click_in(cx, move |this, _, _, cx| {
                                                let value =
                                                    search_icon_text.read(cx).value().to_owned();
                                                this.status =
                                                    format!("图标加文字搜索：{value}").into();
                                                cx.notify();
                                            }),
                                    )
                                    .on_submit_in(cx, |this, value, _, cx| {
                                        this.status = format!("图标加文字搜索：{value}").into();
                                        cx.notify();
                                    }),
                            )
                            .child(
                                Input::new(
                                    "search-attached-icon-text",
                                    self.search_attached_icon_text.clone(),
                                )
                                .size(ComponentSize::Md)
                                .placeholder("搜索")
                                .aria_label("拼接式图标加文字搜索输入")
                                .clearable(InputClear::new("清空拼接式图标加文字搜索"))
                                .attached_suffix(
                                    Button::new("submit-search-attached-icon-text")
                                        .label("搜索")
                                        .start_icon(IconName::Search)
                                        .variant(ButtonVariant::Ghost)
                                        .size(ComponentSize::Md)
                                        .on_click_in(cx, move |this, _, _, cx| {
                                            let value = search_attached_icon_text
                                                .read(cx)
                                                .value()
                                                .to_owned();
                                            this.status =
                                                format!("拼接式图标加文字搜索：{value}").into();
                                            cx.notify();
                                        }),
                                )
                                .on_submit_in(
                                    cx,
                                    |this, value, _, cx| {
                                        this.status =
                                            format!("拼接式图标加文字搜索：{value}").into();
                                        cx.notify();
                                    },
                                ),
                            )
                            .child(
                                Input::new(
                                    "search-attached-icon-only",
                                    self.search_attached_icon.clone(),
                                )
                                .size(ComponentSize::Md)
                                .placeholder("搜索")
                                .aria_label("拼接式纯图标搜索输入")
                                .clearable(InputClear::new("清空拼接式纯图标搜索"))
                                .attached_suffix(
                                    IconButton::new(
                                        "submit-search-attached-icon",
                                        IconName::Search,
                                    )
                                    .variant(IconButtonVariant::Ghost)
                                    .size(ComponentSize::Md)
                                    .aria_label("搜索")
                                    .tooltip("搜索")
                                    .on_click_in(
                                        cx,
                                        move |this, _, _, cx| {
                                            let value =
                                                search_attached_icon.read(cx).value().to_owned();
                                            this.status =
                                                format!("拼接式纯图标搜索：{value}").into();
                                            cx.notify();
                                        },
                                    ),
                                )
                                .on_submit_in(
                                    cx,
                                    |this, value, _, cx| {
                                        this.status = format!("拼接式纯图标搜索：{value}").into();
                                        cx.notify();
                                    },
                                ),
                            ),
                    )
                    .child(div().text_size(px(18.)).child("Variants"))
                    .children(
                        [
                            InputVariant::Outline,
                            InputVariant::Filled,
                            InputVariant::Borderless,
                            InputVariant::Underline,
                        ]
                        .into_iter()
                        .zip(self.variants.iter().cloned())
                        .enumerate()
                        .map(|(index, (variant, state))| {
                            Input::new(("variant", index), state)
                                .variant(variant)
                                .aria_label(format!("{variant:?} 输入"))
                        }),
                    )
                    .child(div().text_size(px(18.)).child("Sizes"))
                    .child(
                        div().flex().flex_col().gap(px(8.)).children(
                            [
                                ComponentSize::Xs,
                                ComponentSize::Sm,
                                ComponentSize::Md,
                                ComponentSize::Lg,
                            ]
                            .into_iter()
                            .zip(self.sizes.iter().cloned())
                            .enumerate()
                            .map(|(index, (size, state))| {
                                Input::new(("size", index), state)
                                    .size(size)
                                    .aria_label(format!("{size:?} 输入"))
                            }),
                        ),
                    )
                    .child(div().text_size(px(18.)).child("States and narrow layout"))
                    .child(
                        Input::new("invalid", self.invalid.clone())
                            .invalid(true)
                            .aria_label("无效输入"),
                    )
                    .child(
                        Input::new("read-only", self.read_only.clone())
                            .read_only(true)
                            .clearable(InputClear::new("清空只读内容"))
                            .aria_label("只读输入"),
                    )
                    .child(
                        Input::new("disabled", self.disabled.clone())
                            .disabled(true)
                            .clearable(InputClear::new("清空禁用内容"))
                            .aria_label("禁用输入"),
                    )
                    .child(
                        div().w(px(180.)).child(
                            Input::new("narrow", self.narrow.clone())
                                .prefix(Icon::new(IconName::Search))
                                .clearable(InputClear::new("清空窄输入").tooltip("清空"))
                                .aria_label("窄宽度输入"),
                        ),
                    ),
            )
    }
}

fn main() {
    application()
        .with_assets(vektra::assets::Assets)
        .run(|cx: &mut App| {
            cx.bind_keys([
                KeyBinding::new("tab", Tab, None),
                KeyBinding::new("shift-tab", TabPrev, None),
            ]);
            let bounds = Bounds::centered(None, size(px(800.), px(760.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| InputExample::new(window, cx)),
            )
            .expect("Input 示例窗口应能成功打开");
            cx.activate(true);
        });
}
