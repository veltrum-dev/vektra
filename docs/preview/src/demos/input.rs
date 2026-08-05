use super::{PreviewApp, PreviewLang};
use gpui::{
    AnyElement, AppContext, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use vektra::{
    Button, ButtonVariant, ComponentSize, Icon, IconButton, IconButtonVariant, IconName, Input,
    InputClear, InputState, InputType, InputVariant, Tooltip, TooltipPlacement,
};

// #region input-example-basic
pub(super) struct InputBasicDemo {
    state: gpui::Entity<InputState>,
}

impl InputBasicDemo {
    pub(super) fn new(cx: &mut Context<PreviewApp>) -> Self {
        Self {
            state: cx.new(|cx| InputState::new("", cx)),
        }
    }

    fn input(&self) -> Input {
        Input::new("name-input", self.state.clone())
            .placeholder("请输入名称")
            .aria_label("名称")
    }
}
// #endregion input-example-basic

impl InputBasicDemo {
    pub(super) fn render(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);
        let title = match language {
            PreviewLang::ZhCn => "基础输入框",
            PreviewLang::EnUs => "Basic input",
        };

        div()
            .id("input-example-basic")
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(16.))
            .p(px(20.))
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(div().text_size(px(18.)).child(title))
            .child(div().w_full().max_w(px(360.)).child(self.input()))
            .into_any_element()
    }
}

// #region input-state
pub(super) struct InputDemo {
    primary: gpui::Entity<InputState>,
    observable: gpui::Entity<InputState>,
    search: gpui::Entity<InputState>,
    password: gpui::Entity<InputState>,
    password_revealed: bool,
    email: gpui::Entity<InputState>,
    phone: gpui::Entity<InputState>,
    url: gpui::Entity<InputState>,
    affixes: gpui::Entity<InputState>,
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
    search_status: gpui::SharedString,
    observable_status: gpui::SharedString,
}

impl InputDemo {
    pub(super) fn new(cx: &mut Context<PreviewApp>) -> Self {
        Self {
            primary: cx
                .new(|cx| InputState::new("双击 English/中文选词，三击全选 👩🏽‍💻 e\u{301}", cx)),
            observable: cx.new(|cx| InputState::new("", cx)),
            search: cx.new(|cx| InputState::new("Vektra Input", cx)),
            password: cx.new(|cx| InputState::new("机密👩🏽‍💻e\u{301}", cx)),
            password_revealed: false,
            email: cx.new(|cx| InputState::new("hello@example.com", cx)),
            phone: cx.new(|cx| InputState::new("+86 138 0000 0000", cx)),
            url: cx.new(|cx| InputState::new("https://vektra.dev", cx)),
            affixes: cx.new(|cx| InputState::new("可组合内容", cx)),
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
            search_status: "尚未提交搜索".into(),
            observable_status: "尝试输入、IME 或按 Enter".into(),
        }
    }

    pub(super) fn render_search(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let (title, placeholder, label, clear) = match language {
            PreviewLang::ZhCn => ("搜索输入", "输入搜索内容", "搜索", "清空搜索内容"),
            PreviewLang::EnUs => (
                "Search input",
                "Enter a search query",
                "Search",
                "Clear search",
            ),
        };
        let status = self.search_status.clone();
        // #region input-example-search
        let example = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                Input::new("typed-search-input", self.search.clone())
                    .input_type(InputType::Search)
                    .placeholder(placeholder)
                    .aria_label(label)
                    .prefix(Icon::new(IconName::Search))
                    .clearable(InputClear::new(clear))
                    .on_submit_in(cx, move |this, value, _, cx| {
                        this.input_demo.search_status = match language {
                            PreviewLang::ZhCn => format!("已搜索：{value}"),
                            PreviewLang::EnUs => format!("Searched: {value}"),
                        }
                        .into();
                        cx.notify();
                    }),
            )
            .child(status);
        // #endregion input-example-search

        self.example_page("input-example-search", title, example, window, cx)
    }

    pub(super) fn render_password(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let (title, input_label, show, hide) = match language {
            PreviewLang::ZhCn => ("密码显隐", "密码", "显示密码", "隐藏密码"),
            PreviewLang::EnUs => (
                "Password reveal",
                "Password",
                "Show password",
                "Hide password",
            ),
        };
        // #region input-example-password
        let revealed = self.password_revealed;
        let action_label = if revealed { hide } else { show };
        let action_icon = if revealed {
            IconName::EyeOff
        } else {
            IconName::Eye
        };
        let example = Input::new("password-input", self.password.clone())
            .input_type(InputType::Password)
            .password_revealed(revealed)
            .aria_label(input_label)
            .suffix(
                IconButton::new("password-reveal", action_icon)
                    .variant(IconButtonVariant::Ghost)
                    .size(ComponentSize::Xs)
                    .selected(revealed)
                    .aria_label(action_label)
                    .tooltip(action_label)
                    .on_click_in(cx, |this, _, window, cx| {
                        this.input_demo.password_revealed = !this.input_demo.password_revealed;
                        let focus = this.input_demo.password.read(cx).focus_handle().clone();
                        window.focus(&focus, cx);
                        cx.notify();
                    }),
            );
        // #endregion input-example-password

        self.example_page("input-example-password", title, example, window, cx)
    }

    pub(super) fn render_types(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let (title, email, phone, url) = match language {
            PreviewLang::ZhCn => ("常用输入语义", "电子邮箱", "电话号码", "网址"),
            PreviewLang::EnUs => ("Common input semantics", "Email", "Phone number", "URL"),
        };
        // #region input-example-types
        let example = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                Input::new("email-input", self.email.clone())
                    .input_type(InputType::Email)
                    .aria_label(email),
            )
            .child(
                Input::new("phone-input", self.phone.clone())
                    .input_type(InputType::Phone)
                    .aria_label(phone),
            )
            .child(
                Input::new("url-input", self.url.clone())
                    .input_type(InputType::Url)
                    .aria_label(url),
            );
        // #endregion input-example-types

        self.example_page("input-example-types", title, example, window, cx)
    }

    pub(super) fn render_affixes(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let (title, label, clear) = match language {
            PreviewLang::ZhCn => ("前后缀与清除", "组合输入", "清空组合输入"),
            PreviewLang::EnUs => (
                "Prefix, suffix, and clear",
                "Composed input",
                "Clear composed input",
            ),
        };
        // #region input-example-affixes
        let example = Input::new("affix-input", self.affixes.clone())
            .aria_label(label)
            .prefix(Icon::new(IconName::Search))
            .clearable(InputClear::new(clear).tooltip(clear))
            .suffix(div().text_size(px(12.)).child("⌘ K"));
        // #endregion input-example-affixes

        self.example_page("input-example-affixes", title, example, window, cx)
    }

    pub(super) fn render_events(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let (title, label, placeholder) = match language {
            PreviewLang::ZhCn => ("可观察交互", "事件输入", "尝试中文 IME 并按 Enter"),
            PreviewLang::EnUs => (
                "Observable interactions",
                "Event input",
                "Try an IME and press Enter",
            ),
        };
        let status = self.observable_status.clone();
        // #region input-example-events
        let example = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                Input::new("observable-input", self.observable.clone())
                    .aria_label(label)
                    .placeholder(placeholder)
                    .on_change_in(cx, |this, value, _, cx| {
                        this.input_demo.observable_status = format!("Changed: {value}").into();
                        cx.notify();
                    })
                    .on_submit_in(cx, |this, value, _, cx| {
                        this.input_demo.observable_status = format!("Submitted: {value}").into();
                        cx.notify();
                    })
                    .on_focus_in(cx, |this, _, cx| {
                        this.input_demo.observable_status = "Focused".into();
                        cx.notify();
                    })
                    .on_blur_in(cx, |this, _, cx| {
                        this.input_demo.observable_status = "Blurred".into();
                        cx.notify();
                    }),
            )
            .child(status);
        // #endregion input-example-events

        self.example_page("input-example-events", title, example, window, cx)
    }

    pub(super) fn render_group(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let (title, placeholder, label) = match language {
            PreviewLang::ZhCn => ("Input Group", "输入搜索内容", "搜索"),
            PreviewLang::EnUs => ("Input group", "Enter a search query", "Search"),
        };
        // #region input-example-group
        let example = Input::new("search-input", self.search_text.clone())
            .size(ComponentSize::Md)
            .placeholder(placeholder)
            .aria_label(label)
            .attached_suffix(
                Button::new("search-button")
                    .label(label)
                    .variant(ButtonVariant::Ghost)
                    .size(ComponentSize::Md)
                    .on_click(|_, _, _| {
                        // 执行搜索。
                    }),
            );
        // #endregion input-example-group

        self.example_page("input-example-group", title, example, window, cx)
    }

    pub(super) fn render_variants(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let title = match language {
            PreviewLang::ZhCn => "外观变体",
            PreviewLang::EnUs => "Visual variants",
        };
        // #region input-example-variants
        let example = div().flex().flex_col().gap(px(8.)).children(
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
        );
        // #endregion input-example-variants

        self.example_page("input-example-variants", title, example, window, cx)
    }

    pub(super) fn render_sizes(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let title = match language {
            PreviewLang::ZhCn => "语义尺寸",
            PreviewLang::EnUs => "Semantic sizes",
        };
        // #region input-example-sizes
        let example = div().flex().flex_col().gap(px(8.)).children(
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
        );
        // #endregion input-example-sizes

        self.example_page("input-example-sizes", title, example, window, cx)
    }

    pub(super) fn render_states(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let title = match language {
            PreviewLang::ZhCn => "输入状态",
            PreviewLang::EnUs => "Input states",
        };
        // #region input-example-states
        let example = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                Input::new("input-invalid", self.invalid.clone())
                    .invalid(true)
                    .aria_label("Invalid Input"),
            )
            .child(
                Input::new("input-read-only", self.read_only.clone())
                    .read_only(true)
                    .aria_label("Read-only Input"),
            )
            .child(
                Input::new("input-disabled", self.disabled.clone())
                    .disabled(true)
                    .aria_label("Disabled Input"),
            );
        // #endregion input-example-states

        self.example_page("input-example-states", title, example, window, cx)
    }

    fn example_page(
        &self,
        id: &'static str,
        title: &'static str,
        example: impl IntoElement,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);

        div()
            .id(id)
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(14.))
            .p(px(20.))
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(div().text_size(px(18.)).child(title))
            .child(div().w_full().max_w(px(520.)).child(example))
            .into_any_element()
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
