use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding, KeyDownEvent, KeyUpEvent,
    Keystroke, Modifiers, MouseButton, ParentElement, Render, ScrollDelta, ScrollWheelEvent,
    Styled, TestAppContext, TouchPhase, Window, actions, div, point, px, size,
};

actions!(select_behavior, [Tab, TabPrev]);
use vektra::{
    Changeable, Checkbox, ComponentSize, Disableable, Focusable, Select, SelectGroup, SelectOption,
    SelectStatus, Sizable, ThemeMode, set_theme_mode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Plan {
    Free,
    Team,
    Pro,
}

#[test]
fn public_types_and_standard_capabilities_compile() {
    fn changeable<C: Changeable<Plan>>(component: C) -> C {
        component.on_change(|_, _, _| {})
    }
    fn disable<C: Disableable>(component: C) -> C {
        component.disabled(true)
    }
    fn sizable<C: Sizable>(component: C) -> C {
        component.size(ComponentSize::Sm)
    }
    fn focusable<C: Focusable>(component: C) -> C {
        component.on_focus(|_, _| {}).on_blur(|_, _| {})
    }

    let option = disable(
        SelectOption::new("free", Plan::Free, "免费版")
            .description("适合个人")
            .aria_label("免费方案")
            .aria_description("无需付费")
            .disabled(false),
    );
    let group = SelectGroup::new("paid", "付费方案")
        .aria_label("付费订阅方案")
        .option(SelectOption::new("team", Plan::Team, "团队版"));
    let _select = focusable(changeable(sizable(disable(
        Select::new("plans")
            .selected_value(Some(Plan::Free))
            .placeholder("选择方案")
            .status(SelectStatus::Ready)
            .aria_label("订阅方案")
            .aria_description("选择一个订阅方案")
            .option(option)
            .group(group)
            .disabled(false)
            .size(ComponentSize::Md)
            .on_change(|_, _, _| {})
            .on_focus(|_, _| {})
            .on_blur(|_, _| {}),
    ))));
}

struct SelectView {
    selected: Option<Plan>,
    requests: Vec<Plan>,
    accept: bool,
    disabled: bool,
    team_disabled: bool,
    status: SelectStatus,
    show_select: bool,
    show_free: bool,
    show_team: bool,
    show_pro: bool,
    reverse: bool,
    all_disabled: bool,
    focus_events: Vec<&'static str>,
    bubbled_key_downs: usize,
    root_focus: FocusHandle,
}

impl SelectView {
    fn new(
        selected: Option<Plan>,
        accept: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.activate(true);
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            selected,
            requests: Vec::new(),
            accept,
            disabled: false,
            team_disabled: false,
            status: SelectStatus::Ready,
            show_select: true,
            show_free: true,
            show_team: true,
            show_pro: true,
            reverse: false,
            all_disabled: false,
            focus_events: Vec::new(),
            bubbled_key_downs: 0,
            root_focus,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}

impl Render for SelectView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut select = Select::new("plans")
            .selected_value(self.selected)
            .placeholder("选择方案")
            .status(self.status.clone())
            .aria_label("订阅方案")
            .aria_description("请选择一个订阅方案")
            .disabled(self.disabled)
            .on_focus_in(cx, |this, _, _| this.focus_events.push("focus"))
            .on_blur_in(cx, |this, _, _| this.focus_events.push("blur"))
            .on_change_in(cx, |this, requested, _, cx| {
                this.requests.push(requested);
                if this.accept {
                    this.selected = Some(requested);
                }
                cx.notify();
            });
        let option = |id, value, label, disabled| {
            SelectOption::new(id, value, label).disabled(disabled || self.all_disabled)
        };
        if self.reverse {
            if self.show_pro {
                select = select.option(option("pro", Plan::Pro, "专业版", false));
            }
            if self.show_team {
                select = select.option(option("team", Plan::Team, "团队版", self.team_disabled));
            }
            if self.show_free {
                select = select.option(option("free", Plan::Free, "免费版", false));
            }
        } else {
            if self.show_free {
                select = select.option(option("free", Plan::Free, "免费版", false));
            }
            let mut group = SelectGroup::new("paid", "付费方案");
            if self.show_team {
                group = group.option(option("team", Plan::Team, "团队版", self.team_disabled));
            }
            if self.show_pro {
                group = group.option(option("pro", Plan::Pro, "专业版", false));
            }
            select = select.group(group);
        }

        let mut root = div()
            .id("select-test-root")
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .on_key_down(cx.listener(|this, _: &KeyDownEvent, _, _| {
                this.bubbled_key_downs += 1;
            }))
            .w(px(260.))
            .h(px(320.));
        if self.show_select {
            root = root.child(select);
        }
        root.child(
            div()
                .id("outside")
                .debug_selector(|| "select-outside".into())
                .h(px(40.)),
        )
        .child(Checkbox::new("after-select").label("后续控件"))
    }
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn focus_trigger(view: &gpui::Entity<SelectView>, cx: &mut gpui::VisualTestContext) {
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&root_focus, cx);
    });
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
}

fn key_down(key: &str, cx: &mut gpui::VisualTestContext) {
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse(key).unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
}

fn key_up(key: &str, cx: &mut gpui::VisualTestContext) {
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).unwrap(),
    });
}

fn key_cycle(key: &str, cx: &mut gpui::VisualTestContext) {
    key_down(key, cx);
    key_up(key, cx);
}

#[gpui::test]
fn enter_and_space_use_full_cycle_and_request_at_most_once(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);
    draw(cx);

    key_down("enter", cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
    key_up("enter", cx);
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_some());
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));

    key_down("enter", cx);
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
    key_up("enter", cx);
    draw(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Free]
    );
    assert!(cx.debug_bounds("vektra-select-popup").is_none());

    key_cycle("space", cx);
    draw(cx);
    key_down("down", cx);
    key_cycle("space", cx);
    draw(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Free, Plan::Team]
    );
}

#[gpui::test]
fn rejected_requests_do_not_become_a_second_business_selection(cx: &mut TestAppContext) {
    let (view, cx) =
        cx.add_window_view(|window, cx| SelectView::new(Some(Plan::Free), false, window, cx));
    draw(cx);
    focus_trigger(&view, cx);
    key_cycle("enter", cx);
    draw(cx);
    key_down("down", cx);
    key_cycle("enter", cx);
    draw(cx);

    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Team]
    );
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );

    key_cycle("enter", cx);
    draw(cx);
    key_cycle("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Team]
    );
}

#[gpui::test]
fn moving_focus_between_key_down_and_key_up_cancels_activation(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);

    for key in ["enter", "space"] {
        focus_trigger(&view, cx);
        key_down(key, cx);
        cx.update(|window, _| window.blur());
        key_up(key, cx);
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-popup").is_none());
    }

    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
}

#[gpui::test]
fn unsupported_modifiers_and_unknown_keys_propagate_without_activation(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);

    key_cycle("cmd-enter", cx);
    key_cycle("cmd-space", cx);
    key_down("cmd-down", cx);
    key_down("x", cx);

    assert_eq!(view.read_with(cx, |view, _| view.bubbled_key_downs), 4);
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
    assert!(cx.debug_bounds("vektra-select-popup").is_none());

    key_down("down", cx);
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_some());
    assert_eq!(view.read_with(cx, |view, _| view.bubbled_key_downs), 4);
    key_down("escape", cx);
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
    assert_eq!(view.read_with(cx, |view, _| view.bubbled_key_downs), 4);

    key_down("escape", cx);
    key_down("home", cx);
    key_down("end", cx);
    assert_eq!(view.read_with(cx, |view, _| view.bubbled_key_downs), 7);
}

#[gpui::test]
fn arrows_home_end_skip_disabled_and_do_not_wrap(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = SelectView::new(Some(Plan::Free), true, window, cx);
        view.team_disabled = true;
        view
    });
    draw(cx);
    focus_trigger(&view, cx);
    key_down("down", cx);
    draw(cx);
    key_down("down", cx);
    key_down("down", cx);
    key_cycle("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(Plan::Pro));

    key_down("up", cx);
    draw(cx);
    key_down("home", cx);
    key_cycle("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
}

#[gpui::test]
fn escape_status_and_disabled_paths_never_request_values(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);
    key_cycle("enter", cx);
    draw(cx);
    key_down("escape", cx);
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));

    view.update(cx, |view, cx| {
        view.status = SelectStatus::loading("正在加载");
        cx.notify();
    });
    draw(cx);
    key_cycle("enter", cx);
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-status").is_some());
    key_down("down", cx);
    key_cycle("enter", cx);
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));

    view.update(cx, |view, cx| {
        view.disabled = true;
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
    key_cycle("enter", cx);
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
}

#[gpui::test]
fn mouse_open_option_submit_and_outside_close(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);
    let trigger = cx.debug_bounds("vektra-select-trigger").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::none());
    draw(cx);
    let option = cx.debug_bounds("vektra-select-option-free").unwrap();
    cx.simulate_click(option.center(), Modifiers::none());
    draw(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
    assert!(cx.debug_bounds("vektra-select-popup").is_none());

    view.update(cx, |view, cx| {
        view.team_disabled = true;
        cx.notify();
    });
    draw(cx);
    let trigger = cx.debug_bounds("vektra-select-trigger").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::none());
    draw(cx);
    let disabled_option = cx.debug_bounds("vektra-select-option-team").unwrap();
    cx.simulate_click(disabled_option.center(), Modifiers::none());
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_some());
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Free]
    );

    cx.simulate_click(trigger.center(), Modifiers::none());
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());

    cx.simulate_click(trigger.center(), Modifiers::none());
    draw(cx);
    cx.simulate_mouse_down(
        point(px(500.), px(500.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
}

struct DuplicateSelectView {
    requests: Vec<Plan>,
}

impl Render for DuplicateSelectView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Select::new("duplicate-select")
            .aria_label("重复项测试")
            .option(SelectOption::new("first", Plan::Free, "首项"))
            .option(SelectOption::new("first", Plan::Team, "重复 ID"))
            .option(SelectOption::new("later-team", Plan::Team, "后续重复值"))
            .option(SelectOption::new("pro", Plan::Pro, "正常项"))
            .on_change_in(cx, |this, requested, _, _| {
                this.requests.push(requested);
            })
    }
}

#[gpui::test]
fn transitive_duplicate_conflicts_stay_disabled_in_the_rendered_popup(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| DuplicateSelectView {
        requests: Vec::new(),
    });
    draw(cx);
    let trigger = cx.debug_bounds("vektra-select-trigger").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::none());
    draw(cx);

    let duplicate_value = cx.debug_bounds("vektra-select-option-later-team").unwrap();
    cx.simulate_click(duplicate_value.center(), Modifiers::none());
    draw(cx);
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
    assert!(cx.debug_bounds("vektra-select-popup").is_some());

    let canonical = cx.debug_bounds("vektra-select-option-pro").unwrap();
    cx.simulate_click(canonical.center(), Modifiers::none());
    draw(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Pro]
    );
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
}

#[gpui::test]
fn tab_and_shift_tab_close_without_trapping_focus(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);
    key_cycle("enter", cx);
    draw(cx);
    key_down("tab", cx);
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());

    cx.update(|window, cx| window.focus_prev(cx));
    key_cycle("enter", cx);
    draw(cx);
    key_down("shift-tab", cx);
    cx.update(|window, cx| window.focus_prev(cx));
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
}

#[gpui::test]
fn focus_callbacks_follow_real_trigger_focus_lifecycle(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.bind_keys([
            KeyBinding::new("tab", Tab, None),
            KeyBinding::new("shift-tab", TabPrev, None),
        ]);
        cx.activate(true);
    });
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&root_focus, cx);
    });
    draw(cx);
    cx.simulate_keystrokes("tab");
    draw(cx);
    let focused = view.read_with(cx, |view, _| view.focus_events.clone());
    assert_eq!(focused, ["focus"]);
    key_cycle("enter", cx);
    key_down("escape", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.focus_events.clone()),
        ["focus"]
    );
    cx.simulate_keystrokes("tab");
    assert_eq!(
        view.read_with(cx, |view, _| view.focus_events.clone()),
        ["focus", "blur"]
    );
}

#[gpui::test]
fn dynamic_options_preserve_identity_and_never_replace_business_value(cx: &mut TestAppContext) {
    let (view, cx) =
        cx.add_window_view(|window, cx| SelectView::new(Some(Plan::Free), true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);
    key_down("down", cx);
    draw(cx);
    key_down("down", cx);
    view.update(cx, |view, cx| {
        view.reverse = true;
        cx.notify();
    });
    draw(cx);
    key_cycle("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Team)
    );

    view.update(cx, |view, cx| {
        view.show_team = false;
        cx.notify();
    });
    draw(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Team)
    );
    assert!(view.read_with(cx, |view, _| view.requests.len()) == 1);

    key_cycle("enter", cx);
    draw(cx);
    key_cycle("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(Plan::Pro));
}

#[gpui::test]
fn disabled_selected_and_all_disabled_options_are_safe(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = SelectView::new(Some(Plan::Team), true, window, cx);
        view.team_disabled = true;
        view
    });
    draw(cx);
    focus_trigger(&view, cx);
    key_cycle("enter", cx);
    draw(cx);
    key_cycle("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );

    view.update(cx, |view, cx| {
        view.all_disabled = true;
        view.requests.clear();
        cx.notify();
    });
    draw(cx);
    key_cycle("enter", cx);
    draw(cx);
    key_down("down", cx);
    key_cycle("enter", cx);
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
}

#[gpui::test]
fn host_status_transitions_clear_active_and_keep_selected_value(cx: &mut TestAppContext) {
    let (view, cx) =
        cx.add_window_view(|window, cx| SelectView::new(Some(Plan::Free), true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);
    key_down("down", cx);
    draw(cx);
    key_down("down", cx);
    for status in [
        SelectStatus::empty("暂无方案"),
        SelectStatus::error("加载失败"),
        SelectStatus::loading("重新加载"),
    ] {
        view.update(cx, |view, cx| {
            view.status = status;
            cx.notify();
        });
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-status").is_some());
        assert_eq!(
            view.read_with(cx, |view, _| view.selected),
            Some(Plan::Free)
        );
    }
    view.update(cx, |view, cx| {
        view.status = SelectStatus::Ready;
        cx.notify();
    });
    draw(cx);
    key_cycle("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
}

struct LongSelectView {
    selected: Option<usize>,
    focus_handle: FocusHandle,
}

impl LongSelectView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.activate(true);
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            selected: None,
            focus_handle,
        }
    }
}

impl Render for LongSelectView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut select = Select::new("long-select")
            .selected_value(self.selected)
            .aria_label("长列表")
            .on_change_in(cx, |this, value, _, cx| {
                this.selected = Some(value);
                cx.notify();
            });
        for index in 0..40 {
            select = select.option(SelectOption::new(
                format!("long-{index}"),
                index,
                format!("选项 {index}"),
            ));
        }
        div()
            .track_focus(&self.focus_handle)
            .w(px(340.))
            .pt(px(180.))
            .child(select)
    }
}

#[gpui::test]
fn long_list_flips_stays_in_viewport_and_end_scrolls_active_visible(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(LongSelectView::new);
    cx.simulate_resize(size(px(360.), px(260.)));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.focus_handle.clone());
    cx.update(|window, cx| {
        window.focus(&root_focus, cx);
        window.focus_next(cx);
    });
    key_down("down", cx);
    draw(cx);
    let trigger = cx.debug_bounds("vektra-select-trigger").unwrap();
    let popup = cx.debug_bounds("vektra-select-popup").unwrap();
    assert!(popup.top() < trigger.top());
    assert!(popup.left() >= px(0.));
    assert!(popup.right() <= px(360.));
    assert!(popup.size.height <= px(280.));

    cx.simulate_event(ScrollWheelEvent {
        position: popup.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-80.))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_some());

    let track_x = popup.right() - px(7.);
    let thumb_y = popup.top() + px(18.);
    cx.simulate_mouse_down(
        point(track_x, thumb_y),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.simulate_mouse_move(
        point(track_x, popup.bottom() - px(28.)),
        Some(MouseButton::Left),
        Modifiers::none(),
    );
    cx.simulate_mouse_up(
        point(track_x, popup.bottom() - px(28.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_some());

    key_down("end", cx);
    cx.update(|window, _| window.refresh());
    draw(cx);
    draw(cx);
    draw(cx);
    let popup = cx.debug_bounds("vektra-select-popup").unwrap();
    let last = cx.debug_bounds("vektra-select-option-long-39").unwrap();
    assert!(last.top() >= popup.top());
    assert!(last.bottom() <= popup.bottom());
    let last_center = last.center();

    let trigger = cx.debug_bounds("vektra-select-trigger").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::none());
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
    cx.simulate_click(trigger.center(), Modifiers::none());
    draw(cx);
    cx.simulate_mouse_move(last_center, None, Modifiers::none());
    draw(cx);
    draw(cx);
    let popup = cx.debug_bounds("vektra-select-popup").unwrap();
    let last = cx.debug_bounds("vektra-select-option-long-39").unwrap();
    assert!(last.top() >= popup.top());
    assert!(last.bottom() <= popup.bottom());

    cx.simulate_mouse_move(last.center(), None, Modifiers::none());
    cx.simulate_click(last.center(), Modifiers::none());
    draw(cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(39));

    key_down("down", cx);
    draw(cx);
    cx.simulate_resize(size(px(240.), px(220.)));
    draw(cx);
    let popup = cx.debug_bounds("vektra-select-popup").unwrap();
    assert!(popup.left() >= px(0.));
    assert!(popup.right() <= px(240.));
}

#[gpui::test]
fn window_deactivation_and_trigger_removal_are_safe(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);
    key_cycle("enter", cx);
    draw(cx);
    cx.deactivate_window();
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());

    view.update(cx, |view, cx| {
        view.show_select = false;
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-trigger").is_none());
}

#[gpui::test]
fn select_renders_in_light_dark_and_system_modes(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
        cx.update(|_, cx| set_theme_mode(mode, cx));
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-trigger").is_some());
    }
}
