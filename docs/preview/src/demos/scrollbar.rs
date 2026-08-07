use super::{PreviewApp, PreviewLang};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, Orientation, ParentElement, Styled,
    Window, div, px,
};
use vektra::{
    Radio, RadioGroup, ScrollAxis, ScrollGutter, ScrollVisibility, ScrollableExt, ScrollbarConfig,
};

pub(super) fn render_basic(
    language: PreviewLang,
    window: &mut Window,
    cx: &mut Context<PreviewApp>,
) -> AnyElement {
    let theme = vektra::current_theme(window, cx);
    let (title, label, axis_hint) = match language {
        PreviewLang::ZhCn => ("双轴自动滚动条", "产品卡片", "← 横向内容：拖动底部 Thumb →"),
        PreviewLang::EnUs => (
            "Automatic two-axis scrollbar",
            "Product card",
            "← Horizontal content: drag the bottom thumb →",
        ),
    };

    // #region scrollbar-example-basic
    let example = div()
        .h(px(260.))
        .w(px(560.))
        .border_1()
        .border_color(theme.semantic.border)
        .bg(theme.semantic.surface)
        .child(scroll_canvas(
            label,
            axis_hint,
            theme.semantic.muted,
            theme.semantic.primary,
        ))
        .scrollbar()
        .scrollbar_aria_label(title);
    // #endregion scrollbar-example-basic

    example_page("scrollbar-example-basic", title, example, window, cx)
}

pub(super) struct ScrollbarDemo {
    axis: ScrollAxis,
    visibility: ScrollVisibility,
    gutter: ScrollGutter,
}

impl ScrollbarDemo {
    pub(super) const fn new() -> Self {
        Self {
            axis: ScrollAxis::Both,
            visibility: ScrollVisibility::Always,
            gutter: ScrollGutter::Overlay,
        }
    }

    pub(super) fn render_configuration(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);
        let (
            title,
            axis_label,
            visibility_label,
            gutter_label,
            item_label,
            axis_hint,
            overlay_hint,
            stable_hint,
        ) = match language {
            PreviewLang::ZhCn => (
                "动态配置滚动条",
                "滚动轴",
                "显隐",
                "布局",
                "产品卡片",
                "← 横向内容：拖动底部 Thumb →",
                "0px · Track 覆盖内容",
                "14px · 独立 Gutter",
            ),
            PreviewLang::EnUs => (
                "Interactive scrollbar configuration",
                "Axis",
                "Visibility",
                "Gutter",
                "Product card",
                "← Horizontal content: drag the bottom thumb →",
                "0px · Track overlays content",
                "14px · Dedicated gutter",
            ),
        };

        // #region scrollbar-example-configuration
        let axis_control = RadioGroup::new("scrollbar-axis")
            .selected_value(Some(self.axis))
            .orientation(Orientation::Horizontal)
            .aria_label(axis_label)
            .on_change_in(cx, |this, axis, _, cx| {
                this.scrollbar_demo.axis = axis;
                cx.notify();
            })
            .child(Radio::new("scrollbar-axis-vertical", ScrollAxis::Vertical).label("Vertical"))
            .child(
                Radio::new("scrollbar-axis-horizontal", ScrollAxis::Horizontal).label("Horizontal"),
            )
            .child(Radio::new("scrollbar-axis-both", ScrollAxis::Both).label("Both"));

        let visibility_control = RadioGroup::new("scrollbar-visibility")
            .selected_value(Some(self.visibility))
            .orientation(Orientation::Horizontal)
            .aria_label(visibility_label)
            .on_change_in(cx, |this, visibility, _, cx| {
                this.scrollbar_demo.visibility = visibility;
                cx.notify();
            })
            .child(Radio::new("scrollbar-visibility-auto", ScrollVisibility::Auto).label("Auto"))
            .child(
                Radio::new("scrollbar-visibility-always", ScrollVisibility::Always).label("Always"),
            )
            .child(
                Radio::new("scrollbar-visibility-never", ScrollVisibility::Never).label("Never"),
            );

        let gutter_control = RadioGroup::new("scrollbar-gutter")
            .selected_value(Some(self.gutter))
            .orientation(Orientation::Horizontal)
            .aria_label(gutter_label)
            .on_change_in(cx, |this, gutter, _, cx| {
                this.scrollbar_demo.gutter = gutter;
                cx.notify();
            })
            .child(Radio::new("scrollbar-gutter-overlay", ScrollGutter::Overlay).label("Overlay"))
            .child(Radio::new("scrollbar-gutter-stable", ScrollGutter::Stable).label("Stable"));

        let controls = div()
            .flex()
            .items_start()
            .justify_between()
            .gap(px(14.))
            .children([
                control_group(axis_label, axis_control),
                control_group(visibility_label, visibility_control),
                gutter_control_group(
                    gutter_label,
                    gutter_control,
                    self.gutter,
                    overlay_hint,
                    stable_hint,
                    theme.semantic.muted,
                    theme.semantic.primary,
                ),
            ]);

        let example = div()
            .w(px(720.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(controls)
            .child(
                div()
                    .h(px(210.))
                    .w_full()
                    .border_1()
                    .border_color(theme.semantic.border)
                    .bg(theme.semantic.surface)
                    .child(scroll_canvas(
                        item_label,
                        axis_hint,
                        theme.semantic.muted,
                        theme.semantic.primary,
                    ))
                    .scrollbar_with(ScrollbarConfig {
                        axis: self.axis,
                        visibility: self.visibility,
                        gutter: self.gutter,
                    })
                    .scrollbar_id("preview-scrollbar-configuration")
                    .scrollbar_aria_label(title),
            );
        // #endregion scrollbar-example-configuration

        example_page(
            "scrollbar-example-configuration",
            title,
            example,
            window,
            cx,
        )
    }
}

fn control_group(label: &'static str, control: impl IntoElement) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(5.))
        .child(div().text_size(px(12.)).child(label))
        .child(control)
}

fn gutter_control_group(
    label: &'static str,
    control: impl IntoElement,
    gutter: ScrollGutter,
    overlay_hint: &'static str,
    stable_hint: &'static str,
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
            overlay_hint,
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
            stable_hint,
        ),
    };

    div()
        .flex()
        .flex_col()
        .gap(px(5.))
        .child(div().text_size(px(12.)).child(label))
        .child(control)
        .child(probe)
        .child(div().text_size(px(11.)).child(hint))
}

fn scroll_canvas(
    label: &'static str,
    axis_hint: &'static str,
    background: gpui::Hsla,
    accent: gpui::Hsla,
) -> gpui::Div {
    div()
        .flex_none()
        .w(px(1160.))
        .flex()
        .flex_col()
        .child(
            div()
                .flex_none()
                .h(px(36.))
                .flex()
                .items_center()
                .px(px(14.))
                .bg(accent.opacity(0.16))
                .child(axis_hint),
        )
        .child(
            div()
                .grid()
                .grid_cols(5)
                .gap(px(10.))
                .p(px(14.))
                .children((1..=35).map(|index| {
                    div()
                        .h(px(64.))
                        .rounded(px(6.))
                        .bg(background)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(format!("{label} {index}"))
                })),
        )
}

fn example_page(
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
        .p(px(24.))
        .bg(theme.semantic.background)
        .text_color(theme.semantic.foreground)
        .child(div().text_size(px(18.)).child(title))
        .child(example)
        .into_any_element()
}
