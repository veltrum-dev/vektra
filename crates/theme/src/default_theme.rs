//! Vektra 默认 Light/Dark 主题。

use crate::{
    ResolvedTheme, ResolvedThemeMode,
    dtcg::{ResolvedTokens, parse_token_sets},
    error::ThemeError,
    profile,
};
use gpui::AssetSource;
use std::{
    borrow::Cow,
    sync::{Arc, OnceLock},
};
use vektra_assets::Assets;

const FOUNDATION: &str = "themes/default/foundation.json";
const LIGHT: &str = "themes/default/light.json";
const DARK: &str = "themes/default/dark.json";
const BUTTON: &str = "themes/default/button.json";

static LIGHT_THEME: OnceLock<Arc<ResolvedTheme>> = OnceLock::new();
static DARK_THEME: OnceLock<Arc<ResolvedTheme>> = OnceLock::new();

/// 解析默认主题的 DTCG token。
pub fn default_tokens(mode: ResolvedThemeMode) -> Result<ResolvedTokens, ThemeError> {
    let foundation = load_builtin_text(FOUNDATION)?;
    let mode_source = load_builtin_text(match mode {
        ResolvedThemeMode::Light => LIGHT,
        ResolvedThemeMode::Dark => DARK,
    })?;
    let button = load_builtin_text(BUTTON)?;

    let tokens = parse_token_sets(&[&foundation, &mode_source, &button])?;
    profile::validate(&tokens)?;
    Ok(tokens)
}

/// 返回缓存的默认主题。
///
/// 默认主题在首次读取时解析并转换；render 路径复用缓存结果，不进行 JSON 解析或文件 I/O。
pub fn default_theme(mode: ResolvedThemeMode) -> Arc<ResolvedTheme> {
    match mode {
        ResolvedThemeMode::Light => LIGHT_THEME
            .get_or_init(|| {
                Arc::new(
                    load_default_theme(ResolvedThemeMode::Light)
                        .expect("Vektra 默认 Light 主题必须通过测试保持有效"),
                )
            })
            .clone(),
        ResolvedThemeMode::Dark => DARK_THEME
            .get_or_init(|| {
                Arc::new(
                    load_default_theme(ResolvedThemeMode::Dark)
                        .expect("Vektra 默认 Dark 主题必须通过测试保持有效"),
                )
            })
            .clone(),
    }
}

fn load_default_theme(mode: ResolvedThemeMode) -> Result<ResolvedTheme, ThemeError> {
    ResolvedTheme::from_tokens(mode, default_tokens(mode)?)
}

fn load_builtin_text(path: &str) -> Result<Cow<'static, str>, ThemeError> {
    let bytes = Assets
        .load(path)
        .map_err(|error| ThemeError::ResourceRead {
            path: path.to_owned(),
            message: error.to_string(),
        })?;
    let Some(bytes) = bytes else {
        return Err(ThemeError::MissingResource {
            path: path.to_owned(),
        });
    };

    match bytes {
        Cow::Borrowed(bytes) => std::str::from_utf8(bytes)
            .map(Cow::Borrowed)
            .map_err(|error| ThemeError::ResourceUtf8 {
                path: path.to_owned(),
                message: error.to_string(),
            }),
        Cow::Owned(bytes) => {
            String::from_utf8(bytes)
                .map(Cow::Owned)
                .map_err(|error| ThemeError::ResourceUtf8 {
                    path: path.to_owned(),
                    message: error.to_string(),
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    #[test]
    fn parses_default_light_and_dark() {
        assert!(!default_tokens(ResolvedThemeMode::Light).unwrap().is_empty());
        assert!(!default_tokens(ResolvedThemeMode::Dark).unwrap().is_empty());
    }

    #[test]
    fn caches_default_theme() {
        let first = default_theme(ResolvedThemeMode::Light);
        let second = default_theme(ResolvedThemeMode::Light);
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn all_button_tokens_exist_for_light_and_dark() {
        for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
            let theme = default_theme(mode);
            for variant in [
                "primary",
                "outline",
                "ghost",
                "destructive",
                "secondary",
                "link",
            ] {
                for state in ["normal", "hover", "pressed", "focus-visible", "disabled"] {
                    theme.button_state(variant, state).unwrap();
                }
            }
            for size in ["xs", "sm", "md", "lg"] {
                theme.button_size(size).unwrap();
            }
        }
    }

    #[test]
    fn icon_and_button_icon_size_tokens_resolve() {
        for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
            let theme = default_theme(mode);
            assert_eq!(theme.icon.default_size, px(16.));

            let expected = [
                ("xs", px(24.), px(12.), px(4.)),
                ("sm", px(32.), px(14.), px(6.)),
                ("md", px(36.), px(16.), px(8.)),
                ("lg", px(40.), px(20.), px(8.)),
            ];
            for (size, height, icon_size, content_gap) in expected {
                let tokens = theme.button_size(size).unwrap();
                assert_eq!(tokens.height, height);
                assert_eq!(tokens.icon_size, icon_size);
                assert_eq!(tokens.content_gap, content_gap);
            }
        }
    }

    #[test]
    fn link_button_keeps_transparent_surfaces_except_focus_ring() {
        for mode in [ResolvedThemeMode::Light, ResolvedThemeMode::Dark] {
            let theme = default_theme(mode);
            for state in ["normal", "hover", "pressed", "disabled"] {
                let state = theme.button_state("link", state).unwrap();
                assert!(state.background.is_transparent());
                assert!(state.border.is_transparent());
            }

            let focus_visible = theme.button_state("link", "focus-visible").unwrap();
            assert!(focus_visible.background.is_transparent());
            assert_eq!(focus_visible.border, theme.semantic.ring);
            assert!(!focus_visible.border.is_transparent());
        }
    }

    #[test]
    fn local_schemas_are_available_without_network() {
        let dtcg = include_str!("../schemas/dtcg-2025.10-format.schema.json");
        let profile = include_str!("../schemas/vektra-theme-profile.schema.json");
        assert!(dtcg.contains("2025.10"));
        assert!(profile.contains("Vektra Theme Profile"));
    }

    #[test]
    fn default_theme_resources_are_loaded_from_assets_crate() {
        for path in [FOUNDATION, LIGHT, DARK, BUTTON] {
            let text = load_builtin_text(path).unwrap();
            assert!(!text.is_empty(), "`{path}` 应从 vektra-assets 读取");
        }
    }
}
