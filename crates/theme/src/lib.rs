//! Vektra 主题 crate。
//!
//! 该 crate 负责 DTCG 子集解析、Vektra Theme Profile 校验、默认主题和 GPUI 类型转换。

pub mod adapter;
pub mod default_theme;
pub mod dtcg;
pub mod error;
pub mod mode;
pub mod profile;

pub use adapter::{
    ButtonSizeTokens, ButtonStateTokens, IconTokens, ResolvedTheme, SemanticColors, TooltipTokens,
};
pub use default_theme::{default_theme, default_tokens};
pub use error::ThemeError;
pub use mode::{ResolvedThemeMode, ThemeMode};
