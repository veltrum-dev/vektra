use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke,
    Modifiers, Orientation, ParentElement, Render, Styled, TestAppContext, Window, div, point, px,
};
use std::{cell::RefCell, rc::Rc};
use vektra::{
    Changeable, Checkbox, ComponentSize, Disableable, Focusable, Radio, RadioGroup, Sizable,
    ThemeMode, set_theme_mode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Plan {
    Free,
    Team,
    Pro,
}

#[test]
fn radio_types_and_inherent_builders_follow_standard_capabilities() {
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

    let item = focusable(disable(
        Radio::new("plan-free", Plan::Free)
            .label("免费版")
            .description("适合个人")
            .aria_label("免费方案")
            .aria_description("无需付费")
            .disabled(false)
            .on_focus(|_, _| {})
            .on_blur(|_, _| {}),
    ));
    let _group = changeable(sizable(disable(
        RadioGroup::new("plan-group")
            .selected_value(None)
            .orientation(Orientation::Horizontal)
            .aria_label("订阅方案")
            .aria_description("选择一个方案")
            .child(item)
            .disabled(false)
            .size(ComponentSize::Md)
            .on_change(|_, _, _| {}),
    )));
}

struct RadioView {
    selected: Option<Plan>,
    requests: Vec<Plan>,
    accept: bool,
    group_disabled: bool,
    team_disabled: bool,
    after_changes: usize,
    root_focus: FocusHandle,
}

impl RadioView {
    fn new(
        selected: Option<Plan>,
        accept: bool,
        group_disabled: bool,
        team_disabled: bool,
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
            group_disabled,
            team_disabled,
            after_changes: 0,
            root_focus,
        }
    }
}

impl Render for RadioView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("radio-test-root")
            .track_focus(&self.root_focus)
            .w(px(260.))
            .child(
                RadioGroup::new("plans")
                    .selected_value(self.selected)
                    .aria_label("订阅方案")
                    .aria_description("使用方向键选择")
                    .disabled(self.group_disabled)
                    .on_change_in(cx, |this, requested, _, cx| {
                        this.requests.push(requested);
                        if this.accept {
                            this.selected = Some(requested);
                        }
                        cx.notify();
                    })
                    .child(Radio::new("free", Plan::Free).label("免费版"))
                    .child(
                        Radio::new("team", Plan::Team)
                            .label("团队版")
                            .disabled(self.team_disabled),
                    )
                    .child(Radio::new("pro", Plan::Pro).label("专业版")),
            )
            .child(
                Checkbox::new("after-radio")
                    .label("后续控件")
                    .on_change_in(cx, |this, _, _, _| this.after_changes += 1),
            )
    }
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn focus_first_tab_stop(view: &gpui::Entity<RadioView>, cx: &mut gpui::VisualTestContext) {
    let root_focus = view.read_with(cx, |view, _| view.root_focus.clone());
    cx.update(|window, cx| {
        cx.activate(true);
        window.activate_window();
        window.focus(&root_focus, cx);
        window.focus_next(cx);
    });
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

#[gpui::test]
fn selected_item_is_the_only_group_tab_stop(cx: &mut TestAppContext) {
    cx.update(|cx| cx.activate(true));
    let (view, cx) = cx.add_window_view(|window, cx| {
        RadioView::new(Some(Plan::Pro), true, false, false, window, cx)
    });
    draw(cx);
    focus_first_tab_stop(&view, cx);
    draw(cx);
    key_down("left", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Team)
    );
    draw(cx);

    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    key_up("space", cx);
    assert_eq!(view.read_with(cx, |view, _| view.after_changes), 1);
}

#[gpui::test]
fn none_selected_uses_first_enabled_tab_stop_and_single_item_can_select(cx: &mut TestAppContext) {
    cx.update(|cx| cx.activate(true));
    let (view, cx) =
        cx.add_window_view(|window, cx| RadioView::new(None, true, false, true, window, cx));
    draw(cx);
    focus_first_tab_stop(&view, cx);
    draw(cx);
    key_up("space", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );

    struct SingleView {
        selected: Option<Plan>,
    }
    impl Render for SingleView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            RadioGroup::new("single")
                .selected_value(self.selected)
                .on_change_in(cx, |this, value, _, cx| {
                    this.selected = Some(value);
                    cx.notify();
                })
                .child(Radio::new("only", Plan::Free).label("唯一选项"))
        }
    }
    let (single, cx) = cx.add_window_view(|_, _| SingleView { selected: None });
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    key_up("space", cx);
    assert_eq!(
        single.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
}

#[gpui::test]
fn directions_wrap_and_skip_disabled_items(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        RadioView::new(Some(Plan::Free), true, false, true, window, cx)
    });
    draw(cx);
    focus_first_tab_stop(&view, cx);

    key_down("down", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(Plan::Pro));
    draw(cx);
    key_down("right", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
    draw(cx);
    key_down("up", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(Plan::Pro));
    draw(cx);
    key_down("left", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Pro, Plan::Free, Plan::Pro, Plan::Free]
    );
}

#[gpui::test]
fn home_end_and_space_share_the_controlled_change_path(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        RadioView::new(Some(Plan::Free), true, false, false, window, cx)
    });
    draw(cx);
    focus_first_tab_stop(&view, cx);

    key_down("end", cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(Plan::Pro));
    draw(cx);
    key_down("home", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
    draw(cx);
    key_up("space", cx);
    assert_eq!(view.read_with(cx, |view, _| view.requests.len()), 2);
}

#[gpui::test]
fn mouse_focuses_and_requests_but_disabled_items_do_not(cx: &mut TestAppContext) {
    cx.update(|cx| cx.activate(true));
    let (view, cx) = cx.add_window_view(|window, cx| {
        RadioView::new(Some(Plan::Free), true, false, true, window, cx)
    });
    draw(cx);
    cx.simulate_click(point(px(12.), px(90.)), Modifiers::none());
    draw(cx);
    assert_eq!(view.read_with(cx, |view, _| view.selected), Some(Plan::Pro));
    key_down("left", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );

    draw(cx);
    cx.simulate_click(point(px(12.), px(52.)), Modifiers::none());
    assert_eq!(view.read_with(cx, |view, _| view.requests.len()), 2);
}

#[gpui::test]
fn group_disabled_removes_all_radio_tab_stops_and_blocks_input(cx: &mut TestAppContext) {
    cx.update(|cx| cx.activate(true));
    let (view, cx) = cx.add_window_view(|window, cx| {
        RadioView::new(Some(Plan::Free), true, true, false, window, cx)
    });
    draw(cx);
    focus_first_tab_stop(&view, cx);
    draw(cx);
    key_up("space", cx);
    assert_eq!(view.read_with(cx, |view, _| view.after_changes), 1);
    key_down("down", cx);
    cx.simulate_click(point(px(12.), px(90.)), Modifiers::none());
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));
}

#[gpui::test]
fn repeated_selection_is_silent_and_rejected_async_request_keeps_authoritative_value(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        RadioView::new(Some(Plan::Free), false, false, true, window, cx)
    });
    draw(cx);
    focus_first_tab_stop(&view, cx);
    key_up("space", cx);
    cx.simulate_click(point(px(12.), px(12.)), Modifiers::none());
    assert!(view.read_with(cx, |view, _| view.requests.is_empty()));

    key_down("down", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Pro]
    );
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
    draw(cx);
    key_down("home", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.requests.clone()),
        [Plan::Pro]
    );
    assert_eq!(
        view.read_with(cx, |view, _| view.selected),
        Some(Plan::Free)
    );
}

#[gpui::test]
fn direct_on_change_and_all_theme_modes_render(cx: &mut TestAppContext) {
    let requested = Rc::new(RefCell::new(Vec::new()));
    let captured = requested.clone();
    struct DirectView {
        requested: Rc<RefCell<Vec<Plan>>>,
    }
    impl Render for DirectView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let requested = self.requested.clone();
            RadioGroup::new("direct")
                .aria_label("直接回调")
                .orientation(Orientation::Horizontal)
                .on_change(move |value, _, _| requested.borrow_mut().push(value))
                .child(Radio::new("direct-free", Plan::Free).label("免费"))
                .child(Radio::new("direct-pro", Plan::Pro).label("专业"))
        }
    }
    let (_view, cx) = cx.add_window_view(move |_, _| DirectView {
        requested: captured,
    });
    for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
        cx.update(|_, cx| set_theme_mode(mode, cx));
        draw(cx);
    }
    cx.simulate_click(point(px(70.), px(12.)), Modifiers::none());
    assert_eq!(requested.borrow().as_slice(), [Plan::Pro]);
}
