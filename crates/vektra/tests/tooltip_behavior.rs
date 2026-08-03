use gpui::{
    App, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding, Modifiers,
    ParentElement, Render, Styled, TestAppContext, Window, actions, div, px,
};
use vektra::{Button, IconButton, IconSource, Tooltip};

actions!(tooltip_behavior_test, [Tab, TabPrev]);

struct NavigationView {
    focus_handle: FocusHandle,
    activations: Vec<&'static str>,
}

impl NavigationView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            focus_handle,
            activations: Vec::new(),
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}

impl Render for NavigationView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("navigation-root")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .child(
                Button::new("first")
                    .label("第一个")
                    .tooltip("第一个提示")
                    .on_click_in(cx, |this, _, _, cx| {
                        this.activations.push("first");
                        cx.notify();
                    }),
            )
            .child(
                Button::new("disabled")
                    .label("禁用")
                    .tooltip("禁用原因")
                    .disabled(true),
            )
            .child(
                Button::new("last")
                    .label("最后一个")
                    .tooltip("最后一个提示")
                    .on_click_in(cx, |this, _, _, cx| {
                        this.activations.push("last");
                        cx.notify();
                    }),
            )
            .child(
                IconButton::new("icon", IconSource::asset("icons/settings.svg"))
                    .aria_label("图标按钮")
                    .tooltip("图标按钮提示")
                    .on_click_in(cx, |this, _, _, cx| {
                        this.activations.push("icon");
                        cx.notify();
                    }),
            )
    }
}

#[gpui::test]
fn real_tab_and_shift_tab_bindings_move_focus_and_skip_disabled(cx: &mut TestAppContext) {
    cx.update(|cx| {
        bind_keys(cx);
        cx.activate(true);
    });
    let (view, cx) = cx.add_window_view(NavigationView::new);
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|window, cx| {
        assert!(view.read(cx).focus_handle.is_focused(window));
    });
    cx.update(|window, cx| {
        window.activate_window();
        let focus_handle = view.read(cx).focus_handle.clone();
        window.focus(&focus_handle, cx);
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));

    cx.simulate_keystrokes("tab enter tab enter tab enter shift-tab enter");

    assert_eq!(
        view.read_with(cx, |view, _| view.activations.clone()),
        ["first", "last", "icon", "last"]
    );
}

struct ManyTooltips;

impl Render for ManyTooltips {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().children((0..300).map(|index| {
            Button::new(format!("button-{index}"))
                .label(format!("按钮 {index}"))
                .tooltip(format!("提示 {index}"))
        }))
    }
}

#[gpui::test]
fn hundreds_of_configured_triggers_render_without_panicking(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| ManyTooltips);
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

struct SingleTooltip {
    placement: vektra::TooltipPlacement,
}

struct ControlledTooltip {
    open: bool,
}

impl Render for ControlledTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().p(px(100.)).child(
            Button::new("controlled-trigger")
                .label("受控")
                .tooltip(Tooltip::new("常驻提示").open(self.open).animated(false)),
        )
    }
}

#[gpui::test]
fn controlled_open_renders_without_hover_and_escape_needs_false_true(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| ControlledTooltip { open: true });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_some());

    cx.simulate_keystrokes("escape");
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_none());

    view.update(cx, |view, cx| {
        view.open = true;
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_none());

    view.update(cx, |view, cx| {
        view.open = false;
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_none());

    view.update(cx, |view, cx| {
        view.open = true;
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_some());
}

#[gpui::test]
fn controlled_false_blocks_hover_and_focus(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| ControlledTooltip { open: false });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = cx.debug_bounds("vektra-button").unwrap();
    cx.simulate_mouse_move(trigger.center(), None, Modifiers::none());
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));

    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_none());
}

impl Render for SingleTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().flex().p(px(100.)).child(
            div()
                .id("trigger-wrapper")
                .debug_selector(|| "trigger-wrapper".into())
                .flex_none()
                .w(px(120.))
                .child(
                    Button::new("bounded-trigger")
                        .label("Trigger")
                        .width(px(120.))
                        .tooltip("Bounds-based Tooltip")
                        .tooltip_placement(self.placement),
                ),
        )
    }
}

struct FocusAndHoverTooltips {
    focus_handle: FocusHandle,
}

impl FocusAndHoverTooltips {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self { focus_handle }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
}

impl Render for FocusAndHoverTooltips {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_tab))
            .flex()
            .flex_col()
            .gap(px(100.))
            .child(
                div().debug_selector(|| "focus-trigger".into()).child(
                    Button::new("focus-button")
                        .label("Focus")
                        .tooltip("Focus Tooltip"),
                ),
            )
            .child(
                div().debug_selector(|| "hover-trigger".into()).child(
                    Button::new("hover-button")
                        .label("Hover")
                        .tooltip("Hover Tooltip"),
                ),
            )
    }
}

#[gpui::test]
fn visible_tooltip_uses_trigger_bounds_and_does_not_cover_the_button(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| SingleTooltip {
        placement: vektra::TooltipPlacement::BottomStart,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = cx.debug_bounds("vektra-button").unwrap();
    cx.simulate_mouse_move(trigger.center(), None, Modifiers::none());
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));

    let tooltip = cx.debug_bounds("vektra-tooltip-bubble").unwrap();
    assert!(tooltip.top() > trigger.bottom());
    assert_eq!(tooltip.left(), trigger.left());
    assert!(
        tooltip.left() >= trigger.right()
            || tooltip.right() <= trigger.left()
            || tooltip.top() >= trigger.bottom()
            || tooltip.bottom() <= trigger.top()
    );
}

#[gpui::test]
fn pointer_can_cross_the_gap_and_keep_the_actual_bubble_visible(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| SingleTooltip {
        placement: vektra::TooltipPlacement::BottomStart,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = cx.debug_bounds("vektra-button").unwrap();
    cx.simulate_mouse_move(trigger.center(), None, Modifiers::none());
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));

    let bubble = cx.debug_bounds("vektra-tooltip-bubble").unwrap();
    assert!(bubble.top() > trigger.bottom());

    let gap = gpui::point(trigger.center().x, (trigger.bottom() + bubble.top()) / 2.);
    cx.simulate_mouse_move(gap, None, Modifiers::none());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(250));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_some());

    cx.simulate_mouse_move(bubble.center(), None, Modifiers::none());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_some());

    cx.simulate_mouse_move(gpui::point(px(1.), px(1.)), None, Modifiers::none());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_some());

    cx.executor()
        .advance_clock(std::time::Duration::from_millis(80));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_none());
}

#[gpui::test]
fn end_placement_aligns_the_tooltip_and_trigger_right_edges(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| SingleTooltip {
        placement: vektra::TooltipPlacement::BottomEnd,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = cx.debug_bounds("vektra-button").unwrap();
    cx.simulate_mouse_move(trigger.center(), None, Modifiers::none());
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));

    let tooltip = cx.debug_bounds("vektra-tooltip-bubble").unwrap();
    assert!(tooltip.top() > trigger.bottom());
    assert_eq!(tooltip.right(), trigger.right());
}

#[gpui::test]
fn center_placement_aligns_the_tooltip_and_trigger_centers(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| SingleTooltip {
        placement: vektra::TooltipPlacement::Bottom,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = cx.debug_bounds("vektra-button").unwrap();
    cx.simulate_mouse_move(trigger.center(), None, Modifiers::none());
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));

    let tooltip = cx.debug_bounds("vektra-tooltip-bubble").unwrap();
    assert!(tooltip.top() > trigger.bottom());
    assert_eq!(tooltip.center().x, trigger.center().x);
}

#[gpui::test]
fn hovered_trigger_wins_after_another_trigger_was_keyboard_eligible(cx: &mut TestAppContext) {
    cx.update(|cx| {
        bind_keys(cx);
        cx.activate(true);
    });
    let (_, cx) = cx.add_window_view(FocusAndHoverTooltips::new);
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_keystrokes("tab");
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));

    let hover_trigger = cx.debug_bounds("hover-trigger").unwrap();
    cx.simulate_mouse_move(hover_trigger.center(), None, Modifiers::none());
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));

    let tooltip = cx.debug_bounds("vektra-tooltip-bubble").unwrap();
    assert!(tooltip.top() > hover_trigger.bottom());
}

fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", Tab, None),
        KeyBinding::new("shift-tab", TabPrev, None),
    ]);
}
