use serde_json::Value;
use vektra_assets::Assets;
use vektra_theme::{ResolvedTheme, ResolvedThemeMode, default_theme, dtcg, profile};

const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const BUTTON: &str = "themes/default/button.json";
const INPUT: &str = "themes/default/input.json";

#[test]
fn default_input_tokens_resolve_for_all_modes_variants_states_and_sizes() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        assert_eq!(theme.input.caret_width, gpui::px(1.));
        for variant in ["outline", "filled", "borderless", "underline"] {
            for state in [
                "normal",
                "hover",
                "focus-visible",
                "invalid",
                "invalid-focus-visible",
                "read-only",
                "disabled",
            ] {
                let tokens = theme.input_state(variant, state).unwrap();
                assert_ne!(tokens.foreground, tokens.background);
                assert!(!tokens.status.is_transparent());
            }
        }
        for (name, height) in [
            ("xs", gpui::px(24.)),
            ("sm", gpui::px(32.)),
            ("md", gpui::px(36.)),
            ("lg", gpui::px(40.)),
        ] {
            let tokens = theme.input_size(name).unwrap();
            assert_eq!(tokens.height, height);
            assert!(tokens.line_height >= tokens.font_size);
            assert!(tokens.slot_size <= tokens.height);
        }
    }
}

#[test]
fn borderless_and_filled_preserve_transparent_structural_borders() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        assert!(
            theme
                .input_state("borderless", "normal")
                .unwrap()
                .border
                .is_transparent()
        );
        assert!(
            theme
                .input_state("filled", "normal")
                .unwrap()
                .border
                .is_transparent()
        );
        assert!(
            !theme
                .input_state("borderless", "focus-visible")
                .unwrap()
                .border
                .is_transparent()
        );
    }
}

#[test]
fn themes_without_input_extension_use_semantic_and_size_fallbacks() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button]).unwrap();
    profile::validate(&tokens).unwrap();
    let theme = ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).unwrap();

    let outline = theme.input_state("outline", "normal").unwrap();
    assert_eq!(outline.background, theme.semantic.background);
    assert_eq!(outline.border, theme.semantic.input_border);
    assert!(
        theme
            .input_state("borderless", "normal")
            .unwrap()
            .border
            .is_transparent()
    );
    assert_eq!(theme.input_size("md").unwrap().height, gpui::px(36.));
    assert_eq!(theme.input.caret_width, gpui::px(1.));
}

#[test]
fn complete_input_extension_is_accepted() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let input = load(INPUT);
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &input]).unwrap();

    profile::validate(&tokens).unwrap();
    ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).unwrap();
}

#[test]
fn custom_input_theme_can_override_caret_width() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let mut input: Value = serde_json::from_str(&load(INPUT)).unwrap();
    input["input"]["caret-width"]["$value"]["value"] = Value::from(3);
    let input = serde_json::to_string(&input).unwrap();
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &input]).unwrap();

    profile::validate(&tokens).unwrap();
    let theme = ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).unwrap();
    assert_eq!(theme.input.caret_width, gpui::px(3.));
}

#[test]
fn partial_input_state_extension_is_rejected() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let mut input: Value = serde_json::from_str(&load(INPUT)).unwrap();
    input["input"]["variant"]["outline"]["normal"]
        .as_object_mut()
        .unwrap()
        .remove("status");
    let input = serde_json::to_string(&input).unwrap();
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &input]).unwrap();

    assert!(profile::validate(&tokens).is_err());
}

#[test]
fn partial_input_size_extension_is_rejected() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let mut input: Value = serde_json::from_str(&load(INPUT)).unwrap();
    input["input"]["size"]["md"]
        .as_object_mut()
        .unwrap()
        .remove("height");
    let input = serde_json::to_string(&input).unwrap();
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &input]).unwrap();

    assert!(profile::validate(&tokens).is_err());
}

fn load(path: &str) -> String {
    Assets::load_text(path).unwrap().unwrap().into_owned()
}
