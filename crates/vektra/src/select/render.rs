//! Select 的键盘分发、Popup、group 与 option 渲染。

use super::*;

pub(super) fn handle_trigger_key(
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
    if let Some(text) = typeahead_text(event) {
        let handled = state
            .update(cx, |state, cx| state.typeahead(&text, cx))
            .unwrap_or(false);
        if handled {
            window.prevent_default();
            cx.stop_propagation();
        }
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
            (true, "pageup") => state.move_active(ActiveMovement::PagePrevious, cx),
            (true, "pagedown") => state.move_active(ActiveMovement::PageNext, cx),
            (true, "escape") => state.close(cx),
            _ => false,
        })
        .unwrap_or(false);
    if handled {
        window.prevent_default();
        cx.stop_propagation();
    }
}

fn typeahead_text(event: &KeyDownEvent) -> Option<String> {
    let modifiers = event.keystroke.modifiers;
    if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
        return None;
    }
    if let Some(text) = event.keystroke.key_char.as_deref()
        && !text.chars().any(char::is_control)
    {
        return Some(text.to_owned());
    }
    let key = event.keystroke.key.as_str();
    if key == "space" {
        return Some(" ".to_owned());
    }
    (key.graphemes(true).count() == 1 && !key.chars().any(char::is_control)).then(|| key.to_owned())
}

#[derive(Clone, Copy)]
pub(super) struct ResolvedTriggerStates {
    pub(super) normal: SelectTriggerStateTokens,
    pub(super) hover: SelectTriggerStateTokens,
    pub(super) pressed: SelectTriggerStateTokens,
    pub(super) focused: SelectTriggerStateTokens,
    pub(super) open: SelectTriggerStateTokens,
    pub(super) disabled: SelectTriggerStateTokens,
}

impl ResolvedTriggerStates {
    pub(super) fn new(theme: &ResolvedTheme) -> Self {
        Self {
            normal: theme.select_trigger_state(SelectTriggerState::Normal),
            hover: theme.select_trigger_state(SelectTriggerState::Hover),
            pressed: theme.select_trigger_state(SelectTriggerState::Pressed),
            focused: theme.select_trigger_state(SelectTriggerState::FocusVisible),
            open: theme.select_trigger_state(SelectTriggerState::Open),
            disabled: theme.select_trigger_state(SelectTriggerState::Disabled),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_popup<T>(
    children: Vec<SelectChild<T>>,
    catalog: Vec<OptionMetadata<T>>,
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
        let mut metadata = catalog.into_iter();
        for child in children {
            match child {
                SelectChild::Option(option) => {
                    let metadata = metadata
                        .next()
                        .expect("Select catalog 必须与渲染 option 一一对应");
                    content = content.child(render_option(
                        option,
                        metadata,
                        selected_value.as_ref(),
                        active_id.as_ref(),
                        &state,
                        &focus_handle,
                        on_change.clone(),
                        &scroll_handle,
                        size,
                        theme,
                    ));
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
                        let metadata = metadata
                            .next()
                            .expect("Select catalog 必须与分组 option 一一对应");
                        group_element = group_element.child(render_option(
                            option,
                            metadata,
                            selected_value.as_ref(),
                            active_id.as_ref(),
                            &state,
                            &focus_handle,
                            on_change.clone(),
                            &scroll_handle,
                            size,
                            theme,
                        ));
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

    let popup = div()
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
        );

    DisabledA11y::new(popup, false, None, None)
}

#[allow(clippy::too_many_arguments)]
fn render_option<T>(
    option: SelectOption<T>,
    metadata: OptionMetadata<T>,
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
    let disabled = metadata.disabled;
    let selected = metadata.canonical && selected_value == Some(&metadata.value);
    let active = metadata.canonical && active_id == Some(&metadata.id) && !disabled;
    let visible = if disabled {
        theme.select_option_state(SelectOptionState::Disabled)
    } else if active {
        theme.select_option_state(SelectOptionState::Active)
    } else if selected {
        theme.select_option_state(SelectOptionState::Selected)
    } else {
        theme.select_option_state(SelectOptionState::Normal)
    };
    let hover = theme.select_option_state(SelectOptionState::Hover);
    let id = metadata.id.clone();
    let debug_id = metadata.id.clone();
    let accessible_label = metadata.accessible_name.clone();
    let accessible_description = option.accessible_description();
    let value = metadata.value.clone();
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
        .aria_position_in_set(metadata.position + 1)
        .aria_size_of_set(metadata.set_size)
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
    let click_id = id.clone();
    let click_focus = focus_handle.clone();
    let current = selected_value.cloned();
    element = element
        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
        .on_click(move |_, window, cx| {
            let can_submit = click_state
                .update(cx, |state, cx| {
                    let can_submit = state.can_submit(&click_id);
                    if can_submit {
                        state.close(cx);
                    }
                    can_submit
                })
                .unwrap_or(false);
            if !can_submit {
                return;
            }
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
