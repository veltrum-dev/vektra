use serde_json::{Value, json};
use vektra_assets::Assets;
use vektra_theme::{ResolvedTheme, ResolvedThemeMode, default_theme, dtcg, profile};

const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const BUTTON: &str = "themes/default/button.json";
const INPUT: &str = "themes/default/input.json";
const SELECT: &str = "themes/default/select.json";

const VARIANTS: &[&str] = &[
    "primary",
    "outline",
    "ghost",
    "destructive",
    "secondary",
    "link",
];
const STATES: &[&str] = &["normal", "hover", "pressed", "focus-visible", "disabled"];

#[test]
fn default_themes_define_complete_selected_state_matrices() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        for variant in VARIANTS {
            for state in STATES {
                assert!(
                    theme
                        .button_selected_state(variant, state)
                        .unwrap()
                        .is_some()
                );
            }
        }
    }
}

#[test]
fn themes_without_selected_extension_remain_compatible() {
    let (foundation, light, button) = theme_sources_without_selected();
    let tokens =
        dtcg::parse_token_sets(&[&foundation, &light, &button, &load(INPUT), &load(SELECT)])
            .unwrap();
    profile::validate(&tokens).unwrap();
    let theme = ResolvedTheme::from_tokens(ResolvedThemeMode::Light, tokens).unwrap();

    for variant in VARIANTS {
        for state in STATES {
            assert_eq!(theme.button_selected_state(variant, state).unwrap(), None);
        }
    }
}

#[test]
fn partial_selected_extension_is_rejected_by_the_profile() {
    let (foundation, light, button) = theme_sources_without_selected();
    let mut button: Value = serde_json::from_str(&button).unwrap();
    button["button"]["variant"]["primary"]["selected"] = json!({
        "normal": {
            "background": { "$type": "color", "$value": "{semantic.primary}" }
        }
    });
    let button = serde_json::to_string(&button).unwrap();
    let tokens =
        dtcg::parse_token_sets(&[&foundation, &light, &button, &load(INPUT), &load(SELECT)])
            .unwrap();

    assert!(profile::validate(&tokens).is_err());
}

fn theme_sources_without_selected() -> (String, String, String) {
    let foundation = Assets::load_text(FOUNDATION).unwrap().unwrap().into_owned();
    let light = Assets::load_text(LIGHT).unwrap().unwrap().into_owned();
    let button = Assets::load_text(BUTTON).unwrap().unwrap().into_owned();
    let mut button: Value = serde_json::from_str(&button).unwrap();
    for variant in button["button"]["variant"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        variant.as_object_mut().unwrap().remove("selected");
    }
    (foundation, light, serde_json::to_string(&button).unwrap())
}

fn load(path: &str) -> String {
    Assets::load_text(path).unwrap().unwrap().into_owned()
}
