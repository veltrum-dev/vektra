#[allow(dead_code)]
mod support;

use serde_json::Value;
use support::assert_contrast_at_least;
use vektra_assets::Assets;
use vektra_theme::{ResolvedTheme, ResolvedThemeMode, default_theme, dtcg, profile};

const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const BUTTON: &str = "themes/default/button.json";
const SELECT: &str = "themes/default/select.json";

#[test]
fn default_select_tokens_resolve_for_all_modes_states_and_sizes() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        assert_ne!(
            theme.select_trigger_state("normal").unwrap().background,
            theme.select_trigger_state("hover").unwrap().background
        );
        assert_ne!(
            theme.select_trigger_state("hover").unwrap().background,
            theme.select_trigger_state("pressed").unwrap().background
        );
        assert_ne!(
            theme.select_option_state("normal").unwrap().background,
            theme.select_option_state("active").unwrap().background
        );
        for state in [
            "normal",
            "hover",
            "pressed",
            "focus-visible",
            "open",
            "disabled",
        ] {
            let state = theme.select_trigger_state(state).unwrap();
            assert_ne!(state.foreground, state.background);
        }
        for state in ["normal", "hover", "active", "selected", "disabled"] {
            let state = theme.select_option_state(state).unwrap();
            assert_ne!(state.foreground, state.background);
        }
        for (name, height) in [
            ("xs", gpui::px(24.)),
            ("sm", gpui::px(32.)),
            ("md", gpui::px(36.)),
            ("lg", gpui::px(40.)),
        ] {
            let size = theme.select_size(name).unwrap();
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
        for state in ["normal", "hover", "pressed", "focus-visible", "open"] {
            let tokens = theme.select_trigger_state(state).unwrap();
            assert_contrast_at_least(
                &format!("{mode:?} Select trigger {state} text"),
                tokens.foreground,
                tokens.background,
                4.5,
            );
            assert_contrast_at_least(
                &format!("{mode:?} Select trigger {state} boundary"),
                tokens.border,
                page,
                3.,
            );
        }
        for state in ["normal", "hover", "active", "selected"] {
            let tokens = theme.select_option_state(state).unwrap();
            assert_contrast_at_least(
                &format!("{mode:?} Select option {state} text"),
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
fn themes_without_select_extension_use_semantic_fallbacks() {
    let tokens = dtcg::parse_token_sets(&[&load(FOUNDATION), &load(LIGHT), &load(BUTTON)]).unwrap();
    profile::validate(&tokens).unwrap();
    let theme = ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).unwrap();
    assert_eq!(
        theme.select_trigger_state("normal").unwrap().border,
        theme.semantic.input_border
    );
    assert_eq!(
        theme.select_option_state("selected").unwrap().background,
        theme.semantic.surface
    );
    assert_eq!(theme.select_size("md").unwrap().height, gpui::px(36.));
    assert_eq!(theme.select.popup_max_height, gpui::px(280.));
    assert_eq!(theme.select.popup_padding, gpui::px(4.));
}

#[test]
fn partial_select_extension_is_rejected() {
    let mut select: Value = serde_json::from_str(&load(SELECT)).unwrap();
    select["select"]["trigger"]["open"]
        .as_object_mut()
        .unwrap()
        .remove("indicator");
    let select = serde_json::to_string(&select).unwrap();
    let tokens =
        dtcg::parse_token_sets(&[&load(FOUNDATION), &load(LIGHT), &load(BUTTON), &select]).unwrap();
    assert!(profile::validate(&tokens).is_err());
}

fn load(path: &str) -> String {
    Assets::load_text(path).unwrap().unwrap().into_owned()
}
