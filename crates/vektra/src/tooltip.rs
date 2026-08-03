//! 纯文本、非交互式 Tooltip 组件。

use crate::theme;
use gpui::{
    Animation, AnimationExt, AnyElement, AnyTooltip, AnyView, App, AppContext, AvailableSpace,
    Bounds, BoxShadow, Context, Div, Element, ElementId, Entity, FocusHandle, GlobalElementId,
    Hsla, InspectorElementId, InteractiveElement, IntoElement, KeystrokeEvent, LayoutId, Modifiers,
    ParentElement, Path, PathBuilder, Pixels, Point, Render, RenderOnce, SharedString, Size,
    Stateful, StatefulInteractiveElement, Style, Styled, Subscription, Task, WeakEntity, Window,
    div, ease_out_quint, point, px,
};
use std::{cell::Cell, rc::Rc, time::Duration};

const SHOW_DELAY: Duration = Duration::from_millis(500);
const CLOSE_GRACE_DURATION: Duration = Duration::from_millis(500);
const ENTER_DURATION: Duration = Duration::from_millis(120);
const EXIT_DURATION: Duration = Duration::from_millis(80);
const ENTER_OFFSET: Pixels = px(2.);

/// Tooltip 相对 trigger 的优先位置。
///
/// `TopStart`/`BottomStart` 让气泡左边与 trigger 左边对齐，`TopEnd`/`BottomEnd`
/// 让右边对齐；`LeftStart`/`RightStart` 让顶部对齐，`LeftEnd`/`RightEnd` 让底部
/// 对齐。未带 `Start` 或 `End` 的变体在对应交叉轴上居中。视口空间不足时，
/// Tooltip 会保留对齐方式并自动翻转或平移。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipPlacement {
    /// trigger 上方，左边对齐。
    TopStart,
    /// trigger 上方，水平居中。
    Top,
    /// trigger 上方，右边对齐。
    TopEnd,
    /// trigger 右侧，顶部对齐。
    RightStart,
    /// trigger 右侧，垂直居中。
    Right,
    /// trigger 右侧，底部对齐。
    RightEnd,
    /// trigger 下方，左边对齐。
    BottomStart,
    /// trigger 下方，水平居中；这是默认位置。
    #[default]
    Bottom,
    /// trigger 下方，右边对齐。
    BottomEnd,
    /// trigger 左侧，顶部对齐。
    LeftStart,
    /// trigger 左侧，垂直居中。
    Left,
    /// trigger 左侧，底部对齐。
    LeftEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alignment {
    Start,
    Center,
    End,
}

impl TooltipPlacement {
    fn side(self) -> Side {
        match self {
            Self::TopStart | Self::Top | Self::TopEnd => Side::Top,
            Self::RightStart | Self::Right | Self::RightEnd => Side::Right,
            Self::BottomStart | Self::Bottom | Self::BottomEnd => Side::Bottom,
            Self::LeftStart | Self::Left | Self::LeftEnd => Side::Left,
        }
    }

    fn alignment(self) -> Alignment {
        match self {
            Self::TopStart | Self::RightStart | Self::BottomStart | Self::LeftStart => {
                Alignment::Start
            }
            Self::Top | Self::Right | Self::Bottom | Self::Left => Alignment::Center,
            Self::TopEnd | Self::RightEnd | Self::BottomEnd | Self::LeftEnd => Alignment::End,
        }
    }

    fn with_side(self, side: Side) -> Self {
        match (side, self.alignment()) {
            (Side::Top, Alignment::Start) => Self::TopStart,
            (Side::Top, Alignment::Center) => Self::Top,
            (Side::Top, Alignment::End) => Self::TopEnd,
            (Side::Right, Alignment::Start) => Self::RightStart,
            (Side::Right, Alignment::Center) => Self::Right,
            (Side::Right, Alignment::End) => Self::RightEnd,
            (Side::Bottom, Alignment::Start) => Self::BottomStart,
            (Side::Bottom, Alignment::Center) => Self::Bottom,
            (Side::Bottom, Alignment::End) => Self::BottomEnd,
            (Side::Left, Alignment::Start) => Self::LeftStart,
            (Side::Left, Alignment::Center) => Self::Left,
            (Side::Left, Alignment::End) => Self::LeftEnd,
        }
    }
}

impl Side {
    fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }
}

/// 简短补充说明使用的纯文本 Tooltip。
///
/// Tooltip 不获取焦点、不接受点击，也不提供富文本或交互子项。
/// 通常通过 [`crate::Button::tooltip`] 或 [`crate::IconButton::tooltip`] 配置触发器。
///
/// # Examples
///
/// ```
/// let tooltip = vektra::Tooltip::new("设置");
/// assert_eq!(tooltip.text_value().as_ref(), "设置");
/// ```
#[derive(Clone, PartialEq, IntoElement)]
pub struct Tooltip {
    text: SharedString,
    open: Option<bool>,
    arrow: bool,
    color: Option<Hsla>,
    bg_color: Option<Hsla>,
    animated: bool,
}

impl Tooltip {
    /// 创建一个纯文本 Tooltip。
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            open: None,
            arrow: true,
            color: None,
            bg_color: None,
            animated: true,
        }
    }

    /// 设置 Tooltip 的显式打开状态。
    ///
    /// 未调用时由 hover 或键盘焦点自动触发；`true` 会立即显示，`false` 会强制
    /// 关闭。Escape 可临时关闭 `true` 状态，之后必须让该值经历 `false -> true`
    /// 才会再次打开。
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    /// 设置是否绘制指向 trigger 的箭头。
    ///
    /// 默认绘制。关闭后定位不会预留箭头高度，但仍保留主题中的 anchor gap。
    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }

    /// 覆盖当前 Tooltip 实例的文字颜色。
    ///
    /// 未调用时继续使用当前主题 token。固定颜色不会自动适配主题，对比度由调用方负责。
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// 覆盖当前 Tooltip 实例的气泡和箭头背景色。
    ///
    /// 未调用时继续使用当前主题 token。边框、阴影和其他视觉属性不受影响。
    pub fn bg_color(mut self, color: impl Into<Hsla>) -> Self {
        self.bg_color = Some(color.into());
        self
    }

    /// 设置是否使用默认显隐动画。
    ///
    /// 默认开启；关闭或宿主启用 reduce-motion 时，Tooltip 直接呈现静态终态。
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// 创建 GPUI `AnyView`，供 Tooltip trigger 或自定义宿主使用。
    ///
    /// 该 factory 不注册全局状态；返回的 view 只负责绘制文本内容。
    pub fn text(text: impl Into<SharedString>, cx: &mut App) -> AnyView {
        cx.new(|_| TooltipView(Self::new(text))).into()
    }

    /// 返回 Tooltip 保存的文本。
    pub fn text_value(&self) -> &SharedString {
        &self.text
    }

    #[cfg(test)]
    pub(crate) fn open_value(&self) -> Option<bool> {
        self.open
    }

    #[cfg(test)]
    pub(crate) fn arrow_value(&self) -> bool {
        self.arrow
    }

    #[cfg(test)]
    pub(crate) fn color_value(&self) -> Option<Hsla> {
        self.color
    }

    #[cfg(test)]
    pub(crate) fn bg_color_value(&self) -> Option<Hsla> {
        self.bg_color
    }

    #[cfg(test)]
    pub(crate) fn animated_value(&self) -> bool {
        self.animated
    }
}

impl From<&str> for Tooltip {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl From<String> for Tooltip {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<SharedString> for Tooltip {
    fn from(text: SharedString) -> Self {
        Self::new(text)
    }
}

impl RenderOnce for Tooltip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        render_content(&self, window, cx)
    }
}

struct TooltipView(Tooltip);

impl Render for TooltipView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        render_content(&self.0, window, cx)
    }
}

fn render_content(tooltip: &Tooltip, window: &mut Window, cx: &mut App) -> Div {
    let tokens = theme::current_theme(window, cx).tooltip;
    let max_width = tokens
        .max_width
        .min((window.viewport_size().width - px(8.)).max(Pixels::ZERO));
    div()
        .max_w(max_width)
        .px(tokens.padding_x)
        .py(tokens.padding_y)
        .rounded(tokens.radius)
        .border(tokens.border_width)
        .border_color(tokens.border)
        .bg(tooltip_background(tooltip, tokens.background))
        .text_color(tooltip.color.unwrap_or(tokens.foreground))
        .text_size(tokens.font_size)
        .line_height(tokens.line_height)
        .shadow(vec![
            BoxShadow::new(
                tokens.shadow_offset_x,
                tokens.shadow_offset_y,
                tokens.shadow_color,
            )
            .blur_radius(tokens.shadow_blur)
            .spread_radius(tokens.shadow_spread),
        ])
        .whitespace_normal()
        .child(tooltip.text.clone())
}

fn tooltip_background(tooltip: &Tooltip, theme_background: Hsla) -> Hsla {
    tooltip.bg_color.unwrap_or(theme_background)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionPhase {
    Hidden,
    Entering,
    Visible,
    Exiting,
}

impl TransitionPhase {
    fn is_present(self) -> bool {
        self != Self::Hidden
    }
}

struct TooltipOverlayView {
    tooltip: Tooltip,
    trigger_bounds: Bounds<Pixels>,
    bubble_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    placement: TooltipPlacement,
    phase: TransitionPhase,
    animation_generation: u64,
}

impl TooltipOverlayView {
    fn new(tooltip: Tooltip, bubble_bounds: Rc<Cell<Option<Bounds<Pixels>>>>) -> Self {
        Self {
            tooltip,
            trigger_bounds: Bounds::default(),
            bubble_bounds,
            placement: TooltipPlacement::default(),
            phase: TransitionPhase::Hidden,
            animation_generation: 0,
        }
    }
}

impl Render for TooltipOverlayView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = theme::current_theme(window, cx).tooltip;
        let background = tooltip_background(&self.tooltip, tokens.background);
        let overlay = TooltipOverlay {
            body: Some(
                div()
                    .id("vektra-tooltip-bubble")
                    .debug_selector(|| "vektra-tooltip-bubble".into())
                    .child(render_content(&self.tooltip, window, cx))
                    .into_any_element(),
            ),
            bubble_bounds: self.bubble_bounds.clone(),
            trigger_bounds: self.trigger_bounds,
            preferred: self.placement,
            viewport_bounds: Bounds::new(Point::default(), window.viewport_size()),
            viewport_padding: tokens.viewport_padding,
            trigger_gap: tokens.anchor_gap,
            arrow_size: Size {
                width: tokens.arrow_width,
                height: tokens.arrow_height,
            },
            corner_radius: tokens.radius,
            border_width: tokens.border_width,
            background,
            border: tokens.border,
            shadow_margin: shadow_margin(tokens),
            arrow: self.tooltip.arrow,
            opacity: 1.,
            enter_offset: Pixels::ZERO,
        };

        if !self.tooltip.animated {
            return overlay.into_any_element();
        }

        match self.phase {
            TransitionPhase::Entering => overlay
                .with_animation(
                    format!("vektra-tooltip-enter-{}", self.animation_generation),
                    Animation::new(ENTER_DURATION).with_easing(ease_out_quint()),
                    |mut overlay, delta| {
                        overlay.opacity = delta;
                        overlay.enter_offset = ENTER_OFFSET * (1. - delta);
                        overlay
                    },
                )
                .into_any_element(),
            TransitionPhase::Exiting => overlay
                .with_animation(
                    format!("vektra-tooltip-exit-{}", self.animation_generation),
                    Animation::new(EXIT_DURATION).with_easing(ease_out_quint()),
                    |mut overlay, delta| {
                        overlay.opacity = 1. - delta;
                        overlay
                    },
                )
                .into_any_element(),
            TransitionPhase::Hidden | TransitionPhase::Visible => overlay.into_any_element(),
        }
    }
}

fn shadow_margin(tokens: vektra_theme::TooltipTokens) -> Pixels {
    let horizontal = tokens.shadow_blur + tokens.shadow_spread + tokens.shadow_offset_x.abs();
    let vertical = tokens.shadow_blur + tokens.shadow_spread + tokens.shadow_offset_y.abs();
    horizontal.max(vertical).max(Pixels::ZERO)
}

struct TooltipOverlay {
    body: Option<AnyElement>,
    bubble_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    trigger_bounds: Bounds<Pixels>,
    preferred: TooltipPlacement,
    viewport_bounds: Bounds<Pixels>,
    viewport_padding: Pixels,
    trigger_gap: Pixels,
    arrow_size: Size<Pixels>,
    corner_radius: Pixels,
    border_width: Pixels,
    background: gpui::Hsla,
    border: gpui::Hsla,
    shadow_margin: Pixels,
    arrow: bool,
    opacity: f32,
    enter_offset: Pixels,
}

struct TooltipOverlayLayout {
    body: Option<AnyElement>,
    body_size: Size<Pixels>,
}

struct TooltipOverlayPaint {
    body: AnyElement,
    arrow_fill: Option<Path<Pixels>>,
    arrow_stroke: Option<Path<Pixels>>,
}

impl IntoElement for TooltipOverlay {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TooltipOverlay {
    type RequestLayoutState = TooltipOverlayLayout;
    type PrepaintState = TooltipOverlayPaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let body = self.body.take().expect("TooltipOverlay 每帧只允许布局一次");
        let mut body = div().opacity(self.opacity).child(body).into_any_element();
        let body_size = body.layout_as_root(AvailableSpace::min_size(), window, cx);
        let layout_id = window.request_layout(Style::default(), [], cx);
        (
            layout_id,
            TooltipOverlayLayout {
                body: Some(body),
                body_size,
            },
        )
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let placement = calculate_placement(PlacementInput {
            trigger_bounds: self.trigger_bounds,
            tooltip_size: layout.body_size,
            preferred: self.preferred,
            viewport_bounds: self.viewport_bounds,
            viewport_padding: self.viewport_padding,
            trigger_gap: self.trigger_gap,
            arrow_size: self.arrow_size,
            corner_radius: self.corner_radius,
            border_width: self.border_width,
            shadow_margin: self.shadow_margin,
            arrow: self.arrow,
        });
        let mut body = layout
            .body
            .take()
            .expect("TooltipOverlay 的气泡必须在 prepaint 前完成布局");
        let visual_offset = transition_offset(placement.placement.side(), self.enter_offset);
        let visible_bubble_bounds = Bounds::new(
            point(
                placement.bubble_bounds.origin.x + visual_offset.x,
                placement.bubble_bounds.origin.y + visual_offset.y,
            ),
            placement.bubble_bounds.size,
        );
        self.bubble_bounds.set(Some(visible_bubble_bounds));
        body.prepaint_at(visible_bubble_bounds.origin, window, cx);
        let arrow_points = placement
            .arrow_points
            .map(|point| gpui::point(point.x + visual_offset.x, point.y + visual_offset.y));
        let (arrow_fill, arrow_stroke) = arrow_paths(self.arrow, arrow_points, self.border_width);
        TooltipOverlayPaint {
            body,
            arrow_fill,
            arrow_stroke,
        }
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _layout: &mut Self::RequestLayoutState,
        paint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        paint.body.paint(window, cx);
        if let Some(path) = paint.arrow_fill.take() {
            window.paint_path(path, self.background.opacity(self.opacity));
        }
        if let Some(path) = paint.arrow_stroke.take() {
            window.paint_path(path, self.border.opacity(self.opacity));
        }
    }
}

fn arrow_paths(
    arrow: bool,
    points: [Point<Pixels>; 3],
    border_width: Pixels,
) -> (Option<Path<Pixels>>, Option<Path<Pixels>>) {
    if !arrow {
        return (None, None);
    }

    let mut fill = PathBuilder::fill();
    fill.add_polygon(&points, true);

    let mut stroke = PathBuilder::stroke(border_width.max(Pixels::ZERO));
    stroke.move_to(points[0]);
    stroke.line_to(points[1]);
    stroke.line_to(points[2]);

    (fill.build().ok(), stroke.build().ok())
}

fn transition_offset(side: Side, distance: Pixels) -> Point<Pixels> {
    match side {
        Side::Top => point(Pixels::ZERO, distance),
        Side::Right => point(-distance, Pixels::ZERO),
        Side::Bottom => point(Pixels::ZERO, -distance),
        Side::Left => point(distance, Pixels::ZERO),
    }
}

#[derive(Debug, Clone, Copy)]
struct PlacementInput {
    trigger_bounds: Bounds<Pixels>,
    tooltip_size: Size<Pixels>,
    preferred: TooltipPlacement,
    viewport_bounds: Bounds<Pixels>,
    viewport_padding: Pixels,
    trigger_gap: Pixels,
    arrow_size: Size<Pixels>,
    corner_radius: Pixels,
    border_width: Pixels,
    shadow_margin: Pixels,
    arrow: bool,
}

#[derive(Debug, Clone, Copy)]
struct PlacementResult {
    placement: TooltipPlacement,
    bubble_bounds: Bounds<Pixels>,
    arrow_points: [Point<Pixels>; 3],
}

fn calculate_placement(input: PlacementInput) -> PlacementResult {
    let trigger = sanitize_bounds(input.trigger_bounds);
    let viewport = sanitize_bounds(input.viewport_bounds);
    let tooltip_size = sanitize_size(input.tooltip_size);
    let padding = sanitize_length(input.viewport_padding);
    let gap = sanitize_length(input.trigger_gap);
    let arrow_size = if input.arrow {
        sanitize_size(input.arrow_size)
    } else {
        Size::default()
    };
    let shadow_margin = sanitize_length(input.shadow_margin);
    let corner_radius = sanitize_length(input.corner_radius);
    let border_width = sanitize_length(input.border_width);
    let safe_inset = padding + shadow_margin;
    let safe_left = viewport.left() + safe_inset;
    let safe_top = viewport.top() + safe_inset;
    let safe_right = viewport.right() - safe_inset;
    let safe_bottom = viewport.bottom() - safe_inset;

    let preferred_side = input.preferred.side();
    let opposite = preferred_side.opposite();
    let preferred_space = available_space(
        preferred_side,
        trigger,
        viewport,
        safe_inset,
        gap,
        arrow_size.height,
    );
    let opposite_space = available_space(
        opposite,
        trigger,
        viewport,
        safe_inset,
        gap,
        arrow_size.height,
    );
    let required = match preferred_side {
        Side::Top | Side::Bottom => tooltip_size.height,
        Side::Left | Side::Right => tooltip_size.width,
    };
    let side = if preferred_space >= required {
        preferred_side
    } else if opposite_space >= required || opposite_space > preferred_space {
        opposite
    } else {
        preferred_side
    };
    let placement = input.preferred.with_side(side);

    let cross_origin = match (side, placement.alignment()) {
        (Side::Top | Side::Bottom, Alignment::Start) => trigger.left(),
        (Side::Top | Side::Bottom, Alignment::Center) => {
            trigger.center().x - tooltip_size.width / 2.
        }
        (Side::Top | Side::Bottom, Alignment::End) => trigger.right() - tooltip_size.width,
        (Side::Left | Side::Right, Alignment::Start) => trigger.top(),
        (Side::Left | Side::Right, Alignment::Center) => {
            trigger.center().y - tooltip_size.height / 2.
        }
        (Side::Left | Side::Right, Alignment::End) => trigger.bottom() - tooltip_size.height,
    };
    let main_origin = match side {
        Side::Top => trigger.top() - gap - arrow_size.height - tooltip_size.height,
        Side::Right => trigger.right() + gap + arrow_size.height,
        Side::Bottom => trigger.bottom() + gap + arrow_size.height,
        Side::Left => trigger.left() - gap - arrow_size.height - tooltip_size.width,
    };
    let origin = match side {
        Side::Top | Side::Bottom => point(
            clamp_origin(cross_origin, tooltip_size.width, safe_left, safe_right),
            clamp_origin(main_origin, tooltip_size.height, safe_top, safe_bottom),
        ),
        Side::Left | Side::Right => point(
            clamp_origin(main_origin, tooltip_size.width, safe_left, safe_right),
            clamp_origin(cross_origin, tooltip_size.height, safe_top, safe_bottom),
        ),
    };
    let bubble_bounds = Bounds::new(origin, tooltip_size);
    let arrow_points = arrow_points(
        side,
        bubble_bounds,
        trigger,
        arrow_size,
        corner_radius,
        border_width,
    );

    PlacementResult {
        placement,
        bubble_bounds,
        arrow_points,
    }
}

fn available_space(
    side: Side,
    trigger: Bounds<Pixels>,
    viewport: Bounds<Pixels>,
    safe_inset: Pixels,
    gap: Pixels,
    arrow_height: Pixels,
) -> Pixels {
    let edge = match side {
        Side::Top => trigger.top() - (viewport.top() + safe_inset),
        Side::Right => viewport.right() - safe_inset - trigger.right(),
        Side::Bottom => viewport.bottom() - safe_inset - trigger.bottom(),
        Side::Left => trigger.left() - (viewport.left() + safe_inset),
    };
    (edge - gap - arrow_height).max(Pixels::ZERO)
}

fn clamp_origin(value: Pixels, length: Pixels, start: Pixels, end: Pixels) -> Pixels {
    let max_origin = end - length;
    if max_origin < start {
        start.min(end.max(Pixels::ZERO))
    } else {
        value.max(start).min(max_origin)
    }
}

fn arrow_points(
    side: Side,
    bubble: Bounds<Pixels>,
    trigger: Bounds<Pixels>,
    arrow_size: Size<Pixels>,
    corner_radius: Pixels,
    border_width: Pixels,
) -> [Point<Pixels>; 3] {
    let half_width = arrow_size.width / 2.;
    let safe = (corner_radius + half_width + border_width).max(half_width);
    let overlap = border_width.max(px(1.));
    match side {
        Side::Top => {
            let center = clamp_anchor(
                trigger.center().x,
                bubble.left() + safe,
                bubble.right() - safe,
            );
            [
                point(center - half_width, bubble.bottom() - overlap),
                point(center, bubble.bottom() + arrow_size.height),
                point(center + half_width, bubble.bottom() - overlap),
            ]
        }
        Side::Right => {
            let center = clamp_anchor(
                trigger.center().y,
                bubble.top() + safe,
                bubble.bottom() - safe,
            );
            [
                point(bubble.left() + overlap, center - half_width),
                point(bubble.left() - arrow_size.height, center),
                point(bubble.left() + overlap, center + half_width),
            ]
        }
        Side::Bottom => {
            let center = clamp_anchor(
                trigger.center().x,
                bubble.left() + safe,
                bubble.right() - safe,
            );
            [
                point(center - half_width, bubble.top() + overlap),
                point(center, bubble.top() - arrow_size.height),
                point(center + half_width, bubble.top() + overlap),
            ]
        }
        Side::Left => {
            let center = clamp_anchor(
                trigger.center().y,
                bubble.top() + safe,
                bubble.bottom() - safe,
            );
            [
                point(bubble.right() - overlap, center - half_width),
                point(bubble.right() + arrow_size.height, center),
                point(bubble.right() - overlap, center + half_width),
            ]
        }
    }
}

fn clamp_anchor(value: Pixels, start: Pixels, end: Pixels) -> Pixels {
    if end < start {
        (start + end) / 2.
    } else {
        value.max(start).min(end)
    }
}

fn sanitize_bounds(bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    Bounds::new(
        point(
            sanitize_coordinate(bounds.origin.x),
            sanitize_coordinate(bounds.origin.y),
        ),
        sanitize_size(bounds.size),
    )
}

fn sanitize_size(size: Size<Pixels>) -> Size<Pixels> {
    Size {
        width: sanitize_length(size.width),
        height: sanitize_length(size.height),
    }
}

fn sanitize_coordinate(value: Pixels) -> Pixels {
    if value.as_f32().is_finite() {
        value
    } else {
        Pixels::ZERO
    }
}

fn sanitize_length(value: Pixels) -> Pixels {
    sanitize_coordinate(value).max(Pixels::ZERO)
}

pub(crate) struct TooltipTrigger {
    focus_handle: FocusHandle,
    tooltip: Tooltip,
    hovered: bool,
    bubble_hovered: bool,
    bubble_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    keyboard_focused: bool,
    hover_dismissed: bool,
    focus_dismissed: bool,
    explicit_dismissed: bool,
    phase: TransitionPhase,
    placement: TooltipPlacement,
    view: Option<Entity<TooltipOverlayView>>,
    generation: u64,
    close_generation: u64,
    delay_task: Option<Task<()>>,
    close_task: Option<Task<()>>,
    transition_task: Option<Task<()>>,
    escape_subscription: Option<Subscription>,
    _focus_subscription: Subscription,
    _blur_subscription: Subscription,
}

impl TooltipTrigger {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle().tab_index(0);
        let focus_subscription = cx.on_focus(&focus_handle, window, |this, window, cx| {
            this.keyboard_focused = window.last_input_was_keyboard();
            this.focus_dismissed = false;
            if this.keyboard_focused {
                this.ensure_escape_listener(window, cx);
                this.reconcile(cx);
            }
        });
        let blur_subscription = cx.on_blur(&focus_handle, window, |this, _window, cx| {
            this.keyboard_focused = false;
            this.focus_dismissed = false;
            if this.hover_dismissed {
                this.schedule_close_grace(cx);
            }
            this.reconcile(cx);
        });

        Self {
            focus_handle,
            tooltip: Tooltip::new(SharedString::default()),
            hovered: false,
            bubble_hovered: false,
            bubble_bounds: Rc::new(Cell::new(None)),
            keyboard_focused: false,
            hover_dismissed: false,
            focus_dismissed: false,
            explicit_dismissed: false,
            phase: TransitionPhase::Hidden,
            placement: TooltipPlacement::default(),
            view: None,
            generation: 0,
            close_generation: 0,
            delay_task: None,
            close_task: None,
            transition_task: None,
            escape_subscription: None,
            _focus_subscription: focus_subscription,
            _blur_subscription: blur_subscription,
        }
    }

    fn update_tooltip(&mut self, tooltip: Tooltip, window: &Window, cx: &mut Context<Self>) {
        let previous_open = self.tooltip.open;
        let changed = self.tooltip != tooltip;
        self.tooltip = tooltip;
        if previous_open != self.tooltip.open {
            self.explicit_dismissed = false;
        }

        let previous_phase = self.phase;
        self.settle_static_transition(cx);
        if changed || previous_phase != self.phase {
            self.sync_view(cx);
        }
        if self.tooltip.open == Some(true) {
            self.ensure_escape_listener(window, cx);
        }
        self.reconcile(cx);
    }

    fn update_placement(&mut self, placement: TooltipPlacement, cx: &mut Context<Self>) {
        if self.placement != placement {
            self.placement = placement;
            self.sync_view(cx);
            if self.phase.is_present() {
                cx.notify();
            }
        }
    }

    fn set_hovered(&mut self, hovered: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.hovered == hovered {
            return;
        }

        self.hovered = hovered;
        if hovered {
            self.cancel_close_grace();
            // A pointer interaction ends eligibility for the old keyboard-focus source.
            self.keyboard_focused = false;
            self.ensure_escape_listener(window, cx);
        } else {
            self.schedule_close_grace(cx);
        }
        self.reconcile(cx);
    }

    fn set_bubble_hovered(&mut self, hovered: bool, cx: &mut Context<Self>) {
        if self.bubble_hovered == hovered {
            return;
        }

        self.bubble_hovered = hovered;
        if hovered {
            self.cancel_close_grace();
        } else {
            self.schedule_close_grace(cx);
        }
        self.reconcile(cx);
    }

    fn pointer_hovered(&self) -> bool {
        self.hovered || self.bubble_hovered
    }

    fn eligible(&self) -> bool {
        match self.tooltip.open {
            Some(true) => !self.explicit_dismissed,
            Some(false) => false,
            None => {
                (self.pointer_hovered() && !self.hover_dismissed)
                    || (self.keyboard_focused && !self.focus_dismissed)
            }
        }
    }

    fn cancel_close_grace(&mut self) {
        if self.close_task.take().is_some() {
            self.close_generation = self.close_generation.wrapping_add(1);
        }
    }

    fn schedule_close_grace(&mut self, cx: &mut Context<Self>) {
        if self.tooltip.open.is_some()
            || self.pointer_hovered()
            || self.keyboard_focused
            || self.close_task.is_some()
            || (!self.phase.is_present() && !self.hover_dismissed)
        {
            return;
        }

        self.close_generation = self.close_generation.wrapping_add(1);
        let close_generation = self.close_generation;
        self.close_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(CLOSE_GRACE_DURATION).await;
            let _ = this.update(cx, |this, cx| {
                if this.close_generation != close_generation {
                    return;
                }
                this.close_task = None;
                if this.tooltip.open.is_some() || this.pointer_hovered() || this.keyboard_focused {
                    return;
                }
                this.hover_dismissed = false;
                this.reconcile(cx);
            });
        }));
    }

    fn schedule_show(&mut self, cx: &mut Context<Self>) {
        if matches!(
            self.phase,
            TransitionPhase::Entering | TransitionPhase::Visible
        ) || self.delay_task.is_some()
            || !self.eligible()
            || self.tooltip.open.is_some()
        {
            return;
        }

        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.delay_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SHOW_DELAY).await;
            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.delay_task = None;
                if this.eligible() && this.tooltip.open.is_none() {
                    this.show_now(cx);
                }
            });
        }));
    }

    fn reconcile(&mut self, cx: &mut Context<Self>) {
        if self.eligible() {
            self.cancel_close_grace();
            if self.tooltip.open == Some(true) {
                if !matches!(
                    self.phase,
                    TransitionPhase::Entering | TransitionPhase::Visible
                ) {
                    self.show_now(cx);
                }
            } else {
                match self.phase {
                    TransitionPhase::Exiting => self.restore_visible(cx),
                    TransitionPhase::Hidden => self.schedule_show(cx),
                    TransitionPhase::Entering | TransitionPhase::Visible => {}
                }
            }
            return;
        }

        if self.tooltip.open.is_none() && self.close_task.is_some() {
            return;
        }

        self.hide(cx);
        if self.tooltip.open != Some(true) && !self.pointer_hovered() && !self.keyboard_focused {
            self.escape_subscription = None;
        }
    }

    fn show_now(&mut self, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.delay_task = None;
        self.cancel_close_grace();
        self.transition_task = None;
        self.phase = if self.tooltip.animated && !cx.reduce_motion() {
            TransitionPhase::Entering
        } else {
            TransitionPhase::Visible
        };
        if self.view.is_none() {
            let bubble_bounds = self.bubble_bounds.clone();
            self.view =
                Some(cx.new(|_| TooltipOverlayView::new(self.tooltip.clone(), bubble_bounds)));
        }
        self.sync_view(cx);
        cx.notify();

        if self.phase == TransitionPhase::Entering {
            self.transition_task = Some(cx.spawn(async move |this, cx| {
                cx.background_executor().timer(ENTER_DURATION).await;
                let _ = this.update(cx, |this, cx| {
                    if this.generation == generation && this.phase == TransitionPhase::Entering {
                        this.transition_task = None;
                        this.phase = TransitionPhase::Visible;
                        this.sync_view(cx);
                        cx.notify();
                    }
                });
            }));
        }
    }

    fn restore_visible(&mut self, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        self.delay_task = None;
        self.cancel_close_grace();
        self.transition_task = None;
        self.phase = TransitionPhase::Visible;
        self.sync_view(cx);
        cx.notify();
    }

    fn hide(&mut self, cx: &mut Context<Self>) {
        self.cancel_close_grace();
        if self.phase == TransitionPhase::Exiting {
            return;
        }
        if self.phase == TransitionPhase::Hidden {
            if self.delay_task.is_some() {
                self.generation = self.generation.wrapping_add(1);
                self.delay_task = None;
            }
            return;
        }

        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.delay_task = None;

        self.transition_task = None;
        if !self.tooltip.animated || cx.reduce_motion() {
            self.phase = TransitionPhase::Hidden;
            self.view = None;
            self.clear_bubble_state();
            self.schedule_close_grace(cx);
            cx.notify();
            return;
        }

        self.phase = TransitionPhase::Exiting;
        self.sync_view(cx);
        cx.notify();
        self.transition_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(EXIT_DURATION).await;
            let _ = this.update(cx, |this, cx| {
                if this.generation == generation && this.phase == TransitionPhase::Exiting {
                    this.transition_task = None;
                    this.phase = TransitionPhase::Hidden;
                    this.view = None;
                    this.clear_bubble_state();
                    this.schedule_close_grace(cx);
                    cx.notify();
                }
            });
        }));
    }

    fn settle_static_transition(&mut self, cx: &mut Context<Self>) {
        if self.tooltip.animated && !cx.reduce_motion() {
            return;
        }

        match self.phase {
            TransitionPhase::Entering => {
                self.generation = self.generation.wrapping_add(1);
                self.transition_task = None;
                self.phase = TransitionPhase::Visible;
            }
            TransitionPhase::Exiting => {
                self.generation = self.generation.wrapping_add(1);
                self.transition_task = None;
                self.phase = TransitionPhase::Hidden;
                self.view = None;
                self.clear_bubble_state();
                self.schedule_close_grace(cx);
            }
            TransitionPhase::Hidden | TransitionPhase::Visible => {}
        }
    }

    fn sync_view(&self, cx: &mut Context<Self>) {
        let Some(view) = &self.view else {
            return;
        };
        let tooltip = self.tooltip.clone();
        let placement = self.placement;
        let phase = self.phase;
        let animation_generation = self.generation;
        view.update(cx, |view, cx| {
            if view.tooltip != tooltip
                || view.placement != placement
                || view.phase != phase
                || view.animation_generation != animation_generation
            {
                view.tooltip = tooltip;
                view.placement = placement;
                view.phase = phase;
                view.animation_generation = animation_generation;
                cx.notify();
            }
        });
    }

    fn clear_bubble_state(&mut self) {
        self.bubble_bounds.set(None);
        self.bubble_hovered = false;
    }

    fn sync_input_modality(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.keyboard_focused && !self.hovered && !window.last_input_was_keyboard() {
            self.keyboard_focused = false;
            self.reconcile(cx);
        }
    }

    fn dismiss(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.phase.is_present() && self.delay_task.is_none() {
            return false;
        }

        if self.tooltip.open == Some(true) {
            self.explicit_dismissed = true;
        } else {
            if self.pointer_hovered() || self.close_task.is_some() {
                self.hover_dismissed = true;
            }
            if self.keyboard_focused {
                self.focus_dismissed = true;
            }
        }
        self.hide(cx);
        true
    }

    fn ensure_escape_listener(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.escape_subscription.is_some() {
            return;
        }

        let owner = cx.weak_entity();
        let window_handle = window.window_handle();
        self.escape_subscription = Some(cx.intercept_keystrokes(move |event, window, cx| {
            if window.window_handle() != window_handle || !is_escape(event) {
                return;
            }
            let dismissed = owner
                .update(cx, |this, cx| this.dismiss(cx))
                .unwrap_or(false);
            if dismissed {
                cx.stop_propagation();
            }
        }));
    }
}

fn is_escape(event: &KeystrokeEvent) -> bool {
    event.keystroke.key == "escape" && event.keystroke.modifiers == Modifiers::none()
}

pub(crate) fn state_for(
    id: &ElementId,
    tooltip: Tooltip,
    placement: TooltipPlacement,
    tab_stop: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TooltipTrigger> {
    let state = window.use_keyed_state((id.clone(), "tooltip-trigger"), cx, TooltipTrigger::new);
    state.update(cx, |state, cx| {
        state.focus_handle = state.focus_handle.clone().tab_stop(tab_stop).tab_index(0);
        state.update_tooltip(tooltip, window, cx);
        state.update_placement(placement, cx);
    });
    state
}

pub(crate) fn attach_interaction(
    element: Stateful<Div>,
    state: &Entity<TooltipTrigger>,
    focusable: bool,
    cx: &App,
) -> Stateful<Div> {
    let focus_handle = state.read(cx).focus_handle.clone();
    let state = state.downgrade();
    let element = if focusable {
        element.track_focus(&focus_handle)
    } else {
        element
    };
    element.on_hover(move |hovered, window, cx| {
        let _ = state.update(cx, |state, cx| state.set_hovered(*hovered, window, cx));
    })
}

pub(crate) fn attach_prepaint_listener(
    element: Stateful<Div>,
    state: Entity<TooltipTrigger>,
) -> AnyElement {
    div()
        .on_children_prepainted(prepaint_listener(state))
        .child(element)
        .into_any_element()
}

pub(crate) fn prepaint_listener(
    state: Entity<TooltipTrigger>,
) -> impl Fn(Vec<Bounds<Pixels>>, &mut Window, &mut App) + 'static {
    move |children, window, cx| {
        let Some(bounds) = union_bounds(&children) else {
            return;
        };
        state.update(cx, |state, cx| state.sync_input_modality(window, cx));
        let view_and_placement = state.read_with(cx, |state, _| {
            state
                .phase
                .is_present()
                .then(|| state.view.clone().map(|view| (view, state.placement)))
                .flatten()
        });
        let Some((view, placement)) = view_and_placement else {
            return;
        };
        view.update(cx, |view, cx| {
            if view.trigger_bounds != bounds || view.placement != placement {
                view.trigger_bounds = bounds;
                view.placement = placement;
                cx.notify();
            }
        });

        let owner: WeakEntity<TooltipTrigger> = state.downgrade();
        window.set_tooltip(AnyTooltip {
            view: view.into(),
            // GPUI 当前公开 Tooltip 请求仍要求一个鼠标点；Vektra 使用固定占位值，
            // 实际 side/alignment/flip/shift 全部由 TooltipOverlay 基于 trigger bounds 决定。
            mouse_position: Point::default(),
            check_visible_and_update: Rc::new(move |_, window, cx| {
                let mouse_position = window.mouse_position();
                owner
                    .update(cx, |owner, cx| {
                        let bubble_hovered = owner
                            .bubble_bounds
                            .get()
                            .is_some_and(|bounds| bounds.contains(&mouse_position));
                        owner.set_bubble_hovered(bubble_hovered, cx);
                        owner.phase.is_present()
                    })
                    .unwrap_or(false)
            }),
        });
    }
}

fn union_bounds(bounds: &[Bounds<Pixels>]) -> Option<Bounds<Pixels>> {
    let first = *bounds.first()?;
    Some(bounds.iter().skip(1).fold(first, |result, bounds| {
        let left = result.left().min(bounds.left());
        let top = result.top().min(bounds.top());
        let right = result.right().max(bounds.right());
        let bottom = result.bottom().max(bounds.bottom());
        Bounds::from_corners(point(left, top), point(right, bottom))
    }))
}

pub(crate) fn into_any(element: Stateful<Div>) -> AnyElement {
    element.into_any_element()
}

#[cfg(test)]
#[path = "../tests/unit/tooltip.rs"]
mod tests;
