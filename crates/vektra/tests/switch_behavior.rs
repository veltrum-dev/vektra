use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding, KeyDownEvent, KeyUpEvent,
    Keystroke, Modifiers, ParentElement, Render, StatefulInteractiveElement, Styled,
    TestAppContext, Window, actions, div, point, px,
};
use std::{cell::RefCell, rc::Rc, time::Duration};
use vektra::{
    Changeable, Clickable, ComponentSize, Disableable, Focusable, IconSource, Sizable, Switch,
    SwitchContent, ThemeMode, set_theme_mode,
};

actions!(switch_behavior_test, [Tab]);

#[test]
fn switch_is_a_root_export_with_inherent_forwarding_builders() {
    fn disable<C: Disableable>(component: C) -> C {
        component.disabled(true)
    }
    fn sizable<C: Sizable>(component: C) -> C {
        component.size(ComponentSize::Sm)
    }
    fn focusable<C: Focusable>(component: C) -> C {
        component.on_focus(|_, _| {}).on_blur(|_, _| {})
    }
    fn clickable<C: Clickable>(component: C) -> C {
        component
            .cursor_style(gpui::CursorStyle::PointingHand)
            .on_click(|_, _, _| {})
    }
    fn changeable<C: Changeable<bool>>(component: C) -> C {
        component.on_change(|_, _, _| {})
    }

    let _ = changeable(clickable(focusable(sizable(disable(
        Switch::new("public")
            .checked(true)
            .label("通知")
            .aria_label("推送通知")
            .aria_description("立即更新")
            .cursor_style(gpui::CursorStyle::PointingHand)
            .checked_content(SwitchContent::icon_text(
                IconSource::asset("components/checkbox/check.svg"),
                "开启",
            ))
            .unchecked_content(SwitchContent::text("关闭"))
            .loading(true)
            .loading(false)
            .transition_duration(Duration::from_millis(240))
            .on_change(|_, _, _| {})
            .on_focus(|_, _| {})
            .on_blur(|_, _| {}),
    )))));
}

struct SwitchView {
    checked: bool,
    disabled: bool,
    loading: bool,
    transition_duration: Duration,
    content_mode: bool,
    changes: Vec<bool>,
    parent_clicks: usize,
    parent_keys: usize,
}

impl SwitchView {
    const fn new(checked: bool, disabled: bool, content_mode: bool) -> Self {
        Self {
            checked,
            disabled,
            loading: false,
            transition_duration: Duration::from_millis(180),
            content_mode,
            changes: Vec::new(),
            parent_clicks: 0,
            parent_keys: 0,
        }
    }

    const fn loading(checked: bool, disabled: bool, content_mode: bool) -> Self {
        Self {
            checked,
            disabled,
            loading: true,
            transition_duration: Duration::from_millis(240),
            content_mode,
            changes: Vec::new(),
            parent_clicks: 0,
            parent_keys: 0,
        }
    }
}

impl Render for SwitchView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let switch = Switch::new("target")
            .checked(self.checked)
            .label("通知")
            .disabled(self.disabled)
            .loading(self.loading)
            .transition_duration(self.transition_duration)
            .on_change_in(cx, |this, next_checked, _, cx| {
                this.changes.push(next_checked);
                this.checked = next_checked;
                cx.notify();
            });
        let switch = if self.content_mode {
            switch
                .checked_content(SwitchContent::icon_text(
                    IconSource::asset("components/checkbox/check.svg"),
                    "开启",
                ))
                .unchecked_content(SwitchContent::text("关闭"))
        } else {
            switch
        };
        div()
            .id("switch-root")
            .size(px(180.))
            .on_click(cx.listener(|this, _, _, _| {
                this.parent_clicks += 1;
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, _| {
                if is_plain_key_down(event, "enter") || is_plain_key_down(event, "space") {
                    this.parent_keys += 1;
                }
            }))
            .on_key_up(cx.listener(|this, event: &KeyUpEvent, _, _| {
                if is_plain_key_up(event, "enter") || is_plain_key_up(event, "space") {
                    this.parent_keys += 1;
                }
            }))
            .child(switch)
    }
}

struct ClickableSwitchView {
    checked: bool,
    requests: Vec<(bool, bool)>,
}

impl Render for ClickableSwitchView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().id("clickable-switch-root").size(px(180.)).child(
            Switch::new("clickable-switch")
                .checked(self.checked)
                .label("远程设置")
                .on_click_in(cx, |this, event, _, cx| {
                    this.requests.push((!this.checked, event.is_keyboard()));
                    cx.notify();
                }),
        )
    }
}

struct HandlerPrecedenceView {
    click_last: bool,
    click_count: usize,
    change_count: usize,
}

impl Render for HandlerPrecedenceView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let switch = Switch::new("handler-precedence").label("处理器优先级");
        let switch = if self.click_last {
            switch
                .on_change_in(cx, |this, _, _, _| this.change_count += 1)
                .on_click_in(cx, |this, _, _, _| this.click_count += 1)
        } else {
            switch
                .on_click_in(cx, |this, _, _, _| this.click_count += 1)
                .on_change_in(cx, |this, _, _, _| this.change_count += 1)
        };
        div()
            .id("handler-precedence-root")
            .size(px(180.))
            .child(switch)
    }
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

#[gpui::test]
fn switch_mouse_space_enter_and_disabled_follow_controlled_contract(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| SwitchView::new(false, false, false));
    draw(cx);
    cx.simulate_click(point(px(18.), px(18.)), Modifiers::none());
    assert_eq!(view.read_with(cx, |view, _| view.changes.clone()), [true]);

    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse("space").unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
    assert_eq!(view.read_with(cx, |view, _| view.changes.len()), 1);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });
    assert_eq!(
        view.read_with(cx, |view, _| view.changes.clone()),
        [true, false]
    );

    cx.simulate_keystrokes("enter");
    assert_eq!(view.read_with(cx, |view, _| view.changes.len()), 2);

    let (disabled, cx) = cx.add_window_view(|_, _| SwitchView::new(false, true, false));
    draw(cx);
    cx.simulate_click(point(px(18.), px(18.)), Modifiers::none());
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });
    assert!(disabled.read_with(cx, |view, _| view.changes.is_empty()));
}

#[gpui::test]
fn clickable_starts_a_host_request_without_optimistically_changing_checked(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|_, _| ClickableSwitchView {
        checked: false,
        requests: Vec::new(),
    });
    draw(cx);
    cx.simulate_click(point(px(18.), px(18.)), Modifiers::none());
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [(true, false)]
    );
    assert!(!view.read_with(cx, |view, _| view.checked));

    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });
    cx.simulate_keystrokes("enter");
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [(true, false), (true, true)]
    );

    view.update(cx, |view, cx| {
        view.checked = true;
        cx.notify();
    });
    draw(cx);
    assert_eq!(view.read_with(cx, |view, _| view.requests.len()), 2);
}

#[gpui::test]
fn on_click_and_on_change_share_one_last_call_wins_activation_slot(cx: &mut TestAppContext) {
    for click_last in [false, true] {
        let (view, cx) = cx.add_window_view(|_, _| HandlerPrecedenceView {
            click_last,
            click_count: 0,
            change_count: 0,
        });
        draw(cx);
        cx.simulate_click(point(px(18.), px(18.)), Modifiers::none());

        let counts = view.read_with(cx, |view, _| (view.click_count, view.change_count));
        assert_eq!(counts, if click_last { (1, 0) } else { (0, 1) });
    }
}

#[gpui::test]
fn switch_enter_does_not_activate_a_focused_control(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| SwitchView::new(false, false, false));
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
    });

    assert!(view.read_with(cx, |view, _| view.changes.is_empty()));
}

#[gpui::test]
fn content_mode_preserves_mouse_space_enter_and_disabled_behavior(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| SwitchView::new(false, false, true));
    draw(cx);
    cx.simulate_click(point(px(30.), px(18.)), Modifiers::none());
    assert_eq!(view.read_with(cx, |view, _| view.changes.clone()), [true]);

    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse("space").unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });
    assert_eq!(
        view.read_with(cx, |view, _| view.changes.clone()),
        [true, false]
    );
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
    });
    assert_eq!(view.read_with(cx, |view, _| view.changes.len()), 2);

    let (disabled, cx) = cx.add_window_view(|_, _| SwitchView::new(false, true, true));
    draw(cx);
    cx.simulate_click(point(px(30.), px(18.)), Modifiers::none());
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });
    assert!(disabled.read_with(cx, |view, _| view.changes.is_empty()));
}

#[gpui::test]
fn content_mode_renders_in_all_theme_modes_and_with_reduced_motion(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| SwitchView::new(false, false, true));
    for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
        cx.update(|_, cx| set_theme_mode(mode, cx));
        draw(cx);
    }

    cx.update(|_, cx| cx.set_reduce_motion(true));
    cx.simulate_click(point(px(30.), px(18.)), Modifiers::none());
    draw(cx);
    assert_eq!(view.read_with(cx, |view, _| view.changes.clone()), [true]);
}

#[gpui::test]
fn loading_renders_checked_and_unchecked_in_all_theme_modes(cx: &mut TestAppContext) {
    for checked in [false, true] {
        let (_view, cx) = cx.add_window_view(|_, _| SwitchView::loading(checked, false, true));
        for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
            cx.update(|_, cx| set_theme_mode(mode, cx));
            draw(cx);
        }
        cx.update(|_, cx| cx.set_reduce_motion(true));
        draw(cx);
    }
}

#[gpui::test]
fn loading_consumes_mouse_space_and_enter_without_activation_or_parent_bubbling(
    cx: &mut TestAppContext,
) {
    for disabled in [false, true] {
        let (view, cx) = cx.add_window_view(|_, _| SwitchView::loading(true, disabled, true));
        draw(cx);
        cx.simulate_click(point(px(30.), px(18.)), Modifiers::none());
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_event(KeyDownEvent {
            keystroke: Keystroke::parse("enter").unwrap(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent {
            keystroke: Keystroke::parse("enter").unwrap(),
        });
        cx.simulate_event(KeyDownEvent {
            keystroke: Keystroke::parse("space").unwrap(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent {
            keystroke: Keystroke::parse("space").unwrap(),
        });

        assert!(view.read_with(cx, |view, _| view.changes.is_empty()));
        assert_eq!(view.read_with(cx, |view, _| view.parent_clicks), 0);
        assert_eq!(view.read_with(cx, |view, _| view.parent_keys), 0);

        if !disabled {
            view.update(cx, |view, cx| {
                view.loading = false;
                cx.notify();
            });
            draw(cx);
            cx.simulate_click(point(px(30.), px(18.)), Modifiers::none());
            assert_eq!(view.read_with(cx, |view, _| view.changes.len()), 1);
        }
    }
}

#[gpui::test]
fn zero_duration_and_reduced_motion_switch_to_the_final_controlled_state(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| {
        let mut view = SwitchView::new(false, false, true);
        view.transition_duration = Duration::ZERO;
        view
    });
    draw(cx);
    cx.simulate_click(point(px(30.), px(18.)), Modifiers::none());
    draw(cx);
    assert!(view.read_with(cx, |view, _| view.checked));

    cx.update(|_, cx| cx.set_reduce_motion(true));
    view.update(cx, |view, cx| {
        view.transition_duration = Duration::from_millis(400);
        cx.notify();
    });
    draw(cx);
    cx.simulate_click(point(px(30.), px(18.)), Modifiers::none());
    draw(cx);
    assert!(!view.read_with(cx, |view, _| view.checked));
}

struct FocusView {
    root_focus: FocusHandle,
    log: Rc<RefCell<Vec<&'static str>>>,
    loading: bool,
    disabled: bool,
    transition_duration: Duration,
}

impl FocusView {
    fn new(
        log: Rc<RefCell<Vec<&'static str>>>,
        loading: bool,
        disabled: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            root_focus,
            log,
            loading,
            disabled,
            transition_duration: Duration::from_millis(180),
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
}

impl Render for FocusView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.log.clone();
        let blurred = self.log.clone();
        let other_focused = self.log.clone();
        div()
            .id("switch-focus-root")
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::on_tab))
            .child(
                Switch::new("switch-focus")
                    .label("通知")
                    .loading(self.loading)
                    .disabled(self.disabled)
                    .transition_duration(self.transition_duration)
                    .on_focus(move |_, _| focused.borrow_mut().push("focus"))
                    .on_blur(move |_, _| blurred.borrow_mut().push("blur")),
            )
            .child(
                Switch::new("other-switch")
                    .label("其他")
                    .on_focus(move |_, _| other_focused.borrow_mut().push("other-focus")),
            )
    }
}

#[gpui::test]
fn switch_focus_callbacks_follow_real_tab_transitions(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.bind_keys([KeyBinding::new("tab", Tab, None)]);
        cx.activate(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let view_log = log.clone();
    let (view, cx) =
        cx.add_window_view(move |window, cx| FocusView::new(view_log, false, false, window, cx));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&root_focus, cx);
    });
    cx.simulate_keystrokes("tab");
    draw(cx);
    cx.simulate_keystrokes("tab");
    assert_eq!(log.borrow().as_slice(), ["focus", "blur", "other-focus"]);
}

#[gpui::test]
fn loading_remains_tabbable_and_builder_changes_do_not_fabricate_focus_events(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| {
        cx.bind_keys([KeyBinding::new("tab", Tab, None)]);
        cx.activate(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let view_log = log.clone();
    let (view, cx) =
        cx.add_window_view(move |window, cx| FocusView::new(view_log, true, false, window, cx));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&root_focus, cx);
    });
    cx.simulate_keystrokes("tab");
    draw(cx);
    assert_eq!(log.borrow().as_slice(), ["focus"]);

    view.update(cx, |view, cx| {
        view.loading = false;
        view.transition_duration = Duration::from_millis(360);
        cx.notify();
    });
    draw(cx);
    assert_eq!(log.borrow().as_slice(), ["focus"]);
}

#[gpui::test]
fn disabled_switch_is_skipped_in_normal_tab_order(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.bind_keys([KeyBinding::new("tab", Tab, None)]);
        cx.activate(true);
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let view_log = log.clone();
    let (view, cx) =
        cx.add_window_view(move |window, cx| FocusView::new(view_log, false, true, window, cx));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&root_focus, cx);
    });
    cx.simulate_keystrokes("tab");
    draw(cx);

    assert_eq!(log.borrow().as_slice(), ["other-focus"]);
}

fn is_plain_key_down(event: &KeyDownEvent, key: &str) -> bool {
    event.keystroke.key == key && event.keystroke.modifiers == Modifiers::none()
}

fn is_plain_key_up(event: &KeyUpEvent, key: &str) -> bool {
    event.keystroke.key == key && event.keystroke.modifiers == Modifiers::none()
}
