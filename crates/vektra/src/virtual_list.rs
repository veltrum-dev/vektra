//! 固定行高、视窗级物化的公开 VirtualList。

use crate::{
    data_source::LazyDataSource,
    scrollbar::{ScrollAxis, ScrollbarConfig, virtual_scroll_area},
    theme,
};
use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, Element, ElementId, GlobalElementId,
    Hitbox, InspectorElementId, InteractiveElement, Interactivity, IntoElement, LayoutId, Overflow,
    ParentElement, Pixels, Refineable, RenderOnce, Role, ScrollHandle, ScrollStrategy,
    SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window, div, point, px,
    size,
};
use std::{cell::RefCell, ops::Range, rc::Rc};

const KEYBOARD_SCROLL_STEP: f64 = 40.;

#[derive(Debug, Clone, Copy)]
struct DeferredVirtualScroll {
    item_index: usize,
    strategy: ScrollStrategy,
    strict: bool,
}

/// VirtualList 当前帧和生命周期内的有界物化统计。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualListMetrics {
    /// 最近一次绘制请求的可见索引范围。
    pub visible_range: Range<usize>,
    /// 最近一次可见范围实际物化的行数。
    pub materialized_rows: usize,
    /// 自状态创建或重置统计以来的 renderer 调用次数。
    pub renderer_calls: u64,
    /// 观察到的单帧最大物化行数。
    pub max_materialized_rows: usize,
    /// Vektra 持有的缓存行数；固定内存模式当前不缓存行。
    pub cached_rows: usize,
    /// 固定内存模式的缓存硬上限；当前为零。
    pub max_cached_rows: usize,
}

impl Default for VirtualListMetrics {
    fn default() -> Self {
        Self {
            visible_range: 0..0,
            materialized_rows: 0,
            renderer_calls: 0,
            max_materialized_rows: 0,
            cached_rows: 0,
            max_cached_rows: 0,
        }
    }
}

#[derive(Debug, Default)]
struct VirtualListStateInner {
    item_count: usize,
    revision: u64,
    metrics: VirtualListMetrics,
    logical_scroll_top: f64,
    synced_handle_top: f32,
    item_height: f64,
    viewport_height: f64,
    deferred_scroll: Option<DeferredVirtualScroll>,
}

/// 固定行高 [`VirtualList`] 的可复用滚动状态。
///
/// 状态只保存 GPUI 的 O(1) uniform-list 滚动句柄与常数级统计，不为每项保存
/// `Element`、高度、metadata 或 Entity。调用方应跨帧复用同一个状态。
#[derive(Clone, Debug)]
pub struct VirtualListState {
    scroll_handle: ScrollHandle,
    inner: Rc<RefCell<VirtualListStateInner>>,
}

impl Default for VirtualListState {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualListState {
    /// 创建位于列表开头的空状态。
    pub fn new() -> Self {
        Self {
            scroll_handle: ScrollHandle::new(),
            inner: Rc::new(RefCell::new(VirtualListStateInner::default())),
        }
    }

    /// 重置逻辑项目数和数据 revision，并清空统计。
    pub fn reset(&self, item_count: usize, revision: u64) {
        let mut inner = self.inner.borrow_mut();
        inner.item_count = item_count;
        inner.revision = revision;
        inner.metrics = VirtualListMetrics::default();
        if item_count == 0 {
            inner.logical_scroll_top = 0.;
            inner.synced_handle_top = 0.;
            inner.deferred_scroll = None;
            self.scroll_handle.set_offset(gpui::Point::default());
        }
    }

    /// 将指定索引严格放在视口顶部。
    pub fn scroll_to_index(&self, index: usize) {
        if let Some(index) = self.clamped_index(index) {
            self.defer_scroll(index, ScrollStrategy::Top, true);
        }
    }

    /// 仅在需要时将指定索引滚入视口。
    pub fn reveal_index(&self, index: usize) {
        if let Some(index) = self.clamped_index(index) {
            self.defer_scroll(index, ScrollStrategy::Nearest, false);
        }
    }

    /// 滚动到第一项。
    pub fn scroll_to_start(&self) {
        self.scroll_to_index(0);
    }

    /// 滚动到当前数据中间项。
    pub fn scroll_to_middle(&self) {
        let count = self.inner.borrow().item_count;
        if count > 0 {
            self.scroll_to_index(count / 2);
        }
    }

    /// 滚动到最后一项。
    pub fn scroll_to_end(&self) {
        let count = self.inner.borrow().item_count;
        if count > 0 {
            self.defer_scroll(count - 1, ScrollStrategy::Bottom, true);
        }
    }

    /// 返回最近一次绘制的统计快照。
    pub fn metrics(&self) -> VirtualListMetrics {
        self.inner.borrow().metrics.clone()
    }

    /// 返回最近一次绘制请求的可见范围。
    pub fn visible_range(&self) -> Range<usize> {
        self.inner.borrow().metrics.visible_range.clone()
    }

    /// 返回底层滚动句柄，供自定义滚动观测使用。
    pub fn scroll_handle(&self) -> ScrollHandle {
        self.scroll_handle.clone()
    }

    pub(crate) fn base_scroll_handle(&self) -> ScrollHandle {
        self.scroll_handle.clone()
    }

    pub(crate) fn reconcile(&self, item_count: usize, revision: u64) {
        let mut inner = self.inner.borrow_mut();
        if inner.item_count != item_count || inner.revision != revision {
            inner.item_count = item_count;
            inner.revision = revision;
            inner.metrics.visible_range = 0..0;
            inner.metrics.materialized_rows = 0;
        }
    }

    pub(crate) fn record_render(&self, range: Range<usize>) {
        let mut inner = self.inner.borrow_mut();
        let materialized = range.len();
        inner.metrics.visible_range = range;
        inner.metrics.materialized_rows = materialized;
        inner.metrics.renderer_calls = inner
            .metrics
            .renderer_calls
            .saturating_add(materialized as u64);
        inner.metrics.max_materialized_rows = inner.metrics.max_materialized_rows.max(materialized);
    }

    fn clamped_index(&self, index: usize) -> Option<usize> {
        let count = self.inner.borrow().item_count;
        (count > 0).then(|| index.min(count - 1))
    }

    fn defer_scroll(&self, item_index: usize, strategy: ScrollStrategy, strict: bool) {
        self.inner.borrow_mut().deferred_scroll = Some(DeferredVirtualScroll {
            item_index,
            strategy,
            strict,
        });
    }

    fn scroll_by(&self, delta: f64) -> bool {
        let mut inner = self.inner.borrow_mut();
        let max_scroll_top = inner.max_scroll_top();
        let next = (inner.logical_scroll_top + delta).clamp(0., max_scroll_top);
        if next == inner.logical_scroll_top {
            return false;
        }
        inner.logical_scroll_top = next;
        let handle_top = pixels_from_f64(next);
        inner.synced_handle_top = handle_top.as_f32();
        drop(inner);
        self.scroll_handle
            .set_offset(point(Pixels::ZERO, -handle_top));
        true
    }

    fn scroll_to_progress(&self, progress: f64) {
        let mut inner = self.inner.borrow_mut();
        let next = inner.max_scroll_top() * progress.clamp(0., 1.);
        inner.logical_scroll_top = next;
        let handle_top = pixels_from_f64(next);
        inner.synced_handle_top = handle_top.as_f32();
        drop(inner);
        self.scroll_handle
            .set_offset(point(Pixels::ZERO, -handle_top));
    }

    fn handle_key(&self, event: &gpui::KeyDownEvent) -> bool {
        if event.keystroke.modifiers != gpui::Modifiers::none() {
            return false;
        }
        let viewport_height = self.inner.borrow().viewport_height;
        match event.keystroke.key.as_str() {
            "up" => self.scroll_by(-KEYBOARD_SCROLL_STEP),
            "down" => self.scroll_by(KEYBOARD_SCROLL_STEP),
            "pageup" => self.scroll_by(-viewport_height * 0.9),
            "pagedown" => self.scroll_by(viewport_height * 0.9),
            "home" => self.set_scroll_top(0.),
            "end" => {
                let max_scroll_top = self.inner.borrow().max_scroll_top();
                self.set_scroll_top(max_scroll_top)
            }
            _ => false,
        }
    }

    fn set_scroll_top(&self, scroll_top: f64) -> bool {
        let mut inner = self.inner.borrow_mut();
        let next = scroll_top.clamp(0., inner.max_scroll_top());
        if next == inner.logical_scroll_top {
            return false;
        }
        inner.logical_scroll_top = next;
        let handle_top = pixels_from_f64(next);
        inner.synced_handle_top = handle_top.as_f32();
        drop(inner);
        self.scroll_handle
            .set_offset(point(Pixels::ZERO, -handle_top));
        true
    }

    fn prepare_frame(
        &self,
        item_height: Pixels,
        viewport_height: Pixels,
        observed_handle_top: Pixels,
    ) -> VisibleRows {
        let mut inner = self.inner.borrow_mut();
        inner.item_height = f64::from(item_height);
        inner.viewport_height = f64::from(viewport_height.max(Pixels::ZERO));

        let observed_handle_top = observed_handle_top.max(Pixels::ZERO).as_f32();
        let external_delta = observed_handle_top - inner.synced_handle_top;
        if external_delta != 0. {
            inner.logical_scroll_top += f64::from(external_delta);
        }

        let max_scroll_top = inner.max_scroll_top();
        inner.logical_scroll_top = inner.logical_scroll_top.clamp(0., max_scroll_top);
        if let Some(deferred) = inner.deferred_scroll.take() {
            inner.apply_deferred_scroll(deferred, max_scroll_top);
        }

        let scroll_top = inner.logical_scroll_top;
        let handle_top = pixels_from_f64(scroll_top);
        inner.synced_handle_top = handle_top.as_f32();
        let visible_rows = inner.visible_rows();
        drop(inner);
        self.scroll_handle
            .set_offset(point(Pixels::ZERO, -handle_top));
        visible_rows
    }
}

impl VirtualListStateInner {
    fn total_height(&self) -> f64 {
        self.item_height * self.item_count as f64
    }

    fn max_scroll_top(&self) -> f64 {
        (self.total_height() - self.viewport_height).max(0.)
    }

    fn apply_deferred_scroll(&mut self, mut deferred: DeferredVirtualScroll, max_scroll_top: f64) {
        if self.item_count == 0 || self.item_height <= 0. {
            return;
        }
        deferred.item_index = deferred.item_index.min(self.item_count - 1);
        let item_top = self.item_height * deferred.item_index as f64;
        let item_bottom = item_top + self.item_height;
        let is_above = item_top < self.logical_scroll_top;
        let is_below = item_bottom > self.logical_scroll_top + self.viewport_height;
        if !deferred.strict && !is_above && !is_below {
            return;
        }

        if deferred.strategy == ScrollStrategy::Nearest {
            deferred.strategy = if is_above {
                ScrollStrategy::Top
            } else if is_below {
                ScrollStrategy::Bottom
            } else {
                return;
            };
        }
        let next = match deferred.strategy {
            ScrollStrategy::Top => item_top,
            ScrollStrategy::Center => item_top + self.item_height / 2. - self.viewport_height / 2.,
            ScrollStrategy::Bottom => item_bottom - self.viewport_height,
            ScrollStrategy::Nearest => self.logical_scroll_top,
        };
        self.logical_scroll_top = next.clamp(0., max_scroll_top);
    }

    fn visible_rows(&self) -> VisibleRows {
        if self.item_count == 0 || self.item_height <= 0. || self.viewport_height <= 0. {
            return VisibleRows {
                range: 0..0,
                first_row_offset: 0.,
            };
        }
        let first = (self.logical_scroll_top / self.item_height).floor() as usize;
        let first = first.min(self.item_count - 1);
        let first_row_offset = self.logical_scroll_top - first as f64 * self.item_height;
        let visible_count = ((first_row_offset + self.viewport_height) / self.item_height)
            .ceil()
            .max(1.) as usize;
        VisibleRows {
            range: first..first.saturating_add(visible_count).min(self.item_count),
            first_row_offset,
        }
    }
}

struct VisibleRows {
    range: Range<usize>,
    first_row_offset: f64,
}

fn pixels_from_f64(value: f64) -> Pixels {
    px(value.clamp(0., f64::from(f32::MAX)) as f32)
}

type KeyRenderer = dyn Fn(usize) -> ElementId + 'static;
type ItemRenderer = dyn Fn(usize, &mut Window, &mut App) -> AnyElement + 'static;
type RangeRequester = dyn Fn(Range<usize>, &mut Window, &mut App) + 'static;

struct PrecisionVirtualListFrameState {
    items: Vec<AnyElement>,
}

struct PrecisionVirtualListElement {
    state: VirtualListState,
    item_count: usize,
    item_height: Pixels,
    key: Rc<KeyRenderer>,
    renderer: Rc<ItemRenderer>,
    request_range: Option<Rc<RangeRequester>>,
    interactivity: Interactivity,
}

impl PrecisionVirtualListElement {
    fn new(
        id: ElementId,
        state: VirtualListState,
        item_count: usize,
        item_height: Pixels,
        key: Rc<KeyRenderer>,
        renderer: Rc<ItemRenderer>,
        request_range: Option<Rc<RangeRequester>>,
    ) -> Self {
        let mut interactivity = Interactivity::new();
        interactivity.element_id = Some(id);
        interactivity.base_style.overflow.y = Some(Overflow::Hidden);
        let wheel_state = state.clone();
        interactivity.on_scroll_wheel(move |event, window, cx| {
            let delta = event.delta.pixel_delta(window.line_height());
            let vertical_delta = if delta.y != Pixels::ZERO {
                delta.y
            } else {
                delta.x
            };
            if vertical_delta != Pixels::ZERO && wheel_state.scroll_by(-f64::from(vertical_delta)) {
                window.prevent_default();
                window.refresh();
                cx.stop_propagation();
            }
        });
        Self {
            state,
            item_count,
            item_height,
            key,
            renderer,
            request_range,
            interactivity,
        }
    }
}

impl Styled for PrecisionVirtualListElement {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.interactivity.base_style
    }
}

impl InteractiveElement for PrecisionVirtualListElement {
    fn interactivity(&mut self) -> &mut Interactivity {
        &mut self.interactivity
    }
}

impl StatefulInteractiveElement for PrecisionVirtualListElement {}

impl IntoElement for PrecisionVirtualListElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for PrecisionVirtualListElement {
    type RequestLayoutState = PrecisionVirtualListFrameState;
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<ElementId> {
        self.interactivity.element_id.clone()
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        self.interactivity.source_location()
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let layout_id = self.interactivity.request_layout(
            global_id,
            inspector_id,
            window,
            cx,
            |style, window, cx| window.request_layout(style, [], cx),
        );
        (
            layout_id,
            PrecisionVirtualListFrameState { items: Vec::new() },
        )
    }

    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        frame_state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let style = self
            .interactivity
            .compute_style(global_id, None, window, cx);
        let border = style.border_widths.to_pixels(window.rem_size());
        let padding = style
            .padding
            .to_pixels(bounds.size.into(), window.rem_size());
        let padded_bounds = Bounds::from_corners(
            bounds.origin + point(border.left + padding.left, border.top + padding.top),
            bounds.bottom_right()
                - point(border.right + padding.right, border.bottom + padding.bottom),
        );
        let content_size = size(
            padded_bounds.size.width,
            pixels_from_f64(f64::from(self.item_height) * self.item_count as f64),
        );
        let state = self.state.clone();
        let key = self.key.clone();
        let renderer = self.renderer.clone();
        let request_range = self.request_range.clone();
        let item_height = self.item_height;

        self.interactivity.prepaint(
            global_id,
            inspector_id,
            bounds,
            content_size,
            window,
            cx,
            |_, scroll_offset, hitbox, window, cx| {
                let visible =
                    state.prepare_frame(item_height, padded_bounds.size.height, -scroll_offset.y);
                state.record_render(visible.range.clone());
                if let Some(request_range) = request_range.as_ref() {
                    request_range(visible.range.clone(), window, cx);
                }
                let first_origin_y =
                    padded_bounds.top() - pixels_from_f64(visible.first_row_offset);
                let available_space = size(
                    AvailableSpace::Definite(padded_bounds.size.width),
                    AvailableSpace::Definite(item_height),
                );
                let content_mask = ContentMask { bounds };
                window.with_content_mask(Some(content_mask), |window| {
                    for (row_offset, index) in visible.range.enumerate() {
                        let mut item = div()
                            .id(key(index))
                            .w_full()
                            .h(item_height)
                            .min_h(item_height)
                            .max_h(item_height)
                            .child(renderer(index, window, cx))
                            .into_any_element();
                        item.layout_as_root(available_space, window, cx);
                        item.prepaint_at(
                            point(
                                padded_bounds.left(),
                                first_origin_y + item_height * row_offset,
                            ),
                            window,
                            cx,
                        );
                        frame_state.items.push(item);
                    }
                });
                hitbox
            },
        )
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        frame_state: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.interactivity.paint(
            global_id,
            inspector_id,
            bounds,
            hitbox.as_ref(),
            window,
            cx,
            |_, window, cx| {
                for item in &mut frame_state.items {
                    item.paint(window, cx);
                }
            },
        );
    }
}

/// 固定行高、只物化可见行的公开虚拟列表。
///
/// 列表以整数索引和视口局部偏移定位可见行，避免百万项高位滚动时巨大 `f32` 像素坐标
/// 抵消造成的精度损失。布局、prepaint、paint 与 AccessKit 子树均只包含当前可见范围。
/// Vektra 不缓存行，附加状态为 O(1)；总高度由 `item_count × item_height` 推导。
/// renderer 不得阻塞，也不应访问正处于渲染中的外部 Entity。
#[derive(IntoElement)]
pub struct VirtualList {
    id: ElementId,
    state: VirtualListState,
    item_count: usize,
    revision: u64,
    item_height: Pixels,
    key: Rc<KeyRenderer>,
    renderer: Rc<ItemRenderer>,
    request_range: Option<Rc<RangeRequester>>,
    scrollbar: ScrollbarConfig,
    aria_label: SharedString,
    style: StyleRefinement,
}

impl VirtualList {
    /// 创建生成式固定行高列表。
    ///
    /// `key` 必须为每个逻辑索引返回稳定 `ElementId`；`renderer` 只会收到 GPUI 请求的
    /// 可见索引。调用方应跨帧复用 `state`；非正行高会安全钳制为 1px。
    pub fn new<R>(
        id: impl Into<ElementId>,
        state: VirtualListState,
        item_count: usize,
        item_height: Pixels,
        key: impl Fn(usize) -> ElementId + 'static,
        renderer: impl Fn(usize, &mut Window, &mut App) -> R + 'static,
    ) -> Self
    where
        R: IntoElement,
    {
        Self {
            id: id.into(),
            state,
            item_count,
            revision: 0,
            item_height,
            key: Rc::new(key),
            renderer: Rc::new(move |index, window, cx| {
                renderer(index, window, cx).into_any_element()
            }),
            request_range: None,
            scrollbar: ScrollbarConfig::default().axis(ScrollAxis::Vertical),
            aria_label: "虚拟列表".into(),
            style: StyleRefinement::default(),
        }
    }

    /// 从统一惰性数据源创建列表；未加载项目以 `None` 交给 renderer。
    pub fn from_data_source<S, R>(
        id: impl Into<ElementId>,
        state: VirtualListState,
        source: Rc<S>,
        item_height: Pixels,
        renderer: impl Fn(usize, Option<S::Item>, &mut Window, &mut App) -> R + 'static,
    ) -> Self
    where
        S: LazyDataSource + ?Sized,
        S::Key: Into<ElementId>,
        R: IntoElement,
    {
        let item_count = source.item_count();
        let revision = source.revision();
        let key_source = source.clone();
        let render_source = source.clone();
        let request_source = source;
        Self::new(
            id,
            state,
            item_count,
            item_height,
            move |index| key_source.key(index).into(),
            move |index, window, cx| renderer(index, render_source.item(index), window, cx),
        )
        .revision(revision)
        .request_visible_range(move |range, window, cx| {
            request_source.request_range(range, window, cx);
        })
    }

    /// 设置数据 revision；顺序、内容或加载状态变化后应更新。
    pub fn revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }

    /// 设置可见范围请求回调；回调不得阻塞 UI 线程。
    pub fn request_visible_range(
        mut self,
        callback: impl Fn(Range<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.request_range = Some(Rc::new(callback));
        self
    }

    /// 设置 Vektra Scrollbar 的显隐和 gutter 配置。
    ///
    /// VirtualList 固定为垂直滚动，传入配置的 `axis` 会被规范为 `Vertical`。
    pub fn scrollbar(mut self, mut config: ScrollbarConfig) -> Self {
        config.axis = ScrollAxis::Vertical;
        self.scrollbar = config;
        self
    }

    /// 设置 ScrollView 的可访问名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = label.into();
        self
    }

    /// 返回列表状态。
    pub fn state(&self) -> &VirtualListState {
        &self.state
    }
}

impl Styled for VirtualList {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for VirtualList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.state.reconcile(self.item_count, self.revision);
        let key = self.key;
        let renderer = self.renderer;
        let request_range = self.request_range;
        let item_height = self.item_height.max(px(1.));
        let base_handle = self.state.base_scroll_handle();
        let keyboard_state = self.state.clone();
        let scrollbar_state = self.state.clone();
        let scrollbar = self.scrollbar;
        let tokens = theme::current_theme(window, cx).scrollbar;
        let focus_color = tokens.focus_ring;
        let focus_width = tokens.focus_width;
        let gutter_width = match scrollbar.gutter {
            crate::ScrollGutter::Overlay => Pixels::ZERO,
            crate::ScrollGutter::Stable => tokens.hit_thickness,
        };

        let mut list = PrecisionVirtualListElement::new(
            self.id.clone(),
            self.state.clone(),
            self.item_count,
            item_height,
            key,
            renderer,
            request_range,
        )
        .track_scroll(&base_handle)
        .w_full()
        .h_full()
        .min_h_0()
        .scrollbar_width(gutter_width);
        list.style().refine(&self.style);

        let viewport = div()
            .id((self.id.clone(), "viewport"))
            .w_full()
            .h_full()
            .min_h_0()
            .role(Role::ScrollView)
            .aria_label(self.aria_label)
            .tab_index(0)
            .focus_visible(move |style| {
                style.shadow(vec![
                    gpui::BoxShadow::new(Pixels::ZERO, Pixels::ZERO, focus_color)
                        .spread_radius(focus_width),
                ])
            })
            .on_key_down(move |event, window, cx| {
                if keyboard_state.handle_key(event) {
                    window.prevent_default();
                    window.refresh();
                    cx.stop_propagation();
                }
            })
            .child(list);

        virtual_scroll_area(
            (self.id, "scrollbar"),
            viewport.into_any_element(),
            base_handle,
            scrollbar,
            Some(Rc::new(move |progress| {
                scrollbar_state.scroll_to_progress(progress);
            })),
        )
    }
}
