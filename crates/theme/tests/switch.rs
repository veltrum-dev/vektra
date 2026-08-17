use serde_json::Value;
use vektra_assets::Assets;
use vektra_theme::{ResolvedTheme, ResolvedThemeMode, default_theme, dtcg, profile};

const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const DARK: &str = "themes/default/dark.json";
const BUTTON: &str = "themes/default/button.json";
const INPUT: &str = "themes/default/input.json";
const SELECT: &str = "themes/default/select.json";
const SWITCH: &str = "themes/default/switch.json";

#[test]
fn default_switch_tokens_resolve_and_have_legal_size_relationships() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        for visual_state in ["unchecked", "checked"] {
            for state in ["normal", "hover", "pressed", "focus-visible", "disabled"] {
                let tokens = theme.switch_state(visual_state, state).unwrap();
                assert_ne!(tokens.track_background, tokens.thumb);
                assert!(tokens.content.a > 0.);
                assert!(tokens.spinner.a > 0.);
            }
        }
        for size in ["xs", "sm", "md", "lg"] {
            let tokens = theme.switch_size(size).unwrap();
            assert!(tokens.track_width > tokens.track_height);
            assert!(tokens.thumb_size + tokens.track_padding * 2. <= tokens.track_height);
            assert!(tokens.hit_size >= tokens.track_height);
            assert!(tokens.line_height >= tokens.font_size);
            assert!(tokens.content_track_height >= tokens.track_height);
            assert!(
                tokens.content_thumb_size + tokens.content_track_padding * 2.
                    <= tokens.content_track_height
            );
            assert!(tokens.hit_size >= tokens.content_track_height);
            assert!(tokens.content_icon_size <= tokens.content_thumb_size);
            assert!(tokens.spinner_size < tokens.thumb_size);
            assert!(tokens.spinner_size < tokens.content_thumb_size);
        }
    }
}

#[test]
fn default_content_mode_uses_one_track_and_thumb_height() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        for size in ["xs", "sm", "md", "lg"] {
            let tokens = theme.switch_size(size).unwrap();
            assert_eq!(tokens.content_track_height, gpui::px(24.));
            assert_eq!(tokens.content_thumb_size, gpui::px(20.));
            assert!(tokens.content_max_text_width >= tokens.font_size * 2. + gpui::px(2.));
            assert!(tokens.content_max_text_width < tokens.font_size * 3.);
        }
    }
}

#[test]
fn themes_without_a_switch_extension_use_semantic_fallbacks() {
    for (mode, palette) in [
        (ResolvedThemeMode::Light, LIGHT),
        (ResolvedThemeMode::Dark, DARK),
    ] {
        let tokens = parse(palette, None);
        profile::validate(&tokens).unwrap();
        let theme = ResolvedTheme::from_tokens(mode, tokens).unwrap();
        assert_eq!(
            theme
                .switch_state("checked", "normal")
                .unwrap()
                .track_background,
            theme.semantic.primary
        );
        assert_eq!(
            theme
                .switch_state("unchecked", "normal")
                .unwrap()
                .track_border,
            theme.semantic.input_border
        );
        assert_eq!(
            theme.switch_state("checked", "normal").unwrap().content,
            theme.semantic.on_primary
        );
        assert_eq!(
            theme.switch_state("checked", "normal").unwrap().spinner,
            theme.semantic.primary
        );
        let size = theme.switch_size("md").unwrap();
        assert_eq!(size.content_track_height, size.hit_size);
        assert_eq!(size.content_track_padding, size.track_padding);
        assert_eq!(size.content_thumb_size, size.thumb_size);
        assert_eq!(size.content_edge_padding, gpui::px(4.));
        assert_eq!(size.content_icon_size, size.thumb_size);
        assert_eq!(size.content_max_text_width, size.track_width);
        assert_eq!(size.spinner_size, gpui::px(8.));
    }
}

#[test]
fn legacy_complete_switch_extension_uses_content_fallbacks() {
    let mut switch: Value = serde_json::from_str(&load(SWITCH)).unwrap();
    for visual_state in ["unchecked", "checked"] {
        for state in ["normal", "hover", "pressed", "focus-visible", "disabled"] {
            switch["switch"]["state"][visual_state][state]
                .as_object_mut()
                .unwrap()
                .remove("content");
            switch["switch"]["state"][visual_state][state]
                .as_object_mut()
                .unwrap()
                .remove("spinner");
        }
    }
    for size in ["xs", "sm", "md", "lg"] {
        let size = switch["switch"]["size"][size].as_object_mut().unwrap();
        for field in [
            "content-track-height",
            "content-track-padding",
            "content-thumb-size",
            "content-slot-gap",
            "content-edge-padding",
            "content-icon-size",
            "content-gap",
            "content-max-text-width",
            "spinner-size",
        ] {
            size.remove(field);
        }
    }
    let switch = serde_json::to_string(&switch).unwrap();
    let tokens = parse(LIGHT, Some(&switch));
    profile::validate(&tokens).unwrap();
    let theme = ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).unwrap();

    assert_eq!(
        theme.switch_state("checked", "normal").unwrap().content,
        theme.semantic.on_primary
    );
    assert_eq!(
        theme.switch_state("checked", "normal").unwrap().spinner,
        theme.semantic.primary
    );
    let size = theme.switch_size("md").unwrap();
    assert_eq!(size.content_track_height, size.hit_size);
    assert_eq!(size.content_track_padding, size.track_padding);
    assert_eq!(size.content_thumb_size, size.thumb_size);
    assert_eq!(size.content_edge_padding, gpui::px(4.));
    assert_eq!(size.content_icon_size, size.thumb_size);
    assert_eq!(size.content_max_text_width, size.track_width);
    assert_eq!(size.spinner_size, gpui::px(8.));
}

#[test]
fn partial_switch_extensions_are_rejected() {
    let mut switch: Value = serde_json::from_str(&load(SWITCH)).unwrap();
    switch["switch"]["state"]["checked"]["normal"]
        .as_object_mut()
        .unwrap()
        .remove("thumb");
    let switch = serde_json::to_string(&switch).unwrap();
    let tokens = parse(LIGHT, Some(&switch));
    assert!(profile::validate(&tokens).is_err());

    let mut switch: Value = serde_json::from_str(&load(SWITCH)).unwrap();
    switch["switch"]["state"]["checked"]["normal"]
        .as_object_mut()
        .unwrap()
        .remove("spinner");
    let switch = serde_json::to_string(&switch).unwrap();
    let tokens = parse(LIGHT, Some(&switch));
    assert!(profile::validate(&tokens).is_err());

    let mut switch: Value = serde_json::from_str(&load(SWITCH)).unwrap();
    switch["switch"]["size"]["md"]
        .as_object_mut()
        .unwrap()
        .remove("spinner-size");
    let switch = serde_json::to_string(&switch).unwrap();
    let tokens = parse(LIGHT, Some(&switch));
    assert!(profile::validate(&tokens).is_err());

    let mut switch: Value = serde_json::from_str(&load(SWITCH)).unwrap();
    switch["switch"]["size"]["md"]
        .as_object_mut()
        .unwrap()
        .remove("thumb-size");
    let switch = serde_json::to_string(&switch).unwrap();
    let tokens = parse(LIGHT, Some(&switch));
    assert!(profile::validate(&tokens).is_err());

    let mut switch: Value = serde_json::from_str(&load(SWITCH)).unwrap();
    switch["switch"]["state"]["checked"]["normal"]
        .as_object_mut()
        .unwrap()
        .remove("content");
    let switch = serde_json::to_string(&switch).unwrap();
    let tokens = parse(LIGHT, Some(&switch));
    assert!(profile::validate(&tokens).is_err());

    let mut switch: Value = serde_json::from_str(&load(SWITCH)).unwrap();
    switch["switch"]["size"]["md"]
        .as_object_mut()
        .unwrap()
        .remove("content-icon-size");
    let switch = serde_json::to_string(&switch).unwrap();
    let tokens = parse(LIGHT, Some(&switch));
    assert!(profile::validate(&tokens).is_err());
}

fn load(path: &str) -> String {
    Assets::load_text(path).unwrap().unwrap().into_owned()
}

fn parse(palette: &str, extension: Option<&str>) -> dtcg::ResolvedTokens {
    let foundation = load(FOUNDATION);
    let palette = load(palette);
    let button = load(BUTTON);
    let input = load(INPUT);
    let select = load(SELECT);
    let mut sources = vec![
        foundation.as_str(),
        palette.as_str(),
        button.as_str(),
        input.as_str(),
        select.as_str(),
    ];
    sources.extend(extension);
    dtcg::parse_token_sets(&sources).unwrap()
}
