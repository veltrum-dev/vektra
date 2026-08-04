//! Vektra 组件共享语义尺寸。

use gpui::{App, BorrowAppContext, Global};

/// Vektra 公开组件共享的语义尺寸。
///
/// 该枚举只描述控件大小语义，不代表统一像素倍率。Button、IconButton、Checkbox
/// 等组件会把同一个语义值映射到各自的主题 token。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComponentSize {
    /// 最小尺寸。
    Xs,
    /// 小尺寸。
    Sm,
    /// 默认中等尺寸。
    #[default]
    Md,
    /// 大尺寸。
    Lg,
}

impl ComponentSize {
    pub(crate) const fn token_key(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ComponentSizeSettings {
    size: ComponentSize,
}

impl Global for ComponentSizeSettings {}

/// 读取当前全局默认组件语义尺寸。
///
/// 调用方无需预先初始化 Vektra；从未设置时返回 [`ComponentSize::Md`]。
pub fn component_size(cx: &App) -> ComponentSize {
    cx.try_global::<ComponentSizeSettings>()
        .map(|settings| settings.size)
        .unwrap_or_default()
}

/// 设置全局默认组件语义尺寸，并刷新所有窗口。
///
/// 只影响未显式调用 `.size(...)` 的组件。已经设置组件级尺寸的 Button、
/// IconButton 和 Checkbox 会继续使用自身的显式尺寸。
pub fn set_component_size(size: ComponentSize, cx: &mut App) {
    if cx.has_global::<ComponentSizeSettings>() {
        cx.update_global::<ComponentSizeSettings, _>(|settings, _cx| {
            settings.size = size;
        });
    } else {
        cx.set_global(ComponentSizeSettings { size });
    }
    cx.refresh_windows();
}
