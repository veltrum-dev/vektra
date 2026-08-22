//! 固定行高、视窗级物化的公开 VirtualList。

use crate::{
    data_source::LazyDataSource,
    scrollbar::{ScrollAxis, ScrollbarConfig, virtual_scroll_area},
    theme,
};
use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels, Refineable,
    RenderOnce, Role, ScrollHandle, ScrollStrategy, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, UniformListScrollHandle, Window, div, px, uniform_list,
};
use std::{cell::RefCell, ops::Range, rc::Rc};

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
}

/// 固定行高 [`VirtualList`] 的可复用滚动状态。
///
/// 状态只保存 GPUI 的 O(1) uniform-list 滚动句柄与常数级统计，不为每项保存
/// `Element`、高度、metadata 或 Entity。调用方应跨帧复用同一个状态。
#[derive(Clone, Debug)]
pub struct VirtualListState {
    scroll_handle: UniformListScrollHandle,
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
            scroll_handle: UniformListScrollHandle::new(),
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
            self.scroll_handle
                .0
                .borrow()
                .base_handle
                .set_offset(gpui::Point::default());
        }
    }

    /// 将指定索引严格放在视口顶部。
    pub fn scroll_to_index(&self, index: usize) {
        if let Some(index) = self.clamped_index(index) {
            self.scroll_handle
                .scroll_to_item_strict(index, ScrollStrategy::Top);
        }
    }

    /// 仅在需要时将指定索引滚入视口。
    pub fn reveal_index(&self, index: usize) {
        if let Some(index) = self.clamped_index(index) {
            self.scroll_handle
                .scroll_to_item(index, ScrollStrategy::Nearest);
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
            self.scroll_handle
                .scroll_to_item_strict(count - 1, ScrollStrategy::Bottom);
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
        self.base_scroll_handle()
    }

    pub(crate) fn uniform_scroll_handle(&self) -> UniformListScrollHandle {
        self.scroll_handle.clone()
    }

    pub(crate) fn base_scroll_handle(&self) -> ScrollHandle {
        self.scroll_handle.0.borrow().base_handle.clone()
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
}

type KeyRenderer = dyn Fn(usize) -> ElementId + 'static;
type ItemRenderer = dyn Fn(usize, &mut Window, &mut App) -> AnyElement + 'static;
type RangeRequester = dyn Fn(Range<usize>, &mut Window, &mut App) + 'static;

/// 固定行高、只物化可见行的公开虚拟列表。
///
/// 列表基于锁定 GPUI 的 `UniformList`：布局、prepaint、paint 与 AccessKit 子树均只包含
/// 当前可见范围。Vektra 不缓存行，附加状态为 O(1)；总高度由 `item_count × item_height`
/// 推导。renderer 不得阻塞，也不应访问正处于渲染中的外部 Entity。
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
        let state = self.state.clone();
        let key = self.key;
        let renderer = self.renderer;
        let request_range = self.request_range;
        let item_height = self.item_height.max(px(1.));
        let uniform_handle = self.state.uniform_scroll_handle();
        let base_handle = self.state.base_scroll_handle();
        let keyboard_handle = base_handle.clone();
        let scrollbar = self.scrollbar;
        let tokens = theme::current_theme(window, cx).scrollbar;
        let focus_color = tokens.focus_ring;
        let focus_width = tokens.focus_width;
        let gutter_width = match scrollbar.gutter {
            crate::ScrollGutter::Overlay => Pixels::ZERO,
            crate::ScrollGutter::Stable => tokens.hit_thickness,
        };

        let mut list = uniform_list(
            self.id.clone(),
            self.item_count,
            move |range, window, cx| {
                state.record_render(range.clone());
                if let Some(request_range) = request_range.as_ref() {
                    request_range(range.clone(), window, cx);
                }
                range
                    .map(|index| {
                        div()
                            .id(key(index))
                            .w_full()
                            .h(item_height)
                            .min_h(item_height)
                            .max_h(item_height)
                            .child(renderer(index, window, cx))
                    })
                    .collect::<Vec<_>>()
            },
        )
        .track_scroll(&uniform_handle)
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
                if crate::scrollbar::handle_virtual_list_key(event, &keyboard_handle) {
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
        )
    }
}
