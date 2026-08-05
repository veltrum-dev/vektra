use super::{PreviewApp, PreviewLang};
use gpui::{
    AnyElement, AppContext, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use vektra::{
    Button, ButtonVariant, ComponentSize, Icon, IconButton, IconButtonVariant, IconName, Input,
    InputClear, InputState, InputVariant, Tooltip, TooltipPlacement,
};

// #region input-state
pub(super) struct InputDemo {
    primary: gpui::Entity<InputState>,
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
}

impl InputDemo {
    pub(super) fn new(cx: &mut Context<PreviewApp>) -> Self {
        Self {
            primary: cx
                .new(|cx| InputState::new("双击 English/中文选词，三击全选 👩🏽‍💻 e\u{301}", cx)),
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
            invalid: cx.new(|cx| InputState::new("外部校验标记", cx)),
            read_only: cx.new(|cx| InputState::new("只读内容仍可选择和复制", cx)),
            disabled: cx.new(|cx| InputState::new("禁用内容", cx)),
            narrow: cx
                .new(|cx| InputState::new("窄宽度长文本：双击选词，三击全选，光标保持可见", cx)),
            status: "尚无用户事件".into(),
        }
    }
}
// #endregion input-state

impl InputDemo {
    pub(super) fn render(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);
        let (
            title,
            intro,
            status_label,
            search_actions_label,
            variants_label,
            sizes_label,
            states_label,
        ) = match language {
            PreviewLang::ZhCn => (
                "Input 预览",
                "观察动画光标；尝试中文 IME、双击选词、三击全选、清除和 Tab 焦点。",
                "事件",
                "五种 Search 操作",
                "四种外观",
                "四种尺寸",
                "状态与窄宽度",
            ),
            PreviewLang::EnUs => (
                "Input preview",
                "Watch the animated caret; try IME, double-click, triple-click, clear, and Tab focus.",
                "Event",
                "Five Search actions",
                "Four variants",
                "Four sizes",
                "States and narrow width",
            ),
        };
        let primary_aria = match language {
            PreviewLang::ZhCn => "搜索",
            PreviewLang::EnUs => "Search",
        };
        let primary_description = match language {
            PreviewLang::ZhCn => "支持中文输入法",
            PreviewLang::EnUs => "Supports input methods",
        };
        let clear_label = match language {
            PreviewLang::ZhCn => "清空搜索内容",
            PreviewLang::EnUs => "Clear search",
        };
        let clear_tooltip = match language {
            PreviewLang::ZhCn => "清空",
            PreviewLang::EnUs => "Clear",
        };
        let search_label = match language {
            PreviewLang::ZhCn => "搜索",
            PreviewLang::EnUs => "Search",
        };
        let search_placeholder = match language {
            PreviewLang::ZhCn => "输入搜索内容",
            PreviewLang::EnUs => "Enter a search query",
        };
        let primary = self.primary.clone();
        let search_icon = self.search_icon.clone();
        let search_text = self.search_text.clone();
        let search_icon_text = self.search_icon_text.clone();
        let search_attached_icon_text = self.search_attached_icon_text.clone();
        let search_attached_icon = self.search_attached_icon.clone();
        let status = self.status.clone();

        div()
            .id("input-basic-demo")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(20.))
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(div().text_size(px(24.)).child(title))
            .child(intro)
            .child(format!("{status_label}: {status}"))
            // #region input-basic
            .child(
                Input::new("input-primary", self.primary.clone())
                    .size(ComponentSize::Md)
                    .placeholder(match language {
                        PreviewLang::ZhCn => "输入搜索内容",
                        PreviewLang::EnUs => "Enter a search query",
                    })
                    .caret_color(theme.semantic.destructive)
                    .aria_label(primary_aria)
                    .aria_description(primary_description)
                    .prefix(Icon::new(IconName::Search))
                    .clearable(
                        InputClear::new(clear_label)
                            .tooltip(Tooltip::new(clear_tooltip))
                            .tooltip_placement(TooltipPlacement::Top),
                    )
                    .attached_suffix(
                        Button::new("input-submit-search")
                            .label(search_label)
                            .variant(ButtonVariant::Ghost)
                            .size(ComponentSize::Md)
                            .on_click_in(cx, move |this, _, _, cx| {
                                let value = primary.read(cx).value().to_owned();
                                this.input_demo.status = match language {
                                    PreviewLang::ZhCn => format!("搜索：{value}"),
                                    PreviewLang::EnUs => format!("Search: {value}"),
                                }
                                .into();
                                cx.notify();
                            }),
                    )
                    .on_change_in(cx, move |this, value, _, cx| {
                        this.input_demo.status = format!("Changed: {value}").into();
                        cx.notify();
                    })
                    .on_submit_in(cx, move |this, value, _, cx| {
                        this.input_demo.status = match language {
                            PreviewLang::ZhCn => format!("搜索：{value}"),
                            PreviewLang::EnUs => format!("Search: {value}"),
                        }
                        .into();
                        cx.notify();
                    }),
            )
            // #endregion input-basic
            .child(div().text_size(px(18.)).child(search_actions_label))
            // #region input-search-actions
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        Input::new("search-icon-only", self.search_icon.clone())
                            .placeholder(search_placeholder)
                            .aria_label(match language {
                                PreviewLang::ZhCn => "纯图标搜索输入",
                                PreviewLang::EnUs => "Icon-only search input",
                            })
                            .clearable(InputClear::new(clear_label))
                            .suffix(
                                IconButton::new("submit-search-icon", IconName::Search)
                                    .variant(IconButtonVariant::Ghost)
                                    .size(ComponentSize::Xs)
                                    .aria_label(search_label)
                                    .tooltip(search_label)
                                    .on_click_in(cx, move |this, _, _, cx| {
                                        let value = search_icon.read(cx).value().to_owned();
                                        this.input_demo.status = match language {
                                            PreviewLang::ZhCn => {
                                                format!("纯图标搜索：{value}")
                                            }
                                            PreviewLang::EnUs => {
                                                format!("Icon-only search: {value}")
                                            }
                                        }
                                        .into();
                                        cx.notify();
                                    }),
                            )
                            .on_submit_in(cx, move |this, value, _, cx| {
                                this.input_demo.status = match language {
                                    PreviewLang::ZhCn => format!("纯图标搜索：{value}"),
                                    PreviewLang::EnUs => {
                                        format!("Icon-only search: {value}")
                                    }
                                }
                                .into();
                                cx.notify();
                            }),
                    )
                    .child(
                        Input::new("search-text-only", self.search_text.clone())
                            .size(ComponentSize::Md)
                            .placeholder(search_placeholder)
                            .aria_label(match language {
                                PreviewLang::ZhCn => "纯文字搜索输入",
                                PreviewLang::EnUs => "Text-only search input",
                            })
                            .clearable(InputClear::new(clear_label))
                            .attached_suffix(
                                Button::new("submit-search-text")
                                    .label(search_label)
                                    .variant(ButtonVariant::Ghost)
                                    .size(ComponentSize::Md)
                                    .on_click_in(cx, move |this, _, _, cx| {
                                        let value = search_text.read(cx).value().to_owned();
                                        this.input_demo.status = match language {
                                            PreviewLang::ZhCn => {
                                                format!("纯文字搜索：{value}")
                                            }
                                            PreviewLang::EnUs => {
                                                format!("Text-only search: {value}")
                                            }
                                        }
                                        .into();
                                        cx.notify();
                                    }),
                            )
                            .on_submit_in(cx, move |this, value, _, cx| {
                                this.input_demo.status = match language {
                                    PreviewLang::ZhCn => format!("纯文字搜索：{value}"),
                                    PreviewLang::EnUs => format!("Text-only search: {value}"),
                                }
                                .into();
                                cx.notify();
                            }),
                    )
                    .child(
                        Input::new("search-icon-text", self.search_icon_text.clone())
                            .placeholder(search_placeholder)
                            .aria_label(match language {
                                PreviewLang::ZhCn => "图标加文字搜索输入",
                                PreviewLang::EnUs => "Icon-and-text search input",
                            })
                            .clearable(InputClear::new(clear_label))
                            .suffix(
                                Button::new("submit-search-icon-text")
                                    .label(search_label)
                                    .start_icon(IconName::Search)
                                    .variant(ButtonVariant::Ghost)
                                    .size(ComponentSize::Xs)
                                    .on_click_in(cx, move |this, _, _, cx| {
                                        let value = search_icon_text.read(cx).value().to_owned();
                                        this.input_demo.status = match language {
                                            PreviewLang::ZhCn => {
                                                format!("图标加文字搜索：{value}")
                                            }
                                            PreviewLang::EnUs => {
                                                format!("Icon-and-text search: {value}")
                                            }
                                        }
                                        .into();
                                        cx.notify();
                                    }),
                            )
                            .on_submit_in(cx, move |this, value, _, cx| {
                                this.input_demo.status = match language {
                                    PreviewLang::ZhCn => {
                                        format!("图标加文字搜索：{value}")
                                    }
                                    PreviewLang::EnUs => {
                                        format!("Icon-and-text search: {value}")
                                    }
                                }
                                .into();
                                cx.notify();
                            }),
                    )
                    .child(
                        Input::new(
                            "search-attached-icon-text",
                            self.search_attached_icon_text.clone(),
                        )
                        .size(ComponentSize::Md)
                        .placeholder(search_placeholder)
                        .aria_label(match language {
                            PreviewLang::ZhCn => "拼接式图标加文字搜索输入",
                            PreviewLang::EnUs => "Attached icon-and-text search input",
                        })
                        .clearable(InputClear::new(clear_label))
                        .attached_suffix(
                            Button::new("submit-search-attached-icon-text")
                                .label(search_label)
                                .start_icon(IconName::Search)
                                .variant(ButtonVariant::Ghost)
                                .size(ComponentSize::Md)
                                .on_click_in(cx, move |this, _, _, cx| {
                                    let value =
                                        search_attached_icon_text.read(cx).value().to_owned();
                                    this.input_demo.status = match language {
                                        PreviewLang::ZhCn => {
                                            format!("拼接式图标加文字搜索：{value}")
                                        }
                                        PreviewLang::EnUs => {
                                            format!("Attached icon-and-text search: {value}")
                                        }
                                    }
                                    .into();
                                    cx.notify();
                                }),
                        )
                        .on_submit_in(cx, move |this, value, _, cx| {
                            this.input_demo.status = match language {
                                PreviewLang::ZhCn => {
                                    format!("拼接式图标加文字搜索：{value}")
                                }
                                PreviewLang::EnUs => {
                                    format!("Attached icon-and-text search: {value}")
                                }
                            }
                            .into();
                            cx.notify();
                        }),
                    )
                    .child(
                        Input::new(
                            "search-attached-icon-only",
                            self.search_attached_icon.clone(),
                        )
                        .size(ComponentSize::Md)
                        .placeholder(search_placeholder)
                        .aria_label(match language {
                            PreviewLang::ZhCn => "拼接式纯图标搜索输入",
                            PreviewLang::EnUs => "Attached icon-only search input",
                        })
                        .clearable(InputClear::new(clear_label))
                        .attached_suffix(
                            IconButton::new("submit-search-attached-icon", IconName::Search)
                                .variant(IconButtonVariant::Ghost)
                                .size(ComponentSize::Md)
                                .aria_label(search_label)
                                .tooltip(search_label)
                                .on_click_in(cx, move |this, _, _, cx| {
                                    let value = search_attached_icon.read(cx).value().to_owned();
                                    this.input_demo.status = match language {
                                        PreviewLang::ZhCn => {
                                            format!("拼接式纯图标搜索：{value}")
                                        }
                                        PreviewLang::EnUs => {
                                            format!("Attached icon-only search: {value}")
                                        }
                                    }
                                    .into();
                                    cx.notify();
                                }),
                        )
                        .on_submit_in(cx, move |this, value, _, cx| {
                            this.input_demo.status = match language {
                                PreviewLang::ZhCn => {
                                    format!("拼接式纯图标搜索：{value}")
                                }
                                PreviewLang::EnUs => {
                                    format!("Attached icon-only search: {value}")
                                }
                            }
                            .into();
                            cx.notify();
                        }),
                    ),
            )
            // #endregion input-search-actions
            .child(div().text_size(px(18.)).child(variants_label))
            // #region input-variants
            .child(
                div().flex().flex_col().gap(px(8.)).children(
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
                        Input::new(("input-variant", index), state)
                            .variant(variant)
                            .aria_label(format!("{variant:?} Input"))
                    }),
                ),
            )
            // #endregion input-variants
            .child(div().text_size(px(18.)).child(sizes_label))
            // #region input-sizes
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
                        Input::new(("input-size", index), state)
                            .size(size)
                            .aria_label(format!("{size:?} Input"))
                    }),
                ),
            )
            // #endregion input-sizes
            .child(div().text_size(px(18.)).child(states_label))
            // #region input-states
            .child(
                Input::new("input-invalid", self.invalid.clone())
                    .invalid(true)
                    .aria_label("Invalid Input"),
            )
            .child(
                Input::new("input-read-only", self.read_only.clone())
                    .read_only(true)
                    .clearable(InputClear::new("Clear read-only Input"))
                    .aria_label("Read-only Input"),
            )
            .child(
                Input::new("input-disabled", self.disabled.clone())
                    .disabled(true)
                    .clearable(InputClear::new("Clear disabled Input"))
                    .aria_label("Disabled Input"),
            )
            .child(
                div().w(px(180.)).child(
                    Input::new("input-narrow", self.narrow.clone())
                        .prefix(Icon::new(IconName::Search))
                        .clearable(
                            InputClear::new("Clear narrow Input")
                                .tooltip(Tooltip::new(clear_tooltip)),
                        )
                        .aria_label("Narrow Input"),
                ),
            )
            // #endregion input-states
            .into_any_element()
    }
}
