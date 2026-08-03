use super::*;
use gpui::{Hsla, TestAppContext, rgb};

#[test]
fn builder_defaults_and_overrides_are_preserved() {
    let tooltip = Tooltip::new("设置");
    assert_eq!(tooltip.text_value().as_ref(), "设置");
    assert_eq!(tooltip.open, None);
    assert!(tooltip.arrow);
    assert_eq!(tooltip.color, None);
    assert_eq!(tooltip.bg_color, None);
    assert!(tooltip.animated);

    let foreground: Hsla = rgb(0xffffff).into();
    let background: Hsla = rgb(0x222222).into();
    let tooltip = Tooltip::new("保存")
        .open(true)
        .arrow(false)
        .color(rgb(0xffffff))
        .bg_color(rgb(0x222222))
        .animated(false);
    assert_eq!(tooltip.open, Some(true));
    assert!(!tooltip.arrow);
    assert_eq!(tooltip.color, Some(foreground));
    assert_eq!(tooltip.bg_color, Some(background));
    assert!(!tooltip.animated);
}

#[test]
fn string_inputs_convert_to_tooltip_without_migration() {
    let borrowed: Tooltip = "借用字符串".into();
    let owned: Tooltip = String::from("字符串").into();
    let shared: Tooltip = SharedString::from("共享字符串").into();

    assert_eq!(borrowed.text.as_ref(), "借用字符串");
    assert_eq!(owned.text.as_ref(), "字符串");
    assert_eq!(shared.text.as_ref(), "共享字符串");
}

#[gpui::test]
fn delay_and_escape_state_machine_is_deterministic(cx: &mut TestAppContext) {
    let (state, cx) = cx.add_window_view(TooltipTrigger::new);

    state.update(cx, |state, cx| {
        state.tooltip = Tooltip::new("自动").animated(false);
        state.hovered = true;
        state.schedule_show(cx);
        assert_eq!(state.phase, TransitionPhase::Hidden);
    });
    cx.executor().advance_clock(SHOW_DELAY);
    cx.run_until_parked();
    assert_eq!(
        state.read_with(cx, |state, _| state.phase),
        TransitionPhase::Visible
    );

    state.update(cx, |state, cx| {
        assert!(state.dismiss(cx));
        assert_eq!(state.phase, TransitionPhase::Hidden);
        state.schedule_show(cx);
        assert!(state.delay_task.is_none());
    });
}

#[gpui::test]
fn leaving_during_delay_cancels_and_a_new_cycle_can_show(cx: &mut TestAppContext) {
    let (state, cx) = cx.add_window_view(TooltipTrigger::new);

    state.update(cx, |state, cx| {
        state.tooltip = Tooltip::new("自动").animated(false);
        state.hovered = true;
        state.schedule_show(cx);
        state.hovered = false;
        state.reconcile(cx);
    });
    cx.executor().advance_clock(SHOW_DELAY);
    cx.run_until_parked();
    assert_eq!(
        state.read_with(cx, |state, _| state.phase),
        TransitionPhase::Hidden
    );

    state.update(cx, |state, cx| {
        state.hovered = true;
        state.schedule_show(cx);
    });
    cx.executor().advance_clock(SHOW_DELAY);
    cx.run_until_parked();
    assert_eq!(
        state.read_with(cx, |state, _| state.phase),
        TransitionPhase::Visible
    );
}

#[gpui::test]
fn explicit_open_is_immediate_false_blocks_auto_and_escape_requires_false_true(
    cx: &mut TestAppContext,
) {
    let (state, cx) = cx.add_window_view(TooltipTrigger::new);

    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.update_tooltip(Tooltip::new("受控").open(true).animated(false), window, cx);
            assert_eq!(state.phase, TransitionPhase::Visible);
            assert!(state.dismiss(cx));
            assert_eq!(state.phase, TransitionPhase::Hidden);
        });
    });

    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.update_tooltip(Tooltip::new("受控").open(true).animated(false), window, cx);
            assert_eq!(state.phase, TransitionPhase::Hidden);
            assert!(state.explicit_dismissed);

            state.hovered = true;
            state.update_tooltip(Tooltip::new("受控").open(false).animated(false), window, cx);
            assert_eq!(state.phase, TransitionPhase::Hidden);
            assert!(state.delay_task.is_none());

            state.update_tooltip(Tooltip::new("受控").open(true).animated(false), window, cx);
            assert_eq!(state.phase, TransitionPhase::Visible);
            assert!(!state.explicit_dismissed);
        });
    });
}

#[gpui::test]
fn enter_and_exit_use_deterministic_transition_timers(cx: &mut TestAppContext) {
    let (state, cx) = cx.add_window_view(TooltipTrigger::new);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.update_tooltip(Tooltip::new("动画").open(true), window, cx);
        });
    });
    assert_eq!(
        state.read_with(cx, |state, _| state.phase),
        TransitionPhase::Entering
    );
    let first_enter_generation = state.read_with(cx, |state, _| state.generation);

    cx.executor().advance_clock(ENTER_DURATION);
    cx.run_until_parked();
    assert_eq!(
        state.read_with(cx, |state, _| state.phase),
        TransitionPhase::Visible
    );

    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.update_tooltip(Tooltip::new("动画").open(false), window, cx);
        });
    });
    assert_eq!(
        state.read_with(cx, |state, _| state.phase),
        TransitionPhase::Exiting
    );
    cx.executor().advance_clock(EXIT_DURATION);
    cx.run_until_parked();
    assert_eq!(
        state.read_with(cx, |state, _| state.phase),
        TransitionPhase::Hidden
    );
    assert!(state.read_with(cx, |state, _| state.view.is_none()));

    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.update_tooltip(Tooltip::new("动画").open(true), window, cx);
        });
    });
    assert_eq!(
        state.read_with(cx, |state, _| state.phase),
        TransitionPhase::Entering
    );
    assert!(state.read_with(cx, |state, _| state.generation > first_enter_generation));
    cx.executor().advance_clock(ENTER_DURATION);
    cx.run_until_parked();
}

#[gpui::test]
fn reduce_motion_settles_without_transition_time(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (state, cx) = cx.add_window_view(TooltipTrigger::new);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.update_tooltip(Tooltip::new("静态").open(true), window, cx);
            assert_eq!(state.phase, TransitionPhase::Visible);
            assert!(state.transition_task.is_none());
            state.update_tooltip(Tooltip::new("静态").open(false), window, cx);
            assert_eq!(state.phase, TransitionPhase::Hidden);
            assert!(state.transition_task.is_none());
        });
    });
}

#[gpui::test]
fn removing_the_owner_releases_state_and_pending_delay(cx: &mut TestAppContext) {
    let (state, cx) = cx.add_window_view(TooltipTrigger::new);
    let weak_state = state.downgrade();

    state.update(cx, |state, cx| {
        state.hovered = true;
        state.schedule_show(cx);
        assert!(state.delay_task.is_some());
    });

    drop(state);
    cx.update(|window, _| window.remove_window());
    cx.executor().advance_clock(SHOW_DELAY);
    cx.run_until_parked();

    assert!(weak_state.upgrade().is_none());
}

#[gpui::test]
fn removing_the_owner_releases_an_active_exit_transition(cx: &mut TestAppContext) {
    let (state, cx) = cx.add_window_view(TooltipTrigger::new);
    let weak_state = state.downgrade();

    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.update_tooltip(Tooltip::new("退出").open(true), window, cx);
            assert_eq!(state.phase, TransitionPhase::Entering);
            assert!(state.dismiss(cx));
            assert_eq!(state.phase, TransitionPhase::Exiting);
            assert!(state.transition_task.is_some());
        });
    });

    drop(state);
    cx.update(|window, _| window.remove_window());
    cx.executor().advance_clock(EXIT_DURATION);
    cx.run_until_parked();

    assert!(weak_state.upgrade().is_none());
}

#[test]
fn all_twelve_placements_use_the_expected_origin_and_arrow_direction() {
    let cases = [
        (
            TooltipPlacement::TopStart,
            point(px(100.), px(58.)),
            Side::Top,
        ),
        (TooltipPlacement::Top, point(px(80.), px(58.)), Side::Top),
        (TooltipPlacement::TopEnd, point(px(60.), px(58.)), Side::Top),
        (
            TooltipPlacement::RightStart,
            point(px(152.), px(100.)),
            Side::Right,
        ),
        (
            TooltipPlacement::Right,
            point(px(152.), px(95.)),
            Side::Right,
        ),
        (
            TooltipPlacement::RightEnd,
            point(px(152.), px(90.)),
            Side::Right,
        ),
        (
            TooltipPlacement::BottomStart,
            point(px(100.), px(132.)),
            Side::Bottom,
        ),
        (
            TooltipPlacement::Bottom,
            point(px(80.), px(132.)),
            Side::Bottom,
        ),
        (
            TooltipPlacement::BottomEnd,
            point(px(60.), px(132.)),
            Side::Bottom,
        ),
        (
            TooltipPlacement::LeftStart,
            point(px(8.), px(100.)),
            Side::Left,
        ),
        (TooltipPlacement::Left, point(px(8.), px(95.)), Side::Left),
        (
            TooltipPlacement::LeftEnd,
            point(px(8.), px(90.)),
            Side::Left,
        ),
    ];

    for (placement, origin, side) in cases {
        let result = calculate_placement(base_input(placement));
        assert_eq!(result.placement, placement);
        assert_eq!(result.bubble_bounds.origin, origin, "{placement:?}");
        assert_arrow_direction(result.arrow_points, side);
    }
}

#[test]
fn top_bottom_and_left_right_flip_without_losing_alignment() {
    let mut top = base_input(TooltipPlacement::TopStart);
    top.trigger_bounds.origin.y = px(10.);
    let top = calculate_placement(top);
    assert_eq!(top.placement, TooltipPlacement::BottomStart);
    assert_eq!(top.bubble_bounds.left(), px(100.));

    let mut bottom = base_input(TooltipPlacement::BottomEnd);
    bottom.trigger_bounds.origin.y = px(370.);
    let bottom = calculate_placement(bottom);
    assert_eq!(bottom.placement, TooltipPlacement::TopEnd);
    assert_eq!(bottom.bubble_bounds.right(), px(140.));

    let mut left = base_input(TooltipPlacement::LeftStart);
    left.trigger_bounds.origin.x = px(10.);
    let left = calculate_placement(left);
    assert_eq!(left.placement, TooltipPlacement::RightStart);
    assert_eq!(left.bubble_bounds.top(), px(100.));

    let mut right = base_input(TooltipPlacement::RightEnd);
    right.trigger_bounds.origin.x = px(370.);
    let right = calculate_placement(right);
    assert_eq!(right.placement, TooltipPlacement::LeftEnd);
    assert_eq!(right.bubble_bounds.bottom(), px(120.));
}

#[test]
fn all_viewport_edges_and_corners_shift_inside_the_safe_area() {
    let trigger_origins = [
        point(px(0.), px(180.)),
        point(px(380.), px(180.)),
        point(px(180.), px(0.)),
        point(px(180.), px(380.)),
        point(px(0.), px(0.)),
        point(px(380.), px(0.)),
        point(px(0.), px(380.)),
        point(px(380.), px(380.)),
    ];
    for origin in trigger_origins {
        let mut input = base_input(TooltipPlacement::Bottom);
        input.viewport_padding = px(8.);
        input.trigger_bounds.origin = origin;
        let result = calculate_placement(input);
        assert!(result.bubble_bounds.left() >= px(8.));
        assert!(result.bubble_bounds.top() >= px(8.));
        assert!(result.bubble_bounds.right() <= px(392.));
        assert!(result.bubble_bounds.bottom() <= px(392.));
    }
}

#[test]
fn shifted_arrow_targets_the_trigger_and_stays_out_of_rounded_corners() {
    let mut input = base_input(TooltipPlacement::BottomEnd);
    input.trigger_bounds.origin.x = px(2.);
    input.viewport_padding = px(8.);
    input.corner_radius = px(6.);
    let result = calculate_placement(input);
    let center = result.arrow_points[1].x;
    let safe = input.corner_radius + input.arrow_size.width / 2. + input.border_width;

    assert!(center >= result.bubble_bounds.left() + safe);
    assert!(center <= result.bubble_bounds.right() - safe);
    assert!(center >= input.trigger_bounds.left());
    assert!(center <= input.trigger_bounds.right());
}

#[test]
fn normal_space_keeps_the_full_callout_clear_of_the_trigger() {
    for placement in all_placements() {
        let input = base_input(placement);
        let result = calculate_placement(input);
        assert!(!intersects(result.bubble_bounds, input.trigger_bounds));
        let tip = result.arrow_points[1];
        match result.placement.side() {
            Side::Top => assert!(tip.y < input.trigger_bounds.top()),
            Side::Right => assert!(tip.x > input.trigger_bounds.right()),
            Side::Bottom => assert!(tip.y > input.trigger_bounds.bottom()),
            Side::Left => assert!(tip.x < input.trigger_bounds.left()),
        }
    }
}

#[test]
fn hidden_arrow_removes_paths_and_height_but_keeps_anchor_gap() {
    let with_arrow = calculate_placement(base_input(TooltipPlacement::Bottom));
    let mut without_arrow = base_input(TooltipPlacement::Bottom);
    without_arrow.arrow = false;
    let without_arrow = calculate_placement(without_arrow);

    assert_eq!(
        with_arrow.bubble_bounds.top() - without_arrow.bubble_bounds.top(),
        px(8.)
    );
    assert_eq!(without_arrow.bubble_bounds.top(), px(124.));
    assert_eq!(without_arrow.bubble_bounds.top(), px(120.) + px(4.));
    let (fill, stroke) = arrow_paths(false, without_arrow.arrow_points, px(1.));
    assert!(fill.is_none());
    assert!(stroke.is_none());
}

#[test]
fn instance_background_is_used_for_both_bubble_and_arrow() {
    let theme_background: Hsla = rgb(0xeeeeee).into();
    let custom_background: Hsla = rgb(0x222222).into();
    let tooltip = Tooltip::new("自定义背景").bg_color(rgb(0x222222));

    let bubble_background = tooltip_background(&tooltip, theme_background);
    let arrow_background = tooltip_background(&tooltip, theme_background);
    assert_eq!(bubble_background, custom_background);
    assert_eq!(arrow_background, custom_background);
}

#[test]
fn oversized_and_degenerate_viewports_remain_finite_and_non_negative() {
    for viewport in [
        gpui::size(px(50.), px(400.)),
        gpui::size(px(400.), px(20.)),
        gpui::size(px(0.), px(0.)),
    ] {
        let mut input = base_input(TooltipPlacement::Right);
        input.viewport_bounds.size = viewport;
        input.tooltip_size = gpui::size(px(180.), px(90.));
        let result = calculate_placement(input);
        assert!(result.bubble_bounds.size.width >= Pixels::ZERO);
        assert!(result.bubble_bounds.size.height >= Pixels::ZERO);
        assert!(result.bubble_bounds.origin.x.as_f32().is_finite());
        assert!(result.bubble_bounds.origin.y.as_f32().is_finite());
    }

    let mut nan = base_input(TooltipPlacement::Top);
    nan.tooltip_size.width = px(f32::NAN);
    nan.viewport_bounds.size.height = px(f32::NAN);
    let result = calculate_placement(nan);
    assert_eq!(result.bubble_bounds.size.width, Pixels::ZERO);
    assert!(result.bubble_bounds.origin.y.as_f32().is_finite());
}

#[test]
fn resize_recalculates_flip_and_shift_from_current_viewport() {
    let wide = calculate_placement(base_input(TooltipPlacement::Right));
    assert_eq!(wide.placement, TooltipPlacement::Right);

    let mut narrow_input = base_input(TooltipPlacement::Right);
    narrow_input.viewport_bounds.size.width = px(150.);
    let narrow = calculate_placement(narrow_input);
    assert_eq!(narrow.placement, TooltipPlacement::Left);
    assert!(narrow.bubble_bounds.right() <= px(150.));
}

fn base_input(preferred: TooltipPlacement) -> PlacementInput {
    PlacementInput {
        trigger_bounds: Bounds::new(point(px(100.), px(100.)), gpui::size(px(40.), px(20.))),
        tooltip_size: gpui::size(px(80.), px(30.)),
        preferred,
        viewport_bounds: Bounds::new(Point::default(), gpui::size(px(400.), px(400.))),
        viewport_padding: Pixels::ZERO,
        trigger_gap: px(4.),
        arrow_size: gpui::size(px(12.), px(8.)),
        corner_radius: px(4.),
        border_width: px(1.),
        shadow_margin: Pixels::ZERO,
        arrow: true,
    }
}

fn all_placements() -> [TooltipPlacement; 12] {
    [
        TooltipPlacement::TopStart,
        TooltipPlacement::Top,
        TooltipPlacement::TopEnd,
        TooltipPlacement::RightStart,
        TooltipPlacement::Right,
        TooltipPlacement::RightEnd,
        TooltipPlacement::BottomStart,
        TooltipPlacement::Bottom,
        TooltipPlacement::BottomEnd,
        TooltipPlacement::LeftStart,
        TooltipPlacement::Left,
        TooltipPlacement::LeftEnd,
    ]
}

fn assert_arrow_direction(points: [Point<Pixels>; 3], side: Side) {
    let base = points[0];
    let tip = points[1];
    match side {
        Side::Top => assert!(tip.y > base.y),
        Side::Right => assert!(tip.x < base.x),
        Side::Bottom => assert!(tip.y < base.y),
        Side::Left => assert!(tip.x > base.x),
    }
}

fn intersects(a: Bounds<Pixels>, b: Bounds<Pixels>) -> bool {
    a.left() < b.right() && a.right() > b.left() && a.top() < b.bottom() && a.bottom() > b.top()
}

impl Render for TooltipTrigger {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
