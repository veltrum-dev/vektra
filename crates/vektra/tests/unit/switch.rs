use super::*;
use gpui::CursorStyle;
use vektra_theme::{ResolvedThemeMode, default_theme};

#[test]
fn defaults_are_controlled_unchecked_and_enabled() {
    let switch = Switch::new("notifications");

    assert!(!switch.is_checked());
    assert!(!switch.is_disabled());
    assert!(!switch.is_loading());
    assert_eq!(
        switch.transition_duration_value(),
        DEFAULT_SWITCH_TRANSITION_DURATION
    );
    assert_eq!(switch.label_text(), None);
    assert_eq!(switch.explicit_size(), None);
    assert_eq!(switch.cursor_style_value(), None);
}

#[test]
fn builders_preserve_the_last_controlled_values() {
    let switch = Switch::new("notifications")
        .checked(true)
        .checked(false)
        .label("通知")
        .aria_label("推送通知")
        .aria_description("立即应用此设置")
        .size(ComponentSize::Lg)
        .cursor_style(CursorStyle::DragCopy)
        .disabled(true)
        .loading(true)
        .loading(false)
        .transition_duration(Duration::from_millis(240));

    assert!(!switch.is_checked());
    assert!(switch.is_disabled());
    assert!(!switch.is_loading());
    assert_eq!(
        switch.transition_duration_value(),
        Duration::from_millis(240)
    );
    assert_eq!(switch.label_text().unwrap().as_ref(), "通知");
    assert_eq!(switch.accessible_label().unwrap().as_ref(), "推送通知");
    assert_eq!(switch.aria_label_text().unwrap().as_ref(), "推送通知");
    assert_eq!(
        switch.aria_description_text().unwrap().as_ref(),
        "立即应用此设置"
    );
    assert_eq!(switch.explicit_size(), Some(ComponentSize::Lg));
    assert_eq!(switch.cursor_style_value(), Some(CursorStyle::DragCopy));
}

#[test]
fn loading_and_transition_duration_builders_are_controlled_and_last_call_wins() {
    let switch = Switch::new("notifications")
        .loading(true)
        .loading(false)
        .loading(true)
        .transition_duration(Duration::from_millis(90))
        .transition_duration(Duration::ZERO);

    assert!(switch.is_loading());
    assert_eq!(switch.transition_duration_value(), Duration::ZERO);
}

#[test]
fn loading_uses_stable_distinct_child_and_animation_ids() {
    let first = Switch::new("notifications");
    let second = Switch::new("notifications");

    assert_eq!(first.loading_indicator_id(), second.loading_indicator_id());
    assert_eq!(first.loading_animation_id(), second.loading_animation_id());
    assert_ne!(first.loading_indicator_id(), first.loading_animation_id());
}

#[test]
fn visible_label_is_the_accessible_name_without_an_override() {
    let switch = Switch::new("notifications").label("通知");
    assert_eq!(switch.accessible_label().unwrap().as_ref(), "通知");
}

#[test]
fn state_content_does_not_replace_the_accessible_name_or_toggle_state() {
    let switch = Switch::new("notifications")
        .checked(true)
        .label("通知")
        .checked_content(SwitchContent::text("开启"))
        .unchecked_content(SwitchContent::text("关闭"));

    assert_eq!(switch.accessible_label().unwrap().as_ref(), "通知");
    assert_eq!(toggled_state(switch.is_checked()), gpui::Toggled::True);
}

#[test]
fn toggled_state_has_only_boolean_values() {
    assert_eq!(toggled_state(false), gpui::Toggled::False);
    assert_eq!(toggled_state(true), gpui::Toggled::True);
}

#[test]
fn thumb_motion_interpolates_between_the_two_track_ends() {
    let off = compact_thumb_offset(
        false,
        gpui::px(40.),
        gpui::px(16.),
        gpui::px(2.),
        gpui::px(1.),
    );
    let on = compact_thumb_offset(
        true,
        gpui::px(40.),
        gpui::px(16.),
        gpui::px(2.),
        gpui::px(1.),
    );

    assert_eq!(off, gpui::px(0.));
    assert_eq!(on, gpui::px(18.));
    assert_eq!(interpolate_pixels(off, on, 0.), off);
    assert_eq!(interpolate_pixels(off, on, 0.5), gpui::px(9.));
    assert_eq!(interpolate_pixels(off, on, 1.), on);
}

#[test]
fn switch_content_supports_text_icon_and_icon_text() {
    let text = SwitchContent::text("开启");
    let icon = SwitchContent::icon(IconSource::asset("icons/check.svg"));
    let icon_text = SwitchContent::icon_text(IconSource::asset("icons/check.svg"), "开启");

    assert!(matches!(
        text.kind,
        SwitchContentKind::Text(ref value) if value.as_ref() == "开启"
    ));
    assert!(matches!(
        icon.kind,
        SwitchContentKind::Icon(ref source) if source.path() == "icons/check.svg"
    ));
    assert!(matches!(
        icon_text.kind,
        SwitchContentKind::IconText { ref icon, ref text }
            if icon.path() == "icons/check.svg" && text.as_ref() == "开启"
    ));
}

#[test]
fn content_builders_are_independent_and_last_call_wins() {
    let switch = Switch::new("notifications")
        .checked_content(SwitchContent::text("旧开启"))
        .checked_content(SwitchContent::text("开启"))
        .unchecked_content(SwitchContent::text("旧关闭"))
        .unchecked_content(SwitchContent::text("关闭"));

    assert!(matches!(
        switch.checked_content_value().unwrap().kind,
        SwitchContentKind::Text(ref value) if value.as_ref() == "开启"
    ));
    assert!(matches!(
        switch.unchecked_content_value().unwrap().kind,
        SwitchContentKind::Text(ref value) if value.as_ref() == "关闭"
    ));
}

#[test]
fn missing_content_keeps_the_compact_track_width() {
    let theme = default_theme(ResolvedThemeMode::Light);
    let size = theme.switch_size("md").unwrap();

    assert_eq!(
        track_width_for(false, gpui::px(0.), size, theme.switch.border_width),
        size.track_width
    );
}

#[test]
fn content_width_depends_on_the_semantic_content_kind() {
    let theme = default_theme(ResolvedThemeMode::Light);
    let size = theme.switch_size("md").unwrap();
    let text = SwitchContent::text("开启");
    let icon = SwitchContent::icon(IconSource::asset("icons/check.svg"));
    let icon_text = SwitchContent::icon_text(IconSource::asset("icons/check.svg"), "开启");

    assert_eq!(
        content_required_width(Some(&text), size),
        size.content_edge_padding + size.content_max_text_width
    );
    assert_eq!(
        content_required_width(Some(&icon), size),
        size.content_icon_size
    );
    assert_eq!(
        content_required_width(Some(&icon_text), size),
        size.content_edge_padding
            + size.content_icon_size
            + size.content_gap
            + size.content_max_text_width
    );
    assert_eq!(content_required_width(None, size), gpui::px(0.));
}

#[test]
fn content_mode_uses_one_stable_content_slot_for_both_states() {
    let theme = default_theme(ResolvedThemeMode::Light);
    let size = theme.switch_size("md").unwrap();
    let checked = SwitchContent::icon(IconSource::asset("icons/check.svg"));
    let unchecked = SwitchContent::icon_text(IconSource::asset("icons/minus.svg"), "关闭");
    let content_width = stable_content_width(Some(&checked), Some(&unchecked), size);
    let track_width = track_width_for(true, content_width, size, theme.switch.border_width);

    assert_eq!(
        content_width,
        size.content_edge_padding
            + size.content_icon_size
            + size.content_gap
            + size.content_max_text_width
    );
    assert_eq!(
        track_width,
        size.content_thumb_size
            + size.content_slot_gap
            + content_width
            + size.content_track_padding * 2.
            + theme.switch.border_width * 2.
    );
    assert_eq!(
        thumb_offset(false, true, content_width, size, theme.switch.border_width),
        gpui::px(0.)
    );
    assert_eq!(
        thumb_offset(true, true, content_width, size, theme.switch.border_width),
        content_width + size.content_slot_gap
    );
}

#[test]
fn content_crossfade_has_a_safe_empty_midpoint() {
    assert_eq!(content_opacities(false, true, 0.), (0., 1.));
    assert_eq!(content_opacities(false, true, 0.25), (0., 0.5));
    assert_eq!(content_opacities(false, true, 0.5), (0., 0.));
    assert_eq!(content_opacities(false, true, 0.75), (0.5, 0.));
    assert_eq!(content_opacities(false, true, 1.), (1., 0.));
    assert_eq!(content_opacities(true, false, 0.), (1., 0.));
    assert_eq!(content_opacities(true, false, 1.), (0., 1.));
}

#[test]
fn ease_out_cubic_has_stable_finite_boundaries() {
    assert_eq!(switch_ease_out_cubic(0.), 0.);
    assert_eq!(switch_ease_out_cubic(1.), 1.);
    for step in 0..=100 {
        let value = switch_ease_out_cubic(step as f32 / 100.);
        assert!(value.is_finite());
        assert!((0. ..=1.).contains(&value));
    }
}

#[test]
fn motion_generation_changes_only_with_checked_and_uses_the_new_duration() {
    let mut state = SwitchMotionState::new(false, DEFAULT_SWITCH_TRANSITION_DURATION);

    state.update(false, Duration::from_millis(240), false);
    assert_eq!(state.generation, 0);
    assert_eq!(state.duration, DEFAULT_SWITCH_TRANSITION_DURATION);
    assert_eq!(state.animate_generation, None);

    state.update(true, Duration::from_millis(240), false);
    assert_eq!(state.generation, 1);
    assert!(!state.from_checked);
    assert_eq!(state.duration, Duration::from_millis(240));
    assert_eq!(state.animate_generation, Some(1));

    state.update(true, Duration::from_millis(360), false);
    assert_eq!(state.generation, 1);
    assert_eq!(state.duration, Duration::from_millis(240));
}

#[test]
fn zero_duration_and_reduced_motion_settle_without_an_animation_generation() {
    let mut zero = SwitchMotionState::new(false, DEFAULT_SWITCH_TRANSITION_DURATION);
    zero.update(true, Duration::ZERO, false);
    assert_eq!(zero.generation, 1);
    assert_eq!(zero.animate_generation, None);

    let mut reduced = SwitchMotionState::new(false, DEFAULT_SWITCH_TRANSITION_DURATION);
    reduced.update(true, Duration::from_millis(240), true);
    assert_eq!(reduced.generation, 1);
    assert_eq!(reduced.animate_generation, None);
}

#[test]
fn disabled_cursor_has_priority_over_loading_cursor() {
    assert_eq!(
        button::resolved_cursor_style(true, true, Some(CursorStyle::DragCopy)),
        CursorStyle::OperationNotAllowed
    );
    assert_eq!(
        button::resolved_cursor_style(false, true, Some(CursorStyle::DragCopy)),
        CursorStyle::Arrow
    );
}

#[test]
fn transition_duration_does_not_change_spinner_period() {
    let _switch = Switch::new("notifications")
        .transition_duration(Duration::from_secs(2))
        .loading(true);
    assert_eq!(SWITCH_LOADING_SPINNER_DURATION, Duration::from_millis(900));
}
