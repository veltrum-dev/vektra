use super::*;
use gpui::{AppContext as _, TestAppContext};

#[test]
fn default_config_is_both_auto_overlay() {
    assert_eq!(
        ScrollbarConfig::default(),
        ScrollbarConfig {
            axis: ScrollAxis::Both,
            visibility: ScrollVisibility::Auto,
            gutter: ScrollGutter::Overlay,
        }
    );
}

#[test]
fn thumb_geometry_is_clamped_and_tracks_offset() {
    let geometry = axis_geometry(
        PhysicalAxis::Vertical,
        Bounds::new(point(px(0.), px(0.)), gpui::size(px(14.), px(100.))),
        px(8.),
        px(24.),
        px(100.),
        px(300.),
        px(150.),
    );
    assert_eq!(geometry.thumb_bounds.size.height, px(25.));
    assert_eq!(geometry.thumb_bounds.top(), px(37.5));
    assert_eq!(geometry.thumb_bounds.size.width, px(8.));
}

#[test]
fn thumb_geometry_handles_tracks_shorter_than_the_minimum_thumb() {
    let geometry = axis_geometry(
        PhysicalAxis::Vertical,
        Bounds::new(point(px(0.), px(0.)), gpui::size(px(14.), px(8.))),
        px(8.),
        px(24.),
        px(4.),
        px(100.),
        px(100.),
    );

    assert_eq!(geometry.thumb_bounds.top(), px(0.));
    assert_eq!(geometry.thumb_bounds.size.height, px(8.));
    assert_eq!(geometry.thumb_bounds.size.width, px(8.));
    assert_eq!(pill_radius(geometry.thumb_bounds), px(4.));
}

#[test]
fn hovered_thumb_expands_across_the_axis_without_changing_its_length() {
    let vertical = axis_geometry(
        PhysicalAxis::Vertical,
        Bounds::new(point(px(0.), px(0.)), gpui::size(px(14.), px(100.))),
        px(8.),
        px(24.),
        px(100.),
        px(300.),
        px(150.),
    );
    let expanded = thumb_bounds_with_thickness(vertical, px(10.));

    assert_eq!(expanded.size.width, px(10.));
    assert_eq!(expanded.size.height, vertical.thumb_bounds.size.height);
    assert_eq!(expanded.left(), px(2.));
    assert_eq!(expanded.top(), vertical.thumb_bounds.top());
}

#[test]
fn opacity_curve_is_smooth_clamped_and_reversible() {
    assert_eq!(fade_opacity(0., 1., -1.), 0.);
    assert_eq!(fade_opacity(0., 1., 0.5), 0.5);
    assert_eq!(fade_opacity(0., 1., 2.), 1.);
    assert_eq!(fade_opacity(1., 0., 0.5), 0.5);
}

#[test]
fn track_is_only_visible_for_the_hovered_or_dragged_axis() {
    assert!(!track_is_visible(PhysicalAxis::Vertical, None, None));
    assert!(track_is_visible(
        PhysicalAxis::Vertical,
        Some(PhysicalAxis::Vertical),
        None,
    ));
    assert!(!track_is_visible(
        PhysicalAxis::Horizontal,
        Some(PhysicalAxis::Vertical),
        None,
    ));
    assert!(track_is_visible(
        PhysicalAxis::Horizontal,
        None,
        Some(PhysicalAxis::Horizontal),
    ));
}

#[test]
fn virtual_drag_edges_stay_locked_until_the_pointer_intentionally_leaves() {
    let mut drag = DragState {
        axis: PhysicalAxis::Vertical,
        pointer_offset: px(12.),
        track_start: Pixels::ZERO,
        track_length: px(280.),
        thumb_length: px(24.),
        max_offset: px(1_000.),
        edge_lock: None,
    };
    let travel = px(256.);

    assert_eq!(drag_progress(&mut drag, Pixels::ZERO, travel, true), 0.);
    assert_eq!(drag.edge_lock, Some(DragEdgeLock::Start));
    assert_eq!(drag_progress(&mut drag, px(12.), travel, true), 0.);
    assert!(drag_progress(&mut drag, px(13.), travel, true) > 0.);
    assert_eq!(drag.edge_lock, None);

    assert_eq!(drag_progress(&mut drag, travel, travel, true), 1.);
    assert_eq!(drag.edge_lock, Some(DragEdgeLock::End));
    assert_eq!(drag_progress(&mut drag, travel - px(12.), travel, true), 1.);
    assert!(drag_progress(&mut drag, travel - px(13.), travel, true) < 1.);
    assert_eq!(drag.edge_lock, None);
}

#[test]
fn regular_scrollbar_drag_does_not_snap_to_virtual_edges() {
    let mut drag = DragState {
        axis: PhysicalAxis::Vertical,
        pointer_offset: px(12.),
        track_start: Pixels::ZERO,
        track_length: px(280.),
        thumb_length: px(24.),
        max_offset: px(1_000.),
        edge_lock: None,
    };

    assert!(drag_progress(&mut drag, px(1.), px(256.), false) > 0.);
    assert_eq!(drag.edge_lock, None);
}

#[gpui::test]
fn auto_visibility_fades_in_and_out_without_dropping_the_hit_area_early(cx: &mut TestAppContext) {
    let state = cx.new(|_| ScrollAreaState::new());
    state.update(cx, |state, cx| state.reveal(cx));
    cx.run_until_parked();
    state.read_with(cx, |state, _| {
        assert!(state.auto_visible);
        assert_eq!(state.opacity, 0.);
        assert_eq!(state.opacity_target, 1.);
    });

    advance_fade(AUTO_FADE_IN_DURATION, 1, cx);
    state.read_with(cx, |state, _| assert!((0. ..1.).contains(&state.opacity)));
    advance_fade(
        AUTO_FADE_IN_DURATION,
        fade_step_count(AUTO_FADE_IN_DURATION) - 1,
        cx,
    );
    state.read_with(cx, |state, _| {
        assert!(state.auto_visible);
        assert_eq!(state.opacity, 1.);
    });

    state.update(cx, |state, cx| {
        state.cancel_hide();
        state.fade_to(0., AUTO_FADE_OUT_DURATION, cx);
    });
    cx.run_until_parked();
    advance_fade(AUTO_FADE_OUT_DURATION, 1, cx);
    state.read_with(cx, |state, _| {
        assert!(state.auto_visible);
        assert!((0. ..1.).contains(&state.opacity));
    });
    advance_fade(
        AUTO_FADE_OUT_DURATION,
        fade_step_count(AUTO_FADE_OUT_DURATION) - 1,
        cx,
    );
    state.read_with(cx, |state, _| {
        assert!(!state.auto_visible);
        assert_eq!(state.opacity, 0.);
    });
}

#[gpui::test]
fn reduced_motion_settles_auto_visibility_immediately(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let state = cx.new(|_| ScrollAreaState::new());

    state.update(cx, |state, cx| state.reveal(cx));
    state.read_with(cx, |state, _| {
        assert!(state.auto_visible);
        assert_eq!(state.opacity, 1.);
        assert!(state.fade_task.is_none());
    });

    state.update(cx, |state, cx| {
        state.cancel_hide();
        state.fade_to(0., AUTO_FADE_OUT_DURATION, cx);
    });
    state.read_with(cx, |state, _| {
        assert!(!state.auto_visible);
        assert_eq!(state.opacity, 0.);
        assert!(state.fade_task.is_none());
    });
}

#[gpui::test]
fn leaving_the_track_hides_the_track_but_keeps_the_auto_thumb(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let state = cx.new(|_| ScrollAreaState::new());

    state.update(cx, |state, cx| {
        state.reveal(cx);
        state.set_hovered(
            Some(PhysicalAxis::Vertical),
            Some(PhysicalAxis::Vertical),
            cx,
        );
    });
    state.read_with(cx, |state, _| {
        assert!(track_is_visible(
            PhysicalAxis::Vertical,
            state.hovered_axis,
            state.drag.map(|drag| drag.axis),
        ));
        assert!(state.auto_visible);
        assert_eq!(state.opacity, 1.);
    });

    state.update(cx, |state, cx| state.set_hovered(None, None, cx));
    state.read_with(cx, |state, _| {
        assert!(!track_is_visible(
            PhysicalAxis::Vertical,
            state.hovered_axis,
            state.drag.map(|drag| drag.axis),
        ));
        assert!(state.auto_visible);
        assert_eq!(state.opacity, 1.);
        assert!(state.hide_task.is_some());
    });
}

#[gpui::test]
fn never_visibility_can_clear_interaction_state_before_a_later_mode_switch(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let state = cx.new(|_| ScrollAreaState::new());

    state.update(cx, |state, cx| {
        state.reveal(cx);
        state.set_hovered(
            Some(PhysicalAxis::Horizontal),
            Some(PhysicalAxis::Horizontal),
            cx,
        );
        state.reset_interaction(cx);
    });
    state.read_with(cx, |state, _| {
        assert_eq!(state.hovered_axis, None);
        assert_eq!(state.hovered_thumb_axis, None);
        assert!(state.drag.is_none());
        assert!(!state.auto_visible);
        assert_eq!(state.opacity, 0.);
        assert_eq!(state.opacity_target, 0.);
        assert!(state.hide_task.is_none());
        assert!(state.fade_task.is_none());
    });
}

fn advance_fade(duration: Duration, steps: u32, cx: &mut TestAppContext) {
    let step_duration = duration / fade_step_count(duration);
    for _ in 0..steps {
        cx.executor().advance_clock(step_duration);
        cx.run_until_parked();
    }
}
