//! 图标路径和纯展示 Icon 组件。
//!
//! 图标 API 只描述 GPUI `AssetSource` 中的相对路径，不负责注册资源或读取文件。

use crate::theme;
use gpui::{App, Hsla, IntoElement, Pixels, RenderOnce, SharedString, Styled, Window, svg};

#[cfg(feature = "icons")]
pub use vektra_icons::IconName;

/// 一个 GPUI `AssetSource` 中可加载的 SVG 图标路径。
///
/// `IconSource` 不在构造时执行文件 I/O，也不解析 SVG。路径会在渲染时交给
/// GPUI 原生 `svg().path(...)`，由 GPUI 的资源系统和 SVG 缓存处理。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconSource {
    path: SharedString,
}

impl IconSource {
    /// 通过 `AssetSource` 相对路径创建图标来源。
    ///
    /// Vektra 约定自带图标和推荐的用户图标路径为 `icons/**/*.svg`。
    pub fn asset(path: impl Into<SharedString>) -> Self {
        Self { path: path.into() }
    }

    /// 返回图标在 GPUI `AssetSource` 中使用的相对路径。
    pub fn path(&self) -> &str {
        self.path.as_ref()
    }

    pub(crate) fn shared_path(&self) -> SharedString {
        self.path.clone()
    }
}

/// 将轻量图标名称或自定义类型转换为 `IconSource`。
///
/// 用户可以为自己的本地 enum 或 ZST 实现该 trait，使 Vektra 的 `Icon`、
/// `Button::start_icon`、`Button::end_icon` 和 `IconButton` 复用同一图标契约。
pub trait IntoIconSource {
    /// 转换为不会立即读取资源的图标路径。
    fn into_icon_source(self) -> IconSource;
}

impl IntoIconSource for IconSource {
    fn into_icon_source(self) -> IconSource {
        self
    }
}

#[cfg(feature = "icons")]
impl IntoIconSource for IconName {
    fn into_icon_source(self) -> IconSource {
        IconSource::asset(self.path())
    }
}

/// 纯展示 SVG 图标。
///
/// `Icon` 默认是装饰性图形，不创建点击区域，也不声明 Button 角色。未显式设置
/// 颜色时，Icon 会使用渲染上下文中继承的文字颜色；SVG 的 `currentColor` 最终
/// 由 GPUI 的 `text_color` 着色。
#[derive(IntoElement)]
pub struct Icon {
    source: IconSource,
    size: Option<Pixels>,
    color: Option<Hsla>,
}

impl Icon {
    /// 创建图标组件。
    ///
    /// 参数可以是 Vektra 内置类型化图标，也可以是 `IconSource::asset(...)` 或用户
    /// 自行实现 `IntoIconSource` 的类型。
    pub fn new(source: impl IntoIconSource) -> Self {
        Self {
            source: source.into_icon_source(),
            size: None,
            color: None,
        }
    }

    /// 设置图标的正方形尺寸。
    ///
    /// 未设置时，渲染阶段使用主题中的 `icon.size.default`，当前默认值为 16px。
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// 设置图标颜色。
    ///
    /// 显式颜色会覆盖父元素或渲染上下文中的前景色。未调用该方法时，Icon 使用
    /// `Window` 当前文字样式中的颜色，并仍通过 GPUI `text_color` 传给 SVG，
    /// 让 `currentColor` 能被正确绘制。
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub(crate) fn resolved_size(&self, window: &Window, cx: &App) -> Pixels {
        self.size
            .unwrap_or_else(|| theme::current_theme(window, cx).icon.default_size)
    }

    pub(crate) fn resolved_color(&self, window: &Window) -> Hsla {
        self.color.unwrap_or_else(|| window.text_style().color)
    }

    #[cfg(test)]
    pub(crate) fn source(&self) -> &IconSource {
        &self.source
    }

    #[cfg(test)]
    pub(crate) fn color_value(&self) -> Option<Hsla> {
        self.color
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let size = self.resolved_size(window, cx);
        let color = self.resolved_color(window);
        svg()
            .path(self.source.shared_path())
            .size(size)
            .flex_none()
            .text_color(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Hsla, ParentElement, Render, TestAppContext, div, px};

    #[derive(Debug, Clone, Copy)]
    enum TestIcon {
        Settings,
    }

    impl IntoIconSource for TestIcon {
        fn into_icon_source(self) -> IconSource {
            match self {
                Self::Settings => IconSource::asset("icons/settings.svg"),
            }
        }
    }

    #[cfg(feature = "icons")]
    #[test]
    fn icon_name_maps_to_stable_path() {
        let source = IconName::Settings.into_icon_source();
        assert_eq!(source.path(), "icons/settings.svg");
    }

    #[test]
    fn custom_asset_path_is_preserved() {
        let source = IconSource::asset("icons/custom.svg");
        assert_eq!(source.path(), "icons/custom.svg");
    }

    #[test]
    fn icon_default_color_is_none() {
        let icon = Icon::new(TestIcon::Settings);
        assert_eq!(icon.color_value(), None);
    }

    #[test]
    fn icon_color_is_preserved() {
        let color = Hsla::red();
        let icon = Icon::new(TestIcon::Settings).color(color);
        assert_eq!(icon.color_value(), Some(color));
    }

    #[gpui::test]
    fn icon_default_size_resolves_from_theme(cx: &mut TestAppContext) {
        let (_view, cx) = cx.add_window_view(|_, _| IconTestView {
            icon: Icon::new(TestIcon::Settings),
        });
        cx.update(|window, cx| {
            let icon = Icon::new(TestIcon::Settings);
            assert_eq!(icon.resolved_size(window, cx), px(16.));
        });
    }

    #[gpui::test]
    fn icon_explicit_size_overrides_default(cx: &mut TestAppContext) {
        let (_view, cx) = cx.add_window_view(|_, _| IconTestView {
            icon: Icon::new(IconSource::asset("icons/custom.svg")).size(px(24.)),
        });
        cx.update(|window, cx| {
            let icon = Icon::new(TestIcon::Settings).size(px(24.));
            assert_eq!(icon.resolved_size(window, cx), px(24.));
        });
    }

    #[gpui::test]
    fn icon_default_color_resolves_from_window_text_style(cx: &mut TestAppContext) {
        let (_view, cx) = cx.add_window_view(|_, _| IconTestView {
            icon: Icon::new(TestIcon::Settings),
        });
        cx.update(|window, _cx| {
            let icon = Icon::new(TestIcon::Settings);
            assert_eq!(icon.resolved_color(window), window.text_style().color);

            let explicit = Hsla::blue();
            let icon = Icon::new(TestIcon::Settings).color(explicit);
            assert_eq!(icon.resolved_color(window), explicit);
        });
    }

    #[gpui::test]
    fn icon_render_path_has_resolved_text_color(cx: &mut TestAppContext) {
        let (_view, cx) = cx.add_window_view(|_, _| IconTestView {
            icon: Icon::new(TestIcon::Settings),
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            let icon = Icon::new(TestIcon::Settings);
            assert_eq!(icon.resolved_color(window), window.text_style().color);
        });
    }

    struct IconTestView {
        icon: Icon,
    }

    impl Render for IconTestView {
        fn render(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child(Icon::new(self.icon.source().clone()))
        }
    }
}
