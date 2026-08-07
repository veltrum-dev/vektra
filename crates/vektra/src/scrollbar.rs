//! 可组合的滚动区域与自绘 Scrollbar。

use crate::theme;
use gpui::{
    App, BorderStyle, Bounds, BoxShadow, Context, CursorStyle, DispatchPhase, Div, Element,
    ElementId, Entity, GlobalElementId, Hitbox, HitboxBehavior, InspectorElementId,
    InteractiveElement, IntoElement, KeyDownEvent, LayoutId, Modifiers, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Role, ScrollHandle,
    ScrollWheelEvent, SharedString, Stateful, StatefulInteractiveElement, Styled, Task, Window,
    point, px, quad,
};
use std::{panic::Location, time::Duration};
use vektra_theme::ScrollbarTokens;

const AUTO_HIDE_DELAY: Duration = Duration::from_millis(900);
const AUTO_FADE_IN_DURATION: Duration = Duration::from_millis(120);
const AUTO_FADE_OUT_DURATION: Duration = Duration::from_millis(180);
const FADE_FRAME_INTERVAL: Duration = Duration::from_millis(15);
const KEYBOARD_STEP: Pixels = px(40.);

/// Scrollbar 启用的滚动轴。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollAxis {
    /// 仅允许垂直滚动。
    Vertical,
    /// 仅允许水平滚动。
    Horizontal,
    /// 同时允许水平和垂直滚动；这是默认值。
    #[default]
    Both,
}

impl ScrollAxis {
    const fn has_vertical(self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }

    const fn has_horizontal(self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }
}

/// Scrollbar 的显隐策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollVisibility {
    /// 交互时显示，停止交互后自动隐藏；这是默认值。
    #[default]
    Auto,
    /// 有溢出内容时始终显示。
    Always,
    /// 永不绘制 Scrollbar，但仍保留内容的滚动能力。
    Never,
}

/// Scrollbar 是否占用布局空间。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollGutter {
    /// 覆盖在内容之上，不改变内容可用尺寸；这是默认值。
    #[default]
    Overlay,
    /// 始终为 Scrollbar 命中区域预留空间，避免内容宽高随显隐变化。
    Stable,
}

/// Scrollbar 的公共配置。
///
/// `Default` 等价于 `Both + Auto + Overlay`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollbarConfig {
    /// 启用的滚动轴。
    pub axis: ScrollAxis,
    /// 显隐策略。
    pub visibility: ScrollVisibility,
    /// 布局 gutter 策略。
    pub gutter: ScrollGutter,
}

impl ScrollbarConfig {
    /// 创建默认配置。
    pub const fn new() -> Self {
        Self {
            axis: ScrollAxis::Both,
            visibility: ScrollVisibility::Auto,
            gutter: ScrollGutter::Overlay,
        }
    }

    /// 设置启用的滚动轴。
    pub const fn axis(mut self, axis: ScrollAxis) -> Self {
        self.axis = axis;
        self
    }

    /// 设置显隐策略。
    pub const fn visibility(mut self, visibility: ScrollVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// 设置 gutter 策略。
    pub const fn gutter(mut self, gutter: ScrollGutter) -> Self {
        self.gutter = gutter;
        self
    }
}

enum ScrollAreaBase {
    Plain(Div),
    Stateful(Stateful<Div>),
}

/// 由 [`ScrollableExt`] 创建的滚动区域。
///
/// Scrollbar 配置方法带有 `scrollbar_` 前缀，避免和被包装元素的 `axis`、
/// `visibility` 等能力发生命名冲突。尺寸、布局和子项应在调用 `.scrollbar()` 前完成。
pub struct ScrollArea {
    base: Option<ScrollAreaBase>,
    prepared: Option<Stateful<Div>>,
    config: ScrollbarConfig,
    external_handle: Option<ScrollHandle>,
    id: Option<ElementId>,
    base_id: Option<ElementId>,
    aria_label: Option<SharedString>,
    caller: &'static Location<'static>,
    state: Option<Entity<ScrollAreaState>>,
    handle: Option<ScrollHandle>,
    tokens: Option<ScrollbarTokens>,
}

impl ScrollArea {
    fn plain(
        base: Div,
        config: ScrollbarConfig,
        external_handle: Option<ScrollHandle>,
        caller: &'static Location<'static>,
    ) -> Self {
        Self::new(ScrollAreaBase::Plain(base), config, external_handle, caller)
    }

    fn stateful(
        base: Stateful<Div>,
        config: ScrollbarConfig,
        external_handle: Option<ScrollHandle>,
        caller: &'static Location<'static>,
    ) -> Self {
        Self::new(
            ScrollAreaBase::Stateful(base),
            config,
            external_handle,
            caller,
        )
    }

    fn new(
        base: ScrollAreaBase,
        config: ScrollbarConfig,
        external_handle: Option<ScrollHandle>,
        caller: &'static Location<'static>,
    ) -> Self {
        let base_id = match &base {
            ScrollAreaBase::Plain(base) => Element::id(base),
            ScrollAreaBase::Stateful(base) => Element::id(base),
        };
        Self {
            base: Some(base),
            prepared: None,
            config,
            external_handle,
            id: None,
            base_id,
            aria_label: None,
            caller,
            state: None,
            handle: None,
            tokens: None,
        }
    }

    /// 覆盖启用的滚动轴。
    pub fn scrollbar_axis(mut self, axis: ScrollAxis) -> Self {
        self.config.axis = axis;
        self
    }

    /// 覆盖显隐策略。
    pub fn scrollbar_visibility(mut self, visibility: ScrollVisibility) -> Self {
        self.config.visibility = visibility;
        self
    }

    /// 覆盖 gutter 策略。
    pub fn scrollbar_gutter(mut self, gutter: ScrollGutter) -> Self {
        self.config.gutter = gutter;
        self
    }

    /// 设置显式稳定 ID。
    ///
    /// 在循环或条件分支中创建多个滚动区域时应设置 ID，以免调用位置相同的实例共享
    /// 短生命周期交互状态。
    pub fn scrollbar_id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 设置滚动区域的无障碍名称。
    pub fn scrollbar_aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    fn state_key(&self) -> ElementId {
        self.id
            .clone()
            .or_else(|| self.base_id.clone())
            .unwrap_or(ElementId::CodeLocation(*self.caller))
    }

    fn element_id(&self) -> ElementId {
        self.id
            .clone()
            .or_else(|| self.base_id.clone())
            .unwrap_or(ElementId::CodeLocation(*self.caller))
    }
}

/// 为 GPUI `Div` 添加 Vektra Scrollbar 的扩展能力。
pub trait ScrollableExt: Sized {
    /// 使用 `Both + Auto + Overlay` 创建滚动区域。
    fn scrollbar(self) -> ScrollArea;

    /// 创建只允许垂直滚动的区域。
    fn vertical_scrollbar(self) -> ScrollArea;

    /// 创建只允许水平滚动的区域。
    fn horizontal_scrollbar(self) -> ScrollArea;

    /// 使用外部 [`ScrollHandle`] 创建滚动区域。
    fn scrollbar_for(self, handle: &ScrollHandle) -> ScrollArea;

    /// 使用外部 [`ScrollHandle`] 创建只允许垂直滚动的区域。
    fn vertical_scrollbar_for(self, handle: &ScrollHandle) -> ScrollArea;

    /// 使用外部 [`ScrollHandle`] 创建只允许水平滚动的区域。
    fn horizontal_scrollbar_for(self, handle: &ScrollHandle) -> ScrollArea;

    /// 使用完整配置创建滚动区域。
    fn scrollbar_with(self, config: ScrollbarConfig) -> ScrollArea;

    /// 使用默认配置并覆盖滚动轴。
    fn scrollbar_with_axis(self, axis: ScrollAxis) -> ScrollArea;

    /// 使用默认配置并覆盖显隐策略。
    fn scrollbar_with_visibility(self, visibility: ScrollVisibility) -> ScrollArea;

    /// 使用默认配置并覆盖 gutter 策略。
    fn scrollbar_with_gutter(self, gutter: ScrollGutter) -> ScrollArea;
}

macro_rules! impl_scrollable_ext {
    ($ty:ty, $constructor:ident) => {
        impl ScrollableExt for $ty {
            #[track_caller]
            fn scrollbar(self) -> ScrollArea {
                ScrollArea::$constructor(self, ScrollbarConfig::default(), None, Location::caller())
            }

            #[track_caller]
            fn vertical_scrollbar(self) -> ScrollArea {
                ScrollArea::$constructor(
                    self,
                    ScrollbarConfig::default().axis(ScrollAxis::Vertical),
                    None,
                    Location::caller(),
                )
            }

            #[track_caller]
            fn horizontal_scrollbar(self) -> ScrollArea {
                ScrollArea::$constructor(
                    self,
                    ScrollbarConfig::default().axis(ScrollAxis::Horizontal),
                    None,
                    Location::caller(),
                )
            }

            #[track_caller]
            fn scrollbar_for(self, handle: &ScrollHandle) -> ScrollArea {
                ScrollArea::$constructor(
                    self,
                    ScrollbarConfig::default(),
                    Some(handle.clone()),
                    Location::caller(),
                )
            }

            #[track_caller]
            fn vertical_scrollbar_for(self, handle: &ScrollHandle) -> ScrollArea {
                ScrollArea::$constructor(
                    self,
                    ScrollbarConfig::default().axis(ScrollAxis::Vertical),
                    Some(handle.clone()),
                    Location::caller(),
                )
            }

            #[track_caller]
            fn horizontal_scrollbar_for(self, handle: &ScrollHandle) -> ScrollArea {
                ScrollArea::$constructor(
                    self,
                    ScrollbarConfig::default().axis(ScrollAxis::Horizontal),
                    Some(handle.clone()),
                    Location::caller(),
                )
            }

            #[track_caller]
            fn scrollbar_with(self, config: ScrollbarConfig) -> ScrollArea {
                ScrollArea::$constructor(self, config, None, Location::caller())
            }

            #[track_caller]
            fn scrollbar_with_axis(self, axis: ScrollAxis) -> ScrollArea {
                ScrollArea::$constructor(
                    self,
                    ScrollbarConfig::default().axis(axis),
                    None,
                    Location::caller(),
                )
            }

            #[track_caller]
            fn scrollbar_with_visibility(self, visibility: ScrollVisibility) -> ScrollArea {
                ScrollArea::$constructor(
                    self,
                    ScrollbarConfig::default().visibility(visibility),
                    None,
                    Location::caller(),
                )
            }

            #[track_caller]
            fn scrollbar_with_gutter(self, gutter: ScrollGutter) -> ScrollArea {
                ScrollArea::$constructor(
                    self,
                    ScrollbarConfig::default().gutter(gutter),
                    None,
                    Location::caller(),
                )
            }
        }
    };
}

impl_scrollable_ext!(Div, plain);
impl_scrollable_ext!(Stateful<Div>, stateful);

impl IntoElement for ScrollArea {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

struct ScrollAreaState {
    internal_handle: ScrollHandle,
    auto_visible: bool,
    opacity: f32,
    opacity_target: f32,
    hovered_axis: Option<PhysicalAxis>,
    hovered_thumb_axis: Option<PhysicalAxis>,
    drag: Option<DragState>,
    hide_generation: u64,
    hide_task: Option<Task<()>>,
    fade_generation: u64,
    fade_task: Option<Task<()>>,
}

impl ScrollAreaState {
    fn new() -> Self {
        Self {
            internal_handle: ScrollHandle::new(),
            auto_visible: false,
            opacity: 0.,
            opacity_target: 0.,
            hovered_axis: None,
            hovered_thumb_axis: None,
            drag: None,
            hide_generation: 0,
            hide_task: None,
            fade_generation: 0,
            fade_task: None,
        }
    }

    fn cancel_hide(&mut self) {
        self.hide_generation = self.hide_generation.wrapping_add(1);
        self.hide_task = None;
    }

    fn reveal(&mut self, cx: &mut Context<Self>) {
        self.auto_visible = true;
        self.fade_to(1., AUTO_FADE_IN_DURATION, cx);
        self.cancel_hide();
        if self.hovered_axis.is_some() || self.drag.is_some() {
            return;
        }

        let generation = self.hide_generation;
        self.hide_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(AUTO_HIDE_DELAY).await;
            let _ = this.update(cx, |this, cx| {
                if this.hide_generation != generation {
                    return;
                }
                this.hide_task = None;
                if this.hovered_axis.is_none() && this.drag.is_none() {
                    this.fade_to(0., AUTO_FADE_OUT_DURATION, cx);
                }
            });
        }));
    }

    fn set_hovered(
        &mut self,
        hovered_axis: Option<PhysicalAxis>,
        hovered_thumb_axis: Option<PhysicalAxis>,
        cx: &mut Context<Self>,
    ) {
        let axis_changed = self.hovered_axis != hovered_axis;
        if !axis_changed && self.hovered_thumb_axis == hovered_thumb_axis {
            return;
        }
        self.hovered_axis = hovered_axis;
        self.hovered_thumb_axis = hovered_thumb_axis;
        if axis_changed {
            if hovered_axis.is_some() {
                self.auto_visible = true;
                self.fade_to(1., AUTO_FADE_IN_DURATION, cx);
                self.cancel_hide();
            } else {
                self.reveal(cx);
            }
        }
        cx.notify();
    }

    fn start_drag(&mut self, drag: DragState, cx: &mut Context<Self>) {
        self.drag = Some(drag);
        self.hovered_axis = Some(drag.axis);
        self.hovered_thumb_axis = Some(drag.axis);
        self.auto_visible = true;
        self.fade_to(1., AUTO_FADE_IN_DURATION, cx);
        self.cancel_hide();
        cx.notify();
    }

    fn end_drag(&mut self, cx: &mut Context<Self>) {
        if self.drag.take().is_some() {
            self.hovered_axis = None;
            self.hovered_thumb_axis = None;
            self.reveal(cx);
        }
    }

    fn reset_interaction(&mut self, cx: &mut Context<Self>) {
        if self.hovered_axis.is_none()
            && self.hovered_thumb_axis.is_none()
            && self.drag.is_none()
            && !self.auto_visible
            && self.opacity == 0.
            && self.opacity_target == 0.
            && self.hide_task.is_none()
            && self.fade_task.is_none()
        {
            return;
        }

        self.hovered_axis = None;
        self.hovered_thumb_axis = None;
        self.drag = None;
        self.hide_generation = self.hide_generation.wrapping_add(1);
        self.hide_task = None;
        self.fade_generation = self.fade_generation.wrapping_add(1);
        self.fade_task = None;
        self.auto_visible = false;
        self.opacity = 0.;
        self.opacity_target = 0.;
        cx.notify();
    }

    fn fade_to(&mut self, target: f32, duration: Duration, cx: &mut Context<Self>) {
        let target = target.clamp(0., 1.);
        if target > 0. {
            self.auto_visible = true;
        }
        if (self.opacity_target - target).abs() <= f32::EPSILON {
            return;
        }

        self.opacity_target = target;
        self.fade_generation = self.fade_generation.wrapping_add(1);
        let generation = self.fade_generation;
        self.fade_task = None;

        if cx.reduce_motion() || duration.is_zero() {
            self.opacity = target;
            self.auto_visible = target > 0.;
            cx.notify();
            return;
        }

        let start = self.opacity;
        let steps = fade_step_count(duration);
        let step_duration = duration / steps;
        self.fade_task = Some(cx.spawn(async move |this, cx| {
            for step in 1..=steps {
                cx.background_executor().timer(step_duration).await;
                let keep_animating = this
                    .update(cx, |this, cx| {
                        if this.fade_generation != generation {
                            return false;
                        }

                        let progress = if cx.reduce_motion() {
                            1.
                        } else {
                            step as f32 / steps as f32
                        };
                        this.opacity = fade_opacity(start, target, progress);
                        let done = progress >= 1.;
                        if done {
                            this.opacity = target;
                            this.opacity_target = target;
                            this.auto_visible = target > 0.;
                            this.fade_task = None;
                        }
                        cx.notify();
                        !done
                    })
                    .unwrap_or(false);
                if !keep_animating {
                    break;
                }
            }
        }));
        cx.notify();
    }
}

fn fade_step_count(duration: Duration) -> u32 {
    let duration_ms = duration.as_millis().max(1);
    let interval_ms = FADE_FRAME_INTERVAL.as_millis().max(1);
    duration_ms.div_ceil(interval_ms).min(u32::MAX as u128) as u32
}

fn fade_opacity(start: f32, target: f32, progress: f32) -> f32 {
    let progress = progress.clamp(0., 1.);
    let eased = progress * progress * (3. - 2. * progress);
    (start + (target - start) * eased).clamp(0., 1.)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PhysicalAxis {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy)]
struct DragState {
    axis: PhysicalAxis,
    pointer_offset: Pixels,
    track_start: Pixels,
    track_length: Pixels,
    thumb_length: Pixels,
    max_offset: Pixels,
}

#[derive(Debug, Clone, Copy)]
struct AxisGeometry {
    axis: PhysicalAxis,
    track_bounds: Bounds<Pixels>,
    visual_track_bounds: Bounds<Pixels>,
    thumb_bounds: Bounds<Pixels>,
    max_offset: Pixels,
}

impl AxisGeometry {
    fn pointer_position(self, position: Point<Pixels>) -> Pixels {
        match self.axis {
            PhysicalAxis::Vertical => position.y,
            PhysicalAxis::Horizontal => position.x,
        }
    }

    fn track_start(self) -> Pixels {
        match self.axis {
            PhysicalAxis::Vertical => self.track_bounds.top(),
            PhysicalAxis::Horizontal => self.track_bounds.left(),
        }
    }

    fn track_length(self) -> Pixels {
        match self.axis {
            PhysicalAxis::Vertical => self.track_bounds.size.height,
            PhysicalAxis::Horizontal => self.track_bounds.size.width,
        }
    }

    fn thumb_start(self) -> Pixels {
        match self.axis {
            PhysicalAxis::Vertical => self.thumb_bounds.top(),
            PhysicalAxis::Horizontal => self.thumb_bounds.left(),
        }
    }

    fn thumb_length(self) -> Pixels {
        match self.axis {
            PhysicalAxis::Vertical => self.thumb_bounds.size.height,
            PhysicalAxis::Horizontal => self.thumb_bounds.size.width,
        }
    }
}

#[doc(hidden)]
pub struct ScrollbarPrepaint {
    inner: <Stateful<Div> as Element>::PrepaintState,
    viewport_hitbox: Option<Hitbox>,
    vertical: Option<(AxisGeometry, Hitbox)>,
    horizontal: Option<(AxisGeometry, Hitbox)>,
    viewport_bounds: Bounds<Pixels>,
    opacity: f32,
}

impl Element for ScrollArea {
    type RequestLayoutState = <Stateful<Div> as Element>::RequestLayoutState;
    type PrepaintState = ScrollbarPrepaint;

    fn id(&self) -> Option<ElementId> {
        Some(self.element_id())
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        Some(self.caller)
    }

    fn a11y_role(&self) -> Option<gpui::accesskit::Role> {
        Some(Role::ScrollView)
    }

    fn write_a11y_info(&self, node: &mut gpui::accesskit::Node) {
        if let Some(prepared) = &self.prepared {
            prepared.write_a11y_info(node);
        }
    }

    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut gpui::A11ySubtreeBuilder,
    ) {
        if let Some(prepared) = &mut self.prepared {
            prepared.a11y_synthetic_children(&mut prepaint.inner, builder);
        }
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let key = self.state_key();
        let state =
            window.use_keyed_state((key.clone(), "state"), cx, |_, _| ScrollAreaState::new());
        let handle = self
            .external_handle
            .clone()
            .unwrap_or_else(|| state.read(cx).internal_handle.clone());
        let tokens = theme::current_theme(window, cx).scrollbar;
        let focus_color = tokens.focus_ring;
        let focus_width = tokens.focus_width;
        let gutter_width = match self.config.gutter {
            ScrollGutter::Overlay => Pixels::ZERO,
            ScrollGutter::Stable => tokens.hit_thickness,
        };

        let base = self.base.take().expect("ScrollArea 每帧只允许布局一次");
        let mut element = match base {
            ScrollAreaBase::Plain(base) => base.id((key, "viewport")),
            ScrollAreaBase::Stateful(base) => base,
        };
        element = match self.config.axis {
            ScrollAxis::Vertical => element.overflow_x_hidden().overflow_y_scroll(),
            ScrollAxis::Horizontal => element.overflow_x_scroll().overflow_y_hidden(),
            ScrollAxis::Both => element.overflow_scroll(),
        };
        element = element
            .track_scroll(&handle)
            .scrollbar_width(gutter_width)
            .role(Role::ScrollView)
            .tab_index(0)
            .focus_visible(move |style| {
                style.shadow(vec![
                    BoxShadow::new(Pixels::ZERO, Pixels::ZERO, focus_color)
                        .spread_radius(focus_width),
                ])
            });

        if let Some(label) = self.aria_label.clone() {
            element = element.aria_label(label);
        }

        let keyboard_handle = handle.clone();
        let axis = self.config.axis;
        element = element.on_key_down(move |event, window, cx| {
            if handle_scroll_key(event, axis, &keyboard_handle) {
                window.prevent_default();
                window.refresh();
                cx.stop_propagation();
            }
        });

        let result = element.request_layout(id, inspector_id, window, cx);
        self.prepared = Some(element);
        self.state = Some(state);
        self.handle = Some(handle);
        self.tokens = Some(tokens);
        result
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let inner = self
            .prepared
            .as_mut()
            .expect("ScrollArea 必须先完成布局")
            .prepaint(id, inspector_id, bounds, request_layout, window, cx);
        let handle = self.handle.as_ref().expect("ScrollArea 缺少 ScrollHandle");
        let tokens = self.tokens.expect("ScrollArea 缺少主题 token");
        let geometry = scrollbar_geometry(bounds, handle, self.config.axis, tokens);
        let (paint_visible, opacity) = match self.config.visibility {
            ScrollVisibility::Always => (true, 1.),
            ScrollVisibility::Never => (false, 0.),
            ScrollVisibility::Auto => self.state.as_ref().map_or((false, 0.), |state| {
                let state = state.read(cx);
                (state.auto_visible, state.opacity)
            }),
        };
        let viewport_hitbox = (self.config.visibility != ScrollVisibility::Never)
            .then(|| window.insert_hitbox(bounds, HitboxBehavior::Normal));
        let vertical = geometry.vertical.filter(|_| paint_visible).map(|geometry| {
            let hitbox = window.insert_hitbox(
                geometry.track_bounds,
                HitboxBehavior::BlockMouseExceptScroll,
            );
            (geometry, hitbox)
        });
        let horizontal = geometry
            .horizontal
            .filter(|_| paint_visible)
            .map(|geometry| {
                let hitbox = window.insert_hitbox(
                    geometry.track_bounds,
                    HitboxBehavior::BlockMouseExceptScroll,
                );
                (geometry, hitbox)
            });

        ScrollbarPrepaint {
            inner,
            viewport_hitbox,
            vertical,
            horizontal,
            viewport_bounds: bounds,
            opacity,
        }
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.prepared
            .as_mut()
            .expect("ScrollArea 必须先完成布局")
            .paint(
                id,
                inspector_id,
                bounds,
                request_layout,
                &mut prepaint.inner,
                window,
                cx,
            );

        let Some(state) = self.state.clone() else {
            return;
        };
        let Some(handle) = self.handle.clone() else {
            return;
        };
        if self.config.visibility == ScrollVisibility::Never {
            state.update(cx, |state, cx| state.reset_interaction(cx));
        }
        let tokens = self.tokens.expect("ScrollArea 缺少主题 token");
        let opacity = prepaint.opacity;
        let (hovered_axis, hovered_thumb_axis, dragging_axis) = {
            let state = state.read(cx);
            (
                state.hovered_axis,
                state.hovered_thumb_axis,
                state.drag.map(|drag| drag.axis),
            )
        };

        for (geometry, hitbox) in [prepaint.vertical.as_ref(), prepaint.horizontal.as_ref()]
            .into_iter()
            .flatten()
        {
            window.set_cursor_style(CursorStyle::Arrow, hitbox);
            if track_is_visible(geometry.axis, hovered_axis, dragging_axis) {
                window.paint_quad(quad(
                    geometry.visual_track_bounds,
                    tokens.radius,
                    tokens.track.opacity(opacity),
                    Pixels::ZERO,
                    gpui::transparent_black(),
                    BorderStyle::default(),
                ));
            }
            let thumb_color = if dragging_axis == Some(geometry.axis) {
                tokens.thumb_pressed
            } else if hovered_thumb_axis == Some(geometry.axis) {
                tokens.thumb_hover
            } else {
                tokens.thumb
            };
            let thumb_active =
                hovered_thumb_axis == Some(geometry.axis) || dragging_axis == Some(geometry.axis);
            let thumb_bounds = thumb_bounds_with_thickness(
                *geometry,
                if thumb_active {
                    tokens.thumb_hover_thickness
                } else {
                    tokens.thickness
                }
                .min(tokens.hit_thickness),
            );
            window.paint_quad(quad(
                thumb_bounds,
                pill_radius(thumb_bounds),
                thumb_color.opacity(opacity),
                Pixels::ZERO,
                gpui::transparent_black(),
                BorderStyle::default(),
            ));
        }

        register_scrollbar_events(
            ScrollbarEventContext {
                state,
                handle,
                visibility: self.config.visibility,
                viewport_bounds: prepaint.viewport_bounds,
                viewport_hitbox: prepaint.viewport_hitbox.clone(),
                vertical: prepaint.vertical.clone(),
                horizontal: prepaint.horizontal.clone(),
                thumb_hover_thickness: tokens.thumb_hover_thickness.min(tokens.hit_thickness),
            },
            window,
        );
    }
}

fn pill_radius(bounds: Bounds<Pixels>) -> Pixels {
    bounds.size.width.min(bounds.size.height).max(Pixels::ZERO) / 2.
}

fn thumb_bounds_with_thickness(geometry: AxisGeometry, thickness: Pixels) -> Bounds<Pixels> {
    match geometry.axis {
        PhysicalAxis::Vertical => {
            let thickness = thickness
                .max(Pixels::ZERO)
                .min(geometry.track_bounds.size.width);
            Bounds::new(
                point(
                    geometry.track_bounds.left()
                        + (geometry.track_bounds.size.width - thickness) / 2.,
                    geometry.thumb_bounds.top(),
                ),
                gpui::size(thickness, geometry.thumb_bounds.size.height),
            )
        }
        PhysicalAxis::Horizontal => {
            let thickness = thickness
                .max(Pixels::ZERO)
                .min(geometry.track_bounds.size.height);
            Bounds::new(
                point(
                    geometry.thumb_bounds.left(),
                    geometry.track_bounds.top()
                        + (geometry.track_bounds.size.height - thickness) / 2.,
                ),
                gpui::size(geometry.thumb_bounds.size.width, thickness),
            )
        }
    }
}

fn track_is_visible(
    axis: PhysicalAxis,
    hovered_axis: Option<PhysicalAxis>,
    dragging_axis: Option<PhysicalAxis>,
) -> bool {
    hovered_axis == Some(axis) || dragging_axis == Some(axis)
}

#[derive(Default)]
struct ScrollbarGeometry {
    vertical: Option<AxisGeometry>,
    horizontal: Option<AxisGeometry>,
}

fn scrollbar_geometry(
    bounds: Bounds<Pixels>,
    handle: &ScrollHandle,
    axis: ScrollAxis,
    tokens: ScrollbarTokens,
) -> ScrollbarGeometry {
    let max = handle.max_offset();
    let offset = handle.offset();
    let viewport = handle.bounds().size;
    let vertical_overflow = axis.has_vertical() && max.y > Pixels::ZERO;
    let horizontal_overflow = axis.has_horizontal() && max.x > Pixels::ZERO;
    let hit = tokens.hit_thickness.max(tokens.thickness).max(Pixels::ZERO);
    let thickness = tokens.thickness.min(hit).max(Pixels::ZERO);

    let vertical = vertical_overflow.then(|| {
        let track_length = (bounds.size.height
            - if horizontal_overflow {
                hit
            } else {
                Pixels::ZERO
            })
        .max(Pixels::ZERO);
        let track_bounds = Bounds::new(
            point(bounds.right() - hit, bounds.top()),
            gpui::size(hit, track_length),
        );
        axis_geometry(
            PhysicalAxis::Vertical,
            track_bounds,
            thickness,
            tokens.min_thumb_length,
            viewport.height,
            max.y,
            -offset.y,
        )
    });
    let horizontal = horizontal_overflow.then(|| {
        let track_length = (bounds.size.width - if vertical_overflow { hit } else { Pixels::ZERO })
            .max(Pixels::ZERO);
        let track_bounds = Bounds::new(
            point(bounds.left(), bounds.bottom() - hit),
            gpui::size(track_length, hit),
        );
        axis_geometry(
            PhysicalAxis::Horizontal,
            track_bounds,
            thickness,
            tokens.min_thumb_length,
            viewport.width,
            max.x,
            -offset.x,
        )
    });

    ScrollbarGeometry {
        vertical,
        horizontal,
    }
}

fn axis_geometry(
    axis: PhysicalAxis,
    track_bounds: Bounds<Pixels>,
    thickness: Pixels,
    min_thumb_length: Pixels,
    viewport_length: Pixels,
    max_offset: Pixels,
    current_offset: Pixels,
) -> AxisGeometry {
    let track_length = match axis {
        PhysicalAxis::Vertical => track_bounds.size.height,
        PhysicalAxis::Horizontal => track_bounds.size.width,
    };
    let viewport_length = viewport_length.max(Pixels::ZERO);
    let content_length = viewport_length + max_offset.max(Pixels::ZERO);
    let proportional = if content_length > Pixels::ZERO {
        track_length * (viewport_length / content_length)
    } else {
        track_length
    };
    let thumb_length = proportional
        .max(min_thumb_length)
        .min(track_length)
        .max(Pixels::ZERO);
    let travel = (track_length - thumb_length).max(Pixels::ZERO);
    let progress = if max_offset > Pixels::ZERO {
        (current_offset.max(Pixels::ZERO).min(max_offset) / max_offset).clamp(0., 1.)
    } else {
        0.
    };
    let thumb_offset = travel * progress;

    let visual_track_bounds = match axis {
        PhysicalAxis::Vertical => Bounds::new(
            point(
                track_bounds.left() + (track_bounds.size.width - thickness) / 2.,
                track_bounds.top(),
            ),
            gpui::size(thickness, track_bounds.size.height),
        ),
        PhysicalAxis::Horizontal => Bounds::new(
            point(
                track_bounds.left(),
                track_bounds.top() + (track_bounds.size.height - thickness) / 2.,
            ),
            gpui::size(track_bounds.size.width, thickness),
        ),
    };
    let thumb_bounds = match axis {
        PhysicalAxis::Vertical => Bounds::new(
            point(
                visual_track_bounds.left(),
                track_bounds.top() + thumb_offset,
            ),
            gpui::size(thickness, thumb_length),
        ),
        PhysicalAxis::Horizontal => Bounds::new(
            point(
                track_bounds.left() + thumb_offset,
                visual_track_bounds.top(),
            ),
            gpui::size(thumb_length, thickness),
        ),
    };

    AxisGeometry {
        axis,
        track_bounds,
        visual_track_bounds,
        thumb_bounds,
        max_offset,
    }
}

struct ScrollbarEventContext {
    state: Entity<ScrollAreaState>,
    handle: ScrollHandle,
    visibility: ScrollVisibility,
    viewport_bounds: Bounds<Pixels>,
    viewport_hitbox: Option<Hitbox>,
    vertical: Option<(AxisGeometry, Hitbox)>,
    horizontal: Option<(AxisGeometry, Hitbox)>,
    thumb_hover_thickness: Pixels,
}

fn register_scrollbar_events(context: ScrollbarEventContext, window: &mut Window) {
    let ScrollbarEventContext {
        state,
        handle,
        visibility,
        viewport_bounds,
        viewport_hitbox,
        vertical,
        horizontal,
        thumb_hover_thickness,
    } = context;
    if visibility == ScrollVisibility::Never {
        return;
    }

    if visibility == ScrollVisibility::Auto
        && let Some(viewport_hitbox) = viewport_hitbox
    {
        let state_for_scroll = state.clone();
        window.on_mouse_event(move |_: &ScrollWheelEvent, phase, window, cx| {
            if phase == DispatchPhase::Bubble && viewport_hitbox.should_handle_scroll(window) {
                state_for_scroll.update(cx, |state, cx| state.reveal(cx));
            }
        });
    }

    let vertical_for_move = vertical.as_ref().map(|(geometry, _)| *geometry);
    let horizontal_for_move = horizontal.as_ref().map(|(geometry, _)| *geometry);
    let state_for_move = state.clone();
    let handle_for_move = handle.clone();
    window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
        if phase != DispatchPhase::Bubble {
            return;
        }

        let drag = state_for_move.read(cx).drag;
        if let Some(drag) = drag {
            if event.dragging() {
                apply_drag(&handle_for_move, drag, event.position);
                window.refresh();
                cx.stop_propagation();
                return;
            }
            state_for_move.update(cx, |state, cx| state.end_drag(cx));
        }

        let hovered = vertical_for_move
            .filter(|geometry| geometry.track_bounds.contains(&event.position))
            .map(|_| PhysicalAxis::Vertical)
            .or_else(|| {
                horizontal_for_move
                    .filter(|geometry| geometry.track_bounds.contains(&event.position))
                    .map(|_| PhysicalAxis::Horizontal)
            });
        let hovered_thumb = vertical_for_move
            .filter(|geometry| {
                thumb_bounds_with_thickness(*geometry, thumb_hover_thickness)
                    .contains(&event.position)
            })
            .map(|_| PhysicalAxis::Vertical)
            .or_else(|| {
                horizontal_for_move
                    .filter(|geometry| {
                        thumb_bounds_with_thickness(*geometry, thumb_hover_thickness)
                            .contains(&event.position)
                    })
                    .map(|_| PhysicalAxis::Horizontal)
            });
        state_for_move.update(cx, |state, cx| {
            state.set_hovered(hovered, hovered_thumb, cx);
            if visibility == ScrollVisibility::Auto
                && viewport_bounds.contains(&event.position)
                && hovered.is_none()
            {
                state.reveal(cx);
            }
        });
    });

    let state_for_down = state.clone();
    let handle_for_down = handle.clone();
    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
        if phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
            return;
        }
        let target = vertical
            .as_ref()
            .filter(|(_, hitbox)| hitbox.is_hovered(window))
            .map(|(geometry, _)| *geometry)
            .or_else(|| {
                horizontal
                    .as_ref()
                    .filter(|(_, hitbox)| hitbox.is_hovered(window))
                    .map(|(geometry, _)| *geometry)
            });
        let Some(geometry) = target else {
            return;
        };

        let pointer = geometry.pointer_position(event.position);
        let pointer_offset = if thumb_bounds_with_thickness(geometry, thumb_hover_thickness)
            .contains(&event.position)
        {
            pointer - geometry.thumb_start()
        } else {
            geometry.thumb_length() / 2.
        };
        let drag = DragState {
            axis: geometry.axis,
            pointer_offset,
            track_start: geometry.track_start(),
            track_length: geometry.track_length(),
            thumb_length: geometry.thumb_length(),
            max_offset: geometry.max_offset,
        };
        apply_drag(&handle_for_down, drag, event.position);
        state_for_down.update(cx, |state, cx| state.start_drag(drag, cx));
        window.prevent_default();
        window.refresh();
        cx.stop_propagation();
    });

    let state_for_up = state;
    window.on_mouse_event(move |event: &MouseUpEvent, phase, _window, cx| {
        if phase == DispatchPhase::Bubble && event.button == MouseButton::Left {
            let dragging = state_for_up.read(cx).drag.is_some();
            if dragging {
                state_for_up.update(cx, |state, cx| state.end_drag(cx));
                cx.stop_propagation();
            }
        }
    });
}

fn apply_drag(handle: &ScrollHandle, drag: DragState, position: Point<Pixels>) {
    let pointer = match drag.axis {
        PhysicalAxis::Vertical => position.y,
        PhysicalAxis::Horizontal => position.x,
    };
    let travel = (drag.track_length - drag.thumb_length).max(Pixels::ZERO);
    let thumb_start = (pointer - drag.pointer_offset - drag.track_start)
        .max(Pixels::ZERO)
        .min(travel);
    let progress = if travel > Pixels::ZERO {
        (thumb_start / travel).clamp(0., 1.)
    } else {
        0.
    };
    let mut offset = handle.offset();
    match drag.axis {
        PhysicalAxis::Vertical => offset.y = -(drag.max_offset * progress),
        PhysicalAxis::Horizontal => offset.x = -(drag.max_offset * progress),
    }
    handle.set_offset(offset);
}

fn handle_scroll_key(event: &KeyDownEvent, axis: ScrollAxis, handle: &ScrollHandle) -> bool {
    if event.keystroke.modifiers != Modifiers::none() {
        return false;
    }
    let max = handle.max_offset();
    let bounds = handle.bounds();
    let mut current = point(-handle.offset().x, -handle.offset().y);
    let before = current;
    match event.keystroke.key.as_str() {
        "up" if axis.has_vertical() => current.y -= KEYBOARD_STEP,
        "down" if axis.has_vertical() => current.y += KEYBOARD_STEP,
        "left" if axis.has_horizontal() => current.x -= KEYBOARD_STEP,
        "right" if axis.has_horizontal() => current.x += KEYBOARD_STEP,
        "pageup" if axis.has_vertical() => current.y -= bounds.size.height * 0.9,
        "pagedown" if axis.has_vertical() => current.y += bounds.size.height * 0.9,
        "pageup" if axis.has_horizontal() => current.x -= bounds.size.width * 0.9,
        "pagedown" if axis.has_horizontal() => current.x += bounds.size.width * 0.9,
        "home" if axis.has_vertical() => current.y = Pixels::ZERO,
        "end" if axis.has_vertical() => current.y = max.y,
        "home" if axis.has_horizontal() => current.x = Pixels::ZERO,
        "end" if axis.has_horizontal() => current.x = max.x,
        _ => return false,
    }
    current.x = current.x.max(Pixels::ZERO).min(max.x.max(Pixels::ZERO));
    current.y = current.y.max(Pixels::ZERO).min(max.y.max(Pixels::ZERO));
    if current == before {
        return false;
    }
    handle.set_offset(point(-current.x, -current.y));
    true
}

#[cfg(test)]
#[path = "../tests/unit/scrollbar.rs"]
mod tests;
