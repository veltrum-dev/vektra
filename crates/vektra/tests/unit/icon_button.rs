use super::*;
use crate::{ThemeMode, set_theme_mode};
use gpui::{
    ClickEvent, Context, Hsla, KeyUpEvent, KeyboardButton, Keystroke, Modifiers, Render,
    TestAppContext, point, px, rgb,
};
use std::{cell::Cell, rc::Rc};
use vektra_theme::ResolvedThemeMode;

#[test]
fn tooltip_does_not_replace_icon_button_accessible_name() {
    let button = IconButton::new("settings", IconSource::asset("icons/settings.svg"))
        .aria_label("设置")
        .tooltip("旧提示")
        .tooltip("设置")
        .aria_description("打开应用设置");

    assert_eq!(button.aria_label_text().unwrap().as_ref(), "设置");
    assert_eq!(button.tooltip_text().unwrap().as_ref(), "设置");
    assert_eq!(
        button.aria_description_text().unwrap().as_ref(),
        "打开应用设置"
    );
}

#[test]
fn tooltip_configuration_is_preserved_by_icon_button() {
    let button = IconButton::new("settings", IconSource::asset("icons/settings.svg")).tooltip(
        Tooltip::new("设置")
            .open(true)
            .arrow(false)
            .color(rgb(0xffffff))
            .bg_color(rgb(0x222222))
            .animated(false),
    );
    let tooltip = button.tooltip_value().unwrap();

    assert_eq!(tooltip.text_value().as_ref(), "设置");
    assert_eq!(tooltip.open_value(), Some(true));
    assert!(!tooltip.arrow_value());
    assert!(tooltip.color_value().is_some());
    assert!(tooltip.bg_color_value().is_some());
    assert!(!tooltip.animated_value());
}

#[test]
fn tooltip_placement_defaults_to_bottom_and_can_be_overridden() {
    let default = IconButton::new("default", IconSource::asset("icons/settings.svg"));
    assert_eq!(default.tooltip_placement_value(), TooltipPlacement::Bottom);
    let placed = IconButton::new("placed", IconSource::asset("icons/settings.svg"))
        .tooltip_placement(TooltipPlacement::RightStart);
    assert_eq!(
        placed.tooltip_placement_value(),
        TooltipPlacement::RightStart
    );
}

#[test]
fn defaults_are_unresolved_until_render() {
    let button = IconButton::new("settings", IconSource::asset("icons/settings.svg"));
    assert_eq!(button.variant, None);
    assert_eq!(button.size, None);
    assert_eq!(button.icon_color_value(), None);
    assert_eq!(button.resolved_variant(), IconButtonVariant::Primary);
    assert_eq!(button.resolved_size(), ButtonSize::Md);
    assert!(!button.is_disabled());
}

#[test]
fn variants_resolve_to_legal_token_keys() {
    let variants = [
        (IconButtonVariant::Primary, "primary"),
        (IconButtonVariant::Outline, "outline"),
        (IconButtonVariant::Ghost, "ghost"),
        (IconButtonVariant::Destructive, "destructive"),
        (IconButtonVariant::Secondary, "secondary"),
    ];
    for (variant, token_key) in variants {
        let button =
            IconButton::new(token_key, IconSource::asset("icons/settings.svg")).variant(variant);
        assert_eq!(button.resolved_variant(), variant);
        assert_eq!(variant.token_key(), token_key);
    }
}

#[test]
fn explicit_primary_is_preserved() {
    let button = IconButton::new("settings", IconSource::asset("icons/settings.svg"))
        .variant(IconButtonVariant::Primary);
    assert_eq!(button.variant, Some(IconButtonVariant::Primary));
    assert_eq!(button.resolved_variant(), IconButtonVariant::Primary);
}

#[test]
fn aria_label_stores_original_text() {
    let button =
        IconButton::new("settings", IconSource::asset("icons/settings.svg")).aria_label("设置");
    assert_eq!(button.aria_label_text(), Some(&SharedString::from("设置")));
}

#[test]
fn icon_color_is_preserved() {
    let color = Hsla::red();
    let button =
        IconButton::new("settings", IconSource::asset("icons/settings.svg")).icon_color(color);
    assert_eq!(button.icon_color_value(), Some(color));
}

#[gpui::test]
fn icon_color_resolves_with_disabled_priority(cx: &mut TestAppContext) {
    let (_view, cx) = cx.add_window_view(|_, _| IconButtonTestView {
        count: 0,
        disabled: false,
    });
    cx.update(|window, cx| {
        let theme = theme::current_theme(window, cx);
        let normal = theme.button_state("primary", "normal").unwrap();
        let disabled = theme.button_state("primary", "disabled").unwrap();
        let custom = Hsla::red();

        let enabled =
            IconButton::new("enabled", IconSource::asset("icons/settings.svg")).icon_color(custom);
        assert_eq!(enabled.resolved_icon_color(normal), custom);

        let default = IconButton::new("default", IconSource::asset("icons/settings.svg"));
        assert_eq!(default.resolved_icon_color(normal), normal.foreground);

        let disabled_button = IconButton::new("disabled", IconSource::asset("icons/settings.svg"))
            .icon_color(custom)
            .disabled(true);
        assert_eq!(
            disabled_button.resolved_icon_color(disabled),
            disabled.foreground
        );
    });
}

#[test]
fn callback_event_shape_is_shared() {
    let seen = Rc::new(Cell::new(None));
    let seen_handler = seen.clone();
    let handler = move |event: &ClickEvent, _window: &mut Window, _cx: &mut App| {
        seen_handler.set(Some(event.is_keyboard()));
    };
    let _button =
        IconButton::new("settings", IconSource::asset("icons/settings.svg")).on_click(handler);

    let event = button::keyboard_click(KeyboardButton::Enter);
    assert!(event.is_keyboard());
    assert_eq!(seen.get(), None);
}

struct IconButtonTestView {
    count: usize,
    disabled: bool,
}

impl Render for IconButtonTestView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        IconButton::new("target", IconSource::asset("icons/settings.svg"))
            .aria_label("设置")
            .disabled(self.disabled)
            .on_click(cx.listener(|this, _, _, cx| {
                this.count += 1;
                cx.notify();
            }))
    }
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

#[gpui::test]
fn enabled_mouse_enter_and_space_activate(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| IconButtonTestView {
        count: 0,
        disabled: false,
    });
    draw(cx);
    cx.simulate_click(point(px(18.), px(18.)), Modifiers::none());
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("enter");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });

    assert_eq!(view.read_with(cx, |view, _| view.count), 3);
}

#[gpui::test]
fn disabled_mouse_enter_and_space_do_not_activate(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| IconButtonTestView {
        count: 0,
        disabled: true,
    });
    draw(cx);
    cx.simulate_click(point(px(18.), px(18.)), Modifiers::none());
    cx.simulate_keystrokes("enter space");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });

    assert_eq!(view.read_with(cx, |view, _| view.count), 0);
}

#[gpui::test]
fn modifier_keys_do_not_activate_keyboard_shortcuts(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| IconButtonTestView {
        count: 0,
        disabled: false,
    });
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("cmd-enter cmd-space");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("cmd-space").unwrap(),
    });

    assert_eq!(view.read_with(cx, |view, _| view.count), 0);
}

#[gpui::test]
fn renders_all_sizes_variants_and_themes(cx: &mut TestAppContext) {
    struct MatrixView;

    impl Render for MatrixView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().flex().gap(px(8.)).children(
                [
                    IconButtonVariant::Primary,
                    IconButtonVariant::Outline,
                    IconButtonVariant::Ghost,
                    IconButtonVariant::Destructive,
                    IconButtonVariant::Secondary,
                ]
                .into_iter()
                .flat_map(|variant| {
                    [
                        ButtonSize::Xs,
                        ButtonSize::Sm,
                        ButtonSize::Md,
                        ButtonSize::Lg,
                    ]
                    .into_iter()
                    .map(move |size| {
                        IconButton::new(
                            format!("{variant:?}-{size:?}"),
                            IconSource::asset("icons/settings.svg"),
                        )
                        .aria_label("设置")
                        .variant(variant)
                        .size(size)
                    })
                }),
            )
        }
    }

    let (_view, cx) = cx.add_window_view(|_, _| MatrixView);
    cx.update(|_, cx| set_theme_mode(ThemeMode::Light, cx));
    draw(cx);
    cx.update(|_, cx| set_theme_mode(ThemeMode::Dark, cx));
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(
            crate::resolved_theme_mode(window, cx),
            ResolvedThemeMode::Dark
        );
    });
}
