//! Button 组件。

use crate::{
    icon::{Icon, IconSource, IntoIconSource},
    theme,
    traits::{Clickable, Disableable},
};
use gpui::{
    App, ClickEvent, Context, CursorStyle, DefiniteLength, ElementId, IntoElement, KeyDownEvent,
    KeyUpEvent, KeyboardButton, KeyboardClickEvent, Modifiers, MouseButton, ParentElement,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, div, relative,
};
use gpui::{InteractiveElement, prelude::FluentBuilder};
use std::rc::Rc;
use unicode_script::{Script, UnicodeScript};
use vektra_theme::{ButtonStateTokens, ResolvedTheme};

pub(crate) type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Button 的视觉语义变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// 主要操作，默认变体。
    Primary,
    /// 带边框的次要操作。
    Outline,
    /// 背景透明、hover 时显示反馈的轻量按钮。
    Ghost,
    /// 危险或不可逆操作。
    Destructive,
    /// 次要实体按钮。
    Secondary,
    /// 文本链接外观，但无障碍角色仍是 Button。
    Link,
}

impl ButtonVariant {
    pub(crate) const fn token_key(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
            Self::Destructive => "destructive",
            Self::Secondary => "secondary",
            Self::Link => "link",
        }
    }

    pub(crate) const fn underlines_on_hover(self) -> bool {
        matches!(self, Self::Link)
    }
}

/// Button 的尺寸。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    /// 24px 高度。
    Xs,
    /// 32px 高度。
    Sm,
    /// 36px 高度，默认尺寸。
    Md,
    /// 40px 高度。
    Lg,
}

impl ButtonSize {
    pub(crate) const fn token_key(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ButtonWidth {
    Fixed(DefiniteLength),
    Full,
}

/// Vektra Button。
///
/// Button 是普通 GPUI component/element，不需要 Vektra `init` 或 Vektra 根容器。
/// 可选的前置/后置图标只保存 `AssetSource` 路径，不读取 SVG 文件。
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    display_label: SharedString,
    variant: Option<ButtonVariant>,
    size: Option<ButtonSize>,
    width: Option<ButtonWidth>,
    start_icon: Option<IconSource>,
    end_icon: Option<IconSource>,
    disabled: bool,
    auto_insert_space: Option<bool>,
    on_click: Option<ClickHandler>,
}

impl Button {
    /// 创建一个带稳定 `ElementId` 的 Button。
    ///
    /// `id` 必填，用于 GPUI 的交互状态、焦点和测试定位。
    pub fn new(id: impl Into<ElementId>) -> Self {
        let label = SharedString::default();
        Self {
            id: id.into(),
            label: label.clone(),
            display_label: label,
            variant: None,
            size: None,
            width: None,
            start_icon: None,
            end_icon: None,
            disabled: false,
            auto_insert_space: None,
            on_click: None,
        }
    }

    /// 设置文本 label。
    ///
    /// 无障碍名称始终使用原始 label；中文自动空格只影响视觉显示文本。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self.recompute_display_label();
        self
    }

    /// 设置 Button 视觉变体。
    ///
    /// 未调用时解析为 `ButtonVariant::Primary`。
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    /// 设置 Button 尺寸。
    ///
    /// 未调用时解析为 `ButtonSize::Md`。
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = Some(size);
        self
    }

    /// 设置显式宽度。
    ///
    /// 接受 GPUI 当前版本的 `DefiniteLength`，例如 `gpui::px(200.)`。
    /// 如果视觉宽度过窄，文本会被安全截断，无障碍名称仍保留完整原始 label。
    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(ButtonWidth::Fixed(width.into()));
        self
    }

    /// 让 Button 填满父布局提供的可用宽度。
    ///
    /// 与 `width(...)` 同属一个宽度状态，后调用者生效。
    pub fn full_width(mut self) -> Self {
        self.width = Some(ButtonWidth::Full);
        self
    }

    /// 设置前置装饰图标。
    ///
    /// 后一次调用会覆盖前一次调用。图标尺寸和图标与文字之间的间距由主题中当前
    /// `ButtonSize` 的 token 决定，调用方不能单独为插槽图标指定像素尺寸。
    pub fn start_icon(mut self, icon: impl IntoIconSource) -> Self {
        self.start_icon = Some(icon.into_icon_source());
        self
    }

    /// 设置后置装饰图标。
    ///
    /// 后一次调用会覆盖前一次调用。图标不会产生额外无障碍名称。
    pub fn end_icon(mut self, icon: impl IntoIconSource) -> Self {
        self.end_icon = Some(icon.into_icon_source());
        self
    }

    /// 设置 disabled 状态。
    ///
    /// disabled 时鼠标和键盘激活都不会触发回调。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 控制两个汉字 label 的视觉自动空格。
    ///
    /// 默认开启；只在 label 恰好由两个 Unicode Han 字符组成时插入普通空格，
    /// 且不会修改原始 label 或无障碍名称。
    pub fn auto_insert_space(mut self, enabled: bool) -> Self {
        self.auto_insert_space = Some(enabled);
        self.recompute_display_label();
        self
    }

    /// 注册激活回调。
    ///
    /// 鼠标点击、聚焦后的 Enter 和 Space 都会通过同一个三参数回调触发。
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的激活回调。
    ///
    /// 这是 `Clickable::on_click_in` 的 inherent forwarding，调用方只导入
    /// `Button` 时也可以使用。内部通过 `Context::listener` 保留 GPUI 原有的弱
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

    /// 返回用户传入的原始 label。
    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    /// 返回视觉显示 label。
    pub fn display_label(&self) -> &SharedString {
        &self.display_label
    }

    pub(crate) fn resolved_variant(&self) -> ButtonVariant {
        self.variant.unwrap_or(ButtonVariant::Primary)
    }

    pub(crate) fn resolved_size(&self) -> ButtonSize {
        self.size.unwrap_or(ButtonSize::Md)
    }

    #[cfg(test)]
    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[cfg(test)]
    pub(crate) fn start_icon_source(&self) -> Option<&IconSource> {
        self.start_icon.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn end_icon_source(&self) -> Option<&IconSource> {
        self.end_icon.as_ref()
    }

    fn recompute_display_label(&mut self) {
        if self.auto_insert_space.unwrap_or(true) {
            self.display_label = auto_spaced_label(&self.label);
        } else {
            self.display_label = self.label.clone();
        }
    }
}

impl Clickable for Button {
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        Button::on_click(self, handler)
    }
}

impl Disableable for Button {
    fn disabled(self, disabled: bool) -> Self {
        Button::disabled(self, disabled)
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = theme::current_theme(window, cx);
        let variant = self.resolved_variant();
        let underlines_on_hover = variant.underlines_on_hover();
        let variant = variant.token_key();
        let size = theme
            .button_size(self.resolved_size().token_key())
            .expect("Vektra 默认 Button size token 必须通过测试保持有效");
        let states = ResolvedButtonStates::new(&theme, variant);
        let visible = if self.disabled {
            states.disabled
        } else {
            states.normal
        };

        let on_click = self.on_click.clone().filter(|_| !self.disabled);

        let content = div()
            .flex()
            .items_center()
            .justify_center()
            .gap(size.content_gap)
            .min_w_0()
            .when_some(self.start_icon, |this, icon| {
                this.child(Icon::new(icon).size(size.icon_size))
            })
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(self.display_label.clone()),
            )
            .when_some(self.end_icon, |this, icon| {
                this.child(Icon::new(icon).size(size.icon_size))
            });

        apply_interaction(
            div()
                .id(self.id.clone())
                .role(Role::Button)
                .aria_label(self.label.clone())
                .flex()
                .items_center()
                .justify_center()
                .flex_none()
                .h(size.height)
                .px(size.padding_x)
                .rounded(size.radius)
                .border(theme.button.border_width)
                .border_color(visible.border)
                .bg(visible.background)
                .text_color(visible.foreground)
                .text_size(size.font_size)
                .line_height(size.height)
                .whitespace_nowrap()
                .overflow_hidden()
                .when_some(self.width, apply_width)
                .child(content),
            self.disabled,
            on_click,
            states,
            theme.button.focus_width,
            underlines_on_hover,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedButtonStates {
    pub(crate) normal: ButtonStateTokens,
    pub(crate) hover: ButtonStateTokens,
    pub(crate) pressed: ButtonStateTokens,
    pub(crate) focused: ButtonStateTokens,
    pub(crate) disabled: ButtonStateTokens,
}

impl ResolvedButtonStates {
    pub(crate) fn new(theme: &ResolvedTheme, variant: &str) -> Self {
        Self {
            normal: theme
                .button_state(variant, "normal")
                .expect("Vektra 默认 Button normal token 必须通过测试保持有效"),
            hover: theme
                .button_state(variant, "hover")
                .expect("Vektra 默认 Button hover token 必须通过测试保持有效"),
            pressed: theme
                .button_state(variant, "pressed")
                .expect("Vektra 默认 Button pressed token 必须通过测试保持有效"),
            focused: theme
                .button_state(variant, "focus-visible")
                .expect("Vektra 默认 Button focus token 必须通过测试保持有效"),
            disabled: theme
                .button_state(variant, "disabled")
                .expect("Vektra 默认 Button disabled token 必须通过测试保持有效"),
        }
    }
}

pub(crate) fn apply_interaction(
    element: gpui::Stateful<gpui::Div>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    states: ResolvedButtonStates,
    focus_width: gpui::Pixels,
    underline_on_hover: bool,
) -> gpui::Stateful<gpui::Div> {
    let on_key_enter = on_click.clone();
    let on_key_space = on_click.clone();

    element
        .when(disabled, |this| {
            this.cursor(CursorStyle::OperationNotAllowed)
        })
        .when(!disabled, |this| {
            this.cursor(CursorStyle::PointingHand)
                .tab_index(0)
                .hover(move |style| {
                    let style = style
                        .bg(states.hover.background)
                        .border_color(states.hover.border)
                        .text_color(states.hover.foreground);

                    if underline_on_hover {
                        style.underline()
                    } else {
                        style
                    }
                })
                .active(move |style| {
                    style
                        .bg(states.pressed.background)
                        .border_color(states.pressed.border)
                        .text_color(states.pressed.foreground)
                })
                .focus_visible(move |style| {
                    style
                        .border(focus_width)
                        .border_color(states.focused.border)
                        .text_color(states.focused.foreground)
                })
        })
        .when_some(on_click, |this, handler| {
            this.on_mouse_down(MouseButton::Left, |_, window, _| {
                window.prevent_default();
            })
            .on_click(move |event, window, cx| {
                cx.stop_propagation();
                (handler)(event, window, cx);
            })
        })
        .when_some(on_key_enter, |this, handler| {
            this.on_key_down(move |event, window, cx| {
                if is_plain_key(event, "enter") && !event.is_held {
                    window.prevent_default();
                    cx.stop_propagation();
                    (handler)(&keyboard_click(KeyboardButton::Enter), window, cx);
                }
            })
        })
        .when_some(on_key_space, |this, handler| {
            this.on_key_down(|event, window, cx| {
                if is_plain_key(event, "space") {
                    window.prevent_default();
                    cx.stop_propagation();
                }
            })
            .on_key_up(move |event, window, cx| {
                if is_plain_key_up(event, "space") {
                    window.prevent_default();
                    cx.stop_propagation();
                    (handler)(&keyboard_click(KeyboardButton::Space), window, cx);
                }
            })
        })
}

fn apply_width(
    element: gpui::Stateful<gpui::Div>,
    width: ButtonWidth,
) -> gpui::Stateful<gpui::Div> {
    match width {
        ButtonWidth::Fixed(width) => element.w(width).justify_center().text_center(),
        ButtonWidth::Full => element.w(relative(1.)).justify_center().text_center(),
    }
}

pub(crate) fn keyboard_click(button: KeyboardButton) -> ClickEvent {
    ClickEvent::Keyboard(KeyboardClickEvent {
        button,
        bounds: Default::default(),
    })
}

fn is_plain_key(event: &KeyDownEvent, key: &str) -> bool {
    event.keystroke.key == key && event.keystroke.modifiers == Modifiers::none()
}

fn is_plain_key_up(event: &KeyUpEvent, key: &str) -> bool {
    event.keystroke.key == key && event.keystroke.modifiers == Modifiers::none()
}

fn auto_spaced_label(label: &SharedString) -> SharedString {
    let mut chars = label.chars();
    let Some(first) = chars.next() else {
        return label.clone();
    };
    let Some(second) = chars.next() else {
        return label.clone();
    };
    if chars.next().is_some() || first.is_whitespace() || second.is_whitespace() {
        return label.clone();
    }
    if first.script() == Script::Han && second.script() == Script::Han {
        SharedString::from(format!("{first} {second}"))
    } else {
        label.clone()
    }
}

#[cfg(test)]
#[path = "../tests/unit/button.rs"]
mod tests;
