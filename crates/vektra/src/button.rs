//! Button 组件。

use crate::{
    focus::{self, FocusHandler},
    icon::{Icon, IconSource, IntoIconSource},
    size::{ComponentSize, component_size},
    theme,
    tooltip::{self, Tooltip, TooltipPlacement},
    traits::{Clickable, Disableable, Focusable, Sizable},
};
use gpui::{
    Animation, AnimationExt, App, ClickEvent, Context, CursorStyle, DefiniteLength, ElementId,
    IntoElement, KeyDownEvent, KeyUpEvent, Modifiers, MouseButton, ParentElement, RenderOnce, Role,
    SharedString, StatefulInteractiveElement, Styled, Toggled, Transformation, Window, div,
    percentage, relative, svg,
};
use gpui::{InteractiveElement, prelude::FluentBuilder};
use std::{rc::Rc, time::Duration};
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum ButtonWidth {
    Fixed(DefiniteLength),
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ButtonActivity {
    Idle,
    Loading,
    Progress(f32),
}

impl ButtonActivity {
    const fn is_busy(self) -> bool {
        !matches!(self, Self::Idle)
    }

    const fn progress(self) -> Option<f32> {
        match self {
            Self::Progress(value) => Some(value),
            Self::Idle | Self::Loading => None,
        }
    }
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
    size: Option<ComponentSize>,
    width: Option<ButtonWidth>,
    start_icon: Option<IconSource>,
    end_icon: Option<IconSource>,
    disabled: bool,
    activity: ButtonActivity,
    selected: Option<bool>,
    auto_insert_space: Option<bool>,
    on_click: Option<ClickHandler>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
    cursor_style: Option<CursorStyle>,
    tooltip: Option<Tooltip>,
    tooltip_placement: TooltipPlacement,
    aria_description: Option<SharedString>,
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
            activity: ButtonActivity::Idle,
            selected: None,
            auto_insert_space: None,
            on_click: None,
            on_focus: None,
            on_blur: None,
            cursor_style: None,
            tooltip: None,
            tooltip_placement: TooltipPlacement::default(),
            aria_description: None,
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
    /// 未调用时在渲染阶段读取当前全局 [`ComponentSize`]，全局值默认是
    /// [`ComponentSize::Md`]。
    pub fn size(mut self, size: ComponentSize) -> Self {
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
    /// `ComponentSize` 的 token 决定，调用方不能单独为插槽图标指定像素尺寸。
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

    /// 设置不确定 loading 状态。
    ///
    /// `true` 会用旋转指示器替代前置图标，并阻止鼠标、Enter 和 Space 激活；`false`
    /// 恢复空闲状态。loading 与 `progress(...)` 互斥，连续调用时后调用者生效。
    /// 异步任务的启动、取消和完成仍由宿主应用负责。
    pub fn loading(mut self, loading: bool) -> Self {
        self.activity = if loading {
            ButtonActivity::Loading
        } else {
            ButtonActivity::Idle
        };
        self
    }

    /// 设置 0.0～1.0 的确定进度状态。
    ///
    /// 有限值会夹取到该范围，正负无穷分别归一为 1 和 0，NaN 归一为 0。进度状态
    /// 保留 label 和两侧图标，并阻止鼠标、Enter 和 Space 激活。与 `loading(...)`
    /// 连续调用时后调用者生效。
    pub fn progress(mut self, progress: f32) -> Self {
        self.activity = ButtonActivity::Progress(normalize_progress(progress));
        self
    }

    /// 设置受控 selected/toggle 状态。
    ///
    /// 调用该方法后 Button 会通过 `aria-toggled` 暴露显式 toggle 语义；未调用时仍是
    /// 普通 Button。selected 不会在点击后自行翻转，宿主应用应在回调中更新状态。
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
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

    /// 设置纯文本 Tooltip。
    ///
    /// Tooltip 不会自动成为无障碍名称或描述；需要补充朗读信息时请显式调用
    /// [`Self::aria_description`]。既可传入字符串沿用自动 hover/键盘焦点触发，也可
    /// 传入 [`Tooltip`] 配置打开状态、箭头、颜色和动画。后一次调用覆盖前一次调用。
    pub fn tooltip(mut self, tooltip: impl Into<Tooltip>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// 设置 Tooltip 相对 Button 的优先位置。
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

    /// 设置可用状态下的鼠标光标。
    ///
    /// disabled、loading 和 progress 会优先显示自身状态光标，不会被此设置绕过。
    pub fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.cursor_style = Some(cursor_style);
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

    /// 注册组件实际获得焦点时调用的回调。
    ///
    /// 重新渲染或 Button 状态 builder 变化不会自行触发该回调。
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

    pub(crate) fn resolved_size(&self, cx: &App) -> ComponentSize {
        self.size.unwrap_or_else(|| component_size(cx))
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
    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[cfg(test)]
    const fn activity(&self) -> ButtonActivity {
        self.activity
    }

    #[cfg(test)]
    const fn selected_state(&self) -> Option<bool> {
        self.selected
    }

    #[cfg(test)]
    fn activity_id(&self) -> ElementId {
        (self.id.clone(), "activity").into()
    }

    #[cfg(test)]
    pub(crate) fn start_icon_source(&self) -> Option<&IconSource> {
        self.start_icon.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn end_icon_source(&self) -> Option<&IconSource> {
        self.end_icon.as_ref()
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

    fn cursor_style(self, cursor_style: CursorStyle) -> Self {
        Button::cursor_style(self, cursor_style)
    }
}

impl Disableable for Button {
    fn disabled(self, disabled: bool) -> Self {
        Button::disabled(self, disabled)
    }
}

impl Focusable for Button {
    fn on_focus(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Button::on_focus(self, handler)
    }

    fn on_blur(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Button::on_blur(self, handler)
    }
}

impl Sizable for Button {
    fn size(self, size: ComponentSize) -> Self {
        Button::size(self, size)
    }
}

impl RenderOnce for Button {
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
        let variant = self.resolved_variant();
        let underlines_on_hover = variant.underlines_on_hover();
        let variant = variant.token_key();
        let size = theme
            .button_size(self.resolved_size(cx).token_key())
            .expect("Vektra 默认 Button size token 必须通过测试保持有效");
        let selected = self.selected;
        let states = if selected == Some(true) {
            ResolvedButtonStates::selected(&theme, variant)
        } else {
            ResolvedButtonStates::new(&theme, variant)
        };
        let visible = if self.disabled {
            states.disabled
        } else {
            states.normal
        };
        let busy = self.activity.is_busy();
        let loading = matches!(self.activity, ButtonActivity::Loading);
        let progress = self.activity.progress();
        let activity_id: ElementId = (self.id.clone(), "activity").into();
        let animation_id: ElementId = (self.id.clone(), "activity-animation").into();

        let on_click = self.on_click.clone().filter(|_| !self.disabled && !busy);

        let content = div()
            .flex()
            .items_center()
            .justify_center()
            .gap(size.content_gap)
            .min_w_0()
            .when(loading, |this| {
                this.child(
                    div()
                        .id(activity_id.clone())
                        .role(Role::ProgressIndicator)
                        .aria_label(self.label.clone())
                        .size(size.icon_size)
                        .flex_none()
                        .child(
                            svg()
                                .path("components/button/loading.svg")
                                .size(size.icon_size)
                                .text_color(visible.foreground)
                                .with_animation(
                                    animation_id,
                                    Animation::new(Duration::from_millis(900)).repeat(),
                                    |icon, delta| {
                                        icon.with_transformation(Transformation::rotate(
                                            percentage(delta),
                                        ))
                                    },
                                ),
                        ),
                )
            })
            .when(!loading, |this| {
                this.when_some(self.start_icon, |this, icon| {
                    this.child(Icon::new(icon).size(size.icon_size))
                })
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

        let element = div()
            .id(self.id.clone())
            .debug_selector(|| "vektra-button".into())
            .role(Role::Button)
            .aria_label(self.label.clone())
            .when_some(self.aria_description, |this, description| {
                this.aria_description(description)
            })
            .when_some(toggled_state(selected), |this, toggled| {
                this.aria_toggled(toggled)
            })
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
            .relative()
            .when_some(self.width, apply_width)
            .child(content)
            .when(selected == Some(true), |this| {
                this.child(
                    div()
                        .absolute()
                        .top(theme.button.border_width)
                        .right(theme.button.border_width)
                        .bottom(theme.button.border_width)
                        .left(theme.button.border_width)
                        .rounded(size.radius)
                        .border(theme.button.border_width)
                        .border_color(visible.border),
                )
            })
            .when_some(progress, |this, progress| {
                this.child(
                    div()
                        .id(activity_id)
                        .role(Role::ProgressIndicator)
                        .aria_label(self.label.clone())
                        .aria_min_numeric_value(0.)
                        .aria_max_numeric_value(100.)
                        .aria_numeric_value(progress_percent(progress))
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(theme.button.focus_width)
                        .bg(visible.foreground.opacity(0.28))
                        .child(div().h_full().w(relative(progress)).bg(visible.foreground)),
                )
            });

        let element = apply_interaction_with_activity(
            element,
            ButtonInteraction {
                disabled: self.disabled,
                busy,
                on_click,
                cursor_style: self.cursor_style,
                states,
                focus_width: theme.button.focus_width,
                underline_on_hover: underlines_on_hover,
            },
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

    pub(crate) fn selected(theme: &ResolvedTheme, variant: &str) -> Self {
        let base = Self::new(theme, variant);
        Self {
            normal: selected_state(theme, variant, "normal").unwrap_or(base.pressed),
            hover: selected_state(theme, variant, "hover").unwrap_or(base.hover),
            pressed: selected_state(theme, variant, "pressed").unwrap_or(base.pressed),
            focused: selected_state(theme, variant, "focus-visible").unwrap_or(base.focused),
            disabled: selected_state(theme, variant, "disabled").unwrap_or(base.disabled),
        }
    }
}

fn selected_state(theme: &ResolvedTheme, variant: &str, state: &str) -> Option<ButtonStateTokens> {
    theme
        .button_selected_state(variant, state)
        .expect("Vektra Button selected token 必须完整且类型正确")
}

pub(crate) fn apply_interaction(
    element: gpui::Stateful<gpui::Div>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    cursor_style: Option<CursorStyle>,
    states: ResolvedButtonStates,
    focus_width: gpui::Pixels,
    underline_on_hover: bool,
) -> gpui::Stateful<gpui::Div> {
    apply_interaction_with_activity(
        element,
        ButtonInteraction {
            disabled,
            busy: false,
            on_click,
            cursor_style,
            states,
            focus_width,
            underline_on_hover,
        },
    )
}

pub(crate) struct ButtonInteraction {
    disabled: bool,
    busy: bool,
    on_click: Option<ClickHandler>,
    cursor_style: Option<CursorStyle>,
    states: ResolvedButtonStates,
    focus_width: gpui::Pixels,
    underline_on_hover: bool,
}

fn apply_interaction_with_activity(
    element: gpui::Stateful<gpui::Div>,
    interaction: ButtonInteraction,
) -> gpui::Stateful<gpui::Div> {
    let ButtonInteraction {
        disabled,
        busy,
        on_click,
        cursor_style,
        states,
        focus_width,
        underline_on_hover,
    } = interaction;
    element
        .when(disabled, |this| {
            this.cursor(CursorStyle::OperationNotAllowed)
        })
        .when(!disabled, |this| {
            this.cursor(resolved_cursor_style(disabled, busy, cursor_style))
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
                .focus_visible(move |style| {
                    style
                        .border(focus_width)
                        .border_color(states.focused.border)
                        .text_color(states.focused.foreground)
                })
        })
        .when(!disabled && !busy, |this| {
            this.active(move |style| {
                style
                    .bg(states.pressed.background)
                    .border_color(states.pressed.border)
                    .text_color(states.pressed.foreground)
            })
        })
        .when(busy, |this| {
            this.on_mouse_down(MouseButton::Left, |_, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .on_click(|_, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .on_key_down(|event, window, cx| {
                if is_plain_key(event, "enter") || is_plain_key(event, "space") {
                    window.prevent_default();
                    cx.stop_propagation();
                }
            })
            .on_key_up(|event, window, cx| {
                if is_plain_key_up(event, "enter") || is_plain_key_up(event, "space") {
                    window.prevent_default();
                    cx.stop_propagation();
                }
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
}

pub(crate) fn resolved_cursor_style(
    disabled: bool,
    busy: bool,
    cursor_style: Option<CursorStyle>,
) -> CursorStyle {
    if disabled {
        CursorStyle::OperationNotAllowed
    } else if busy {
        CursorStyle::Arrow
    } else {
        cursor_style.unwrap_or(CursorStyle::PointingHand)
    }
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

fn normalize_progress(progress: f32) -> f32 {
    if progress.is_nan() {
        0.
    } else {
        progress.clamp(0., 1.)
    }
}

pub(crate) fn toggled_state(selected: Option<bool>) -> Option<Toggled> {
    selected.map(|selected| {
        if selected {
            Toggled::True
        } else {
            Toggled::False
        }
    })
}

fn progress_percent(progress: f32) -> f64 {
    f64::from(progress * 100.)
}

#[cfg(test)]
#[path = "../tests/unit/button.rs"]
mod tests;
