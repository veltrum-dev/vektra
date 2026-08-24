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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    free_disabled: bool,
    team_disabled: bool,
    pro_disabled: bool,
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
            free_disabled: false,
            team_disabled: false,
            pro_disabled: false,
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
                select = select.option(option("pro", Plan::Pro, "专业版", self.pro_disabled));
            }
            if self.show_team {
                select = select.option(option("team", Plan::Team, "团队版", self.team_disabled));
            }
            if self.show_free {
                select = select.option(option("free", Plan::Free, "免费版", self.free_disabled));
            }
        } else {
            if self.show_free {
                select = select.option(option("free", Plan::Free, "免费版", self.free_disabled));
            }
            let mut group = SelectGroup::new("paid", "付费方案");
            if self.show_team {
                group = group.option(option("team", Plan::Team, "团队版", self.team_disabled));
            }
            if self.show_pro {
                group = group.option(option("pro", Plan::Pro, "专业版", self.pro_disabled));
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
fn empty_group_and_disabled_runs_never_enter_navigation(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = SelectView::new(None, true, window, cx);
        view.show_team = false;
        view.show_pro = false;
        view
    });
    draw(cx);
    focus_trigger(&view, cx);
    key_down("down", cx);
    draw(cx);
    key_cycle("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Free]
    );

    view.update(cx, |view, cx| {
        view.selected = None;
        view.requests.clear();
        view.show_team = true;
        view.show_pro = true;
        view.free_disabled = true;
        view.team_disabled = true;
        view.pro_disabled = false;
        cx.notify();
    });
    draw(cx);
    key_down("down", cx);
    draw(cx);
    key_cycle("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Pro]
    );

    view.update(cx, |view, cx| {
        view.selected = None;
        view.requests.clear();
        view.free_disabled = true;
        view.team_disabled = false;
        view.pro_disabled = true;
        cx.notify();
    });
    draw(cx);
    key_down("down", cx);
    draw(cx);
    key_cycle("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Team]
    );
}

#[gpui::test]
fn non_ready_statuses_are_inert_and_close_with_mouse_keyboard_and_focus(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.bind_keys([
            KeyBinding::new("tab", Tab, None),
            KeyBinding::new("shift-tab", TabPrev, None),
        ]);
        cx.activate(true);
    });
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);

    for status in [
        SelectStatus::loading("正在加载"),
        SelectStatus::empty("暂无方案"),
        SelectStatus::error("加载失败"),
    ] {
        let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
        cx.update(|window, cx| window.focus(&root_focus, cx));
        view.update(cx, |view, cx| {
            view.status = status;
            view.requests.clear();
            view.focus_events.clear();
            cx.notify();
        });
        draw(cx);
        focus_trigger(&view, cx);

        let trigger = cx.debug_bounds("vektra-select-trigger").unwrap();
        cx.simulate_click(trigger.center(), Modifiers::none());
        draw(cx);
        assert_eq!(
            view.read_with(cx, |view, _| view.focus_events.clone()),
            ["focus"]
        );
        assert!(cx.debug_bounds("vektra-select-status").is_some());
        assert!(cx.debug_bounds("vektra-select-popup").is_some());
        assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
        cx.simulate_mouse_down(
            point(px(500.), px(500.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-popup").is_none());

        key_cycle("enter", cx);
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-status").is_some());
        key_down("down", cx);
        key_cycle("enter", cx);
        assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-popup").is_none());

        key_cycle("enter", cx);
        draw(cx);
        for key in ["down", "up", "home", "end"] {
            key_down(key, cx);
        }
        assert!(cx.debug_bounds("vektra-select-popup").is_some());
        key_cycle("enter", cx);
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-popup").is_none());
        assert!(view.read_with(cx, |view, _| view.requests.is_empty()));

        key_cycle("enter", cx);
        draw(cx);
        key_down("escape", cx);
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-popup").is_none());

        key_cycle("enter", cx);
        draw(cx);
        cx.simulate_keystrokes("tab");
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-popup").is_none());
        assert_eq!(
            view.read_with(cx, |view, _| view.focus_events.clone()),
            ["focus", "blur"]
        );
        assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
    }

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
fn typeahead_opens_from_closed_and_matches_accessible_names(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);

    key_down("专", cx);
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_some());
    key_cycle("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(Plan::Pro));

    focus_trigger(&view, cx);
    key_down("x", cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(Plan::Pro));
}

struct TypeaheadSelectView {
    selected: Option<usize>,
    status: SelectStatus,
    root_focus: FocusHandle,
}

impl TypeaheadSelectView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.activate(true);
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            selected: None,
            status: SelectStatus::Ready,
            root_focus,
        }
    }
}

impl Render for TypeaheadSelectView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let group = SelectGroup::new("letters", "字母")
            .option(
                SelectOption::new("beta", 1, "禁用项")
                    .aria_label("Beta")
                    .disabled(true),
            )
            .option(SelectOption::new("bravo", 2, "二号").aria_label("Bravo"))
            .option(SelectOption::new("berlin", 3, "三号").aria_label("Berlin"))
            .option(SelectOption::new("echo", 4, "四号").aria_label("Echo"));
        div().track_focus(&self.root_focus).child(
            Select::new("typeahead-select")
                .selected_value(self.selected)
                .status(self.status.clone())
                .option(SelectOption::new("alpha", 0, "一号").aria_label("Älpha"))
                .group(group)
                .option(SelectOption::new("berlin", 5, "重复 ID").aria_label("Beryl"))
                .option(SelectOption::new("duplicate-value", 3, "重复值").aria_label("Broken"))
                .on_change_in(cx, |this, value, _, cx| {
                    this.selected = Some(value);
                    cx.notify();
                }),
        )
    }
}

fn focus_typeahead_trigger(
    view: &gpui::Entity<TypeaheadSelectView>,
    cx: &mut gpui::VisualTestContext,
) {
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&root_focus, cx);
        window.focus_next(cx);
    });
    draw(cx);
}

#[gpui::test]
fn typeahead_handles_unicode_prefixes_repetition_timeout_groups_and_conflicts(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(TypeaheadSelectView::new);
    draw(cx);
    focus_typeahead_trigger(&view, cx);

    key_down("ä", cx);
    key_cycle("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(0));

    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    key_down("b", cx);
    key_down("b", cx);
    key_cycle("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(3));

    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    key_down("b", cx);
    key_down("r", cx);
    key_cycle("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(2));

    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    key_down("b", cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    key_down("e", cx);
    key_cycle("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(4));

    key_down("x", cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());

    view.update(cx, |view, cx| {
        view.status = SelectStatus::loading("加载中");
        cx.notify();
    });
    draw(cx);
    key_down("b", cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(4));
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
    cx.update(|cx| {
        cx.bind_keys([
            KeyBinding::new("tab", Tab, None),
            KeyBinding::new("shift-tab", TabPrev, None),
        ]);
        cx.activate(true);
    });
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);
    key_cycle("enter", cx);
    draw(cx);
    cx.simulate_keystrokes("tab");
    draw(cx);
    assert!(cx.debug_bounds("vektra-select-popup").is_none());

    cx.update(|window, cx| window.focus_prev(cx));
    key_cycle("enter", cx);
    draw(cx);
    cx.simulate_keystrokes("shift-tab");
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

#[gpui::test]
fn status_loading_to_ready_reinitializes_active_for_submission(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| SelectView::new(None, true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);
    key_cycle("enter", cx);
    draw(cx);

    view.update(cx, |view, cx| {
        view.status = SelectStatus::loading("正在加载");
        cx.notify();
    });
    draw(cx);
    view.update(cx, |view, cx| {
        view.status = SelectStatus::Ready;
        cx.notify();
    });
    draw(cx);

    key_cycle("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Free]
    );
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
}

#[gpui::test]
fn status_loading_to_ready_prefers_the_authoritative_selection(cx: &mut TestAppContext) {
    let (view, cx) =
        cx.add_window_view(|window, cx| SelectView::new(Some(Plan::Pro), true, window, cx));
    draw(cx);
    focus_trigger(&view, cx);
    key_cycle("enter", cx);
    draw(cx);

    view.update(cx, |view, cx| {
        view.status = SelectStatus::loading("正在加载");
        cx.notify();
    });
    draw(cx);
    view.update(cx, |view, cx| {
        view.status = SelectStatus::Ready;
        cx.notify();
    });
    draw(cx);

    key_down("down", cx);
    key_cycle("enter", cx);
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(Plan::Pro));
    assert!(cx.debug_bounds("vektra-select-popup").is_none());
}

struct ShortPopupView {
    focus_handle: FocusHandle,
}

impl ShortPopupView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.activate(true);
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self { focus_handle }
    }
}

impl Render for ShortPopupView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .w(px(340.))
            .pt(px(250.))
            .child(
                Select::new("short-select")
                    .aria_label("短列表")
                    .option(
                        SelectOption::new("short-free", Plan::Free, "免费版")
                            .description("适合个人体验与小型项目"),
                    )
                    .group(
                        SelectGroup::new("short-paid", "付费方案")
                            .option(
                                SelectOption::new("short-team", Plan::Team, "专业版")
                                    .description("适合持续交付的专业团队"),
                            )
                            .option(
                                SelectOption::new("short-pro", Plan::Pro, "企业版")
                                    .description("请联系销售获取报价")
                                    .disabled(true),
                            ),
                    ),
            )
    }
}

#[gpui::test]
fn short_popup_shrinks_to_content_and_stays_below_when_content_fits(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(ShortPopupView::new);
    cx.simulate_resize(size(px(360.), px(500.)));
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
    let last = cx.debug_bounds("vektra-select-option-short-pro").unwrap();
    assert!(
        popup.top() >= trigger.bottom(),
        "内容能在下方放下时不应按最大高度错误翻转"
    );
    assert!(
        popup.size.height < px(220.),
        "四个固定行高条目不应撑满 Popup 最大高度"
    );
    assert!(
        popup.bottom() - last.bottom() <= px(2.),
        "最后一行之后不应留下大块空白"
    );
}

struct LongSelectView {
    selected: Option<usize>,
    option_count: usize,
    descriptions: bool,
    focus_handle: FocusHandle,
}

impl LongSelectView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.activate(true);
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            selected: None,
            option_count: 40,
            descriptions: false,
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
        for index in 0..self.option_count {
            let mut option =
                SelectOption::new(format!("long-{index}"), index, format!("选项 {index}"));
            if self.descriptions {
                option = option.description(format!("选项 {index} 的补充说明"));
            }
            select = select.option(option);
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
fn page_down_uses_the_measured_popup_page_instead_of_a_fixed_item_count(cx: &mut TestAppContext) {
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

    let popup = cx.debug_bounds("vektra-select-popup").unwrap();
    let candidates = [
        ("vektra-select-option-long-0", 0),
        ("vektra-select-option-long-1", 1),
        ("vektra-select-option-long-2", 2),
        ("vektra-select-option-long-3", 3),
        ("vektra-select-option-long-4", 4),
        ("vektra-select-option-long-5", 5),
        ("vektra-select-option-long-6", 6),
        ("vektra-select-option-long-7", 7),
        ("vektra-select-option-long-8", 8),
        ("vektra-select-option-long-9", 9),
        ("vektra-select-option-long-10", 10),
        ("vektra-select-option-long-11", 11),
        ("vektra-select-option-long-12", 12),
    ];
    let expected = candidates
        .into_iter()
        .take_while(|(selector, _)| {
            cx.debug_bounds(selector)
                .is_some_and(|bounds| bounds.bottom() <= popup.bottom())
        })
        .map(|(_, index)| index)
        .last()
        .unwrap();
    assert_ne!(expected, 10, "夹具必须避免把固定十项误当成实际页");

    key_down("pagedown", cx);
    key_cycle("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(expected));

    key_down("down", cx);
    draw(cx);
    key_down("pageup", cx);
    key_cycle("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(0));
}

struct MixedPageSelectView {
    selected: Option<usize>,
    root_focus: FocusHandle,
}

impl MixedPageSelectView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.activate(true);
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            selected: None,
            root_focus,
        }
    }
}

impl Render for MixedPageSelectView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut group = SelectGroup::new("mixed-page-group", "分组选项").option(SelectOption::new(
            "duplicate-zero",
            0,
            "重复值",
        ));
        for index in 1..30 {
            group = group.option(
                SelectOption::new(format!("mixed-{index}"), index, format!("混合选项 {index}"))
                    .disabled(index == 2),
            );
        }
        div().track_focus(&self.root_focus).w(px(340.)).child(
            Select::new("mixed-page-select")
                .selected_value(self.selected)
                .option(SelectOption::new("mixed-0", 0, "混合选项 0"))
                .group(group)
                .on_change_in(cx, |this, value, _, cx| {
                    this.selected = Some(value);
                    cx.notify();
                }),
        )
    }
}

#[gpui::test]
fn measured_page_navigation_skips_grouped_disabled_and_duplicate_options(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(MixedPageSelectView::new);
    cx.simulate_resize(size(px(360.), px(220.)));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    cx.update(|window, cx| {
        window.focus(&root_focus, cx);
        window.focus_next(cx);
    });
    key_down("down", cx);
    draw(cx);

    let popup = cx.debug_bounds("vektra-select-popup").unwrap();
    let candidates = [
        ("vektra-select-option-mixed-0", 0),
        ("vektra-select-option-mixed-1", 1),
        ("vektra-select-option-mixed-3", 3),
        ("vektra-select-option-mixed-4", 4),
        ("vektra-select-option-mixed-5", 5),
        ("vektra-select-option-mixed-6", 6),
        ("vektra-select-option-mixed-7", 7),
        ("vektra-select-option-mixed-8", 8),
    ];
    let expected = candidates
        .into_iter()
        .filter(|(selector, _)| {
            cx.debug_bounds(selector)
                .is_some_and(|bounds| bounds.bottom() <= popup.bottom())
        })
        .map(|(_, value)| value)
        .last()
        .unwrap();
    assert!(
        cx.debug_bounds("vektra-select-option-duplicate-zero")
            .is_some()
    );
    assert!(
        cx.debug_bounds("vektra-select-option-mixed-2").is_none(),
        "虚拟 Popup 不应为视口外 option 创建 Element"
    );

    key_down("pagedown", cx);
    key_cycle("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(expected));
}

#[gpui::test]
fn popup_placement_survives_fractional_scale_factors_and_resizes(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(LongSelectView::new);
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.focus_handle.clone());
    cx.update(|window, cx| {
        window.focus(&root_focus, cx);
        window.focus_next(cx);
    });

    for scale_factor in [1.0, 1.25, 1.5, 2.0] {
        cx.simulate_resize(size(px(360.), px(260.)));
        cx.update(|window, _| window.set_scale_factor(scale_factor));
        draw(cx);
        key_down("down", cx);
        draw(cx);

        cx.update(|window, _| assert_eq!(window.scale_factor(), scale_factor));
        let trigger = cx.debug_bounds("vektra-select-trigger").unwrap();
        let popup = cx.debug_bounds("vektra-select-popup").unwrap();
        assert!(popup.top() < trigger.top());
        assert!(popup.left() >= px(0.));
        assert!(popup.right() <= px(360.));
        assert!(popup.top() >= px(0.));
        assert!(popup.bottom() <= px(260.));
        assert!(popup.size.width > px(0.));
        assert!(popup.size.height > px(0.));

        cx.simulate_resize(size(px(240.), px(220.)));
        cx.update(|window, _| window.set_scale_factor(scale_factor));
        draw(cx);
        draw(cx);
        cx.update(|window, _| assert_eq!(window.scale_factor(), scale_factor));
        let popup = cx.debug_bounds("vektra-select-popup").unwrap();
        assert!(popup.left() >= px(0.));
        assert!(popup.right() <= px(240.));
        assert!(popup.top() >= px(0.));
        assert!(popup.bottom() <= px(220.));

        key_down("escape", cx);
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-popup").is_none());
    }
}

#[gpui::test]
fn long_list_dynamic_data_and_height_keep_active_in_a_legal_scroll_range(cx: &mut TestAppContext) {
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
    key_down("end", cx);
    cx.update(|window, _| window.refresh());
    draw(cx);
    draw(cx);

    view.update(cx, |view, cx| {
        view.option_count = 10;
        view.descriptions = true;
        cx.notify();
    });
    draw(cx);
    draw(cx);
    let popup = cx.debug_bounds("vektra-select-popup").unwrap();
    let last = cx.debug_bounds("vektra-select-option-long-9").unwrap();
    assert!(last.top() >= popup.top());
    assert!(last.bottom() <= popup.bottom());

    view.update(cx, |view, cx| {
        view.option_count = 50;
        cx.notify();
    });
    draw(cx);
    key_down("end", cx);
    cx.update(|window, _| window.refresh());
    draw(cx);
    draw(cx);
    let popup = cx.debug_bounds("vektra-select-popup").unwrap();
    let last = cx.debug_bounds("vektra-select-option-long-49").unwrap();
    assert!(last.top() >= popup.top());
    assert!(last.bottom() <= popup.bottom());
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
fn long_list_keeps_active_and_selected_visible_in_all_theme_modes(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(LongSelectView::new);
    cx.simulate_resize(size(px(360.), px(260.)));
    draw(cx);
    let root_focus = view.read_with(cx, |view, _| view.focus_handle.clone());
    cx.update(|window, cx| {
        window.focus(&root_focus, cx);
        window.focus_next(cx);
    });

    for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
        cx.update(|_, cx| set_theme_mode(mode, cx));
        draw(cx);

        key_down("down", cx);
        draw(cx);
        key_down("end", cx);
        cx.update(|window, _| window.refresh());
        draw(cx);
        draw(cx);
        let popup = cx.debug_bounds("vektra-select-popup").unwrap();
        let last = cx.debug_bounds("vektra-select-option-long-39").unwrap();
        assert!(last.top() >= popup.top());
        assert!(last.bottom() <= popup.bottom());

        cx.simulate_click(last.center(), Modifiers::none());
        draw(cx);
        assert_eq!(view.read_with(cx, |view, _| view.selected), Some(39));

        let trigger = cx.debug_bounds("vektra-select-trigger").unwrap();
        cx.simulate_click(trigger.center(), Modifiers::none());
        draw(cx);
        draw(cx);
        let popup = cx.debug_bounds("vektra-select-popup").unwrap();
        let selected = cx.debug_bounds("vektra-select-option-long-39").unwrap();
        assert!(selected.top() >= popup.top());
        assert!(selected.bottom() <= popup.bottom());

        cx.simulate_click(trigger.center(), Modifiers::none());
        draw(cx);
        assert!(cx.debug_bounds("vektra-select-popup").is_none());
    }
}
