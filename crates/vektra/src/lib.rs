//! Vektra GPUI 组件库。
//!
//! 第一阶段提供默认主题、Button、Icon 和 IconButton。Button 的 loading、progress
//! 和 selected 状态由调用方控制，不在组件内管理异步任务。Vektra 是组件库，不要求
//! 应用调用 `vektra::init(cx)`，也不要求使用 Vektra 根容器。
//!
//! 默认构建会携带框架默认主题资源。启用 `icons` feature 后，同一个
//! `vektra::assets::Assets` 还会携带 Vektra 内置 SVG 图标。

mod button;
pub mod icon;
mod icon_button;
mod theme;
pub mod traits;

pub use button::{Button, ButtonSize, ButtonVariant};
#[cfg(feature = "icons")]
pub use icon::IconName;
pub use icon::{Icon, IconSource, IntoIconSource};
pub use icon_button::{IconButton, IconButtonVariant};
pub use theme::{current_theme, resolved_theme_mode, semantic_colors, set_theme_mode, theme_mode};
pub use traits::{Clickable, Disableable};
pub use vektra_macros::IntoIconSource;
pub use vektra_theme::{ResolvedTheme, ResolvedThemeMode, SemanticColors, ThemeMode};

/// Vektra 自带资源。
///
/// 传给 GPUI 原生 `with_assets` 后即可加载 Vektra 默认主题资源和 Button loading
/// 指示器。启用 `icons` feature 时，还可加载 `icons/settings.svg` 等内置 SVG 图标。
/// 应用有自己的资源源时，可使用 `Assets::with_overrides(AppAssets)` 组合为单个 GPUI
/// 资源源。
pub mod assets {
    pub use vektra_assets::{Assets, AssetsWithOverrides};
}
