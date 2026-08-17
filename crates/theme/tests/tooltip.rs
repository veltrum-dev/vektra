use serde_json::Value;
use vektra_assets::Assets;
use vektra_theme::{ResolvedTheme, ResolvedThemeMode, default_theme, dtcg, profile};

const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const BUTTON: &str = "themes/default/button.json";
const INPUT: &str = "themes/default/input.json";
const SELECT: &str = "themes/default/select.json";
const TOOLTIP: &str = "themes/default/tooltip.json";

#[test]
fn default_tooltip_tokens_resolve_in_light_and_dark() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        assert!(theme.tooltip.max_width > gpui::px(0.));
        assert!(theme.tooltip.line_height >= theme.tooltip.font_size);
        assert!(theme.tooltip.anchor_gap >= gpui::px(0.));
        assert!(theme.tooltip.viewport_padding > gpui::px(0.));
        assert!(theme.tooltip.arrow_width > gpui::px(0.));
        assert!(theme.tooltip.arrow_height > gpui::px(0.));
        assert!(theme.tooltip.shadow_blur >= gpui::px(0.));
        assert_ne!(theme.tooltip.background, theme.tooltip.foreground);
    }
}

#[test]
fn themes_without_tooltip_extension_use_semantic_fallbacks() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let input = load(INPUT);
    let select = load(SELECT);
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &input, &select]).unwrap();
    profile::validate(&tokens).unwrap();
    let theme = ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).unwrap();

    assert_eq!(theme.tooltip.background, theme.semantic.surface);
    assert_eq!(theme.tooltip.foreground, theme.semantic.foreground);
    assert_eq!(theme.tooltip.max_width, gpui::px(280.));
    assert_eq!(theme.tooltip.arrow_width, gpui::px(12.));
    assert_eq!(theme.tooltip.arrow_height, gpui::px(6.));
}

#[test]
fn legacy_complete_tooltip_extension_uses_new_geometry_fallbacks() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let input = load(INPUT);
    let select = load(SELECT);
    let mut tooltip: Value = serde_json::from_str(&load(TOOLTIP)).unwrap();
    for field in [
        "anchor-gap",
        "viewport-padding",
        "arrow-width",
        "arrow-height",
        "shadow-color",
        "shadow-offset-x",
        "shadow-offset-y",
        "shadow-blur",
        "shadow-spread",
    ] {
        tooltip["tooltip"].as_object_mut().unwrap().remove(field);
    }
    let tooltip = serde_json::to_string(&tooltip).unwrap();
    let tokens =
        dtcg::parse_token_sets(&[&foundation, &light, &button, &input, &select, &tooltip]).unwrap();

    profile::validate(&tokens).unwrap();
    let theme = ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).unwrap();
    assert_eq!(theme.tooltip.anchor_gap, gpui::px(4.));
    assert_eq!(theme.tooltip.viewport_padding, gpui::px(8.));
    assert_eq!(theme.tooltip.arrow_width, gpui::px(12.));
    assert_eq!(theme.tooltip.shadow_offset_y, gpui::px(4.));
}

#[test]
fn partial_tooltip_extension_is_rejected() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let input = load(INPUT);
    let select = load(SELECT);
    let mut tooltip: Value = serde_json::from_str(&load(TOOLTIP)).unwrap();
    tooltip["tooltip"]
        .as_object_mut()
        .unwrap()
        .remove("max-width");
    let tooltip = serde_json::to_string(&tooltip).unwrap();
    let tokens =
        dtcg::parse_token_sets(&[&foundation, &light, &button, &input, &select, &tooltip]).unwrap();

    assert!(profile::validate(&tokens).is_err());
}

fn load(path: &str) -> String {
    Assets::load_text(path).unwrap().unwrap().into_owned()
}
