//! 只包含图标的 Button 组件。

use crate::{
    ButtonSize,
    button::{self, ClickHandler},
    icon::{Icon, IconSource, IntoIconSource},
    theme,
    traits::{Clickable, Disableable},
};
use gpui::{
    App, ClickEvent, Context, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};
use vektra_theme::ButtonStateTokens;

/// IconButton 的视觉语义变体。
///
/// 该 enum 与 `ButtonVariant` 独立，故意不提供 `Link`，以保证纯图标按钮只能表达
/// 合法状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconButtonVariant {
    /// 主要操作，未显式指定时使用。
    Primary,
    /// 带边框的次要操作。
    Outline,
    /// 背景透明、hover 时显示反馈的轻量按钮。
    Ghost,
    /// 危险或不可逆操作。
    Destructive,
    /// 次要实体按钮。
    Secondary,
}

impl IconButtonVariant {
    pub(crate) const fn token_key(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
            Self::Destructive => "destructive",
            Self::Secondary => "secondary",
        }
    }
}

/// 固定正方形、只显示图标的按钮。
///
/// `IconButton` 与 `Button` 使用相同的鼠标、键盘和 focus-visible 行为。因为没有
/// 可见文字，推荐始终调用 `aria_label(...)` 为辅助技术提供名称。
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: IconSource,
    aria_label: Option<SharedString>,
    variant: Option<IconButtonVariant>,
    size: Option<ButtonSize>,
    icon_color: Option<Hsla>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl IconButton {
    /// 创建一个带稳定 `ElementId` 和图标路径的 IconButton。
    ///
    /// `aria_label` 不是构造器必填项，但没有可见文字的按钮应设置它。
    pub fn new(id: impl Into<ElementId>, icon: impl IntoIconSource) -> Self {
        Self {
            id: id.into(),
            icon: icon.into_icon_source(),
            aria_label: None,
            variant: None,
            size: None,
            icon_color: None,
            disabled: false,
            on_click: None,
        }
    }

    /// 设置辅助技术朗读的名称。
    ///
    /// 该文本不会显示在界面上。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 设置 IconButton 视觉变体。
    ///
    /// 未调用时在渲染阶段解析为 `IconButtonVariant::Primary`。
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    /// 设置 IconButton 尺寸。
    ///
    /// 未调用时在渲染阶段解析为 `ButtonSize::Md`。
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = Some(size);
        self
    }

    /// 设置内部图标颜色。
    ///
    /// 该颜色只改变图标，不改变按钮背景、边框或焦点环。未指定时，图标继续使用
    /// 当前 variant/state 的前景色；disabled 状态会优先使用主题 disabled
    /// foreground，避免自定义颜色破坏禁用态表达。
    pub fn icon_color(mut self, color: Hsla) -> Self {
        self.icon_color = Some(color);
        self
    }

    /// 设置 disabled 状态。
    ///
    /// disabled 时鼠标和键盘激活都不会触发回调。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 注册激活回调。
    ///
    /// 鼠标点击、聚焦后的 Enter 和 Space 都会通过同一个三参数回调触发。
    pub fn on_click(
        mut self,
        handler: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(std::rc::Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的激活回调。
    ///
    /// 这是 `Clickable::on_click_in` 的 inherent forwarding，调用方只导入
    /// `IconButton` 时也可以使用。内部通过 `Context::listener` 保留 GPUI 原有的弱
    /// Entity 生命周期语义；handler 可以访问宿主 `&mut T`、`ClickEvent`、`Window`
    /// 和 `Context<T>`，并可在修改状态后调用 `cx.notify()`。
    pub fn on_click_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &ClickEvent, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        <Self as Clickable>::on_click_in(self, cx, handler)
    }

    /// 返回稳定 ElementId。
    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub(crate) fn resolved_variant(&self) -> IconButtonVariant {
        self.variant.unwrap_or(IconButtonVariant::Primary)
    }

    pub(crate) fn resolved_size(&self) -> ButtonSize {
        self.size.unwrap_or(ButtonSize::Md)
    }

    pub(crate) fn resolved_icon_color(&self, visible: ButtonStateTokens) -> Hsla {
        if self.disabled {
            visible.foreground
        } else {
            self.icon_color.unwrap_or(visible.foreground)
        }
    }

    #[cfg(test)]
    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[cfg(test)]
    pub(crate) fn aria_label_text(&self) -> Option<&SharedString> {
        self.aria_label.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn icon_color_value(&self) -> Option<Hsla> {
        self.icon_color
    }
}

impl Clickable for IconButton {
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        IconButton::on_click(self, handler)
    }
}

impl Disableable for IconButton {
    fn disabled(self, disabled: bool) -> Self {
        IconButton::disabled(self, disabled)
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = theme::current_theme(window, cx);
        let variant = self.resolved_variant().token_key();
        let size = theme
            .button_size(self.resolved_size().token_key())
            .expect("Vektra 默认 Button size token 必须通过测试保持有效");
        let states = button::ResolvedButtonStates::new(&theme, variant);
        let visible = if self.disabled {
            states.disabled
        } else {
            states.normal
        };

        let on_click = self.on_click.clone().filter(|_| !self.disabled);
        let icon_color = self.resolved_icon_color(visible);

        button::apply_interaction(
            div()
                .id(self.id.clone())
                .role(Role::Button)
                .flex()
                .items_center()
                .justify_center()
                .flex_none()
                .w(size.height)
                .h(size.height)
                .rounded(size.radius)
                .border(theme.button.border_width)
                .border_color(visible.border)
                .bg(visible.background)
                .text_color(visible.foreground)
                .when_some(self.aria_label, |this, label| this.aria_label(label))
                .child(Icon::new(self.icon).size(size.icon_size).color(icon_color)),
            self.disabled,
            on_click,
            states,
            theme.button.focus_width,
            false,
        )
    }
}

#[cfg(test)]
#[path = "../tests/unit/icon_button.rs"]
mod tests;
