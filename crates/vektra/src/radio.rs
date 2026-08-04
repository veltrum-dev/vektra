//! Radio 与 RadioGroup 受控单选组件。

use crate::{
    focus::{self, FocusHandler},
    size::{ComponentSize, component_size},
    theme,
    traits::{Changeable, Disableable, Focusable, Sizable},
};
use gpui::{
    App, Context, CursorStyle, ElementId, FocusHandle, InteractiveElement, IntoElement,
    KeyDownEvent, KeyUpEvent, Modifiers, MouseButton, Orientation, ParentElement, RenderOnce, Role,
    SharedString, StatefulInteractiveElement, Styled, Toggled, Window, div, prelude::FluentBuilder,
};
use std::rc::Rc;
use vektra_theme::{RadioSizeTokens, RadioStateTokens, ResolvedTheme};

type ChangeHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App) + 'static>;

/// RadioGroup 中的强类型单选项。
///
/// `Radio<T>` 只保存单项的值、语义文本和交互配置，不实现 `IntoElement`，因此不能脱离
/// [`RadioGroup`] 独立渲染。
///
/// ```compile_fail
/// use gpui::IntoElement;
/// use vektra::Radio;
///
/// fn require_element(_: impl IntoElement) {}
/// require_element(Radio::new("standalone", 1_u8));
/// ```
pub struct Radio<T> {
    id: ElementId,
    value: T,
    label: Option<SharedString>,
    description: Option<SharedString>,
    aria_label: Option<SharedString>,
    aria_description: Option<SharedString>,
    disabled: bool,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
}

impl<T> Radio<T> {
    /// 创建一个带稳定 `ElementId` 和强类型值的单选项。
    pub fn new(id: impl Into<ElementId>, value: T) -> Self {
        Self {
            id: id.into(),
            value,
            label: None,
            description: None,
            aria_label: None,
            aria_description: None,
            disabled: false,
            on_focus: None,
            on_blur: None,
        }
    }

    /// 设置单选项的可见标签。
    ///
    /// 未设置显式可访问名称时，该标签同时作为辅助技术朗读的名称。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// 设置显示在标签下方的补充描述。
    ///
    /// 未设置显式可访问描述时，该文本同时作为辅助技术朗读的描述。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 设置辅助技术朗读的名称，并覆盖可见标签提供的名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 设置辅助技术朗读的补充描述，并覆盖可见描述提供的内容。
    pub fn aria_description(mut self, description: impl Into<SharedString>) -> Self {
        self.aria_description = Some(description.into());
        self
    }

    /// 设置单项禁用状态。
    ///
    /// 禁用项不会进入 Tab 顺序、不会被方向键选中，也不会产生变化请求。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 注册该单选项真实获得焦点时调用的回调。
    pub fn on_focus(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_focus = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的聚焦回调。
    pub fn on_focus_in<U: 'static>(
        self,
        cx: &Context<U>,
        handler: impl Fn(&mut U, &mut Window, &mut Context<U>) + 'static,
    ) -> Self {
        Focusable::on_focus_in(self, cx, handler)
    }

    /// 注册该单选项真实失去焦点时调用的回调。
    pub fn on_blur(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_blur = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的失焦回调。
    pub fn on_blur_in<U: 'static>(
        self,
        cx: &Context<U>,
        handler: impl Fn(&mut U, &mut Window, &mut Context<U>) + 'static,
    ) -> Self {
        Focusable::on_blur_in(self, cx, handler)
    }

    /// 返回单选项的稳定 `ElementId`。
    pub fn id(&self) -> &ElementId {
        &self.id
    }

    fn accessible_label(&self) -> Option<SharedString> {
        self.aria_label.clone().or_else(|| self.label.clone())
    }

    fn accessible_description(&self) -> Option<SharedString> {
        self.aria_description
            .clone()
            .or_else(|| self.description.clone())
    }
}

impl<T> Disableable for Radio<T> {
    fn disabled(self, disabled: bool) -> Self {
        Radio::disabled(self, disabled)
    }
}

impl<T> Focusable for Radio<T> {
    fn on_focus(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Radio::on_focus(self, handler)
    }

    fn on_blur(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Radio::on_blur(self, handler)
    }
}

/// 一组受控、可访问并使用 roving focus 的强类型单选项。
///
/// `selected_value` 是权威业务状态；用户交互只通过 [`Changeable::on_change`] 请求下一
/// 值。宿主可以立即采用请求，也可以异步审批后再传回新的权威值。
#[derive(IntoElement)]
pub struct RadioGroup<T>
where
    T: Clone + PartialEq + 'static,
{
    id: ElementId,
    selected_value: Option<T>,
    radios: Vec<Radio<T>>,
    disabled: bool,
    size: Option<ComponentSize>,
    orientation: Orientation,
    aria_label: Option<SharedString>,
    aria_description: Option<SharedString>,
    on_change: Option<ChangeHandler<T>>,
}

impl<T> RadioGroup<T>
where
    T: Clone + PartialEq + 'static,
{
    /// 创建默认无选中项的 RadioGroup。
    ///
    /// 默认方向为垂直；通过 [`Self::child`] 添加一个或多个强类型 [`Radio`]。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected_value: None,
            radios: Vec::new(),
            disabled: false,
            size: None,
            orientation: Orientation::Vertical,
            aria_label: None,
            aria_description: None,
            on_change: None,
        }
    }

    /// 设置当前权威选中值；`None` 表示没有选中项。
    pub fn selected_value(mut self, selected_value: Option<T>) -> Self {
        self.selected_value = selected_value;
        self
    }

    /// 添加一个与组值类型一致的 Radio 子项。
    ///
    /// 该固有方法只接收 `Radio<T>`；RadioGroup 不实现可接收任意元素的
    /// `ParentElement`。
    ///
    /// ```compile_fail
    /// use gpui::div;
    /// use vektra::RadioGroup;
    ///
    /// let _ = RadioGroup::<u8>::new("typed-group").child(div());
    /// ```
    pub fn child(mut self, radio: Radio<T>) -> Self {
        self.radios.push(radio);
        self
    }

    /// 设置整组禁用状态。
    ///
    /// 组级禁用优先于单项配置，并禁止所有鼠标、键盘和焦点导航交互。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 设置整组统一使用的语义尺寸。
    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = Some(size);
        self
    }

    /// 设置组布局与辅助技术报告的方向。
    ///
    /// 无论方向为何，组件都响应 Up/Down/Left/Right，方便不同平台和书写方向下使用。
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// 设置辅助技术朗读的组级名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 设置辅助技术朗读的组级补充描述。
    pub fn aria_description(mut self, description: impl Into<SharedString>) -> Self {
        self.aria_description = Some(description.into());
        self
    }

    /// 注册下一选中值的受控变化请求。
    ///
    /// 再次激活当前权威选中项不会调用回调。
    pub fn on_change(mut self, handler: impl Fn(T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 注册可访问宿主 Entity 状态的受控变化请求。
    pub fn on_change_in<U: 'static>(
        self,
        cx: &Context<U>,
        handler: impl Fn(&mut U, T, &mut Window, &mut Context<U>) + 'static,
    ) -> Self {
        let listener = cx.listener(move |this, value: &T, window, cx| {
            handler(this, value.clone(), window, cx);
        });
        self.on_change(move |value, window, cx| listener(&value, window, cx))
    }

    /// 返回组根节点的稳定 `ElementId`。
    pub fn id(&self) -> &ElementId {
        &self.id
    }

    fn resolved_size(&self, cx: &App) -> ComponentSize {
        self.size.unwrap_or_else(|| component_size(cx))
    }
}

impl<T> Changeable<T> for RadioGroup<T>
where
    T: Clone + PartialEq + 'static,
{
    fn on_change(self, handler: impl Fn(T, &mut Window, &mut App) + 'static) -> Self {
        RadioGroup::on_change(self, handler)
    }

    fn on_change_in<U: 'static>(
        self,
        cx: &Context<U>,
        handler: impl Fn(&mut U, T, &mut Window, &mut Context<U>) + 'static,
    ) -> Self {
        RadioGroup::on_change_in(self, cx, handler)
    }
}

impl<T> Disableable for RadioGroup<T>
where
    T: Clone + PartialEq + 'static,
{
    fn disabled(self, disabled: bool) -> Self {
        RadioGroup::disabled(self, disabled)
    }
}

impl<T> Sizable for RadioGroup<T>
where
    T: Clone + PartialEq + 'static,
{
    fn size(self, size: ComponentSize) -> Self {
        RadioGroup::size(self, size)
    }
}

impl<T> RenderOnce for RadioGroup<T>
where
    T: Clone + PartialEq + 'static,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = theme::current_theme(window, cx);
        let size = theme
            .radio_size(self.resolved_size(cx).token_key())
            .expect("Vektra 默认 Radio size token 必须通过测试保持有效");
        let effective_disabled: Vec<bool> = self
            .radios
            .iter()
            .map(|radio| self.disabled || radio.disabled)
            .collect();
        let tab_stop_index = self
            .radios
            .iter()
            .enumerate()
            .find(|(index, radio)| {
                !effective_disabled[*index] && self.selected_value.as_ref() == Some(&radio.value)
            })
            .map(|(index, _)| index)
            .or_else(|| effective_disabled.iter().position(|disabled| !disabled));

        let focus_states = self
            .radios
            .iter()
            .enumerate()
            .map(|(index, radio)| {
                focus::state_for(
                    &radio.id,
                    Some(index) == tab_stop_index,
                    radio.on_focus.clone(),
                    radio.on_blur.clone(),
                    window,
                    cx,
                )
            })
            .collect::<Vec<_>>();
        let enabled_targets = self
            .radios
            .iter()
            .enumerate()
            .filter(|(index, _)| !effective_disabled[*index])
            .map(|(index, radio)| {
                (
                    index,
                    focus::handle(&focus_states[index], cx),
                    radio.value.clone(),
                )
            })
            .collect::<Vec<_>>();
        let set_size = self.radios.len();

        let mut group = div()
            .id(self.id)
            .debug_selector(|| "vektra-radio-group".into())
            .role(radio_group_role())
            .aria_orientation(self.orientation)
            .when_some(self.aria_label, |element, label| element.aria_label(label))
            .when_some(self.aria_description, |element, description| {
                element.aria_description(description)
            })
            .flex()
            .gap(size.group_gap);
        group = match self.orientation {
            Orientation::Horizontal => group.flex_row().items_start(),
            Orientation::Vertical => group.flex_col().items_start(),
        };

        for (index, radio) in self.radios.into_iter().enumerate() {
            let disabled = effective_disabled[index];
            let selected = self.selected_value.as_ref() == Some(&radio.value);
            let states = ResolvedRadioStates::new(&theme, selected);
            let visible = if disabled {
                states.disabled
            } else {
                states.normal
            };
            let enabled_position = enabled_targets
                .iter()
                .position(|(original_index, _, _)| *original_index == index);
            let item = render_radio_item(
                radio,
                RadioRenderContext {
                    index,
                    set_size,
                    selected,
                    disabled,
                    size,
                    states,
                    visible,
                    selected_value: self.selected_value.clone(),
                    on_change: self.on_change.clone(),
                    enabled_targets: enabled_targets.clone(),
                    enabled_position,
                    focus_handle: focus::handle(&focus_states[index], cx),
                    focus_width: theme.radio.focus_width,
                    border_width: theme.radio.border_width,
                },
            );
            group = group.child(focus::attach_interaction(
                item,
                &focus_states[index],
                !disabled,
                cx,
            ));
        }

        group
    }
}

#[derive(Debug, Clone, Copy)]
struct ResolvedRadioStates {
    normal: RadioStateTokens,
    hover: RadioStateTokens,
    pressed: RadioStateTokens,
    focused: RadioStateTokens,
    disabled: RadioStateTokens,
}

impl ResolvedRadioStates {
    fn new(theme: &ResolvedTheme, selected: bool) -> Self {
        Self {
            normal: theme
                .radio_state(selected, "normal")
                .expect("Vektra 默认 Radio normal token 必须通过测试保持有效"),
            hover: theme
                .radio_state(selected, "hover")
                .expect("Vektra 默认 Radio hover token 必须通过测试保持有效"),
            pressed: theme
                .radio_state(selected, "pressed")
                .expect("Vektra 默认 Radio pressed token 必须通过测试保持有效"),
            focused: theme
                .radio_state(selected, "focus-visible")
                .expect("Vektra 默认 Radio focus token 必须通过测试保持有效"),
            disabled: theme
                .radio_state(selected, "disabled")
                .expect("Vektra 默认 Radio disabled token 必须通过测试保持有效"),
        }
    }
}

struct RadioRenderContext<T> {
    index: usize,
    set_size: usize,
    selected: bool,
    disabled: bool,
    size: RadioSizeTokens,
    states: ResolvedRadioStates,
    visible: RadioStateTokens,
    selected_value: Option<T>,
    on_change: Option<ChangeHandler<T>>,
    enabled_targets: Vec<(usize, FocusHandle, T)>,
    enabled_position: Option<usize>,
    focus_handle: FocusHandle,
    focus_width: gpui::Pixels,
    border_width: gpui::Pixels,
}

fn render_radio_item<T>(
    radio: Radio<T>,
    context: RadioRenderContext<T>,
) -> gpui::Stateful<gpui::Div>
where
    T: Clone + PartialEq + 'static,
{
    let RadioRenderContext {
        index,
        set_size,
        selected,
        disabled,
        size,
        states,
        visible,
        selected_value,
        on_change,
        enabled_targets,
        enabled_position,
        focus_handle,
        focus_width,
        border_width,
    } = context;
    let interaction_group: SharedString = format!("vektra-radio-{:?}", radio.id).into();
    let indicator_id: ElementId = (radio.id.clone(), "indicator").into();
    let indicator = div()
        .id(indicator_id)
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .size(size.indicator_size)
        .rounded(size.indicator_size / 2.)
        .border(border_width)
        .border_color(visible.border)
        .bg(visible.indicator_background)
        .when(!disabled, |element| {
            element
                .group_hover(interaction_group.clone(), move |style| {
                    style
                        .border_color(states.hover.border)
                        .bg(states.hover.indicator_background)
                })
                .group_active(interaction_group.clone(), move |style| {
                    style
                        .border_color(states.pressed.border)
                        .bg(states.pressed.indicator_background)
                })
        })
        .when(selected, |element| {
            element.child(
                div()
                    .size(size.dot_size)
                    .rounded(size.dot_size / 2.)
                    .bg(visible.dot),
            )
        });

    let mut item = div()
        .id(radio.id.clone())
        .group(interaction_group)
        .debug_selector(|| "vektra-radio".into())
        .role(radio_role())
        .aria_toggled(toggled_state(selected))
        .aria_position_in_set(index + 1)
        .aria_size_of_set(set_size)
        .when_some(radio.accessible_label(), |element, label| {
            element.aria_label(label)
        })
        .when_some(radio.accessible_description(), |element, description| {
            element.aria_description(description)
        })
        .flex()
        .items_start()
        .gap(size.label_gap)
        .min_h(size.hit_size)
        .min_w(size.hit_size)
        .px(size.hit_padding_x)
        .py(size.hit_padding_y)
        .rounded(size.hit_size / 6.)
        .bg(visible.background)
        .text_color(visible.label)
        .cursor(if disabled {
            CursorStyle::OperationNotAllowed
        } else {
            CursorStyle::PointingHand
        })
        .when(!disabled, |element| {
            element
                .hover(move |style| style.bg(states.hover.background))
                .active(move |style| style.bg(states.pressed.background))
                .focus_visible(move |style| {
                    style
                        .border(focus_width)
                        .border_color(states.focused.border)
                })
        })
        .child(indicator);

    if radio.label.is_some() || radio.description.is_some() {
        item = item.child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(size.description_gap)
                .when_some(radio.label, |element, label| {
                    element.child(
                        div()
                            .whitespace_normal()
                            .text_size(size.font_size)
                            .line_height(size.line_height)
                            .text_color(visible.label)
                            .child(label),
                    )
                })
                .when_some(radio.description, |element, description| {
                    element.child(
                        div()
                            .whitespace_normal()
                            .text_size(size.description_font_size)
                            .line_height(size.description_line_height)
                            .text_color(visible.description)
                            .child(description),
                    )
                }),
        );
    }

    if disabled {
        return item;
    }

    let click_selected = selected_value.clone();
    let click_value = radio.value.clone();
    let click_handler = on_change.clone();
    let click_focus = focus_handle.clone();
    item = item
        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            click_focus.focus(window, cx);
            request_change(
                &click_selected,
                click_value.clone(),
                click_handler.as_ref(),
                window,
                cx,
            );
        });

    let key_selected = selected_value.clone();
    let key_handler = on_change.clone();
    let key_targets = enabled_targets.clone();
    item = item.on_key_down(move |event: &KeyDownEvent, window, cx| {
        if event.keystroke.modifiers != Modifiers::none() {
            return;
        }

        if event.keystroke.key == "space" {
            window.prevent_default();
            cx.stop_propagation();
            return;
        }

        let Some(current) = enabled_position else {
            return;
        };
        let Some(target) = navigation_target(&event.keystroke.key, current, key_targets.len())
        else {
            return;
        };
        window.prevent_default();
        cx.stop_propagation();
        let (_, handle, value) = &key_targets[target];
        handle.focus(window, cx);
        request_change(
            &key_selected,
            value.clone(),
            key_handler.as_ref(),
            window,
            cx,
        );
    });

    let space_selected = selected_value;
    let space_value = radio.value;
    item.on_key_up(move |event: &KeyUpEvent, window, cx| {
        if event.keystroke.key == "space" && event.keystroke.modifiers == Modifiers::none() {
            window.prevent_default();
            cx.stop_propagation();
            request_change(
                &space_selected,
                space_value.clone(),
                on_change.as_ref(),
                window,
                cx,
            );
        }
    })
}

fn request_change<T>(
    selected_value: &Option<T>,
    requested_value: T,
    handler: Option<&ChangeHandler<T>>,
    window: &mut Window,
    cx: &mut App,
) where
    T: PartialEq,
{
    if selected_value.as_ref() == Some(&requested_value) {
        return;
    }
    if let Some(handler) = handler {
        handler(requested_value, window, cx);
    }
}

fn navigation_target(key: &str, current: usize, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key {
        "up" | "left" => Some(if current == 0 { len - 1 } else { current - 1 }),
        "down" | "right" => Some((current + 1) % len),
        "home" => Some(0),
        "end" => Some(len - 1),
        _ => None,
    }
}

const fn radio_group_role() -> Role {
    Role::RadioGroup
}

const fn radio_role() -> Role {
    Role::RadioButton
}

const fn toggled_state(selected: bool) -> Toggled {
    if selected {
        Toggled::True
    } else {
        Toggled::False
    }
}

#[cfg(test)]
#[path = "../tests/unit/radio.rs"]
mod tests;
