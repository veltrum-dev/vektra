use serde_json::Value;
use vektra_assets::Assets;
use vektra_theme::{ResolvedTheme, ResolvedThemeMode, default_theme, dtcg, profile};

const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const BUTTON: &str = "themes/default/button.json";
const SCROLLBAR: &str = "themes/default/scrollbar.json";

#[test]
fn default_scrollbar_tokens_have_legal_geometry_in_light_and_dark() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        assert!(theme.scrollbar.thickness > gpui::px(0.));
        assert!(theme.scrollbar.thumb_hover_thickness >= theme.scrollbar.thickness);
        assert!(theme.scrollbar.hit_thickness >= theme.scrollbar.thumb_hover_thickness);
        assert!(theme.scrollbar.hit_thickness >= theme.scrollbar.thickness);
        assert!(theme.scrollbar.min_thumb_length >= theme.scrollbar.hit_thickness);
        assert!(theme.scrollbar.radius >= gpui::px(0.));
        assert!(theme.scrollbar.focus_width > gpui::px(0.));
        assert_eq!(theme.scrollbar.focus_ring, theme.semantic.ring);
    }
}

#[test]
fn themes_without_scrollbar_extension_use_semantic_and_geometry_fallbacks() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button]).unwrap();
    profile::validate(&tokens).unwrap();
    let theme = ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).unwrap();

    assert_eq!(theme.scrollbar.track, theme.semantic.secondary);
    assert_eq!(theme.scrollbar.thumb, theme.semantic.on_muted);
    assert_eq!(theme.scrollbar.thumb_hover, theme.semantic.foreground);
    assert_eq!(theme.scrollbar.thumb_pressed, theme.semantic.primary);
    assert_eq!(theme.scrollbar.thickness, gpui::px(8.));
    assert_eq!(theme.scrollbar.thumb_hover_thickness, gpui::px(10.));
    assert_eq!(theme.scrollbar.hit_thickness, gpui::px(14.));
    assert_eq!(theme.scrollbar.min_thumb_length, gpui::px(24.));
}

#[test]
fn partial_scrollbar_extension_is_rejected() {
    let foundation = load(FOUNDATION);
    let light = load(LIGHT);
    let button = load(BUTTON);
    let mut scrollbar: Value = serde_json::from_str(&load(SCROLLBAR)).unwrap();
    scrollbar["scrollbar"]
        .as_object_mut()
        .unwrap()
        .remove("min-thumb-length");
    let scrollbar = serde_json::to_string(&scrollbar).unwrap();
    let tokens = dtcg::parse_token_sets(&[&foundation, &light, &button, &scrollbar]).unwrap();

    assert!(profile::validate(&tokens).is_err());
}

fn load(path: &str) -> String {
    Assets::load_text(path).unwrap().unwrap().into_owned()
}
