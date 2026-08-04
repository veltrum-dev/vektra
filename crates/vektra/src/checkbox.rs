//! Checkbox 组件。

use crate::{
    button::{self, ClickHandler},
    focus::{self, FocusHandler},
    icon::{Icon, IconSource, IntoIconSource},
    size::{ComponentSize, component_size},
    theme,
    traits::{Changeable, Disableable, Focusable, Sizable},
};
use gpui::{
    App, Context, CursorStyle, ElementId, InteractiveElement, IntoElement, KeyDownEvent,
    KeyUpEvent, Modifiers, MouseButton, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Toggled, Window, div, prelude::FluentBuilder,
};
use std::rc::Rc;
use vektra_theme::{CheckboxStateTokens, ResolvedTheme};

const DEFAULT_CHECKED_ICON: &str = "components/checkbox/check.svg";
const DEFAULT_INDETERMINATE_ICON: &str = "components/checkbox/minus.svg";

type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// Vektra Checkbox。
///
/// Checkbox 是受控组件，不在内部保存业务 checked 状态。调用方应在每次 render 时通过
/// [`Self::checked`] 传入当前值，并在 [`Self::on_change`] 或 [`Self::on_change_in`]
/// 中更新宿主状态。
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    label: Option<SharedString>,
    size: Option<ComponentSize>,
    cursor_style: Option<CursorStyle>,
    unchecked_icon: Option<IconSource>,
    checked_icon: Option<IconSource>,
    indeterminate_icon: Option<IconSource>,
    icon_indicator: bool,
    aria_label: Option<SharedString>,
    aria_description: Option<SharedString>,
    on_change: Option<ChangeHandler>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
}

impl Checkbox {
    /// 创建一个带稳定 `ElementId` 的 Checkbox。
    ///
    /// 新建后默认 `checked = false`、`indeterminate = false`、`disabled = false`。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            indeterminate: false,
            disabled: false,
            label: None,
            size: None,
            cursor_style: None,
            unchecked_icon: None,
            checked_icon: None,
            indeterminate_icon: None,
            icon_indicator: false,
            aria_label: None,
            aria_description: None,
            on_change: None,
            on_focus: None,
            on_blur: None,
        }
    }

    /// 设置当前受控 checked 值。
    ///
    /// 该值不是初始值；宿主状态变化后应在下一次 render 中继续传入最新值。
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// 设置当前受控部分选中状态。
    ///
    /// `true` 时视觉与无障碍状态优先于 `checked`，用户激活后回调的下一值固定为
    /// `true`。
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// 设置 disabled 状态。
    ///
    /// disabled 时鼠标、触摸和键盘激活都不会触发 `on_change`。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 设置可见文本 label。
    ///
    /// label 与方框共享同一个交互区域，并默认作为可访问名称来源。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// 设置组件级显式尺寸。
    ///
    /// 未调用时在渲染阶段读取当前全局 [`ComponentSize`]。
    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = Some(size);
        self
    }

    /// 设置可用状态下的鼠标光标。
    ///
    /// disabled 状态会优先显示不可操作光标，不会被此设置绕过。
    pub fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.cursor_style = Some(cursor_style);
        self
    }

    /// 设置未选中状态显示的图标。
    ///
    /// 默认未选中状态不显示图标。
    pub fn unchecked_icon(mut self, icon: impl IntoIconSource) -> Self {
        self.unchecked_icon = Some(icon.into_icon_source());
        self
    }

    /// 设置已选中状态显示的图标。
    ///
    /// 未设置时使用 Vektra 内置对勾图标。
    pub fn checked_icon(mut self, icon: impl IntoIconSource) -> Self {
        self.checked_icon = Some(icon.into_icon_source());
        self
    }

    /// 设置部分选中状态显示的图标。
    ///
    /// 未设置时使用 Vektra 内置横线图标。
    pub fn indeterminate_icon(mut self, icon: impl IntoIconSource) -> Self {
        self.indeterminate_icon = Some(icon.into_icon_source());
        self
    }

    /// 使用一对状态图标替代默认方框指示器。
    ///
    /// 未选中时显示 `unchecked_icon`，选中时显示 `checked_icon`；部分选中仍使用
    /// [`Self::indeterminate_icon`]，未设置时回退到内置横线。该设置不会移除可见
    /// label。构建纯图标 Checkbox 时不要设置 label，并应通过 [`Self::aria_label`]
    /// 提供可访问名称。
    pub fn indicator_icons(
        mut self,
        unchecked_icon: impl IntoIconSource,
        checked_icon: impl IntoIconSource,
    ) -> Self {
        self.unchecked_icon = Some(unchecked_icon.into_icon_source());
        self.checked_icon = Some(checked_icon.into_icon_source());
        self.icon_indicator = true;
        self
    }

    /// 设置辅助技术朗读的名称。
    ///
    /// 该名称会覆盖可见 label。无可见 label 时应提供该名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 设置辅助技术朗读的补充描述。
    pub fn aria_description(mut self, description: impl Into<SharedString>) -> Self {
        self.aria_description = Some(description.into());
        self
    }

    /// 注册受控状态变化回调。
    ///
    /// 回调收到的 bool 是下一 checked 值。Checkbox 不会自行持久化该值，宿主应在
    /// 回调中更新自己的状态并触发重绘。
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的受控变化回调。
    pub fn on_change_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, bool, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        let listener = cx.listener(move |this, next_checked: &bool, window, cx| {
            handler(this, *next_checked, window, cx);
        });
        self.on_change(move |next_checked, window, cx| listener(&next_checked, window, cx))
    }

    /// 注册组件实际获得焦点时调用的回调。
    ///
    /// checked、indeterminate 或其他 builder 状态变化不会自行触发该回调。
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

    /// 返回稳定 ElementId。
    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub(crate) fn resolved_size(&self, cx: &App) -> ComponentSize {
        self.size.unwrap_or_else(|| component_size(cx))
    }

    fn visual_state(&self) -> CheckboxVisualState {
        CheckboxVisualState::new(self.checked, self.indeterminate)
    }

    fn accessible_label(&self) -> Option<SharedString> {
        self.aria_label.clone().or_else(|| self.label.clone())
    }

    fn icon_for_state(&self, state: CheckboxVisualState) -> Option<IconSource> {
        match state {
            CheckboxVisualState::Unchecked => self.unchecked_icon.clone(),
            CheckboxVisualState::Checked => Some(
                self.checked_icon
                    .clone()
                    .unwrap_or_else(|| IconSource::asset(DEFAULT_CHECKED_ICON)),
            ),
            CheckboxVisualState::Indeterminate => Some(
                self.indeterminate_icon
                    .clone()
                    .unwrap_or_else(|| IconSource::asset(DEFAULT_INDETERMINATE_ICON)),
            ),
        }
    }

    #[cfg(test)]
    pub(crate) fn is_checked(&self) -> bool {
        self.checked
    }

    #[cfg(test)]
    pub(crate) fn is_indeterminate(&self) -> bool {
        self.indeterminate
    }

    #[cfg(test)]
    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[cfg(test)]
    pub(crate) fn label_text(&self) -> Option<&SharedString> {
        self.label.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn aria_label_text(&self) -> Option<&SharedString> {
        self.aria_label.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn aria_description_text(&self) -> Option<&SharedString> {
        self.aria_description.as_ref()
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
    pub(crate) fn uses_icon_indicator(&self) -> bool {
        self.icon_indicator
    }
}

impl Disableable for Checkbox {
    fn disabled(self, disabled: bool) -> Self {
        Checkbox::disabled(self, disabled)
    }
}

impl Changeable<bool> for Checkbox {
    fn on_change(self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        Checkbox::on_change(self, handler)
    }

    fn on_change_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, bool, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        Checkbox::on_change_in(self, cx, handler)
    }
}

impl Focusable for Checkbox {
    fn on_focus(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Checkbox::on_focus(self, handler)
    }

    fn on_blur(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Checkbox::on_blur(self, handler)
    }
}

impl Sizable for Checkbox {
    fn size(self, size: ComponentSize) -> Self {
        Checkbox::size(self, size)
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_state = focus::state_for(
            &self.id,
            !self.disabled,
            self.on_focus.clone(),
            self.on_blur.clone(),
            window,
            cx,
        );
        let theme = theme::current_theme(window, cx);
        let size = theme
            .checkbox_size(self.resolved_size(cx).token_key())
            .expect("Vektra 默认 Checkbox size token 必须通过测试保持有效");
        let state = self.visual_state();
        let states = ResolvedCheckboxStates::new(&theme, state.token_key());
        let visible = if self.disabled {
            states.disabled
        } else {
            states.normal
        };
        let next_checked = next_checked(self.checked, self.indeterminate);
        let on_change = self.on_change.clone().filter(|_| !self.disabled);
        let on_click: Option<ClickHandler> = on_change.map(|handler| {
            Rc::new(
                move |_: &gpui::ClickEvent, window: &mut Window, cx: &mut App| {
                    handler(next_checked, window, cx);
                },
            ) as ClickHandler
        });
        let icon_indicator = self.icon_indicator;
        let indicator_icon_size = if icon_indicator {
            size.box_size
        } else {
            size.icon_size
        };
        let base_indicator_icon_color =
            indicator_icon_color(state, visible, icon_indicator, self.disabled);
        let indicator_id: ElementId = (self.id.clone(), "indicator").into();

        let box_element = div()
            .id(indicator_id)
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .size(size.box_size)
            .text_color(base_indicator_icon_color)
            .when(!icon_indicator, |this| {
                this.rounded(size.radius)
                    .border(theme.checkbox.border_width)
                    .border_color(visible.border)
                    .bg(visible.box_background)
            })
            .when(!self.disabled, |this| {
                this.hover(move |style| {
                    if icon_indicator {
                        style.text_color(indicator_icon_color(state, states.hover, true, false))
                    } else {
                        style
                            .bg(states.hover.box_background)
                            .border_color(states.hover.border)
                            .text_color(states.hover.icon)
                    }
                })
                .active(move |style| {
                    if icon_indicator {
                        style.text_color(indicator_icon_color(state, states.pressed, true, false))
                    } else {
                        style
                            .bg(states.pressed.box_background)
                            .border_color(states.pressed.border)
                            .text_color(states.pressed.icon)
                    }
                })
            })
            .when_some(self.icon_for_state(state), |this, icon| {
                this.child(Icon::new(icon).size(indicator_icon_size))
            });

        let element = div()
            .id(self.id.clone())
            .debug_selector(|| "vektra-checkbox".into())
            .role(Role::CheckBox)
            .aria_toggled(toggled_state(state))
            .when_some(self.accessible_label(), |this, label| {
                this.aria_label(label)
            })
            .when_some(self.aria_description, |this, description| {
                this.aria_description(description)
            })
            .flex()
            .items_center()
            .gap(size.label_gap)
            .min_h(size.hit_size)
            .min_w(size.hit_size)
            .py(size.hit_padding_y)
            .pr(size.hit_padding_x)
            .text_size(size.font_size)
            .line_height(size.line_height)
            .text_color(visible.label)
            .relative()
            .child(box_element)
            .when_some(self.label, |this, label| {
                this.child(
                    div()
                        .min_w_0()
                        .whitespace_normal()
                        .text_color(visible.label)
                        .child(label),
                )
            });

        let element = apply_interaction(
            element,
            self.disabled,
            on_click,
            self.cursor_style,
            states.focused.border,
            theme.checkbox.focus_width,
        );
        focus::attach_interaction(element, &focus_state, !self.disabled, cx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckboxVisualState {
    Unchecked,
    Checked,
    Indeterminate,
}

impl CheckboxVisualState {
    const fn new(checked: bool, indeterminate: bool) -> Self {
        if indeterminate {
            Self::Indeterminate
        } else if checked {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }

    const fn token_key(self) -> &'static str {
        match self {
            Self::Unchecked => "unchecked",
            Self::Checked => "checked",
            Self::Indeterminate => "indeterminate",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ResolvedCheckboxStates {
    normal: CheckboxStateTokens,
    hover: CheckboxStateTokens,
    pressed: CheckboxStateTokens,
    focused: CheckboxStateTokens,
    disabled: CheckboxStateTokens,
}

impl ResolvedCheckboxStates {
    fn new(theme: &ResolvedTheme, state: &str) -> Self {
        Self {
            normal: theme
                .checkbox_state(state, "normal")
                .expect("Vektra 默认 Checkbox normal token 必须通过测试保持有效"),
            hover: theme
                .checkbox_state(state, "hover")
                .expect("Vektra 默认 Checkbox hover token 必须通过测试保持有效"),
            pressed: theme
                .checkbox_state(state, "pressed")
                .expect("Vektra 默认 Checkbox pressed token 必须通过测试保持有效"),
            focused: theme
                .checkbox_state(state, "focus-visible")
                .expect("Vektra 默认 Checkbox focus token 必须通过测试保持有效"),
            disabled: theme
                .checkbox_state(state, "disabled")
                .expect("Vektra 默认 Checkbox disabled token 必须通过测试保持有效"),
        }
    }
}

fn apply_interaction(
    element: gpui::Stateful<gpui::Div>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    cursor_style: Option<CursorStyle>,
    focus_color: gpui::Hsla,
    focus_width: gpui::Pixels,
) -> gpui::Stateful<gpui::Div> {
    let on_key_space = on_click.clone();

    element
        .when(disabled, |this| {
            this.cursor(CursorStyle::OperationNotAllowed)
        })
        .when(!disabled, |this| {
            this.cursor(button::resolved_cursor_style(false, false, cursor_style))
                .tab_index(0)
                .focus_visible(move |style| style.border(focus_width).border_color(focus_color))
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
                    (handler)(
                        &button::keyboard_click(gpui::KeyboardButton::Space),
                        window,
                        cx,
                    );
                }
            })
        })
}

fn indicator_icon_color(
    state: CheckboxVisualState,
    tokens: CheckboxStateTokens,
    icon_indicator: bool,
    disabled: bool,
) -> gpui::Hsla {
    if icon_indicator && !disabled {
        match state {
            CheckboxVisualState::Unchecked => tokens.border,
            CheckboxVisualState::Checked | CheckboxVisualState::Indeterminate => {
                tokens.box_background
            }
        }
    } else {
        tokens.icon
    }
}

pub(crate) const fn next_checked(checked: bool, indeterminate: bool) -> bool {
    if indeterminate { true } else { !checked }
}

pub(crate) const fn toggled_state(state: CheckboxVisualState) -> Toggled {
    match state {
        CheckboxVisualState::Unchecked => Toggled::False,
        CheckboxVisualState::Checked => Toggled::True,
        CheckboxVisualState::Indeterminate => Toggled::Mixed,
    }
}

fn is_plain_key(event: &KeyDownEvent, key: &str) -> bool {
    event.keystroke.key == key && event.keystroke.modifiers == Modifiers::none()
}

fn is_plain_key_up(event: &KeyUpEvent, key: &str) -> bool {
    event.keystroke.key == key && event.keystroke.modifiers == Modifiers::none()
}

#[cfg(test)]
#[path = "../tests/unit/checkbox.rs"]
mod tests;
