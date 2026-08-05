use super::{PreviewApp, PreviewLang};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Styled, Window, div, px,
};
use vektra::{ComponentSize, IconButton, IconButtonVariant, IconName};

pub(super) fn render_basic(
    language: PreviewLang,
    window: &mut Window,
    cx: &mut Context<PreviewApp>,
) -> AnyElement {
    let title = match language {
        PreviewLang::ZhCn => "基础图标按钮",
        PreviewLang::EnUs => "Basic icon button",
    };
    // #region icon-button-example-basic
    let example = IconButton::new("settings-button", IconName::Settings)
        .aria_label("设置")
        .tooltip("设置");
    // #endregion icon-button-example-basic

    example_page("icon-button-example-basic", title, example, window, cx)
}

pub(super) fn render_variants(
    language: PreviewLang,
    window: &mut Window,
    cx: &mut Context<PreviewApp>,
) -> AnyElement {
    let title = match language {
        PreviewLang::ZhCn => "视觉变体",
        PreviewLang::EnUs => "Visual variants",
    };
    // #region icon-button-example-variants
    let example = div().flex().gap(px(8.)).flex_wrap().children(
        [
            IconButtonVariant::Primary,
            IconButtonVariant::Outline,
            IconButtonVariant::Ghost,
            IconButtonVariant::Destructive,
            IconButtonVariant::Secondary,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, variant)| {
            IconButton::new(("icon-variant", index), IconName::Settings)
                .aria_label(format!("{variant:?}"))
                .variant(variant)
        }),
    );
    // #endregion icon-button-example-variants

    example_page("icon-button-example-variants", title, example, window, cx)
}

pub(super) fn render_sizes(
    language: PreviewLang,
    window: &mut Window,
    cx: &mut Context<PreviewApp>,
) -> AnyElement {
    let title = match language {
        PreviewLang::ZhCn => "语义尺寸",
        PreviewLang::EnUs => "Semantic sizes",
    };
    // #region icon-button-example-sizes
    let example = div().flex().items_center().gap(px(8.)).children(
        [
            ComponentSize::Xs,
            ComponentSize::Sm,
            ComponentSize::Md,
            ComponentSize::Lg,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, size)| {
            IconButton::new(("icon-size", index), IconName::Settings)
                .aria_label(format!("{size:?}"))
                .size(size)
        }),
    );
    // #endregion icon-button-example-sizes

    example_page("icon-button-example-sizes", title, example, window, cx)
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
        .p(px(20.))
        .bg(theme.semantic.background)
        .text_color(theme.semantic.foreground)
        .child(div().text_size(px(18.)).child(title))
        .child(example)
        .into_any_element()
}

pub(super) fn render(
    language: PreviewLang,
    focus_status: gpui::SharedString,
    window: &mut Window,
    cx: &mut Context<PreviewApp>,
) -> AnyElement {
    let theme = vektra::current_theme(window, cx);
    let (title, intro, disabled) = match language {
        PreviewLang::ZhCn => (
            "IconButton 预览",
            "Tab 聚焦，Enter/Space 激活；每个纯图标按钮都有 aria_label。",
            "禁用设置",
        ),
        PreviewLang::EnUs => (
            "IconButton preview",
            "Use Tab to focus and Enter/Space to activate. Every icon-only button has an aria_label.",
            "Disabled settings",
        ),
    };

    div()
        .id("icon-button-basic-demo")
        .size_full()
        .flex()
        .flex_col()
        .gap(px(16.))
        .p(px(20.))
        .bg(theme.semantic.background)
        .text_color(theme.semantic.foreground)
        .child(div().text_size(px(24.)).child(title))
        .child(intro)
        .child(focus_status)
        .child(
            div().flex().gap(px(10.)).flex_wrap().children([
                icon_button(
                    "icon-primary",
                    "Primary",
                    IconButtonVariant::Primary,
                    ComponentSize::Xs,
                    cx,
                ),
                icon_button(
                    "icon-outline",
                    "Outline",
                    IconButtonVariant::Outline,
                    ComponentSize::Sm,
                    cx,
                ),
                icon_button(
                    "icon-ghost",
                    "Ghost",
                    IconButtonVariant::Ghost,
                    ComponentSize::Md,
                    cx,
                ),
                icon_button(
                    "icon-danger",
                    "Destructive",
                    IconButtonVariant::Destructive,
                    ComponentSize::Lg,
                    cx,
                ),
                IconButton::new("icon-disabled", IconName::Settings)
                    .aria_label(disabled)
                    .tooltip(disabled)
                    .disabled(true),
            ]),
        )
        .into_any_element()
}

fn icon_button(
    id: &'static str,
    label: &'static str,
    variant: IconButtonVariant,
    size: ComponentSize,
    cx: &mut Context<PreviewApp>,
) -> IconButton {
    let clicked = gpui::SharedString::new_static(label);
    IconButton::new(id, IconName::Settings)
        .aria_label(label)
        .aria_description("Open settings")
        .tooltip(label)
        .variant(variant)
        .size(size)
        .on_click_in(cx, move |this, _, window, cx| {
            this.record_button_click(clicked.clone(), window, cx);
        })
        .on_focus_in(cx, move |this, _, cx| {
            this.record_focus(label, true, cx);
        })
        .on_blur_in(cx, move |this, _, cx| {
            this.record_focus(label, false, cx);
        })
}
