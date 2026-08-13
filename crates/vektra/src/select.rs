//! 强类型、受控的单值 Select 组件。

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
    Style, Styled, Subscription, WeakEntity, Window, deferred, div, point, prelude::FluentBuilder,
};
use std::{cell::Cell, rc::Rc};
use vektra_theme::{ResolvedTheme, SelectSizeTokens, SelectTriggerStateTokens};

type ChangeHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App) + 'static>;

const SELECT_CHEVRON_ICON: &str = "components/select/chevron-down.svg";
const SELECT_CHEVRON_UP_ICON: &str = "components/select/chevron-up.svg";
const SELECT_CHECK_ICON: &str = "components/checkbox/check.svg";

/// Select Popup 当前由宿主控制的互斥内容状态。
///
/// 状态内容不是 option，不参与键盘导航，也不会产生 [`Changeable`] 回调。Select
/// 不发起异步请求；宿主完成加载、重试或数据更新后，应传入新的状态与 option。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SelectStatus {
    /// 显示结构化 option 与 group。
    #[default]
    Ready,
    /// 显示宿主提供的加载文案。
    Loading(SharedString),
    /// 显示宿主提供的空状态文案。
    Empty(SharedString),
    /// 显示宿主提供的错误文案。
    Error(SharedString),
}

impl SelectStatus {
    /// 创建 loading 状态。
    pub fn loading(message: impl Into<SharedString>) -> Self {
        Self::Loading(message.into())
    }

    /// 创建 empty 状态。
    pub fn empty(message: impl Into<SharedString>) -> Self {
        Self::Empty(message.into())
    }

    /// 创建 error 状态。
    pub fn error(message: impl Into<SharedString>) -> Self {
        Self::Error(message.into())
    }

    fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// Select 中一个结构化、强类型的可选项。
///
/// `id` 与 `value` 在同一个 Select 中都应唯一。若出现重复，Select 按输入顺序只把
/// 第一个同时拥有未重复 ID 与未重复值的项作为 canonical option；后续冲突项仍可见，
/// 但按禁用项处理，不会形成第二个选中视觉或重复变化回调。
pub struct SelectOption<T> {
    id: ElementId,
    value: T,
    label: SharedString,
    icon: Option<IconSource>,
    description: Option<SharedString>,
    aria_label: Option<SharedString>,
    aria_description: Option<SharedString>,
    disabled: bool,
}

impl<T> SelectOption<T> {
    /// 创建带稳定 `ElementId`、业务值和可见标签的 option。
    pub fn new(id: impl Into<ElementId>, value: T, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value,
            label: label.into(),
            icon: None,
            description: None,
            aria_label: None,
            aria_description: None,
            disabled: false,
        }
    }

    /// 设置 option 的可选前置图标。
    pub fn icon(mut self, icon: IconSource) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 设置显示在主标签下方的补充描述。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 覆盖辅助技术使用的 option 名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 覆盖辅助技术使用的 option 描述。
    pub fn aria_description(mut self, description: impl Into<SharedString>) -> Self {
        self.aria_description = Some(description.into());
        self
    }

    /// 设置单项禁用状态。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 返回 option 的稳定标识。
    pub fn id(&self) -> &ElementId {
        &self.id
    }

    fn accessible_label(&self) -> SharedString {
        self.aria_label
            .clone()
            .unwrap_or_else(|| self.label.clone())
    }

    fn accessible_description(&self) -> Option<SharedString> {
        self.aria_description
            .clone()
            .or_else(|| self.description.clone())
    }
}

impl<T> Disableable for SelectOption<T> {
    fn disabled(self, disabled: bool) -> Self {
        SelectOption::disabled(self, disabled)
    }
}

/// Select 中带可见标题的一组结构化 option。
pub struct SelectGroup<T> {
    id: ElementId,
    label: SharedString,
    aria_label: Option<SharedString>,
    options: Vec<SelectOption<T>>,
}

impl<T> SelectGroup<T> {
    /// 创建带稳定 `ElementId` 和可见标题的 group。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            aria_label: None,
            options: Vec::new(),
        }
    }

    /// 覆盖辅助技术使用的 group 名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 向 group 添加一个类型一致的 option。
    pub fn option(mut self, option: SelectOption<T>) -> Self {
        self.options.push(option);
        self
    }

    /// 返回 group 的稳定标识。
    pub fn id(&self) -> &ElementId {
        &self.id
    }

    fn accessible_label(&self) -> SharedString {
        self.aria_label
            .clone()
            .unwrap_or_else(|| self.label.clone())
    }
}

enum SelectChild<T> {
    Option(SelectOption<T>),
    Group(SelectGroup<T>),
}

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

#[derive(Clone, PartialEq, Eq)]
struct OptionSnapshot {
    id: ElementId,
    disabled: bool,
}

struct SelectInteractionState {
    open: bool,
    active_id: Option<ElementId>,
    previous: Vec<OptionSnapshot>,
    scroll_handle: ScrollHandle,
    pending_scroll: bool,
    trigger_bounds: Rc<Cell<Bounds<Pixels>>>,
    _activation_subscription: Subscription,
}

impl SelectInteractionState {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let activation_subscription = cx.observe_window_activation(window, |this, window, cx| {
            if !window.is_window_active() && this.open {
                this.open = false;
                cx.notify();
            }
        });
        Self {
            open: false,
            active_id: None,
            previous: Vec::new(),
            scroll_handle: ScrollHandle::new(),
            pending_scroll: false,
            trigger_bounds: Rc::new(Cell::new(Bounds::default())),
            _activation_subscription: activation_subscription,
        }
    }

    fn reconcile(
        &mut self,
        next: Vec<OptionSnapshot>,
        ready: bool,
        enabled: bool,
        preferred: Option<&ElementId>,
        cx: &mut Context<Self>,
    ) {
        if !enabled {
            let changed = self.open || self.active_id.take().is_some();
            self.open = false;
            self.previous = next;
            self.pending_scroll = false;
            if changed {
                cx.notify();
            }
            return;
        }
        if !ready {
            let changed = self.active_id.take().is_some();
            self.previous = next;
            self.pending_scroll = false;
            if changed {
                cx.notify();
            }
            return;
        }

        let previous_active = self.active_id.clone();
        let options_changed = self.previous != next;
        if let Some(active) = self.active_id.as_ref()
            && next
                .iter()
                .any(|option| option.id == *active && !option.disabled)
        {
            self.previous = next;
            if options_changed {
                self.pending_scroll = true;
            }
            return;
        }

        self.active_id = reconciled_active_id(&self.previous, &next, self.active_id.as_ref());
        if self.open && self.active_id.is_none() {
            self.active_id = preferred
                .filter(|id| {
                    next.iter()
                        .any(|option| option.id == **id && !option.disabled)
                })
                .cloned()
                .or_else(|| {
                    next.iter()
                        .find(|option| !option.disabled)
                        .map(|option| option.id.clone())
                });
        }
        self.previous = next;
        if self.active_id != previous_active {
            self.pending_scroll = true;
            cx.notify();
        }
    }

    fn open_with(
        &mut self,
        preferred: Option<ElementId>,
        from_end: bool,
        scroll_to_active: bool,
        cx: &mut Context<Self>,
    ) {
        self.open = true;
        self.active_id = preferred
            .filter(|id| {
                self.previous
                    .iter()
                    .any(|option| option.id == *id && !option.disabled)
            })
            .or_else(|| {
                if from_end {
                    self.previous.iter().rev().find(|option| !option.disabled)
                } else {
                    self.previous.iter().find(|option| !option.disabled)
                }
                .map(|option| option.id.clone())
            });
        self.pending_scroll = scroll_to_active;
        cx.notify();
    }

    fn close(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.open {
            return false;
        }
        self.open = false;
        self.pending_scroll = false;
        cx.notify();
        true
    }

    fn move_active(&mut self, movement: ActiveMovement, cx: &mut Context<Self>) -> bool {
        let enabled = self
            .previous
            .iter()
            .filter(|option| !option.disabled)
            .collect::<Vec<_>>();
        if enabled.is_empty() {
            return false;
        }
        let current = self
            .active_id
            .as_ref()
            .and_then(|id| enabled.iter().position(|option| option.id == *id));
        let target = match movement {
            ActiveMovement::Previous => current.unwrap_or(0).saturating_sub(1),
            ActiveMovement::Next => current.map_or(0, |index| (index + 1).min(enabled.len() - 1)),
            ActiveMovement::First => 0,
            ActiveMovement::Last => enabled.len() - 1,
        };
        let next = enabled[target].id.clone();
        if self.active_id.as_ref() == Some(&next) {
            return true;
        }
        self.active_id = Some(next);
        self.pending_scroll = true;
        cx.notify();
        true
    }

    fn set_hovered(&mut self, id: ElementId, cx: &mut Context<Self>) {
        self.pending_scroll = false;
        if self.active_id.as_ref() != Some(&id) {
            self.active_id = Some(id);
            cx.notify();
        }
    }

    fn take_scroll_request(&mut self, id: &ElementId) -> bool {
        if self.pending_scroll && self.active_id.as_ref() == Some(id) {
            self.pending_scroll = false;
            true
        } else {
            false
        }
    }
}

fn reconciled_active_id(
    previous: &[OptionSnapshot],
    next: &[OptionSnapshot],
    active_id: Option<&ElementId>,
) -> Option<ElementId> {
    let old_position =
        active_id.and_then(|active| previous.iter().position(|option| option.id == *active))?;
    next.iter()
        .skip(old_position.min(next.len()))
        .find(|option| !option.disabled)
        .or_else(|| {
            next.iter()
                .take(old_position.min(next.len()))
                .rev()
                .find(|option| !option.disabled)
        })
        .map(|option| option.id.clone())
}

#[derive(Clone, Copy)]
enum ActiveMovement {
    Previous,
    Next,
    First,
    Last,
}

struct FlatOption<'a, T> {
    option: &'a SelectOption<T>,
    canonical: bool,
}

fn flat_options<T: PartialEq>(children: &[SelectChild<T>]) -> Vec<FlatOption<'_, T>> {
    let mut result: Vec<FlatOption<'_, T>> = Vec::new();
    for child in children {
        match child {
            SelectChild::Option(option) => {
                let canonical = !result.iter().any(|previous| {
                    previous.option.id == option.id || previous.option.value == option.value
                });
                result.push(FlatOption { option, canonical });
            }
            SelectChild::Group(group) => {
                for option in &group.options {
                    let canonical = !result.iter().any(|previous| {
                        previous.option.id == option.id || previous.option.value == option.value
                    });
                    result.push(FlatOption { option, canonical });
                }
            }
        }
    }
    result
}

impl<T> RenderOnce for Select<T>
where
    T: Clone + PartialEq + 'static,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let resolved_size = self.size.unwrap_or_else(|| component_size(cx));
        let theme = theme::current_theme(window, cx);
        let size = theme
            .select_size(resolved_size.token_key())
            .expect("Vektra 默认 Select size token 必须通过测试保持有效");
        let flat = flat_options(&self.children);
        let selected = self.selected_value.as_ref().and_then(|selected| {
            flat.iter()
                .find(|entry| entry.canonical && entry.option.value == *selected)
                .map(|entry| entry.option)
        });
        let selected_enabled_id = selected
            .filter(|option| !option.disabled)
            .map(|option| option.id.clone());
        let snapshots = flat
            .iter()
            .filter(|entry| entry.canonical)
            .map(|entry| OptionSnapshot {
                id: entry.option.id.clone(),
                disabled: entry.option.disabled,
            })
            .collect::<Vec<_>>();
        let canonical_values = flat
            .iter()
            .filter(|entry| entry.canonical && !entry.option.disabled)
            .map(|entry| (entry.option.id.clone(), entry.option.value.clone()))
            .collect::<Vec<_>>();
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
        let trigger_name = self
            .aria_label
            .clone()
            .or_else(|| display_label.clone())
            .unwrap_or_else(|| self.placeholder.clone());
        let popup_name = trigger_name.clone();
        let trigger_value = display_label
            .clone()
            .unwrap_or_else(|| self.placeholder.clone());
        let trigger_placeholder = display_label.is_none().then(|| self.placeholder.clone());

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
            .aria_value(trigger_value)
            .aria_expanded(is_open)
            .when_some(trigger_placeholder, |element, placeholder| {
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
                            let requested = state.active_id.as_ref().and_then(|active| {
                                click_values
                                    .iter()
                                    .find(|(id, _)| id == active)
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
            let popup = render_popup(
                self.children,
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
        }

        DisabledA11y::new(trigger, self.disabled, Some(trigger_bounds), None)
    }
}

fn handle_trigger_key(
    event: &KeyDownEvent,
    state: &WeakEntity<SelectInteractionState>,
    selected: Option<&ElementId>,
    window: &mut Window,
    cx: &mut App,
) {
    let modifiers = event.keystroke.modifiers;
    let key = event.keystroke.key.as_str();
    if key == "tab" && (modifiers == Modifiers::none() || modifiers == Modifiers::shift()) {
        let _ = state.update(cx, |state, cx| {
            state.close(cx);
        });
        return;
    }
    if modifiers != Modifiers::none() {
        return;
    }

    let handled = state
        .update(cx, |state, cx| match (state.open, key) {
            (false, "down") => {
                state.open_with(selected.cloned(), false, true, cx);
                true
            }
            (false, "up") => {
                state.open_with(selected.cloned(), true, true, cx);
                true
            }
            (true, "down") => state.move_active(ActiveMovement::Next, cx),
            (true, "up") => state.move_active(ActiveMovement::Previous, cx),
            (true, "home") => state.move_active(ActiveMovement::First, cx),
            (true, "end") => state.move_active(ActiveMovement::Last, cx),
            (true, "escape") => state.close(cx),
            _ => false,
        })
        .unwrap_or(false);
    if handled {
        window.prevent_default();
        cx.stop_propagation();
    }
}

#[derive(Clone, Copy)]
struct ResolvedTriggerStates {
    normal: SelectTriggerStateTokens,
    hover: SelectTriggerStateTokens,
    pressed: SelectTriggerStateTokens,
    focused: SelectTriggerStateTokens,
    open: SelectTriggerStateTokens,
    disabled: SelectTriggerStateTokens,
}

impl ResolvedTriggerStates {
    fn new(theme: &ResolvedTheme) -> Self {
        Self {
            normal: theme
                .select_trigger_state("normal")
                .expect("Select normal token"),
            hover: theme
                .select_trigger_state("hover")
                .expect("Select hover token"),
            pressed: theme
                .select_trigger_state("pressed")
                .expect("Select pressed token"),
            focused: theme
                .select_trigger_state("focus-visible")
                .expect("Select focus token"),
            open: theme
                .select_trigger_state("open")
                .expect("Select open token"),
            disabled: theme
                .select_trigger_state("disabled")
                .expect("Select disabled token"),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_popup<T>(
    children: Vec<SelectChild<T>>,
    status: SelectStatus,
    selected_value: Option<T>,
    active_id: Option<ElementId>,
    state: Entity<SelectInteractionState>,
    focus_handle: gpui::FocusHandle,
    scroll_handle: ScrollHandle,
    on_change: Option<ChangeHandler<T>>,
    size: SelectSizeTokens,
    theme: &ResolvedTheme,
    trigger_bounds_cell: Rc<Cell<Bounds<Pixels>>>,
    popup_name: SharedString,
) -> impl IntoElement
where
    T: Clone + PartialEq + 'static,
{
    let outside_state = state.downgrade();
    let mut content = div().flex().flex_col().p(theme.select.popup_padding);
    if status.is_ready() {
        let mut seen: Vec<(ElementId, T)> = Vec::new();
        let option_count = children
            .iter()
            .map(|child| match child {
                SelectChild::Option(_) => 1,
                SelectChild::Group(group) => group.options.len(),
            })
            .sum::<usize>();
        let mut option_position = 0;
        for child in children {
            match child {
                SelectChild::Option(option) => {
                    let canonical = !seen
                        .iter()
                        .any(|(id, value)| *id == option.id || *value == option.value);
                    seen.push((option.id.clone(), option.value.clone()));
                    content = content.child(render_option(
                        option,
                        canonical,
                        option_position,
                        option_count,
                        selected_value.as_ref(),
                        active_id.as_ref(),
                        &state,
                        &focus_handle,
                        on_change.clone(),
                        &scroll_handle,
                        size,
                        theme,
                    ));
                    option_position += 1;
                }
                SelectChild::Group(group) => {
                    let group_label = group.accessible_label();
                    let mut group_element = div()
                        .id(group.id.clone())
                        .debug_selector(|| "vektra-select-group".into())
                        .role(Role::Group)
                        .aria_label(group_label)
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .id((group.id.clone(), "label"))
                                .debug_selector(|| "vektra-select-group-label".into())
                                .role(Role::Label)
                                .px(size.option_padding_x)
                                .py(size.group_padding_y)
                                .text_size(size.description_font_size)
                                .line_height(size.description_line_height)
                                .text_color(theme.select.group_label)
                                .child(group.label),
                        );
                    for option in group.options {
                        let canonical = !seen
                            .iter()
                            .any(|(id, value)| *id == option.id || *value == option.value);
                        seen.push((option.id.clone(), option.value.clone()));
                        group_element = group_element.child(render_option(
                            option,
                            canonical,
                            option_position,
                            option_count,
                            selected_value.as_ref(),
                            active_id.as_ref(),
                            &state,
                            &focus_handle,
                            on_change.clone(),
                            &scroll_handle,
                            size,
                            theme,
                        ));
                        option_position += 1;
                    }
                    content = content.child(group_element);
                }
            }
        }
    } else {
        let (message, color, role, marker) = match status {
            SelectStatus::Loading(message) => {
                (message, theme.select.status_loading, Role::Status, "…")
            }
            SelectStatus::Empty(message) => (message, theme.select.status_empty, Role::Status, "—"),
            SelectStatus::Error(message) => (message, theme.select.status_error, Role::Alert, "!"),
            SelectStatus::Ready => unreachable!(),
        };
        content = content.child(
            div()
                .id("vektra-select-status")
                .debug_selector(|| "vektra-select-status".into())
                .role(role)
                .aria_label(message.clone())
                .flex()
                .items_center()
                .gap(size.content_gap)
                .px(size.option_padding_x)
                .py(size.option_padding_y)
                .text_color(color)
                .child(marker)
                .child(message),
        );
    }

    div()
        .id("vektra-select-popup")
        .debug_selector(|| "vektra-select-popup".into())
        .role(Role::ListBox)
        .aria_label(popup_name)
        .occlude()
        .w_full()
        .max_h_full()
        .flex()
        .flex_col()
        .border(theme.select.popup_border_width)
        .border_color(theme.select.popup_border)
        .rounded(theme.select.popup_radius)
        .bg(theme.select.popup_background)
        .shadow(vec![
            BoxShadow::new(
                Pixels::ZERO,
                theme.select.popup_shadow_offset_y,
                theme.select.popup_shadow_color.opacity(0.16),
            )
            .blur_radius(theme.select.popup_shadow_blur)
            .spread_radius(theme.select.popup_shadow_spread),
        ])
        .on_mouse_down_out(move |event, _, cx| {
            if trigger_bounds_cell.get().contains(&event.position) {
                return;
            }
            let _ = outside_state.update(cx, |state, cx| {
                state.close(cx);
            });
        })
        .child(
            content
                .flex_1()
                .min_h_0()
                .vertical_scrollbar_for(&scroll_handle)
                .scrollbar_id("vektra-select-scroll")
                .scrollbar_aria_label("选项列表"),
        )
}

#[allow(clippy::too_many_arguments)]
fn render_option<T>(
    option: SelectOption<T>,
    canonical: bool,
    position: usize,
    set_size: usize,
    selected_value: Option<&T>,
    active_id: Option<&ElementId>,
    state: &Entity<SelectInteractionState>,
    focus_handle: &gpui::FocusHandle,
    on_change: Option<ChangeHandler<T>>,
    scroll_handle: &ScrollHandle,
    size: SelectSizeTokens,
    theme: &ResolvedTheme,
) -> impl IntoElement
where
    T: Clone + PartialEq + 'static,
{
    let disabled = option.disabled || !canonical;
    let selected = canonical && selected_value == Some(&option.value);
    let active = canonical && active_id == Some(&option.id) && !disabled;
    let visible = if disabled {
        theme.select_option_state("disabled")
    } else if active {
        theme.select_option_state("active")
    } else if selected {
        theme.select_option_state("selected")
    } else {
        theme.select_option_state("normal")
    }
    .expect("Vektra 默认 Select option token 必须保持有效");
    let hover = theme
        .select_option_state("hover")
        .expect("Vektra 默认 Select option hover token 必须保持有效");
    let id = option.id.clone();
    let debug_id = option.id.clone();
    let accessible_label = option.accessible_label();
    let accessible_description = option.accessible_description();
    let value = option.value.clone();
    let scroll_request = ScrollRequest {
        state: state.downgrade(),
        id: id.clone(),
        handle: scroll_handle.clone(),
    };

    let mut element = div()
        .id(option.id)
        .debug_selector(move || format!("vektra-select-option-{debug_id}"))
        .role(Role::ListBoxOption)
        .aria_label(accessible_label)
        .aria_selected(selected)
        .aria_position_in_set(position + 1)
        .aria_size_of_set(set_size)
        .when_some(accessible_description, |element, description| {
            element.aria_description(description)
        })
        .when(active, |element| element.aria_active_descendant())
        .w_full()
        .min_w_0()
        .flex()
        .items_center()
        .gap(size.content_gap)
        .px(size.option_padding_x)
        .py(size.option_padding_y)
        .rounded(size.radius)
        .bg(visible.background)
        .text_color(visible.foreground)
        .cursor(if disabled {
            CursorStyle::OperationNotAllowed
        } else {
            CursorStyle::PointingHand
        })
        .when(!disabled, |element| {
            element.hover(move |style| style.bg(hover.background).text_color(hover.foreground))
        })
        .when_some(option.icon, |element, icon| {
            element.child(
                Icon::new(icon)
                    .size(size.icon_size)
                    .color(visible.foreground),
            )
        })
        .child(
            div()
                .min_w_0()
                .flex_1()
                .flex()
                .flex_col()
                .child(
                    div()
                        .truncate()
                        .text_size(size.font_size)
                        .line_height(size.line_height)
                        .child(option.label),
                )
                .when_some(option.description, |element, description| {
                    element.child(
                        div()
                            .truncate()
                            .text_size(size.description_font_size)
                            .line_height(size.description_line_height)
                            .text_color(visible.description)
                            .child(description),
                    )
                }),
        )
        .child(
            div()
                .flex_none()
                .size(size.indicator_size)
                .flex()
                .items_center()
                .justify_center()
                .when(selected, |indicator| {
                    indicator.child(
                        Icon::new(IconSource::asset(SELECT_CHECK_ICON))
                            .size(size.indicator_size)
                            .color(visible.indicator),
                    )
                }),
        );
    if disabled {
        return DisabledA11y::new(element, true, None, Some(scroll_request));
    }

    let hover_state = state.downgrade();
    let hover_id = id.clone();
    element = element.on_hover(move |hovered, _, cx| {
        if *hovered {
            let _ = hover_state.update(cx, |state, cx| state.set_hovered(hover_id.clone(), cx));
        }
    });
    let click_state = state.downgrade();
    let click_focus = focus_handle.clone();
    let current = selected_value.cloned();
    element = element
        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
        .on_click(move |_, window, cx| {
            let _ = click_state.update(cx, |state, cx| {
                state.close(cx);
            });
            click_focus.focus(window, cx);
            if current.as_ref() != Some(&value)
                && let Some(handler) = on_change.as_ref()
            {
                handler(value.clone(), window, cx);
            }
            cx.stop_propagation();
        });
    DisabledA11y::new(element, false, None, Some(scroll_request))
}

struct ScrollRequest {
    state: WeakEntity<SelectInteractionState>,
    id: ElementId,
    handle: ScrollHandle,
}

/// 为锁定 GPUI 高层元素尚未暴露的 AccessKit disabled 属性提供私有委托层。
struct DisabledA11y {
    inner: Stateful<Div>,
    disabled: bool,
    measured_bounds: Option<Rc<Cell<Bounds<Pixels>>>>,
    scroll_request: Option<ScrollRequest>,
}

impl DisabledA11y {
    fn new(
        inner: Stateful<Div>,
        disabled: bool,
        measured_bounds: Option<Rc<Cell<Bounds<Pixels>>>>,
        scroll_request: Option<ScrollRequest>,
    ) -> Self {
        Self {
            inner,
            disabled,
            measured_bounds,
            scroll_request,
        }
    }
}

impl IntoElement for DisabledA11y {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for DisabledA11y {
    type RequestLayoutState = <Stateful<Div> as Element>::RequestLayoutState;
    type PrepaintState = <Stateful<Div> as Element>::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        Element::id(&self.inner)
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }

    fn a11y_role(&self) -> Option<gpui::accesskit::Role> {
        self.inner.a11y_role()
    }

    fn write_a11y_info(&self, node: &mut gpui::accesskit::Node) {
        self.inner.write_a11y_info(node);
        if self.disabled {
            node.set_disabled();
        } else {
            node.clear_disabled();
        }
    }

    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut A11ySubtreeBuilder,
    ) {
        Element::a11y_synthetic_children(&mut self.inner, prepaint, builder);
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.inner
            .request_layout(global_id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if let Some(measured_bounds) = self.measured_bounds.as_ref()
            && measured_bounds.get() != bounds
        {
            measured_bounds.set(bounds);
            window.refresh();
        }
        if let Some(request) = self.scroll_request.as_ref() {
            let viewport = request.handle.bounds();
            if viewport.size.height > Pixels::ZERO {
                let should_scroll = request
                    .state
                    .update(cx, |state, _| state.take_scroll_request(&request.id))
                    .unwrap_or(false);
                if should_scroll {
                    let mut offset = request.handle.offset();
                    if bounds.top() < viewport.top() {
                        offset.y += viewport.top() - bounds.top();
                    } else if bounds.bottom() > viewport.bottom() {
                        offset.y += viewport.bottom() - bounds.bottom();
                    }
                    request.handle.set_offset(offset);
                    window.refresh();
                }
            }
        }
        self.inner
            .prepaint(global_id, inspector_id, bounds, layout, window, cx)
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner.paint(
            global_id,
            inspector_id,
            bounds,
            layout,
            prepaint,
            window,
            cx,
        );
    }
}

struct SelectPopupOverlay {
    body: Option<AnyElement>,
    trigger_bounds: Rc<Cell<Bounds<Pixels>>>,
    viewport_bounds: Bounds<Pixels>,
    anchor_gap: Pixels,
    viewport_padding: Pixels,
    max_height: Pixels,
}

struct PopupLayout {
    body: Option<AnyElement>,
    origin: Point<Pixels>,
}

impl IntoElement for SelectPopupOverlay {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SelectPopupOverlay {
    type RequestLayoutState = PopupLayout;
    type PrepaintState = AnyElement;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let trigger = self.trigger_bounds.get();
        let safe_top = self.viewport_bounds.top() + self.viewport_padding;
        let safe_bottom = self.viewport_bounds.bottom() - self.viewport_padding;
        let below = (safe_bottom - trigger.bottom() - self.anchor_gap).max(Pixels::ZERO);
        let above = (trigger.top() - safe_top - self.anchor_gap).max(Pixels::ZERO);
        let open_above = below < self.max_height.min(above) && above > below;
        let available_height = self.max_height.min(if open_above { above } else { below });
        let available_width =
            (self.viewport_bounds.size.width - self.viewport_padding * 2.).max(Pixels::ZERO);
        let popup_width = trigger.size.width.min(available_width);
        let mut body = self.body.take().expect("Select Popup 每帧只允许布局一次");
        let body_size = body.layout_as_root(
            Size {
                width: AvailableSpace::Definite(popup_width),
                height: AvailableSpace::Definite(available_height),
            },
            window,
            cx,
        );
        let safe_left = self.viewport_bounds.left() + self.viewport_padding;
        let x = trigger
            .left()
            .min(self.viewport_bounds.right() - self.viewport_padding - body_size.width)
            .max(safe_left);
        let y = if open_above {
            trigger.top() - self.anchor_gap - body_size.height
        } else {
            trigger.bottom() + self.anchor_gap
        };
        let layout_id = window.request_layout(Style::default(), [], cx);
        (
            layout_id,
            PopupLayout {
                body: Some(body),
                origin: point(x, y.max(safe_top)),
            },
        )
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let mut body = layout.body.take().expect("Select Popup 必须先完成布局");
        body.prepaint_at(layout.origin, window, cx);
        body
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        body: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        body.paint(window, cx);
    }
}

#[cfg(test)]
#[path = "../tests/unit/select.rs"]
mod tests;
