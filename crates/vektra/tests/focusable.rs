use gpui::{
    App, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding, KeyUpEvent, Keystroke,
    Modifiers, ParentElement, Render, Styled, TestAppContext, Window, actions, div,
    prelude::FluentBuilder, px,
};
use std::{cell::RefCell, rc::Rc, time::Duration};
use vektra::{Button, Checkbox, Focusable, IconButton, IconSource};

actions!(focusable_test, [Tab]);

#[derive(Debug, Clone, Copy)]
enum Target {
    Button,
    IconButton,
    Checkbox,
}

fn assert_focusable<C: Focusable>(component: C) -> C {
    component.on_focus(|_, _| {}).on_blur(|_, _| {})
}

#[test]
fn focusable_is_exported_from_root_and_traits_module() {
    fn assert_traits_path<C: vektra::traits::Focusable>(component: C) -> C {
        component.on_focus(|_, _| {}).on_blur(|_, _| {})
    }

    let _ = assert_focusable(Button::new("root-button"));
    let _ = assert_focusable(IconButton::new(
        "root-icon-button",
        IconSource::asset("icons/settings.svg"),
    ));
    let _ = assert_focusable(Checkbox::new("root-checkbox"));
    let _ = assert_traits_path(Button::new("traits-button"));
}

struct OrdinaryFocusView {
    target: Target,
    root_focus: FocusHandle,
    log: Rc<RefCell<Vec<&'static str>>>,
}

impl OrdinaryFocusView {
    fn new(
        target: Target,
        log: Rc<RefCell<Vec<&'static str>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            target,
            root_focus,
            log,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
}

impl Render for OrdinaryFocusView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_log = self.log.clone();
        let blur_log = self.log.clone();
        let target = match self.target {
            Target::Button => Button::new("target")
                .label("Target")
                .on_focus(move |_, _| focus_log.borrow_mut().push("target-focus"))
                .on_blur(move |_, _| blur_log.borrow_mut().push("target-blur"))
                .into_any_element(),
            Target::IconButton => {
                IconButton::new("target", IconSource::asset("icons/settings.svg"))
                    .aria_label("Target")
                    .on_focus(move |_, _| focus_log.borrow_mut().push("target-focus"))
                    .on_blur(move |_, _| blur_log.borrow_mut().push("target-blur"))
                    .into_any_element()
            }
            Target::Checkbox => Checkbox::new("target")
                .label("Target")
                .on_focus(move |_, _| focus_log.borrow_mut().push("target-focus"))
                .on_blur(move |_, _| blur_log.borrow_mut().push("target-blur"))
                .into_any_element(),
        };
        let other_log = self.log.clone();

        div()
            .id("focus-root")
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::on_tab))
            .child(target)
            .child(
                Button::new("other")
                    .label("Other")
                    .on_focus(move |_, _| other_log.borrow_mut().push("other-focus")),
            )
    }
}

fn bind_tab(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("tab", Tab, None)]);
    cx.activate(true);
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn activate_root(cx: &mut gpui::VisualTestContext, focus_handle: FocusHandle) {
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&focus_handle, cx);
    });
    draw(cx);
}

fn assert_ordinary_focus_and_blur(cx: &mut TestAppContext, target: Target) {
    cx.update(bind_tab);
    let log = Rc::new(RefCell::new(Vec::new()));
    let view_log = log.clone();
    let (view, cx) =
        cx.add_window_view(move |window, cx| OrdinaryFocusView::new(target, view_log, window, cx));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    activate_root(cx, root_focus);

    cx.simulate_keystrokes("tab");
    draw(cx);
    cx.simulate_keystrokes("tab");

    assert_eq!(
        log.borrow().as_slice(),
        ["target-focus", "target-blur", "other-focus"]
    );
}

#[gpui::test]
fn button_on_focus_and_on_blur_follow_real_tab_transitions(cx: &mut TestAppContext) {
    assert_ordinary_focus_and_blur(cx, Target::Button);
}

#[gpui::test]
fn icon_button_on_focus_and_on_blur_follow_real_tab_transitions(cx: &mut TestAppContext) {
    assert_ordinary_focus_and_blur(cx, Target::IconButton);
}

#[gpui::test]
fn checkbox_on_focus_and_on_blur_follow_real_tab_transitions(cx: &mut TestAppContext) {
    assert_ordinary_focus_and_blur(cx, Target::Checkbox);
}

#[gpui::test]
fn programmatic_gpui_focus_uses_the_same_component_lifecycle(cx: &mut TestAppContext) {
    cx.update(bind_tab);
    let log = Rc::new(RefCell::new(Vec::new()));
    let view_log = log.clone();
    let (view, cx) = cx.add_window_view(move |window, cx| {
        OrdinaryFocusView::new(Target::Button, view_log, window, cx)
    });
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    activate_root(cx, root_focus);

    cx.simulate_keystrokes("tab");
    draw(cx);
    let target_focus = cx.update(|window, cx| window.focused(cx).unwrap());
    cx.simulate_keystrokes("tab");
    draw(cx);
    cx.update(|window, cx| window.focus(&target_focus, cx));

    assert_eq!(
        log.borrow().as_slice(),
        ["target-focus", "target-blur", "other-focus", "target-focus",]
    );
}

struct EntityFocusView {
    target: Target,
    root_focus: FocusHandle,
    focus_count: usize,
    blur_count: usize,
    notify_count: usize,
}

impl EntityFocusView {
    fn new(target: Target, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            target,
            root_focus,
            focus_count: 0,
            blur_count: 0,
            notify_count: 0,
        }
    }

    fn focused(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.focus_count += 1;
        self.notify_count += 1;
        cx.notify();
    }

    fn blurred(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.blur_count += 1;
        self.notify_count += 1;
        cx.notify();
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
}

impl Render for EntityFocusView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target = match self.target {
            Target::Button => Button::new("target")
                .label("Target")
                .on_focus_in(cx, Self::focused)
                .on_blur_in(cx, Self::blurred)
                .into_any_element(),
            Target::IconButton => {
                IconButton::new("target", IconSource::asset("icons/settings.svg"))
                    .aria_label("Target")
                    .on_focus_in(cx, Self::focused)
                    .on_blur_in(cx, Self::blurred)
                    .into_any_element()
            }
            Target::Checkbox => Checkbox::new("target")
                .label("Target")
                .on_focus_in(cx, Self::focused)
                .on_blur_in(cx, Self::blurred)
                .into_any_element(),
        };

        div()
            .id("entity-focus-root")
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::on_tab))
            .child(target)
            .child(Button::new("other").label("Other"))
    }
}

fn assert_entity_callbacks(cx: &mut TestAppContext, target: Target) {
    cx.update(bind_tab);
    let (view, cx) = cx.add_window_view(move |window, cx| EntityFocusView::new(target, window, cx));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    activate_root(cx, root_focus);
    cx.simulate_keystrokes("tab");
    draw(cx);
    cx.simulate_keystrokes("tab");

    assert_eq!(
        view.read_with(cx, |view, _| (
            view.focus_count,
            view.blur_count,
            view.notify_count,
        )),
        (1, 1, 2)
    );
}

#[gpui::test]
fn button_entity_bound_focus_callbacks_update_and_notify(cx: &mut TestAppContext) {
    assert_entity_callbacks(cx, Target::Button);
}

#[gpui::test]
fn icon_button_entity_bound_focus_callbacks_update_and_notify(cx: &mut TestAppContext) {
    assert_entity_callbacks(cx, Target::IconButton);
}

#[gpui::test]
fn checkbox_entity_bound_focus_callbacks_update_and_notify(cx: &mut TestAppContext) {
    assert_entity_callbacks(cx, Target::Checkbox);
}

struct RerenderView {
    root_focus: FocusHandle,
    generation: usize,
    log: Vec<(&'static str, usize)>,
}

impl RerenderView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            root_focus,
            generation: 1,
            log: Vec::new(),
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
}

impl Render for RerenderView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_generation = self.generation;
        let blur_generation = self.generation;
        div()
            .id("rerender-root")
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::on_tab))
            .child(
                Button::new("stable-target")
                    .label("Stable")
                    .on_focus_in(cx, move |this, _, _| {
                        this.log.push(("focus", focus_generation));
                    })
                    .on_blur_in(cx, move |this, _, _| {
                        this.log.push(("blur", blur_generation));
                    }),
            )
            .child(Button::new("other").label("Other"))
    }
}

#[gpui::test]
fn rerender_keeps_one_subscription_and_uses_latest_handlers(cx: &mut TestAppContext) {
    cx.update(bind_tab);
    let (view, cx) = cx.add_window_view(RerenderView::new);
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    activate_root(cx, root_focus);
    cx.simulate_keystrokes("tab");

    view.update(cx, |view, cx| {
        view.generation = 2;
        cx.notify();
    });
    draw(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.log.clone()),
        [("focus", 1)]
    );

    cx.simulate_keystrokes("tab tab");
    assert_eq!(
        view.read_with(cx, |view, _| view.log.clone()),
        [("focus", 1), ("blur", 2), ("focus", 2)]
    );
}

struct DisabledView {
    target: Target,
    root_focus: FocusHandle,
    disabled: bool,
    focus_count: usize,
    blur_count: usize,
    other_focus_count: usize,
}

impl DisabledView {
    fn new(target: Target, disabled: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            target,
            root_focus,
            disabled,
            focus_count: 0,
            blur_count: 0,
            other_focus_count: 0,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
}

impl Render for DisabledView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target = match self.target {
            Target::Button => Button::new("target")
                .label("Target")
                .disabled(self.disabled)
                .on_focus_in(cx, |this, _, _| this.focus_count += 1)
                .on_blur_in(cx, |this, _, _| this.blur_count += 1)
                .into_any_element(),
            Target::IconButton => {
                IconButton::new("target", IconSource::asset("icons/settings.svg"))
                    .aria_label("Target")
                    .disabled(self.disabled)
                    .on_focus_in(cx, |this, _, _| this.focus_count += 1)
                    .on_blur_in(cx, |this, _, _| this.blur_count += 1)
                    .into_any_element()
            }
            Target::Checkbox => Checkbox::new("target")
                .label("Target")
                .disabled(self.disabled)
                .on_focus_in(cx, |this, _, _| this.focus_count += 1)
                .on_blur_in(cx, |this, _, _| this.blur_count += 1)
                .into_any_element(),
        };

        div()
            .id("disabled-root")
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::on_tab))
            .child(target)
            .child(
                Button::new("other")
                    .label("Other")
                    .on_focus_in(cx, |this, _, _| this.other_focus_count += 1),
            )
    }
}

fn assert_disabled_is_skipped(cx: &mut TestAppContext, target: Target) {
    cx.update(bind_tab);
    let (view, cx) =
        cx.add_window_view(move |window, cx| DisabledView::new(target, true, window, cx));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    activate_root(cx, root_focus);
    cx.simulate_keystrokes("tab");
    assert_eq!(
        view.read_with(cx, |view, _| (view.focus_count, view.other_focus_count)),
        (0, 1)
    );
}

#[gpui::test]
fn disabled_button_is_not_a_tab_stop(cx: &mut TestAppContext) {
    assert_disabled_is_skipped(cx, Target::Button);
}

#[gpui::test]
fn disabled_icon_button_is_not_a_tab_stop(cx: &mut TestAppContext) {
    assert_disabled_is_skipped(cx, Target::IconButton);
}

#[gpui::test]
fn disabled_checkbox_is_not_a_tab_stop(cx: &mut TestAppContext) {
    assert_disabled_is_skipped(cx, Target::Checkbox);
}

#[gpui::test]
fn focused_component_blurs_once_when_it_becomes_disabled(cx: &mut TestAppContext) {
    cx.update(bind_tab);
    let (view, cx) =
        cx.add_window_view(|window, cx| DisabledView::new(Target::Button, false, window, cx));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    activate_root(cx, root_focus);
    cx.simulate_keystrokes("tab");

    view.update(cx, |view, cx| {
        view.disabled = true;
        cx.notify();
    });
    draw(cx);

    assert_eq!(
        view.read_with(cx, |view, _| (view.focus_count, view.blur_count)),
        (1, 1)
    );
}

struct RemovalView {
    root_focus: FocusHandle,
    present: bool,
    focus_count: usize,
    blur_count: usize,
}

impl RemovalView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            root_focus,
            present: true,
            focus_count: 0,
            blur_count: 0,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
}

impl Render for RemovalView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("removal-root")
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::on_tab))
            .when(self.present, |root| {
                root.child(
                    Button::new("removable")
                        .label("Removable")
                        .on_focus_in(cx, |this, _, _| this.focus_count += 1)
                        .on_blur_in(cx, |this, _, _| this.blur_count += 1),
                )
            })
    }
}

#[gpui::test]
fn removing_a_focused_component_drops_its_listener_before_gpui_clears_focus(
    cx: &mut TestAppContext,
) {
    cx.update(bind_tab);
    let (view, cx) = cx.add_window_view(RemovalView::new);
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    activate_root(cx, root_focus);
    cx.simulate_keystrokes("tab");

    view.update(cx, |view, cx| {
        view.present = false;
        cx.notify();
    });
    draw(cx);

    assert_eq!(
        view.read_with(cx, |view, _| (view.focus_count, view.blur_count)),
        (1, 0)
    );
}

struct CheckboxIndependenceView {
    root_focus: FocusHandle,
    checked: bool,
    focus_count: usize,
    blur_count: usize,
    change_count: usize,
}

impl CheckboxIndependenceView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            root_focus,
            checked: false,
            focus_count: 0,
            blur_count: 0,
            change_count: 0,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
}

impl Render for CheckboxIndependenceView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("checkbox-independence-root")
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::on_tab))
            .child(
                Checkbox::new("checkbox")
                    .label("Checkbox")
                    .checked(self.checked)
                    .on_focus_in(cx, |this, _, _| this.focus_count += 1)
                    .on_blur_in(cx, |this, _, _| this.blur_count += 1)
                    .on_change_in(cx, |this, next, _, _, cx| {
                        this.checked = next;
                        this.change_count += 1;
                        cx.notify();
                    }),
            )
            .child(Button::new("other").label("Other"))
    }
}

#[gpui::test]
fn checkbox_change_and_focus_lifecycles_are_independent(cx: &mut TestAppContext) {
    cx.update(bind_tab);
    let (view, cx) = cx.add_window_view(CheckboxIndependenceView::new);
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    activate_root(cx, root_focus);
    cx.simulate_keystrokes("tab");
    draw(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });
    draw(cx);
    assert_eq!(
        view.read_with(cx, |view, _| (
            view.focus_count,
            view.blur_count,
            view.change_count,
        )),
        (1, 0, 1)
    );

    cx.simulate_keystrokes("tab");
    assert_eq!(
        view.read_with(cx, |view, _| (
            view.focus_count,
            view.blur_count,
            view.change_count,
        )),
        (1, 1, 1)
    );
}

struct TooltipFocusView {
    icon: bool,
    tooltip: bool,
    root_focus: FocusHandle,
    focus_count: usize,
    blur_count: usize,
}

impl TooltipFocusView {
    fn new(icon: bool, tooltip: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            icon,
            tooltip,
            root_focus,
            focus_count: 0,
            blur_count: 0,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
}

impl Render for TooltipFocusView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target = if self.icon {
            IconButton::new("target", IconSource::asset("icons/settings.svg"))
                .aria_label("Target")
                .when(self.tooltip, |button| button.tooltip("Tooltip"))
                .on_focus_in(cx, |this, _, _| this.focus_count += 1)
                .on_blur_in(cx, |this, _, _| this.blur_count += 1)
                .into_any_element()
        } else {
            Button::new("target")
                .label("Target")
                .when(self.tooltip, |button| button.tooltip("Tooltip"))
                .on_focus_in(cx, |this, _, _| this.focus_count += 1)
                .on_blur_in(cx, |this, _, _| this.blur_count += 1)
                .into_any_element()
        };

        div()
            .id("tooltip-focus-root")
            .p(px(100.))
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::on_tab))
            .child(target)
            .child(Button::new("other").label("Other"))
    }
}

fn assert_tooltip_focus_path(cx: &mut TestAppContext, icon: bool, tooltip: bool) {
    cx.update(bind_tab);
    let (view, cx) =
        cx.add_window_view(move |window, cx| TooltipFocusView::new(icon, tooltip, window, cx));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    activate_root(cx, root_focus);
    cx.simulate_keystrokes("tab");
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.run_until_parked();
    draw(cx);
    assert_eq!(cx.debug_bounds("vektra-tooltip-bubble").is_some(), tooltip);

    cx.simulate_keystrokes("tab");
    assert_eq!(
        view.read_with(cx, |view, _| (view.focus_count, view.blur_count)),
        (1, 1)
    );
}

#[gpui::test]
fn button_without_tooltip_has_one_focus_lifecycle(cx: &mut TestAppContext) {
    assert_tooltip_focus_path(cx, false, false);
}

#[gpui::test]
fn button_with_tooltip_reuses_the_business_focus_lifecycle(cx: &mut TestAppContext) {
    assert_tooltip_focus_path(cx, false, true);
}

#[gpui::test]
fn icon_button_without_tooltip_has_one_focus_lifecycle(cx: &mut TestAppContext) {
    assert_tooltip_focus_path(cx, true, false);
}

#[gpui::test]
fn icon_button_with_tooltip_reuses_the_business_focus_lifecycle(cx: &mut TestAppContext) {
    assert_tooltip_focus_path(cx, true, true);
}

#[gpui::test]
fn mouse_down_focuses_an_enabled_component_once(cx: &mut TestAppContext) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let view_log = log.clone();
    let (view, cx) = cx.add_window_view(move |window, cx| {
        OrdinaryFocusView::new(Target::Checkbox, view_log, window, cx)
    });
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    activate_root(cx, root_focus);
    let bounds = cx.debug_bounds("vektra-checkbox").unwrap();

    cx.simulate_click(bounds.center(), Modifiers::none());

    assert_eq!(log.borrow().as_slice(), ["target-focus"]);
}
