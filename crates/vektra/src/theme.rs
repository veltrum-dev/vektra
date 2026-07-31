//! Vektra 主题读取与切换入口。

use gpui::{App, BorrowAppContext, Global, Window};
use std::sync::Arc;
use vektra_theme::{ResolvedTheme, ResolvedThemeMode, SemanticColors, ThemeMode, default_theme};

#[derive(Debug, Clone, Default)]
struct ThemeSettings {
    mode: ThemeMode,
}

impl Global for ThemeSettings {}

/// 读取当前 Vektra ThemeMode。
///
/// 如果调用方从未设置过 Vektra 全局主题状态，本函数返回 `ThemeMode::System`。
pub fn theme_mode(cx: &App) -> ThemeMode {
    cx.try_global::<ThemeSettings>()
        .map(|settings| settings.mode)
        .unwrap_or_default()
}

/// 切换 Vektra ThemeMode，并刷新所有窗口。
///
/// 该函数不要求用户预先调用 Vektra 初始化入口；全局状态不存在时会按需创建。
pub fn set_theme_mode(mode: ThemeMode, cx: &mut App) {
    if cx.has_global::<ThemeSettings>() {
        cx.update_global::<ThemeSettings, _>(|settings, _cx| {
            settings.mode = mode;
        });
    } else {
        cx.set_global(ThemeSettings { mode });
    }
    cx.refresh_windows();
}

/// 根据当前窗口外观解析实际使用的 Light/Dark 模式。
pub fn resolved_theme_mode(window: &Window, cx: &App) -> ResolvedThemeMode {
    theme_mode(cx).resolve(window.appearance())
}

/// 读取当前窗口使用的缓存解析主题。
pub fn current_theme(window: &Window, cx: &App) -> Arc<ResolvedTheme> {
    default_theme(resolved_theme_mode(window, cx))
}

/// 读取当前窗口的语义颜色。
pub fn semantic_colors(window: &Window, cx: &App) -> SemanticColors {
    current_theme(window, cx).semantic
}
