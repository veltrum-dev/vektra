//! 强类型、受控的单值 Select 组件。

mod a11y;
mod overlay;
mod render;
mod state;
mod types;

use a11y::*;
use overlay::*;
use render::*;
use state::*;
use types::SelectChild;
pub use types::{SelectGroup, SelectOption, SelectStatus};

use crate::{
    Icon, IconSource,
    focus::{self, FocusHandler},
    scrollbar::ScrollableExt,
    size::{ComponentSize, component_size},
    theme,
    traits::{Changeable, Disableable, Focusable, Sizable},
};
use gpui::{
    A11ySubtreeBuilder, AnyElement, App, AvailableSpace, Bounds, BoxShadow, Context, CursorStyle,
    Div, Element, ElementId, Entity, GlobalElementId, InspectorElementId, InteractiveElement,
    IntoElement, KeyDownEvent, LayoutId, Modifiers, MouseButton, ParentElement, Pixels, Point,
    RenderOnce, Role, ScrollHandle, SharedString, Size, Stateful, StatefulInteractiveElement,
    Style, Styled, WeakEntity, Window, deferred, div, point, prelude::FluentBuilder,
};
use std::hash::{Hash, Hasher};
use std::{cell::Cell, rc::Rc};
use unicode_segmentation::UnicodeSegmentation as _;
use vektra_theme::{
    ResolvedTheme, SelectOptionState, SelectSizeTokens, SelectTriggerState,
    SelectTriggerStateTokens,
};

type ChangeHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App) + 'static>;

const SELECT_CHEVRON_ICON: &str = "components/select/chevron-down.svg";
const SELECT_CHEVRON_UP_ICON: &str = "components/select/chevron-up.svg";
const SELECT_CHECK_ICON: &str = "components/checkbox/check.svg";

/// 强类型、受控的单值下拉选择组件。
///
/// `selected_value` 始终是宿主持有的权威业务值；组件只通过 `on_change` 请求下一值。
/// Popup 的 open、active option、焦点句柄与滚动位置是按根 `ElementId` 保存的窗口私有
/// 交互状态，不是第二份业务选择。
#[derive(IntoElement)]
pub struct Select<T>
where
    T: Clone + PartialEq + 'static,
{
    id: ElementId,
    selected_value: Option<T>,
    children: Vec<SelectChild<T>>,
    placeholder: SharedString,
    status: SelectStatus,
    disabled: bool,
    size: Option<ComponentSize>,
    aria_label: Option<SharedString>,
    aria_description: Option<SharedString>,
    on_change: Option<ChangeHandler<T>>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
}

impl<T> Select<T>
where
    T: Clone + PartialEq + 'static,
{
    /// 创建一个默认无选中值的 Select。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected_value: None,
            children: Vec::new(),
            placeholder: "请选择".into(),
            status: SelectStatus::Ready,
            disabled: false,
            size: None,
            aria_label: None,
            aria_description: None,
            on_change: None,
            on_focus: None,
            on_blur: None,
        }
    }

    /// 设置宿主持有的权威选中值。
    pub fn selected_value(mut self, selected_value: Option<T>) -> Self {
        self.selected_value = selected_value;
        self
    }

    /// 添加一个顶层结构化 option。
    pub fn option(mut self, option: SelectOption<T>) -> Self {
        self.children.push(SelectChild::Option(option));
        self
    }

    /// 添加一组结构化 option。
    pub fn group(mut self, group: SelectGroup<T>) -> Self {
        self.children.push(SelectChild::Group(group));
        self
    }

    /// 设置没有有效受控选中项时 Trigger 显示的 placeholder。
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 设置 Popup 的宿主控制内容状态。
    pub fn status(mut self, status: SelectStatus) -> Self {
        self.status = status;
        self
    }

    /// 设置整组禁用状态。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 设置 Select 使用的共享语义尺寸。
    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = Some(size);
        self
    }

    /// 设置 Trigger 的可访问名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 设置 Trigger 的可访问补充描述。
    pub fn aria_description(mut self, description: impl Into<SharedString>) -> Self {
        self.aria_description = Some(description.into());
        self
    }

    /// 注册下一选中值的受控变化请求。
    ///
    /// 再次提交当前权威值不会调用回调。
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

    /// 注册 Trigger 真实获得焦点时调用的回调。
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

    /// 注册 Trigger 真实失去焦点时调用的回调。
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

    /// 返回 Select 根节点的稳定标识。
    pub fn id(&self) -> &ElementId {
        &self.id
    }
}

impl<T> Changeable<T> for Select<T>
where
    T: Clone + PartialEq + 'static,
{
    fn on_change(self, handler: impl Fn(T, &mut Window, &mut App) + 'static) -> Self {
        Select::on_change(self, handler)
    }

    fn on_change_in<U: 'static>(
        self,
        cx: &Context<U>,
        handler: impl Fn(&mut U, T, &mut Window, &mut Context<U>) + 'static,
    ) -> Self {
        Select::on_change_in(self, cx, handler)
    }
}

impl<T> Disableable for Select<T>
where
    T: Clone + PartialEq + 'static,
{
    fn disabled(self, disabled: bool) -> Self {
        Select::disabled(self, disabled)
    }
}

impl<T> Sizable for Select<T>
where
    T: Clone + PartialEq + 'static,
{
    fn size(self, size: ComponentSize) -> Self {
        Select::size(self, size)
    }
}

impl<T> Focusable for Select<T>
where
    T: Clone + PartialEq + 'static,
{
    fn on_focus(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Select::on_focus(self, handler)
    }

    fn on_blur(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Select::on_blur(self, handler)
    }
}

#[derive(Clone)]
struct OptionMetadata<T> {
    id: ElementId,
    value: T,
    label: SharedString,
    accessible_name: SharedString,
    canonical: bool,
    disabled: bool,
    position: usize,
    set_size: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct TriggerAccessibility {
    value: Option<SharedString>,
    placeholder: Option<SharedString>,
}

fn trigger_accessibility<T>(
    selected: Option<&OptionMetadata<T>>,
    placeholder: &SharedString,
) -> TriggerAccessibility {
    TriggerAccessibility {
        value: selected.map(|option| option.accessible_name.clone()),
        placeholder: selected.is_none().then(|| placeholder.clone()),
    }
}

fn option_catalog<T: Clone + PartialEq>(children: &[SelectChild<T>]) -> Vec<OptionMetadata<T>> {
    let set_size = children
        .iter()
        .map(|child| match child {
            SelectChild::Option(_) => 1,
            SelectChild::Group(group) => group.options.len(),
        })
        .sum();
    let mut result: Vec<OptionMetadata<T>> = Vec::with_capacity(set_size);
    for child in children {
        match child {
            SelectChild::Option(option) => {
                push_option_metadata(&mut result, option, set_size);
            }
            SelectChild::Group(group) => {
                for option in &group.options {
                    push_option_metadata(&mut result, option, set_size);
                }
            }
        }
    }
    result
}

fn push_option_metadata<T: Clone + PartialEq>(
    result: &mut Vec<OptionMetadata<T>>,
    option: &SelectOption<T>,
    set_size: usize,
) {
    let canonical = !result
        .iter()
        .any(|previous| previous.id == option.id || previous.value == option.value);
    result.push(OptionMetadata {
        id: option.id.clone(),
        value: option.value.clone(),
        label: option.label.clone(),
        accessible_name: option.accessible_label(),
        canonical,
        disabled: option.disabled || !canonical,
        position: result.len(),
        set_size,
    });
}

impl<T> RenderOnce for Select<T>
where
    T: Clone + PartialEq + 'static,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let resolved_size = self.size.unwrap_or_else(|| component_size(cx));
        let theme = theme::current_theme(window, cx);
        let size = theme.select_size(resolved_size.theme_size());
        let catalog = option_catalog(&self.children);
        let selected = self.selected_value.as_ref().and_then(|selected| {
            catalog
                .iter()
                .find(|entry| entry.canonical && entry.value == *selected)
        });
        let selected_enabled_id = selected
            .filter(|option| !option.disabled)
            .map(|option| option.id.clone());
        let snapshots = catalog
            .iter()
            .filter(|entry| entry.canonical)
            .map(|entry| OptionSnapshot {
                id: entry.id.clone(),
                disabled: entry.disabled,
                accessible_name: entry.accessible_name.clone(),
            })
            .collect::<Vec<_>>();
        let canonical_values = if self.status.is_ready() {
            catalog
                .iter()
                .filter(|entry| entry.canonical && !entry.disabled)
                .map(|entry| (entry.id.clone(), entry.value.clone()))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let state = window.use_keyed_state(
            (self.id.clone(), "select-interaction"),
            cx,
            SelectInteractionState::new,
        );
        state.update(cx, |state, cx| {
            state.reconcile(
                snapshots,
                self.status.is_ready(),
                !self.disabled,
                selected_enabled_id.as_ref(),
                cx,
            )
        });
        let focus_state = focus::state_for(
            &self.id,
            !self.disabled,
            self.on_focus.clone(),
            self.on_blur.clone(),
            window,
            cx,
        );
        let focus_handle = focus::handle(&focus_state, cx);
        let blur_state = state.downgrade();
        focus::set_observers(
            &focus_state,
            Rc::new(|_, _| {}),
            Rc::new(move |_, cx| {
                let _ = blur_state.update(cx, |state, cx| {
                    state.close(cx);
                });
            }),
            cx,
        );
        let is_open = state.read(cx).open;
        let active_id = state.read(cx).active_id.clone();
        let trigger_bounds = state.read(cx).trigger_bounds.clone();
        let scroll_handle = state.read(cx).scroll_handle.clone();
        let trigger_states = ResolvedTriggerStates::new(&theme);
        let focus_width = theme.select.focus_width;
        let trigger_visible = if self.disabled {
            trigger_states.disabled
        } else if is_open {
            trigger_states.open
        } else {
            trigger_states.normal
        };
        let display_label = selected.map(|option| option.label.clone());
        let trigger_accessibility = trigger_accessibility(selected, &self.placeholder);
        let trigger_name = self
            .aria_label
            .clone()
            .or_else(|| display_label.clone())
            .unwrap_or_else(|| self.placeholder.clone());
        let popup_name = trigger_name.clone();

        let trigger_content = div()
            .debug_selector(|| "vektra-select-trigger-content".into())
            .w_full()
            .h(size.height)
            .flex()
            .items_center()
            .gap(size.content_gap)
            .px(size.padding_x)
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .truncate()
                    .text_size(size.font_size)
                    .line_height(size.line_height)
                    .text_color(if display_label.is_some() {
                        trigger_visible.foreground
                    } else {
                        trigger_visible.placeholder
                    })
                    .child(display_label.unwrap_or_else(|| self.placeholder.clone())),
            )
            .child(
                div()
                    .flex_none()
                    .size(size.indicator_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Icon::new(IconSource::asset(if is_open {
                            SELECT_CHEVRON_UP_ICON
                        } else {
                            SELECT_CHEVRON_ICON
                        }))
                        .size(size.indicator_size)
                        .color(trigger_visible.indicator),
                    ),
            );

        let state_for_click = state.downgrade();
        let selected_for_click = selected_enabled_id.clone();
        let authoritative_value = self.selected_value.clone();
        let click_values = canonical_values;
        let trigger_handler = self.on_change.clone();
        let click_focus = focus_handle.clone();
        let mut trigger = div()
            .id(self.id.clone())
            .debug_selector(|| "vektra-select-trigger".into())
            .role(Role::ComboBox)
            .aria_label(trigger_name)
            .aria_expanded(is_open)
            .when_some(trigger_accessibility.value, |element, value| {
                element.aria_value(value)
            })
            .when_some(trigger_accessibility.placeholder, |element, placeholder| {
                element.aria_placeholder(placeholder)
            })
            .when_some(self.aria_description, |element, description| {
                element.aria_description(description)
            })
            .w_full()
            .min_w_0()
            .h(size.height)
            .min_h(size.height)
            .rounded(size.radius)
            .border(theme.select.border_width)
            .border_color(trigger_visible.border)
            .bg(trigger_visible.background)
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .when(!self.disabled, |element| {
                element
                    .hover(move |style| {
                        style
                            .bg(trigger_states.hover.background)
                            .border_color(trigger_states.hover.border)
                    })
                    .active(move |style| {
                        style
                            .bg(trigger_states.pressed.background)
                            .border_color(trigger_states.pressed.border)
                    })
                    .focus_visible(move |style| {
                        style
                            .border(focus_width)
                            .border_color(trigger_states.focused.border)
                    })
            })
            .child(trigger_content);

        trigger = focus::attach_interaction(trigger, &focus_state, !self.disabled, cx);
        if !self.disabled {
            trigger = trigger
                .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                .on_click(move |event, window, cx| {
                    let _ = state_for_click.update(cx, |state, cx| match event {
                        gpui::ClickEvent::Keyboard(_) if state.open => {
                            let requested = state.submittable_active_id().and_then(|active| {
                                click_values
                                    .iter()
                                    .find(|(id, _)| *id == active)
                                    .map(|(_, value)| value.clone())
                            });
                            state.close(cx);
                            click_focus.focus(window, cx);
                            if let Some(requested) = requested
                                && authoritative_value.as_ref() != Some(&requested)
                                && let Some(handler) = trigger_handler.as_ref()
                            {
                                handler(requested, window, cx);
                            }
                        }
                        gpui::ClickEvent::Keyboard(_) => {
                            state.open_with(selected_for_click.clone(), false, true, cx);
                        }
                        gpui::ClickEvent::Mouse(_) | gpui::ClickEvent::Touch(_) => {
                            if state.open {
                                state.close(cx);
                            } else {
                                state.open_with(selected_for_click.clone(), false, false, cx);
                            }
                        }
                    });
                    cx.stop_propagation();
                });
        }

        let key_state = state.downgrade();
        let key_selected = selected_enabled_id;
        trigger = trigger.on_key_down(move |event: &KeyDownEvent, window, cx| {
            handle_trigger_key(event, &key_state, key_selected.as_ref(), window, cx);
        });

        if is_open {
            let popup_node_id = select_popup_node_id(self.id.clone(), window);
            let popup = render_popup(
                self.children,
                catalog,
                self.status,
                self.selected_value,
                active_id,
                state.clone(),
                focus_handle,
                scroll_handle,
                self.on_change,
                size,
                &theme,
                trigger_bounds.clone(),
                popup_name,
            );
            trigger = trigger.child(
                deferred(SelectPopupOverlay {
                    body: Some(popup.into_any_element()),
                    trigger_bounds: trigger_bounds.clone(),
                    viewport_bounds: Bounds::new(Point::default(), window.viewport_size()),
                    anchor_gap: theme.select.popup_anchor_gap,
                    viewport_padding: theme.select.popup_viewport_padding,
                    max_height: theme.select.popup_max_height,
                })
                .priority(1),
            );

            return DisabledA11y::new(trigger, self.disabled, Some(trigger_bounds), None)
                .controls(popup_node_id);
        }

        DisabledA11y::new(trigger, self.disabled, Some(trigger_bounds), None)
    }
}

#[cfg(test)]
#[path = "../tests/unit/select.rs"]
mod tests;
