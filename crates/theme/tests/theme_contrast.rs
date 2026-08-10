mod support;

use std::sync::Arc;

use gpui::{Hsla, WindowAppearance};
use support::{assert_contrast_at_least, assert_neutral, contrast_ratio};
use vektra_assets::Assets;
use vektra_theme::{ResolvedTheme, ResolvedThemeMode, ThemeMode, default_theme, dtcg, profile};

const ENABLED_STATES: [&str; 4] = ["normal", "hover", "pressed", "focus-visible"];
const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const DARK: &str = "themes/default/dark.json";
const BUTTON: &str = "themes/default/button.json";

#[test]
fn default_switch_enabled_states_meet_contrast_and_hierarchy_requirements() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        let page = theme.semantic.background;

        for visual_state in ["unchecked", "checked"] {
            for state in ENABLED_STATES {
                let tokens = theme.switch_state(visual_state, state).unwrap();
                let context = format!("{mode:?} Switch {visual_state} {state}");
                assert_contrast_at_least(
                    &format!("{context} track/page"),
                    tokens.track_background,
                    page,
                    3.,
                );
                assert_contrast_at_least(
                    &format!("{context} border/page"),
                    tokens.track_border,
                    page,
                    3.,
                );
                assert_contrast_at_least(
                    &format!("{context} thumb/track"),
                    tokens.thumb,
                    tokens.track_background,
                    3.,
                );
                assert_contrast_at_least(
                    &format!("{context} content/track"),
                    tokens.content,
                    tokens.track_background,
                    4.5,
                );
                assert_contrast_at_least(
                    &format!("{context} spinner/thumb"),
                    tokens.spinner,
                    tokens.thumb,
                    3.,
                );
                assert_contrast_at_least(&format!("{context} label/page"), tokens.label, page, 4.5);
            }

            let normal = theme.switch_state(visual_state, "normal").unwrap();
            let hover = theme.switch_state(visual_state, "hover").unwrap();
            let pressed = theme.switch_state(visual_state, "pressed").unwrap();
            assert_contrast_at_least(
                &format!("{mode:?} Switch {visual_state} normal/hover feedback"),
                normal.track_background,
                hover.track_background,
                1.1,
            );
            assert_contrast_at_least(
                &format!("{mode:?} Switch {visual_state} hover/pressed feedback"),
                hover.track_background,
                pressed.track_background,
                1.1,
            );
        }

        for state in ENABLED_STATES {
            let unchecked = theme.switch_state("unchecked", state).unwrap();
            let checked = theme.switch_state("checked", state).unwrap();
            assert_contrast_at_least(
                &format!("{mode:?} Switch checked/unchecked {state} state difference"),
                checked.track_background,
                unchecked.track_background,
                3.,
            );
        }

        let unchecked = theme.switch_state("unchecked", "normal").unwrap();
        let checked = theme.switch_state("checked", "normal").unwrap();
        let disabled = theme.switch_state("unchecked", "disabled").unwrap();
        let unchecked_emphasis = contrast_ratio(unchecked.track_background, page);
        let checked_emphasis = contrast_ratio(checked.track_background, page);
        let disabled_emphasis = contrast_ratio(disabled.track_background, page);
        assert!(checked_emphasis > unchecked_emphasis);
        assert!(checked_emphasis <= unchecked_emphasis * 4.);
        assert!(disabled_emphasis > 1.);
        assert!(disabled_emphasis < unchecked_emphasis);
        assert_ne!(disabled.track_background, unchecked.track_background);
        assert_ne!(disabled.thumb, disabled.track_background);
    }
}

#[test]
fn shared_enabled_control_tokens_keep_required_boundaries_and_text_legible() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        let page = theme.semantic.background;

        for visual_state in ["unchecked", "checked", "indeterminate"] {
            for state in ENABLED_STATES {
                let tokens = theme.checkbox_state(visual_state, state).unwrap();
                let context = format!("{mode:?} Checkbox {visual_state} {state}");
                assert_contrast_at_least(
                    &format!("{context} border/page"),
                    tokens.border,
                    page,
                    3.,
                );
                assert_contrast_at_least(
                    &format!("{context} label/background"),
                    tokens.label,
                    opaque_or(tokens.background, page),
                    4.5,
                );
                if visual_state != "unchecked" {
                    assert_contrast_at_least(
                        &format!("{context} icon/box"),
                        tokens.icon,
                        tokens.box_background,
                        3.,
                    );
                }
            }
        }

        for selected in [false, true] {
            for state in ENABLED_STATES {
                let tokens = theme.radio_state(selected, state).unwrap();
                let context = format!("{mode:?} Radio selected={selected} {state}");
                assert_contrast_at_least(
                    &format!("{context} border/page"),
                    tokens.border,
                    page,
                    3.,
                );
                let background = opaque_or(tokens.background, page);
                assert_contrast_at_least(
                    &format!("{context} label/background"),
                    tokens.label,
                    background,
                    4.5,
                );
                assert_contrast_at_least(
                    &format!("{context} description/background"),
                    tokens.description,
                    background,
                    4.5,
                );
                if selected {
                    assert_contrast_at_least(
                        &format!("{context} dot/indicator"),
                        tokens.dot,
                        tokens.indicator_background,
                        3.,
                    );
                }
            }
        }

        for variant in ["outline", "filled", "underline"] {
            for state in ["normal", "hover", "focus-visible"] {
                let tokens = theme.input_state(variant, state).unwrap();
                let background = opaque_or(tokens.background, page);
                let context = format!("{mode:?} Input {variant} {state}");
                assert_contrast_at_least(
                    &format!("{context} border/page"),
                    tokens.border,
                    page,
                    3.,
                );
                assert_contrast_at_least(
                    &format!("{context} foreground/background"),
                    tokens.foreground,
                    background,
                    4.5,
                );
            }
        }

        for variant in ["primary", "outline"] {
            for state in ENABLED_STATES {
                let tokens = theme.button_state(variant, state).unwrap();
                let background = opaque_or(tokens.background, page);
                let context = format!("{mode:?} Button {variant} {state}");
                assert_contrast_at_least(
                    &format!("{context} foreground/background"),
                    tokens.foreground,
                    background,
                    4.5,
                );
                assert_contrast_at_least(
                    &format!("{context} boundary/page"),
                    if variant == "primary" {
                        tokens.background
                    } else {
                        tokens.border
                    },
                    page,
                    3.,
                );
            }
        }

        for variant in [
            "primary",
            "outline",
            "ghost",
            "destructive",
            "secondary",
            "link",
        ] {
            for state in ENABLED_STATES {
                let tokens = theme.button_state(variant, state).unwrap();
                let background = opaque_or(tokens.background, page);
                assert_contrast_at_least(
                    &format!("{mode:?} Button {variant} {state} foreground/background"),
                    tokens.foreground,
                    background,
                    4.5,
                );
                if state == "focus-visible" {
                    assert_contrast_at_least(
                        &format!("{mode:?} Button {variant} focus boundary/page"),
                        tokens.border,
                        page,
                        3.,
                    );
                }
            }
        }

        assert_contrast_at_least(
            &format!("{mode:?} Tooltip foreground/background"),
            theme.tooltip.foreground,
            theme.tooltip.background,
            4.5,
        );
        assert_contrast_at_least(
            &format!("{mode:?} Tooltip border/page"),
            theme.tooltip.border,
            page,
            3.,
        );

        for (state, thumb) in [
            ("normal", theme.scrollbar.thumb),
            ("hover", theme.scrollbar.thumb_hover),
            ("pressed", theme.scrollbar.thumb_pressed),
        ] {
            assert_contrast_at_least(
                &format!("{mode:?} Scrollbar {state} thumb/track"),
                thumb,
                theme.scrollbar.track,
                3.,
            );
        }
        assert!(
            contrast_ratio(theme.scrollbar.thumb_pressed, theme.scrollbar.track)
                >= contrast_ratio(theme.scrollbar.thumb_hover, theme.scrollbar.track)
        );
        assert_contrast_at_least(
            &format!("{mode:?} Scrollbar focus/page"),
            theme.scrollbar.focus_ring,
            page,
            3.,
        );
    }
}

#[test]
fn default_interaction_palette_stays_neutral() {
    for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
        let theme = default_theme(mode);
        for (name, color) in [
            ("primary", theme.semantic.primary),
            (
                "accent-pressed",
                theme.button_state("ghost", "pressed").unwrap().background,
            ),
            ("border", theme.semantic.border),
            ("input-border", theme.semantic.input_border),
            ("ring", theme.semantic.ring),
        ] {
            assert_neutral(&format!("{mode:?} semantic.{name}"), color);
        }
        for state in ENABLED_STATES {
            let unchecked = theme.switch_state("unchecked", state).unwrap();
            let checked = theme.switch_state("checked", state).unwrap();
            assert_neutral(
                &format!("{mode:?} Switch unchecked {state}"),
                unchecked.track_background,
            );
            assert_neutral(
                &format!("{mode:?} Switch checked {state}"),
                checked.track_background,
            );
        }
    }
}

#[test]
fn system_mode_uses_the_same_contrast_qualified_palettes() {
    for (appearance, expected) in [
        (WindowAppearance::Light, ResolvedThemeMode::Light),
        (WindowAppearance::VibrantLight, ResolvedThemeMode::Light),
        (WindowAppearance::Dark, ResolvedThemeMode::Dark),
        (WindowAppearance::VibrantDark, ResolvedThemeMode::Dark),
    ] {
        let resolved = ThemeMode::System.resolve(appearance);
        assert_eq!(resolved, expected);
        let system_theme = default_theme(resolved);
        let explicit_theme = default_theme(expected);
        assert!(Arc::ptr_eq(&system_theme, &explicit_theme));
    }
}

#[test]
fn themes_without_component_extensions_keep_contrast_qualified_semantic_fallbacks() {
    for (mode, palette) in [
        (ResolvedThemeMode::Light, LIGHT),
        (ResolvedThemeMode::Dark, DARK),
    ] {
        let tokens =
            dtcg::parse_token_sets(&[&load(FOUNDATION), &load(palette), &load(BUTTON)]).unwrap();
        profile::validate(&tokens).unwrap();
        let theme = ResolvedTheme::from_tokens(mode, tokens).unwrap();
        let page = theme.semantic.background;

        for state in ENABLED_STATES {
            for visual_state in ["unchecked", "checked"] {
                let tokens = theme.switch_state(visual_state, state).unwrap();
                let context = format!("{mode:?} fallback Switch {visual_state} {state}");
                assert_contrast_at_least(
                    &format!("{context} track/page"),
                    tokens.track_background,
                    page,
                    3.,
                );
                assert_contrast_at_least(
                    &format!("{context} thumb/track"),
                    tokens.thumb,
                    tokens.track_background,
                    3.,
                );
                assert_contrast_at_least(
                    &format!("{context} content/track"),
                    tokens.content,
                    tokens.track_background,
                    4.5,
                );
            }

            for visual_state in ["unchecked", "checked", "indeterminate"] {
                let tokens = theme.checkbox_state(visual_state, state).unwrap();
                assert_contrast_at_least(
                    &format!("{mode:?} fallback Checkbox {visual_state} {state} boundary/page"),
                    tokens.border,
                    page,
                    3.,
                );
            }

            for selected in [false, true] {
                let tokens = theme.radio_state(selected, state).unwrap();
                assert_contrast_at_least(
                    &format!("{mode:?} fallback Radio selected={selected} {state} boundary/page"),
                    tokens.border,
                    page,
                    3.,
                );
            }
        }

        for variant in ["outline", "filled", "underline"] {
            for state in ["normal", "hover", "focus-visible"] {
                let tokens = theme.input_state(variant, state).unwrap();
                assert_contrast_at_least(
                    &format!("{mode:?} fallback Input {variant} {state} boundary/page"),
                    tokens.border,
                    page,
                    3.,
                );
            }
        }
    }
}

fn opaque_or(color: Hsla, fallback: Hsla) -> Hsla {
    if (color.a - 1.).abs() <= f32::EPSILON {
        color
    } else {
        fallback
    }
}

fn load(path: &str) -> String {
    Assets::load_text(path).unwrap().unwrap().into_owned()
}
