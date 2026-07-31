use super::{PreviewApp, PreviewLang};
use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, relative,
};
use vektra::{Button, ButtonSize, ButtonVariant, IconName, IconSource};

pub(super) struct ButtonDemo {
    clicks: usize,
    last_clicked: SharedString,
}

impl ButtonDemo {
    pub(super) fn new(language: PreviewLang) -> Self {
        Self {
            clicks: 0,
            last_clicked: language.no_recent_click().into(),
        }
    }

    pub(super) fn clicks(&self) -> usize {
        self.clicks
    }

    pub(super) fn last_clicked(&self) -> &SharedString {
        &self.last_clicked
    }

    pub(super) fn record_click(&mut self, label: SharedString) {
        self.clicks += 1;
        self.last_clicked = label;
    }

    pub(super) fn render_basic(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);
        let copy = ButtonCopy::new(language);
        let resolved_theme =
            super::resolved_theme_mode_label_for(vektra::resolved_theme_mode(window, cx), language);
        let theme_mode = super::theme_mode_label_for(vektra::theme_mode(cx), language);

        div()
            .id("button-basic-demo")
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
                    .child(self.header(copy.basic_title, copy.basic_intro))
                    .child(self.status_line(&copy, theme_mode, resolved_theme))
                    .child(
                        self.section(copy.section_basic).child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .flex_wrap()
                                .items_center()
                                // #region button-basic
                                .child(self.click_button("button-basic-interactive", copy.add, cx))
                                .child(
                                    Button::new("button-basic-disabled")
                                        .label(copy.disabled)
                                        .variant(ButtonVariant::Secondary)
                                        .disabled(true),
                                )
                                .child(self.click_button("button-basic-default", copy.default, cx)),
                            // #endregion button-basic
                        ),
                    )
                    .child(
                        self.section(copy.section_variants).child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(10.))
                                // #region button-variants
                                .child(self.variant_row(
                                    "button-variant-primary",
                                    "Primary",
                                    ButtonVariant::Primary,
                                    &copy,
                                    cx,
                                ))
                                .child(self.variant_row(
                                    "button-variant-outline",
                                    "Outline",
                                    ButtonVariant::Outline,
                                    &copy,
                                    cx,
                                ))
                                .child(self.variant_row(
                                    "button-variant-ghost",
                                    "Ghost",
                                    ButtonVariant::Ghost,
                                    &copy,
                                    cx,
                                ))
                                .child(self.variant_row(
                                    "button-variant-destructive",
                                    "Destructive",
                                    ButtonVariant::Destructive,
                                    &copy,
                                    cx,
                                ))
                                .child(self.variant_row(
                                    "button-variant-secondary",
                                    "Secondary",
                                    ButtonVariant::Secondary,
                                    &copy,
                                    cx,
                                ))
                                .child(self.variant_row(
                                    "button-variant-link",
                                    "Link",
                                    ButtonVariant::Link,
                                    &copy,
                                    cx,
                                )),
                            // #endregion button-variants
                        ),
                    )
                    .child(
                        self.section(copy.section_sizes)
                            .child(self.size_notice(&copy))
                            .child(self.size_row("button-size-comparison", &copy, cx)),
                    )
                    .child(
                        self.section(copy.section_icons).child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(10.))
                                // #region button-icons
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.))
                                        .flex_wrap()
                                        .child(
                                            self.click_button(
                                                "button-icon-start",
                                                copy.settings,
                                                cx,
                                            )
                                            .start_icon(IconName::Settings),
                                        )
                                        .child(
                                            self.click_button("button-icon-end", copy.next, cx)
                                                .end_icon(IconName::Settings),
                                        )
                                        .child(
                                            self.click_button(
                                                "button-icon-both",
                                                copy.both_icons,
                                                cx,
                                            )
                                            .start_icon(IconName::Settings)
                                            .end_icon(IconSource::asset("icons/settings.svg")),
                                        )
                                        .child(
                                            self.click_button(
                                                "button-icon-fixed",
                                                copy.fixed_width,
                                                cx,
                                            )
                                            .start_icon(IconName::Settings)
                                            .width(px(112.)),
                                        )
                                        .child(
                                            Button::new("button-icon-disabled")
                                                .label(copy.disabled)
                                                .start_icon(IconName::Settings)
                                                .disabled(true),
                                        )
                                        .child(
                                            self.click_button(
                                                "button-icon-link",
                                                copy.link_icon,
                                                cx,
                                            )
                                            .variant(ButtonVariant::Link)
                                            .start_icon(IconName::Settings),
                                        ),
                                )
                                .child(
                                    div().w(px(360.)).max_w(relative(1.)).child(
                                        self.click_button("button-icon-full", copy.full_width, cx)
                                            .start_icon(IconName::Settings)
                                            .end_icon(IconName::Settings)
                                            .full_width(),
                                    ),
                                ),
                            // #endregion button-icons
                        ),
                    )
                    .child(
                        self.section(copy.section_states).child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .flex_wrap()
                                // #region button-states
                                .child(self.click_button("button-state-normal", copy.normal, cx))
                                .child(
                                    Button::new("button-state-disabled")
                                        .label(copy.disabled)
                                        .disabled(true),
                                )
                                .child(self.click_button("button-state-default", copy.default, cx)),
                            // #endregion button-states
                        ),
                    )
                    .child(
                        self.section(copy.section_auto_space).child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .flex_wrap()
                                // #region button-auto-space
                                .child(self.click_button("button-cn-default", "保存", cx))
                                .child(
                                    self.click_button("button-cn-enabled", "确定", cx)
                                        .auto_insert_space(true),
                                )
                                .child(
                                    self.click_button("button-cn-disabled", "取消", cx)
                                        .auto_insert_space(false),
                                )
                                .child(self.click_button("button-cn-long", "保存设置", cx))
                                .child(self.click_button("button-cn-mixed", "Save 1", cx)),
                            // #endregion button-auto-space
                        ),
                    )
                    .child(
                        self.section(copy.section_width).child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(10.))
                                // #region button-width
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.))
                                        .flex_wrap()
                                        .child(self.click_button(
                                            "button-width-auto",
                                            copy.auto,
                                            cx,
                                        ))
                                        .child(
                                            self.click_button(
                                                "button-width-fixed",
                                                copy.fixed_200,
                                                cx,
                                            )
                                            .width(px(200.)),
                                        )
                                        .child(
                                            self.click_button(
                                                "button-width-narrow",
                                                copy.long_text,
                                                cx,
                                            )
                                            .width(px(72.)),
                                        )
                                        .child(
                                            self.click_button("button-width-cn", "保存", cx)
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
                                            self.click_button(
                                                "button-width-full",
                                                copy.full_width,
                                                cx,
                                            )
                                            .full_width(),
                                        )
                                        .child(
                                            Button::new("button-width-disabled-full")
                                                .label(copy.disabled_full)
                                                .full_width()
                                                .disabled(true),
                                        )
                                        .child(
                                            self.click_button(
                                                "button-width-fixed-then-full",
                                                copy.fixed_then_full,
                                                cx,
                                            )
                                            .width(px(120.))
                                            .full_width(),
                                        )
                                        .child(
                                            self.click_button(
                                                "button-width-full-then-fixed",
                                                copy.full_then_fixed,
                                                cx,
                                            )
                                            .full_width()
                                            .width(px(180.)),
                                        ),
                                ),
                            // #endregion button-width
                        ),
                    ),
            )
            .into_any_element()
    }

    pub(super) fn render_showcase(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);
        let copy = ButtonCopy::new(language);
        let resolved_theme =
            super::resolved_theme_mode_label_for(vektra::resolved_theme_mode(window, cx), language);
        let theme_mode = super::theme_mode_label_for(vektra::theme_mode(cx), language);

        div()
            .id("button-showcase-demo")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .p(px(18.))
                    .max_w(px(760.))
                    .child(self.header(copy.showcase_title, copy.showcase_intro))
                    .child(self.status_line(&copy, theme_mode, resolved_theme))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .flex_wrap()
                            .child(
                                self.click_button("button-showcase-save", copy.save, cx)
                                    .start_icon(IconName::Settings),
                            )
                            .child(
                                self.click_button("button-showcase-outline", copy.secondary, cx)
                                    .variant(ButtonVariant::Outline),
                            )
                            .child(
                                self.click_button("button-showcase-danger", copy.delete, cx)
                                    .variant(ButtonVariant::Destructive),
                            )
                            .child(
                                Button::new("button-showcase-disabled")
                                    .label(copy.disabled)
                                    .variant(ButtonVariant::Secondary)
                                    .disabled(true),
                            ),
                    )
                    .child(self.size_notice(&copy))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .flex_wrap()
                            .child(
                                self.click_button("button-showcase-xs", "XS", cx)
                                    .size(ButtonSize::Xs)
                                    .variant(ButtonVariant::Ghost),
                            )
                            .child(
                                self.click_button("button-showcase-sm", "SM", cx)
                                    .size(ButtonSize::Sm)
                                    .variant(ButtonVariant::Secondary),
                            )
                            .child(
                                self.click_button("button-showcase-md", "MD", cx)
                                    .size(ButtonSize::Md)
                                    .variant(ButtonVariant::Outline),
                            )
                            .child(
                                self.click_button("button-showcase-lg", "LG", cx)
                                    .size(ButtonSize::Lg)
                                    .end_icon(IconName::Settings),
                            ),
                    )
                    .child(
                        div().w(px(360.)).max_w(relative(1.)).child(
                            self.click_button("button-showcase-full", copy.full_width, cx)
                                .full_width(),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn header(&self, title: &'static str, intro: &'static str) -> gpui::Div {
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .text_size(px(24.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .line_height(px(30.))
                    .child(title),
            )
            .child(
                div()
                    .text_size(px(15.))
                    .line_height(px(23.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(intro),
            )
    }

    fn status_line(
        &self,
        copy: &ButtonCopy,
        theme_mode: &'static str,
        resolved_theme: &'static str,
    ) -> AnyElement {
        div()
            .id("button-preview-state")
            .text_size(px(14.))
            .font_weight(FontWeight::SEMIBOLD)
            .line_height(px(22.))
            .child(format!(
                "{}: {}  {}: {}  {}: {}  {}: {}",
                copy.click_count,
                self.clicks,
                copy.last_clicked,
                self.last_clicked,
                copy.theme_setting,
                theme_mode,
                copy.current_theme,
                resolved_theme
            ))
            .into_any_element()
    }

    fn section(&self, title: &'static str) -> gpui::Div {
        div().flex().flex_col().gap(px(10.)).child(
            div()
                .text_size(px(16.))
                .font_weight(FontWeight::SEMIBOLD)
                .line_height(px(22.))
                .child(title),
        )
    }

    fn size_notice(&self, copy: &ButtonCopy) -> gpui::Div {
        div()
            .text_size(px(13.))
            .line_height(px(20.))
            .font_weight(FontWeight::MEDIUM)
            .child(copy.size_notice)
    }

    fn size_row(
        &self,
        id_prefix: &'static str,
        copy: &ButtonCopy,
        cx: &mut Context<PreviewApp>,
    ) -> gpui::Div {
        div()
            .flex()
            .gap(px(8.))
            .flex_wrap()
            .items_center()
            .child(
                self.click_button(format!("{id_prefix}-xs"), copy.size_xs, cx)
                    .size(ButtonSize::Xs),
            )
            .child(
                self.click_button(format!("{id_prefix}-sm"), copy.size_sm, cx)
                    .size(ButtonSize::Sm),
            )
            .child(
                self.click_button(format!("{id_prefix}-md"), copy.size_md, cx)
                    .size(ButtonSize::Md),
            )
            .child(
                self.click_button(format!("{id_prefix}-lg"), copy.size_lg, cx)
                    .size(ButtonSize::Lg),
            )
    }

    fn variant_row(
        &self,
        id_prefix: &'static str,
        label: &'static str,
        variant: ButtonVariant,
        copy: &ButtonCopy,
        cx: &mut Context<PreviewApp>,
    ) -> gpui::Div {
        div()
            .flex()
            .gap(px(8.))
            .flex_wrap()
            .items_center()
            .child(
                div()
                    .w(px(110.))
                    .text_size(px(14.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(label),
            )
            .child(
                self.click_button(format!("{id_prefix}-default"), copy.default, cx)
                    .variant(variant),
            )
            .child(
                Button::new(format!("{id_prefix}-disabled"))
                    .label(copy.disabled)
                    .variant(variant)
                    .disabled(true),
            )
    }

    fn click_button(
        &self,
        id: impl Into<gpui::ElementId>,
        label: &'static str,
        cx: &mut Context<PreviewApp>,
    ) -> Button {
        let clicked = SharedString::new_static(label);
        Button::new(id)
            .label(label)
            .on_click_in(cx, move |this, _, window, cx| {
                this.record_button_click(clicked.clone(), window, cx);
            })
    }
}

struct ButtonCopy {
    basic_title: &'static str,
    basic_intro: &'static str,
    showcase_title: &'static str,
    showcase_intro: &'static str,
    click_count: &'static str,
    last_clicked: &'static str,
    theme_setting: &'static str,
    current_theme: &'static str,
    section_basic: &'static str,
    section_variants: &'static str,
    section_icons: &'static str,
    section_states: &'static str,
    section_auto_space: &'static str,
    section_width: &'static str,
    section_sizes: &'static str,
    size_notice: &'static str,
    add: &'static str,
    disabled: &'static str,
    default: &'static str,
    size_xs: &'static str,
    size_sm: &'static str,
    size_md: &'static str,
    size_lg: &'static str,
    settings: &'static str,
    next: &'static str,
    both_icons: &'static str,
    fixed_width: &'static str,
    link_icon: &'static str,
    full_width: &'static str,
    normal: &'static str,
    auto: &'static str,
    fixed_200: &'static str,
    long_text: &'static str,
    disabled_full: &'static str,
    fixed_then_full: &'static str,
    full_then_fixed: &'static str,
    save: &'static str,
    secondary: &'static str,
    delete: &'static str,
}

impl ButtonCopy {
    fn new(language: PreviewLang) -> Self {
        match language {
            PreviewLang::ZhCn => Self {
                basic_title: "Button 预览",
                basic_intro: "主题由外层 VitePress 控制；切换主题会保留点击状态和焦点。",
                showcase_title: "Button 展示",
                showcase_intro: "用于首页的紧凑预览，展示常用形态和点击反馈。",
                click_count: "点击次数",
                last_clicked: "最近点击",
                theme_setting: "主题设置",
                current_theme: "当前主题",
                section_basic: "基础",
                section_variants: "变体",
                section_icons: "图标",
                section_states: "交互状态",
                section_auto_space: "中文自动空格",
                section_width: "宽度",
                section_sizes: "尺寸对比",
                size_notice: "以下按钮刻意使用不同尺寸，用于展示 XS / SM / MD / LG size token；高度差异并非渲染不一致。",
                add: "增加",
                disabled: "禁用",
                default: "默认配置",
                size_xs: "XS",
                size_sm: "SM",
                size_md: "MD",
                size_lg: "LG",
                settings: "设置",
                next: "下一步",
                both_icons: "两端图标",
                fixed_width: "固定宽度",
                link_icon: "链接图标",
                full_width: "填满父容器",
                normal: "正常",
                auto: "自动宽度",
                fixed_200: "固定宽度 200px",
                long_text: "较长文本",
                disabled_full: "禁用并填满",
                fixed_then_full: "先固定后填满",
                full_then_fixed: "先填满后固定",
                save: "保存",
                secondary: "次要操作",
                delete: "删除",
            },
            PreviewLang::EnUs => Self {
                basic_title: "Button preview",
                basic_intro: "VitePress controls the theme. Theme changes keep click state and focus.",
                showcase_title: "Button showcase",
                showcase_intro: "A compact home-page preview with common shapes and click feedback.",
                click_count: "Clicks",
                last_clicked: "Last clicked",
                theme_setting: "Theme setting",
                current_theme: "Current theme",
                section_basic: "Basics",
                section_variants: "Variants",
                section_icons: "Icons",
                section_states: "Interaction states",
                section_auto_space: "Chinese auto spacing",
                section_width: "Width",
                section_sizes: "Size comparison",
                size_notice: "The buttons below intentionally use different sizes to show the XS / SM / MD / LG size tokens; height differences are not rendering inconsistencies.",
                add: "Add",
                disabled: "Disabled",
                default: "Default",
                size_xs: "XS",
                size_sm: "SM",
                size_md: "MD",
                size_lg: "LG",
                settings: "Settings",
                next: "Next",
                both_icons: "Both icons",
                fixed_width: "Fixed width",
                link_icon: "Link icon",
                full_width: "Full width",
                normal: "Normal",
                auto: "Auto width",
                fixed_200: "Fixed 200px",
                long_text: "Long label",
                disabled_full: "Disabled full",
                fixed_then_full: "Fixed then full",
                full_then_fixed: "Full then fixed",
                save: "Save",
                secondary: "Secondary",
                delete: "Delete",
            },
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/button.rs"]
mod tests;
