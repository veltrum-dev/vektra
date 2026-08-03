use super::{PreviewApp, PreviewLang};
use gpui::{
    AnyElement, Context, Div, InteractiveElement, IntoElement, ParentElement, Styled, Window, div,
    px, rgb,
};
use vektra::{Button, ButtonVariant, IconButton, IconName, Tooltip, TooltipPlacement};

pub(super) fn render(
    language: PreviewLang,
    window: &mut Window,
    cx: &mut Context<PreviewApp>,
) -> AnyElement {
    let theme = vektra::current_theme(window, cx);
    let (title, intro, save, settings, disabled, long) = match language {
        PreviewLang::ZhCn => (
            "Tooltip 预览",
            "悬停或使用 Tab 聚焦 500ms 后显示；指针可移入气泡保持显示，Escape 关闭并保留焦点。",
            "保存",
            "设置",
            "当前操作不可用",
            "这是一段用于验证窄窗口、长中文文本换行和视口边缘避让的补充说明。",
        ),
        PreviewLang::EnUs => (
            "Tooltip preview",
            "Hover or focus with Tab for 500ms. Moving into the bubble keeps it visible; Escape dismisses without moving focus.",
            "Save",
            "Settings",
            "This action is unavailable",
            "This longer tooltip verifies wrapping in a narrow viewport and viewport-edge avoidance.",
        ),
    };

    div()
        .id("tooltip-basic-demo")
        .size_full()
        .flex()
        .flex_col()
        .gap(px(14.))
        .p(px(16.))
        .bg(theme.semantic.background)
        .text_color(theme.semantic.foreground)
        .child(div().text_size(px(24.)).child(title))
        .child(intro)
        .child(
            Button::new("tooltip-controlled")
                .label(match language {
                    PreviewLang::ZhCn => "常驻、自定义颜色、无箭头、无动画",
                    PreviewLang::EnUs => "Persistent, custom colors, no arrow or motion",
                })
                .variant(ButtonVariant::Outline)
                .tooltip(
                    Tooltip::new(match language {
                        PreviewLang::ZhCn => "这里点击保存",
                        PreviewLang::EnUs => "Click here to save",
                    })
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
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    Button::new("tooltip-edge-left")
                        .label("Edge flip")
                        .variant(ButtonVariant::Outline)
                        .tooltip(long)
                        .tooltip_placement(TooltipPlacement::Left),
                )
                .child(
                    IconButton::new("tooltip-settings", IconName::Settings)
                        .aria_label(settings)
                        .tooltip(settings)
                        .tooltip_placement(TooltipPlacement::Top),
                )
                .child(
                    Button::new("tooltip-disabled")
                        .label(disabled)
                        .disabled(true)
                        .tooltip(disabled)
                        .tooltip_placement(TooltipPlacement::Right),
                ),
        )
        .child(
            Button::new("tooltip-long")
                .label(save)
                .variant(ButtonVariant::Outline)
                .tooltip(long)
                .tooltip_placement(TooltipPlacement::Top),
        )
        .into_any_element()
}

fn placement_row(side: &'static str, placements: [(&'static str, TooltipPlacement); 3]) -> Div {
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
