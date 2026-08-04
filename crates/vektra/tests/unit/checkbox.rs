use super::*;
use gpui::CursorStyle;

#[test]
fn defaults_are_controlled_unchecked_and_enabled() {
    let checkbox = Checkbox::new("terms");

    assert!(!checkbox.is_checked());
    assert!(!checkbox.is_indeterminate());
    assert!(!checkbox.is_disabled());
    assert_eq!(checkbox.label_text(), None);
    assert_eq!(checkbox.explicit_size(), None);
    assert_eq!(checkbox.cursor_style_value(), None);
    assert!(!checkbox.uses_icon_indicator());
    assert_eq!(checkbox.visual_state(), CheckboxVisualState::Unchecked);
    assert_eq!(
        checkbox.icon_for_state(CheckboxVisualState::Unchecked),
        None
    );
}

#[test]
fn checked_and_indeterminate_builders_are_controlled_and_last_call_wins() {
    let checkbox = Checkbox::new("terms")
        .checked(true)
        .checked(false)
        .indeterminate(true)
        .indeterminate(false)
        .checked(true);

    assert!(checkbox.is_checked());
    assert!(!checkbox.is_indeterminate());
    assert_eq!(checkbox.visual_state(), CheckboxVisualState::Checked);
}

#[test]
fn indeterminate_visual_state_takes_priority_over_checked() {
    let checkbox = Checkbox::new("mixed").checked(false).indeterminate(true);

    assert_eq!(checkbox.visual_state(), CheckboxVisualState::Indeterminate);
    assert_eq!(toggled_state(checkbox.visual_state()), gpui::Toggled::Mixed);
}

#[test]
fn state_transitions_produce_next_checked_value() {
    assert!(next_checked(false, false));
    assert!(!next_checked(true, false));
    assert!(next_checked(false, true));
    assert!(next_checked(true, true));
}

#[test]
fn toggled_state_maps_all_accessibility_values() {
    assert_eq!(
        toggled_state(CheckboxVisualState::Unchecked),
        gpui::Toggled::False
    );
    assert_eq!(
        toggled_state(CheckboxVisualState::Checked),
        gpui::Toggled::True
    );
    assert_eq!(
        toggled_state(CheckboxVisualState::Indeterminate),
        gpui::Toggled::Mixed
    );
}

#[test]
fn label_aria_description_size_cursor_and_disabled_are_preserved() {
    let checkbox = Checkbox::new("terms")
        .label("接受条款")
        .aria_label("条款")
        .aria_description("必须同意后才能继续")
        .size(ComponentSize::Lg)
        .cursor_style(CursorStyle::DragCopy)
        .disabled(true);

    assert_eq!(checkbox.label_text().unwrap().as_ref(), "接受条款");
    assert_eq!(checkbox.aria_label_text().unwrap().as_ref(), "条款");
    assert_eq!(
        checkbox.aria_description_text().unwrap().as_ref(),
        "必须同意后才能继续"
    );
    assert_eq!(checkbox.accessible_label().unwrap().as_ref(), "条款");
    assert_eq!(checkbox.explicit_size(), Some(ComponentSize::Lg));
    assert_eq!(checkbox.cursor_style_value(), Some(CursorStyle::DragCopy));
    assert!(checkbox.is_disabled());
}

#[test]
fn visible_label_is_accessible_name_when_aria_label_is_absent() {
    let checkbox = Checkbox::new("terms").label("接受条款");
    assert_eq!(checkbox.accessible_label().unwrap().as_ref(), "接受条款");
}

#[test]
fn default_and_custom_icons_are_selected_by_state() {
    let checkbox = Checkbox::new("terms");
    assert_eq!(
        checkbox.icon_for_state(CheckboxVisualState::Unchecked),
        None
    );
    assert_eq!(
        checkbox
            .icon_for_state(CheckboxVisualState::Checked)
            .unwrap()
            .path(),
        DEFAULT_CHECKED_ICON
    );
    assert_eq!(
        checkbox
            .icon_for_state(CheckboxVisualState::Indeterminate)
            .unwrap()
            .path(),
        DEFAULT_INDETERMINATE_ICON
    );

    let checkbox = Checkbox::new("custom")
        .unchecked_icon(IconSource::asset("icons/empty.svg"))
        .checked_icon(IconSource::asset("icons/done.svg"))
        .indeterminate_icon(IconSource::asset("icons/mixed.svg"));
    assert_eq!(
        checkbox
            .icon_for_state(CheckboxVisualState::Unchecked)
            .unwrap()
            .path(),
        "icons/empty.svg"
    );
    assert_eq!(
        checkbox
            .icon_for_state(CheckboxVisualState::Checked)
            .unwrap()
            .path(),
        "icons/done.svg"
    );
    assert_eq!(
        checkbox
            .icon_for_state(CheckboxVisualState::Indeterminate)
            .unwrap()
            .path(),
        "icons/mixed.svg"
    );
}

#[test]
fn repeated_icon_builders_use_last_value() {
    let checkbox = Checkbox::new("custom")
        .checked_icon(IconSource::asset("icons/old.svg"))
        .checked_icon(IconSource::asset("icons/new.svg"));

    assert_eq!(
        checkbox
            .icon_for_state(CheckboxVisualState::Checked)
            .unwrap()
            .path(),
        "icons/new.svg"
    );
}

#[test]
fn indicator_icons_replace_the_box_and_preserve_both_state_icons() {
    let checkbox = Checkbox::new("favorite").indicator_icons(
        IconSource::asset("components/checkbox/heart.svg"),
        IconSource::asset("components/checkbox/heart-filled.svg"),
    );

    assert!(checkbox.uses_icon_indicator());
    assert_eq!(
        checkbox
            .icon_for_state(CheckboxVisualState::Unchecked)
            .unwrap()
            .path(),
        "components/checkbox/heart.svg"
    );
    assert_eq!(
        checkbox
            .icon_for_state(CheckboxVisualState::Checked)
            .unwrap()
            .path(),
        "components/checkbox/heart-filled.svg"
    );
}

#[test]
fn icon_indicator_uses_border_for_outline_and_box_background_for_fill() {
    let tokens = CheckboxStateTokens {
        background: gpui::Hsla::black(),
        box_background: gpui::Hsla::green(),
        border: gpui::Hsla::blue(),
        icon: gpui::Hsla::red(),
        label: gpui::Hsla::white(),
    };

    assert_eq!(
        indicator_icon_color(CheckboxVisualState::Unchecked, tokens, true, false),
        tokens.border
    );
    assert_eq!(
        indicator_icon_color(CheckboxVisualState::Checked, tokens, true, false),
        tokens.box_background
    );
    assert_eq!(
        indicator_icon_color(CheckboxVisualState::Indeterminate, tokens, true, false),
        tokens.box_background
    );
    assert_eq!(
        indicator_icon_color(CheckboxVisualState::Unchecked, tokens, false, false),
        tokens.icon
    );
    assert_eq!(
        indicator_icon_color(CheckboxVisualState::Unchecked, tokens, true, true),
        tokens.icon
    );
}
