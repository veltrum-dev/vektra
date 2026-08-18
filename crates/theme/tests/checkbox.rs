use serde_json::Value;
use vektra_assets::Assets;
use vektra_theme::{ResolvedTheme, ResolvedThemeMode, default_theme, dtcg, profile};

const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const DARK: &str = "themes/default/dark.json";
const BUTTON: &str = "themes/default/button.json";
const INPUT: &str = "themes/default/input.json";
const SELECT: &str = "themes/default/select.json";
const CHECKBOX: &str = "themes/default/checkbox.json";

#[test]
fn default_checkbox_tokens_resolve_in_light_and_dark() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        let hover = theme.checkbox_state("unchecked", "hover").unwrap();
        let pressed = theme.checkbox_state("unchecked", "pressed").unwrap();
        let primary_pressed = theme.button_state("primary", "pressed").unwrap();
        assert_eq!(hover.border, theme.semantic.primary);
        assert_eq!(pressed.border, primary_pressed.background);

        for visual_state in ["unchecked", "checked", "indeterminate"] {
            for state in ["normal", "hover", "pressed", "focus-visible", "disabled"] {
                let tokens = theme.checkbox_state(visual_state, state).unwrap();
                assert_ne!(tokens.label, tokens.background);
            }
        }
        for size in ["xs", "sm", "md", "lg"] {
            let tokens = theme.checkbox_size(size).unwrap();
            assert!(tokens.box_size > gpui::px(0.));
            assert!(tokens.hit_size >= tokens.box_size);
            assert!(tokens.line_height >= tokens.font_size);
        }
    }
}

#[test]
fn themes_without_checkbox_extension_use_semantic_fallbacks() {
    for (mode, palette) in [
        (ResolvedThemeMode::Light, LIGHT),
        (ResolvedThemeMode::Dark, DARK),
    ] {
        let foundation = load(FOUNDATION);
        let palette = load(palette);
        let button = load(BUTTON);
        let input = load(INPUT);
        let select = load(SELECT);
        let tokens =
            dtcg::parse_token_sets(&[&foundation, &palette, &button, &input, &select]).unwrap();
        profile::validate(&tokens).unwrap();
        let theme = ResolvedTheme::from_tokens(mode, tokens).unwrap();

        let normal = theme.checkbox_state("unchecked", "normal").unwrap();
        assert_eq!(normal.box_background, theme.semantic.background);
        assert_eq!(normal.border, theme.semantic.input_border);

        let hover = theme.checkbox_state("unchecked", "hover").unwrap();
        assert_eq!(hover.border, theme.semantic.primary);

        let pressed = theme.checkbox_state("unchecked", "pressed").unwrap();
        let primary_pressed = theme.button_state("primary", "pressed").unwrap();
        assert_eq!(pressed.border, primary_pressed.background);

        let checked = theme.checkbox_state("checked", "normal").unwrap();
        assert_eq!(checked.box_background, theme.semantic.primary);
        assert_eq!(checked.icon, theme.semantic.on_primary);

        let md = theme.checkbox_size("md").unwrap();
        assert_eq!(md.box_size, gpui::px(16.));
        assert_eq!(md.hit_size, gpui::px(16.));
    }
}

#[test]
fn partial_checkbox_state_extension_is_rejected() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let input = load(INPUT);
    let select = load(SELECT);
    let mut checkbox: Value = serde_json::from_str(&load(CHECKBOX)).unwrap();
    checkbox["checkbox"]["state"]["checked"]["normal"]
        .as_object_mut()
        .unwrap()
        .remove("label");
    let checkbox = serde_json::to_string(&checkbox).unwrap();
    let tokens =
        dtcg::parse_token_sets(&[&foundation, &light, &button, &input, &select, &checkbox])
            .unwrap();

    assert!(profile::validate(&tokens).is_err());
}

#[test]
fn partial_checkbox_size_extension_is_rejected() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let input = load(INPUT);
    let select = load(SELECT);
    let mut checkbox: Value = serde_json::from_str(&load(CHECKBOX)).unwrap();
    checkbox["checkbox"]["size"]["md"]
        .as_object_mut()
        .unwrap()
        .remove("hit-size");
    let checkbox = serde_json::to_string(&checkbox).unwrap();
    let tokens =
        dtcg::parse_token_sets(&[&foundation, &light, &button, &input, &select, &checkbox])
            .unwrap();

    assert!(profile::validate(&tokens).is_err());
}

fn load(path: &str) -> String {
    Assets::load_text(path).unwrap().unwrap().into_owned()
}
