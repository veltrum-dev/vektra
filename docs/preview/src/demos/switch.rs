use super::{PreviewApp, PreviewLang};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use std::time::Duration;
use vektra::{ComponentSize, IconSource, Switch, SwitchContent};

// #region switch-example-basic
pub(super) struct SwitchBasicDemo {
    checked: bool,
}

impl SwitchBasicDemo {
    pub(super) const fn new() -> Self {
        Self { checked: false }
    }

    fn switch(&self, cx: &mut Context<PreviewApp>) -> Switch {
        Switch::new("analytics-switch")
            .checked(self.checked)
            .label("使用分析")
            .on_change_in(cx, |this, next_checked, _, cx| {
                this.switch_basic_demo.checked = next_checked;
                cx.notify();
            })
    }
}
// #endregion switch-example-basic

impl SwitchBasicDemo {
    pub(super) fn render(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);
        let title = match language {
            PreviewLang::ZhCn => "基础开关",
            PreviewLang::EnUs => "Basic switch",
        };

        div()
            .id("switch-example-basic")
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
            .child(self.switch(cx))
            .into_any_element()
    }
}

// #region switch-state
pub(super) struct SwitchDemo {
    notifications: bool,
    no_label: bool,
    xs: bool,
    sm: bool,
    md: bool,
    lg: bool,
    fast: bool,
    slow: bool,
    instant: bool,
    loading: bool,
}

impl SwitchDemo {
    pub(super) const fn new() -> Self {
        Self {
            notifications: true,
            no_label: false,
            xs: false,
            sm: true,
            md: false,
            lg: true,
            fast: false,
            slow: true,
            instant: false,
            loading: true,
        }
    }

    pub(super) fn render_focus(
        &self,
        language: PreviewLang,
        focus_status: gpui::SharedString,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);
        let label = match language {
            PreviewLang::ZhCn => "推送通知",
            PreviewLang::EnUs => "Push notifications",
        };
        // #region switch-example-focus
        let example = Switch::new("focus-switch")
            .checked(self.notifications)
            .label(label)
            .on_change_in(cx, |this, next_checked, _, cx| {
                this.switch_demo.notifications = next_checked;
                cx.notify();
            })
            .on_focus_in(cx, move |this, _, cx| {
                this.record_focus(label, true, cx);
            })
            .on_blur_in(cx, move |this, _, cx| {
                this.record_focus(label, false, cx);
            });
        // #endregion switch-example-focus

        div()
            .id("switch-example-focus")
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(12.))
            .p(px(20.))
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(example)
            .child(focus_status)
            .into_any_element()
    }

    pub(super) fn render_loading(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);
        let (control, pending) = match language {
            PreviewLang::ZhCn => ("控制 loading", "正在同步"),
            PreviewLang::EnUs => ("Control loading", "Syncing"),
        };
        // #region switch-example-loading
        let example = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                Switch::new("loading-control")
                    .checked(self.loading)
                    .label(control)
                    .on_change_in(cx, |this, next_checked, _, cx| {
                        this.switch_demo.loading = next_checked;
                        cx.notify();
                    }),
            )
            .child(
                Switch::new("loading-state")
                    .checked(true)
                    .label(pending)
                    .loading(self.loading),
            );
        // #endregion switch-example-loading

        div()
            .id("switch-example-loading")
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .p(px(20.))
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(example)
            .into_any_element()
    }

    pub(super) fn render_states(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let (title, enabled, disabled) = match language {
            PreviewLang::ZhCn => ("启用与禁用", "可交互开关", "禁用开关"),
            PreviewLang::EnUs => (
                "Enabled and disabled",
                "Interactive switch",
                "Disabled switch",
            ),
        };
        // #region switch-example-states
        let example = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                Switch::new("switch-enabled")
                    .checked(self.notifications)
                    .label(enabled)
                    .on_change_in(cx, |this, next, _, cx| {
                        this.switch_demo.notifications = next;
                        cx.notify();
                    }),
            )
            .child(
                Switch::new("switch-disabled-example")
                    .checked(true)
                    .label(disabled)
                    .disabled(true),
            );
        // #endregion switch-example-states

        self.example_page("switch-example-states", title, example, window, cx)
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
        // #region switch-example-sizes
        let example = div()
            .flex()
            .items_center()
            .gap(px(12.))
            .flex_wrap()
            .children(
                [
                    (ComponentSize::Xs, self.xs),
                    (ComponentSize::Sm, self.sm),
                    (ComponentSize::Md, self.md),
                    (ComponentSize::Lg, self.lg),
                ]
                .into_iter()
                .enumerate()
                .map(|(index, (size, checked))| {
                    Switch::new(("switch-size", index))
                        .checked(checked)
                        .label(format!("{size:?}"))
                        .size(size)
                        .on_change_in(cx, move |this, next, _, cx| {
                            match index {
                                0 => this.switch_demo.xs = next,
                                1 => this.switch_demo.sm = next,
                                2 => this.switch_demo.md = next,
                                _ => this.switch_demo.lg = next,
                            }
                            cx.notify();
                        })
                }),
            );
        // #endregion switch-example-sizes

        self.example_page("switch-example-sizes", title, example, window, cx)
    }

    pub(super) fn render_content(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let (title, text, icon, combined, on, off) = match language {
            PreviewLang::ZhCn => ("文字与图标内容", "文字", "图标", "图标加文字", "开", "关"),
            PreviewLang::EnUs => (
                "Text and icon content",
                "Text",
                "Icon",
                "Icon and text",
                "On",
                "Off",
            ),
        };
        // #region switch-example-content
        let example = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                Switch::new("switch-text-content")
                    .checked(self.sm)
                    .label(text)
                    .checked_content(SwitchContent::text(on))
                    .unchecked_content(SwitchContent::text(off))
                    .on_change_in(cx, |this, next, _, cx| {
                        this.switch_demo.sm = next;
                        cx.notify();
                    }),
            )
            .child(
                Switch::new("switch-icon-content")
                    .checked(self.md)
                    .label(icon)
                    .checked_content(SwitchContent::icon(IconSource::asset(
                        "components/checkbox/check.svg",
                    )))
                    .unchecked_content(SwitchContent::icon(IconSource::asset(
                        "components/checkbox/minus.svg",
                    )))
                    .on_change_in(cx, |this, next, _, cx| {
                        this.switch_demo.md = next;
                        cx.notify();
                    }),
            )
            .child(
                Switch::new("switch-icon-text-content")
                    .checked(self.lg)
                    .label(combined)
                    .checked_content(SwitchContent::icon_text(
                        IconSource::asset("components/checkbox/check.svg"),
                        on,
                    ))
                    .unchecked_content(SwitchContent::icon_text(
                        IconSource::asset("components/checkbox/minus.svg"),
                        off,
                    ))
                    .on_change_in(cx, |this, next, _, cx| {
                        this.switch_demo.lg = next;
                        cx.notify();
                    }),
            );
        // #endregion switch-example-content

        self.example_page("switch-example-content", title, example, window, cx)
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
            .gap(px(16.))
            .p(px(20.))
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(div().text_size(px(18.)).child(title))
            .child(example)
            .into_any_element()
    }
}
// #endregion switch-state

impl SwitchDemo {
    pub(super) fn render(
        &self,
        language: PreviewLang,
        focus_status: gpui::SharedString,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);
        let (
            title,
            intro,
            notifications,
            disabled,
            no_label,
            checked_text,
            unchecked_text,
            xs_checked_text,
            xs_unchecked_text,
            xs_label,
            sm_label,
            md_label,
            lg_label,
        ) = match language {
            PreviewLang::ZhCn => (
                "Switch 预览",
                "受控状态由宿主保存；Space 激活，Enter 不激活。",
                "MD · 推送通知（紧凑）",
                "MD · 禁用设置",
                "无可见标签的开关",
                "开启",
                "关闭",
                "开",
                "关",
                "XS · 精简提示",
                "SM · 自动更新",
                "MD · 声音提醒",
                "LG · 在线状态",
            ),
            PreviewLang::EnUs => (
                "Switch preview",
                "State is controlled by the host; Space activates while Enter does not.",
                "MD · Push notifications (compact)",
                "MD · Disabled setting",
                "Switch without a visible label",
                "On",
                "Off",
                "On",
                "Off",
                "XS · Compact hint",
                "SM · Auto update",
                "MD · Sound alerts",
                "LG · Online status",
            ),
        };
        let (
            animation_title,
            default_duration_label,
            fast_duration_label,
            slow_duration_label,
            zero_duration_label,
            loading_controller_label,
            loading_checked_label,
            loading_unchecked_label,
            disabled_loading_label,
        ) = match language {
            PreviewLang::ZhCn => (
                "动画与 loading",
                "默认 180ms · 固定 ease-out cubic",
                "快速 100ms",
                "较慢 400ms",
                "Duration::ZERO",
                "控制 loading",
                "Loading · checked（保留 Tab）",
                "Loading · unchecked（保留 Tab）",
                "Disabled + loading",
            ),
            PreviewLang::EnUs => (
                "Motion and loading",
                "Default 180ms · fixed ease-out cubic",
                "Fast 100ms",
                "Slower 400ms",
                "Duration::ZERO",
                "Control loading",
                "Loading · checked (keeps Tab stop)",
                "Loading · unchecked (keeps Tab stop)",
                "Disabled + loading",
            ),
        };

        div()
            .id("switch-basic-demo")
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
            .child(focus_status)
            // #region switch-basic
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        Switch::new("switch-notifications")
                            .checked(self.notifications)
                            .label(notifications)
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.switch_demo.notifications = next_checked;
                                cx.notify();
                            })
                            // #region switch-focus
                            .on_focus_in(cx, move |this, _, cx| {
                                this.record_focus(notifications, true, cx);
                            })
                            .on_blur_in(cx, move |this, _, cx| {
                                this.record_focus(notifications, false, cx);
                            }),
                        // #endregion switch-focus
                    )
                    .child(
                        Switch::new("switch-disabled")
                            .checked(true)
                            .checked_content(SwitchContent::text(checked_text))
                            .unchecked_content(SwitchContent::text(unchecked_text))
                            .label(disabled)
                            .disabled(true),
                    )
                    .child(
                        Switch::new("switch-no-label")
                            .checked(self.no_label)
                            .aria_label(no_label)
                            .aria_description("Standalone controlled switch")
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.switch_demo.no_label = next_checked;
                                cx.notify();
                            }),
                    ),
            )
            // #endregion switch-basic
            .child(
                div().flex().gap(px(10.)).flex_wrap().children([
                    Switch::new("switch-xs")
                        .checked(self.xs)
                        .checked_content(SwitchContent::text(xs_checked_text))
                        .unchecked_content(SwitchContent::text(xs_unchecked_text))
                        .label(xs_label)
                        .size(ComponentSize::Xs)
                        .on_change_in(cx, |this, next_checked, _, cx| {
                            this.switch_demo.xs = next_checked;
                            cx.notify();
                        }),
                    Switch::new("switch-sm")
                        .checked(self.sm)
                        .checked_content(SwitchContent::text(checked_text))
                        .unchecked_content(SwitchContent::text(unchecked_text))
                        .label(sm_label)
                        .size(ComponentSize::Sm)
                        .on_change_in(cx, |this, next_checked, _, cx| {
                            this.switch_demo.sm = next_checked;
                            cx.notify();
                        }),
                    Switch::new("switch-md")
                        .checked(self.md)
                        .checked_content(SwitchContent::icon(IconSource::asset(
                            "components/checkbox/check.svg",
                        )))
                        .unchecked_content(SwitchContent::icon(IconSource::asset(
                            "components/checkbox/minus.svg",
                        )))
                        .label(md_label)
                        .size(ComponentSize::Md)
                        .on_change_in(cx, |this, next_checked, _, cx| {
                            this.switch_demo.md = next_checked;
                            cx.notify();
                        }),
                    Switch::new("switch-lg")
                        .checked(self.lg)
                        .checked_content(SwitchContent::icon_text(
                            IconSource::asset("components/checkbox/check.svg"),
                            checked_text,
                        ))
                        .unchecked_content(SwitchContent::icon_text(
                            IconSource::asset("components/checkbox/minus.svg"),
                            unchecked_text,
                        ))
                        .label(lg_label)
                        .size(ComponentSize::Lg)
                        .on_change_in(cx, |this, next_checked, _, cx| {
                            this.switch_demo.lg = next_checked;
                            cx.notify();
                        }),
                ]),
            )
            // #region switch-motion-loading
            .child(div().text_size(px(18.)).child(animation_title))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        Switch::new("switch-default-duration")
                            .checked(self.notifications)
                            .label(default_duration_label)
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.switch_demo.notifications = next_checked;
                                cx.notify();
                            }),
                    )
                    .child(
                        Switch::new("switch-fast-duration")
                            .checked(self.fast)
                            .checked_content(SwitchContent::text(checked_text))
                            .unchecked_content(SwitchContent::text(unchecked_text))
                            .label(fast_duration_label)
                            .transition_duration(Duration::from_millis(100))
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.switch_demo.fast = next_checked;
                                cx.notify();
                            }),
                    )
                    .child(
                        Switch::new("switch-slow-duration")
                            .checked(self.slow)
                            .checked_content(SwitchContent::text(checked_text))
                            .unchecked_content(SwitchContent::text(unchecked_text))
                            .label(slow_duration_label)
                            .transition_duration(Duration::from_millis(400))
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.switch_demo.slow = next_checked;
                                cx.notify();
                            }),
                    )
                    .child(
                        Switch::new("switch-zero-duration")
                            .checked(self.instant)
                            .checked_content(SwitchContent::text(checked_text))
                            .unchecked_content(SwitchContent::text(unchecked_text))
                            .label(zero_duration_label)
                            .transition_duration(Duration::ZERO)
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.switch_demo.instant = next_checked;
                                cx.notify();
                            }),
                    )
                    .child(
                        Switch::new("switch-loading-controller")
                            .checked(self.loading)
                            .label(loading_controller_label)
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.switch_demo.loading = next_checked;
                                cx.notify();
                            }),
                    )
                    .child(
                        Switch::new("switch-loading-checked")
                            .checked(true)
                            .checked_content(SwitchContent::text(checked_text))
                            .unchecked_content(SwitchContent::text(unchecked_text))
                            .label(loading_checked_label)
                            .loading(self.loading),
                    )
                    .child(
                        Switch::new("switch-loading-unchecked")
                            .checked(false)
                            .checked_content(SwitchContent::text(checked_text))
                            .unchecked_content(SwitchContent::text(unchecked_text))
                            .label(loading_unchecked_label)
                            .loading(self.loading),
                    )
                    .child(
                        Switch::new("switch-disabled-loading")
                            .checked(true)
                            .checked_content(SwitchContent::icon_text(
                                IconSource::asset("components/checkbox/check.svg"),
                                checked_text,
                            ))
                            .unchecked_content(SwitchContent::text(unchecked_text))
                            .label(disabled_loading_label)
                            .disabled(true)
                            .loading(self.loading),
                    ),
            )
            // #endregion switch-motion-loading
            .into_any_element()
    }
}
