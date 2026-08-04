//! 只包含图标的 Button 组件。

use crate::{
    button::{self, ClickHandler},
    focus::{self, FocusHandler},
    icon::{Icon, IconSource, IntoIconSource},
    size::{ComponentSize, component_size},
    theme,
    tooltip::{self, Tooltip, TooltipPlacement},
    traits::{Clickable, Disableable, Focusable, Sizable},
};
use gpui::{
    App, ClickEvent, Context, CursorStyle, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};
use std::rc::Rc;
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
    size: Option<ComponentSize>,
    icon_color: Option<Hsla>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
    cursor_style: Option<CursorStyle>,
    tooltip: Option<Tooltip>,
    tooltip_placement: TooltipPlacement,
    aria_description: Option<SharedString>,
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
            on_focus: None,
            on_blur: None,
            cursor_style: None,
            tooltip: None,
            tooltip_placement: TooltipPlacement::default(),
            aria_description: None,
        }
    }

    /// 设置辅助技术朗读的名称。
    ///
    /// 该文本不会显示在界面上。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 设置纯文本 Tooltip。
    ///
    /// Tooltip 不能替代纯图标按钮必需的可访问名称，也不会自动成为可访问描述。
    /// 既可传入字符串沿用自动触发，也可传入 [`Tooltip`] 配置完整显示行为。
    pub fn tooltip(mut self, tooltip: impl Into<Tooltip>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// 设置 Tooltip 相对 IconButton 的优先位置。
    ///
    /// 默认是 [`TooltipPlacement::Bottom`]；视口空间不足时仍会自动翻转或平移。
    pub fn tooltip_placement(mut self, placement: TooltipPlacement) -> Self {
        self.tooltip_placement = placement;
        self
    }

    /// 设置辅助技术朗读的补充描述。
    pub fn aria_description(mut self, description: impl Into<SharedString>) -> Self {
        self.aria_description = Some(description.into());
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
    /// 未调用时在渲染阶段读取当前全局 [`ComponentSize`]，全局值默认是
    /// [`ComponentSize::Md`]。
    pub fn size(mut self, size: ComponentSize) -> Self {
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

    /// 设置可用状态下的鼠标光标。
    ///
    /// disabled 状态会优先显示不可操作光标，不会被此设置绕过。
    pub fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.cursor_style = Some(cursor_style);
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

    /// 注册组件实际获得焦点时调用的回调。
    pub fn on_focus(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_focus = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的聚焦回调。
    pub fn on_focus_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        Focusable::on_focus_in(self, cx, handler)
    }

    /// 注册组件实际失去焦点时调用的回调。
    pub fn on_blur(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_blur = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的失焦回调。
    pub fn on_blur_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        Focusable::on_blur_in(self, cx, handler)
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

    pub(crate) fn resolved_size(&self, cx: &App) -> ComponentSize {
        self.size.unwrap_or_else(|| component_size(cx))
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

    #[cfg(test)]
    pub(crate) fn explicit_size(&self) -> Option<ComponentSize> {
        self.size
    }

    #[cfg(test)]
    pub(crate) fn cursor_style_value(&self) -> Option<CursorStyle> {
        self.cursor_style
    }

    #[cfg(test)]
    pub(crate) fn tooltip_text(&self) -> Option<&SharedString> {
        self.tooltip.as_ref().map(Tooltip::text_value)
    }

    #[cfg(test)]
    pub(crate) fn tooltip_value(&self) -> Option<&Tooltip> {
        self.tooltip.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn tooltip_placement_value(&self) -> TooltipPlacement {
        self.tooltip_placement
    }

    #[cfg(test)]
    pub(crate) fn aria_description_text(&self) -> Option<&SharedString> {
        self.aria_description.as_ref()
    }
}

impl Clickable for IconButton {
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        IconButton::on_click(self, handler)
    }

    fn cursor_style(self, cursor_style: CursorStyle) -> Self {
        IconButton::cursor_style(self, cursor_style)
    }
}

impl Disableable for IconButton {
    fn disabled(self, disabled: bool) -> Self {
        IconButton::disabled(self, disabled)
    }
}

impl Focusable for IconButton {
    fn on_focus(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        IconButton::on_focus(self, handler)
    }

    fn on_blur(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        IconButton::on_blur(self, handler)
    }
}

impl Sizable for IconButton {
    fn size(self, size: ComponentSize) -> Self {
        IconButton::size(self, size)
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_state = focus::state_for(
            &self.id,
            !self.disabled,
            self.on_focus.clone(),
            self.on_blur.clone(),
            window,
            cx,
        );
        let tooltip_state = self.tooltip.clone().map(|tooltip| {
            tooltip::state_for(
                &self.id,
                tooltip,
                self.tooltip_placement,
                &focus_state,
                window,
                cx,
            )
        });
        let theme = theme::current_theme(window, cx);
        let variant = self.resolved_variant().token_key();
        let size = theme
            .button_size(self.resolved_size(cx).token_key())
            .expect("Vektra 默认 Button size token 必须通过测试保持有效");
        let states = button::ResolvedButtonStates::new(&theme, variant);
        let visible = if self.disabled {
            states.disabled
        } else {
            states.normal
        };

        let on_click = self.on_click.clone().filter(|_| !self.disabled);
        let icon_color = self.resolved_icon_color(visible);

        let element = button::apply_interaction(
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
                .when_some(self.aria_description, |this, description| {
                    this.aria_description(description)
                })
                .child(Icon::new(self.icon).size(size.icon_size).color(icon_color)),
            self.disabled,
            on_click,
            self.cursor_style,
            states,
            theme.button.focus_width,
            false,
        );
        let element = focus::attach_interaction(element, &focus_state, !self.disabled, cx);

        if let Some(state) = tooltip_state {
            let element = tooltip::attach_interaction(element, &state);
            tooltip::attach_prepaint_listener(element, state)
        } else {
            tooltip::into_any(element)
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/icon_button.rs"]
mod tests;
