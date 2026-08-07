//! Vektra GPUI 组件库。
//!
//! 提供默认主题和可组合的 Button、Input、Checkbox、RadioGroup、Switch、Icon、IconButton、
//! Tooltip 与 Scrollbar。受控组件的业务值由调用方持有，并通过 [`Changeable`] 请求下一值；
//! Vektra 不在组件内管理业务状态或异步任务。Vektra 是组件库，不要求应用调用
//! `vektra::init(cx)`，也不要求使用 Vektra 根容器。
//!
//! 默认构建会携带框架默认主题资源。启用 `icons` feature 后，同一个
//! `vektra::assets::Assets` 还会携带 Vektra 内置 SVG 图标。

mod button;
mod checkbox;
mod focus;
pub mod icon;
mod icon_button;
mod input;
mod radio;
mod scrollbar;
mod size;
mod switch;
mod theme;
mod tooltip;
pub mod traits;

pub use button::{Button, ButtonVariant};
pub use checkbox::Checkbox;
#[cfg(feature = "icons")]
pub use icon::IconName;
pub use icon::{Icon, IconSource, IntoIconSource};
pub use icon_button::{IconButton, IconButtonVariant};
pub use input::{Input, InputClear, InputEvent, InputState, InputType, InputVariant};
pub use radio::{Radio, RadioGroup};
pub use scrollbar::{
    ScrollArea, ScrollAxis, ScrollGutter, ScrollVisibility, ScrollableExt, ScrollbarConfig,
};
pub use size::{ComponentSize, component_size, set_component_size};
pub use switch::{Switch, SwitchContent};
pub use theme::{current_theme, resolved_theme_mode, semantic_colors, set_theme_mode, theme_mode};
pub use tooltip::{Tooltip, TooltipPlacement};
pub use traits::{Changeable, Clickable, Disableable, Focusable, Sizable};
pub use vektra_macros::IntoIconSource;
pub use vektra_theme::{
    InputSizeTokens, InputStateTokens, InputTokens, RadioSizeTokens, RadioStateTokens, RadioTokens,
    ResolvedTheme, ResolvedThemeMode, ScrollbarTokens, SemanticColors, ThemeMode, TooltipTokens,
};

/// Vektra 自带资源。
///
/// 传给 GPUI 原生 `with_assets` 后即可加载 Vektra 默认主题资源和 Button loading
/// 指示器。启用 `icons` feature 时，还可加载 Eye、EyeOff、Search、Settings 等内置 SVG
/// 图标。应用有自己的资源源时，可使用 `Assets::with_overrides(AppAssets)` 组合为单个
/// GPUI 资源源。
pub mod assets {
    pub use vektra_assets::{Assets, AssetsWithOverrides};
}
