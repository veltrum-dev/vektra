//! 主题模式定义。

use gpui::WindowAppearance;

/// Vektra 的主题模式。
///
/// `System` 是默认值，会根据当前 GPUI window appearance 在 Light/Dark 之间解析。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    /// 跟随当前系统或 GPUI 窗口外观。
    #[default]
    System,
    /// 强制使用默认 Light 主题。
    Light,
    /// 强制使用默认 Dark 主题。
    Dark,
}

/// 已解析到具体调色板的主题模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedThemeMode {
    /// Light 调色板。
    Light,
    /// Dark 调色板。
    Dark,
}

impl ThemeMode {
    /// 根据 GPUI 窗口外观解析 `System` 模式。
    pub fn resolve(self, appearance: WindowAppearance) -> ResolvedThemeMode {
        match self {
            Self::Light => ResolvedThemeMode::Light,
            Self::Dark => ResolvedThemeMode::Dark,
            Self::System => match appearance {
                WindowAppearance::Dark | WindowAppearance::VibrantDark => ResolvedThemeMode::Dark,
                WindowAppearance::Light | WindowAppearance::VibrantLight => {
                    ResolvedThemeMode::Light
                }
            },
        }
    }
}
