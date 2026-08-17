use serde_json::Value;
use vektra_assets::Assets;
use vektra_theme::{
    InputVariantKind, InputVisualState, ResolvedTheme, ResolvedThemeMode, ThemeError, ThemeSize,
    default_theme, dtcg, profile,
};

const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const BUTTON: &str = "themes/default/button.json";
const INPUT: &str = "themes/default/input.json";
const SELECT: &str = "themes/default/select.json";

#[test]
fn default_input_tokens_resolve_for_all_modes_variants_states_and_sizes() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        assert_eq!(theme.input.caret_width, gpui::px(1.));
        for variant in [
            InputVariantKind::Outline,
            InputVariantKind::Filled,
            InputVariantKind::Borderless,
            InputVariantKind::Underline,
        ] {
            for state in [
                InputVisualState::Normal,
                InputVisualState::Hover,
                InputVisualState::FocusVisible,
                InputVisualState::Invalid,
                InputVisualState::InvalidFocusVisible,
                InputVisualState::ReadOnly,
                InputVisualState::Disabled,
            ] {
                let tokens = theme.input_state(variant, state);
                assert_ne!(tokens.foreground, tokens.background);
                assert!(!tokens.status.is_transparent());
            }
        }
        for (size, height) in [
            (ThemeSize::Xs, gpui::px(24.)),
            (ThemeSize::Sm, gpui::px(32.)),
            (ThemeSize::Md, gpui::px(36.)),
            (ThemeSize::Lg, gpui::px(40.)),
        ] {
            let tokens = theme.input_size(size);
            assert_eq!(tokens.height, height);
            assert!(tokens.line_height >= tokens.font_size);
            assert!(tokens.slot_size <= tokens.height);
        }
    }
}

#[test]
fn borderless_stays_transparent_while_filled_keeps_a_required_boundary() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        assert!(
            theme
                .input_state(InputVariantKind::Borderless, InputVisualState::Normal)
                .border
                .is_transparent()
        );
        assert_eq!(
            theme
                .input_state(InputVariantKind::Filled, InputVisualState::Normal)
                .border,
            theme.semantic.input_border
        );
        assert!(
            !theme
                .input_state(InputVariantKind::Borderless, InputVisualState::FocusVisible,)
                .border
                .is_transparent()
        );
    }
}

#[test]
fn themes_without_input_extension_are_rejected_at_construction() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let select = load(SELECT);
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &select]).unwrap();
    assert!(profile::validate(&tokens).is_err());
    assert!(ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).is_err());
}

#[test]
fn complete_input_extension_is_accepted() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let input = load(INPUT);
    let select = load(SELECT);
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &input, &select]).unwrap();

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
    let select = load(SELECT);
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &input, &select]).unwrap();

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
    let select = load(SELECT);
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &input, &select]).unwrap();

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
    let select = load(SELECT);
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &input, &select]).unwrap();

    assert!(profile::validate(&tokens).is_err());
}

#[test]
fn wrong_input_token_type_is_rejected_by_theme_construction() {
    let mut input: Value = serde_json::from_str(&load(INPUT)).unwrap();
    input["input"]["caret-width"]["$type"] = Value::from("color");
    input["input"]["caret-width"]["$value"] = Value::from("{semantic.primary}");
    let input = serde_json::to_string(&input).unwrap();
    let tokens = dtcg::parse_token_sets(&[
        &load(FOUNDATION),
        &load(LIGHT),
        &load(BUTTON),
        &input,
        &load(SELECT),
    ])
    .unwrap();

    assert!(matches!(
        ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens),
        Err(ThemeError::TypeMismatch { path, .. }) if path == "input.caret-width"
    ));
}

#[test]
fn invalid_input_token_reference_is_rejected_during_loading() {
    let mut input: Value = serde_json::from_str(&load(INPUT)).unwrap();
    input["input"]["caret-width"]["$value"] = Value::from("{input.missing-width}");
    let input = serde_json::to_string(&input).unwrap();
    assert!(matches!(
        dtcg::parse_token_sets(&[
            &load(FOUNDATION),
            &load(LIGHT),
            &load(BUTTON),
            &input,
            &load(SELECT),
        ]),
        Err(ThemeError::MissingReference { reference, .. }) if reference == "input.missing-width"
    ));
}

fn load(path: &str) -> String {
    Assets::load_text(path).unwrap().unwrap().into_owned()
}
