#[allow(dead_code)]
mod support;

use serde_json::Value;
use support::assert_contrast_at_least;
use vektra_assets::Assets;
use vektra_theme::{
    ResolvedTheme, ResolvedThemeMode, SelectOptionState, SelectTriggerState, ThemeError, ThemeSize,
    default_theme, dtcg, profile,
};

const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const BUTTON: &str = "themes/default/button.json";
const INPUT: &str = "themes/default/input.json";
const SELECT: &str = "themes/default/select.json";

#[test]
fn default_select_tokens_resolve_for_all_modes_states_and_sizes() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        assert_ne!(
            theme
                .select_trigger_state(SelectTriggerState::Normal)
                .background,
            theme
                .select_trigger_state(SelectTriggerState::Hover)
                .background
        );
        assert_ne!(
            theme
                .select_trigger_state(SelectTriggerState::Hover)
                .background,
            theme
                .select_trigger_state(SelectTriggerState::Pressed)
                .background
        );
        assert_ne!(
            theme
                .select_option_state(SelectOptionState::Normal)
                .background,
            theme
                .select_option_state(SelectOptionState::Active)
                .background
        );
        for state in [
            SelectTriggerState::Normal,
            SelectTriggerState::Hover,
            SelectTriggerState::Pressed,
            SelectTriggerState::FocusVisible,
            SelectTriggerState::Open,
            SelectTriggerState::Disabled,
        ] {
            let state = theme.select_trigger_state(state);
            assert_ne!(state.foreground, state.background);
        }
        for state in [
            SelectOptionState::Normal,
            SelectOptionState::Hover,
            SelectOptionState::Active,
            SelectOptionState::Selected,
            SelectOptionState::Disabled,
        ] {
            let state = theme.select_option_state(state);
            assert_ne!(state.foreground, state.background);
        }
        for (size, height) in [
            (ThemeSize::Xs, gpui::px(24.)),
            (ThemeSize::Sm, gpui::px(32.)),
            (ThemeSize::Md, gpui::px(36.)),
            (ThemeSize::Lg, gpui::px(40.)),
        ] {
            let size = theme.select_size(size);
            assert_eq!(size.height, height);
            assert!(size.line_height >= size.font_size);
        }
        assert!(theme.select.popup_max_height > gpui::px(0.));
        assert!(theme.select.popup_padding > gpui::px(0.));
        assert!(theme.select.popup_viewport_padding > gpui::px(0.));
    }
}

#[test]
fn default_select_text_and_boundaries_meet_contrast_requirements() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        let page = theme.semantic.background;
        for (name, state) in [
            ("normal", SelectTriggerState::Normal),
            ("hover", SelectTriggerState::Hover),
            ("pressed", SelectTriggerState::Pressed),
            ("focus-visible", SelectTriggerState::FocusVisible),
            ("open", SelectTriggerState::Open),
        ] {
            let tokens = theme.select_trigger_state(state);
            assert_contrast_at_least(
                &format!("{mode:?} Select trigger {name} text"),
                tokens.foreground,
                tokens.background,
                4.5,
            );
            assert_contrast_at_least(
                &format!("{mode:?} Select trigger {name} boundary"),
                tokens.border,
                page,
                3.,
            );
        }
        for (name, state) in [
            ("normal", SelectOptionState::Normal),
            ("hover", SelectOptionState::Hover),
            ("active", SelectOptionState::Active),
            ("selected", SelectOptionState::Selected),
        ] {
            let tokens = theme.select_option_state(state);
            assert_contrast_at_least(
                &format!("{mode:?} Select option {name} text"),
                tokens.foreground,
                tokens.background,
                4.5,
            );
        }
        assert_contrast_at_least(
            &format!("{mode:?} Select popup boundary"),
            theme.select.popup_border,
            page,
            3.,
        );
        assert_contrast_at_least(
            &format!("{mode:?} Select error marker"),
            theme.select.status_error,
            theme.select.popup_background,
            3.,
        );
    }
}

#[test]
fn themes_without_select_extension_are_rejected_at_construction() {
    let tokens =
        dtcg::parse_token_sets(&[&load(FOUNDATION), &load(LIGHT), &load(BUTTON), &load(INPUT)])
            .unwrap();
    assert!(profile::validate(&tokens).is_err());
    assert!(ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).is_err());
}

#[test]
fn partial_select_extension_is_rejected() {
    let mut select: Value = serde_json::from_str(&load(SELECT)).unwrap();
    select["select"]["trigger"]["open"]
        .as_object_mut()
        .unwrap()
        .remove("indicator");
    let select = serde_json::to_string(&select).unwrap();
    let tokens = dtcg::parse_token_sets(&[
        &load(FOUNDATION),
        &load(LIGHT),
        &load(BUTTON),
        &load(INPUT),
        &select,
    ])
    .unwrap();
    assert!(profile::validate(&tokens).is_err());
}

#[test]
fn wrong_select_token_type_is_rejected_by_theme_construction() {
    let mut select: Value = serde_json::from_str(&load(SELECT)).unwrap();
    select["select"]["group-label"]["$type"] = Value::from("dimension");
    select["select"]["group-label"]["$value"] = Value::from("{foundation.border.width}");
    let select = serde_json::to_string(&select).unwrap();
    let tokens = dtcg::parse_token_sets(&[
        &load(FOUNDATION),
        &load(LIGHT),
        &load(BUTTON),
        &load(INPUT),
        &select,
    ])
    .unwrap();

    assert!(matches!(
        ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens),
        Err(ThemeError::TypeMismatch { path, .. }) if path == "select.group-label"
    ));
}

#[test]
fn invalid_select_token_reference_is_rejected_during_loading() {
    let mut select: Value = serde_json::from_str(&load(SELECT)).unwrap();
    select["select"]["group-label"]["$value"] = Value::from("{select.missing-color}");
    let select = serde_json::to_string(&select).unwrap();
    assert!(matches!(
        dtcg::parse_token_sets(&[
            &load(FOUNDATION),
            &load(LIGHT),
            &load(BUTTON),
            &load(INPUT),
            &select,
        ]),
        Err(ThemeError::MissingReference { reference, .. }) if reference == "select.missing-color"
    ));
}

fn load(path: &str) -> String {
    Assets::load_text(path).unwrap().unwrap().into_owned()
}
