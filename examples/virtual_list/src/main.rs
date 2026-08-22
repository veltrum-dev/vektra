#[path = "../../shared.rs"]
mod shared;

use gpui::{
    App, AppContext, Bounds, Context, ElementId, IntoElement, ParentElement, Render, Styled,
    TitlebarOptions, Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui_platform::application;
use vektra::{
    Button, ScrollGutter, ScrollVisibility, ScrollbarConfig, VirtualList, VirtualListState,
    current_theme,
};

const NORMAL_ITEMS: usize = 100;
const LARGE_ITEMS: usize = 10_000_000;

struct VirtualListExample {
    normal: VirtualListState,
    large: VirtualListState,
}

impl VirtualListExample {
    fn new() -> Self {
        Self {
            normal: VirtualListState::new(),
            large: VirtualListState::new(),
        }
    }

    fn metrics(label: &str, state: &VirtualListState) -> impl IntoElement {
        let metrics = state.metrics();
        div().child(format!(
            "{label}: visible {}..{} · 当前物化 {} · 单帧最大 {} · 缓存 {}/{}",
            metrics.visible_range.start,
            metrics.visible_range.end,
            metrics.materialized_rows,
            metrics.max_materialized_rows,
            metrics.cached_rows,
            metrics.max_cached_rows,
        ))
    }
}

impl Render for VirtualListExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);
        let normal = VirtualList::new(
            "normal-virtual-list",
            self.normal.clone(),
            NORMAL_ITEMS,
            px(32.),
            |index| ElementId::named_usize("normal-row", index),
            |index, _, _| {
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .px(px(10.))
                    .child(format!("普通列表行 {index:03}"))
            },
        )
        .aria_label("100 项普通 VirtualList")
        .scrollbar(
            ScrollbarConfig::new()
                .visibility(ScrollVisibility::Always)
                .gutter(ScrollGutter::Stable),
        );

        let large = VirtualList::new(
            "large-virtual-list",
            self.large.clone(),
            LARGE_ITEMS,
            px(32.),
            |index| ElementId::named_usize("large-row", index),
            |index, _, _| {
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .px(px(10.))
                    .child(format!("大数据生成行 {index:08}"))
            },
        )
        .aria_label("一千万项大数据 VirtualList")
        .scrollbar(
            ScrollbarConfig::new()
                .visibility(ScrollVisibility::Always)
                .gutter(ScrollGutter::Stable),
        );

        div()
            .size_full()
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .p(px(24.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .max_w(px(760.))
                    .child(div().text_size(px(24.)).child("Vektra VirtualList"))
                    .child(
                        "同一个组件入口同时展示普通与大数据场景；两者都只渲染可见行。大数据场景直接按索引生成，不预构建 10,000,000 项 Vec。",
                    )
                    .child(shared::theme_selector("virtual-list-example", window, cx))
                    .child(div().text_size(px(18.)).child("普通场景：100 项"))
                    .child(Self::metrics("普通列表", &self.normal))
                    .child(div().w_full().h(px(220.)).child(normal))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                Button::new("normal-start")
                                    .label("开头")
                                    .on_click_in(cx, |this, _, _, cx| {
                                        this.normal.scroll_to_start();
                                        cx.notify();
                                    }),
                            )
                            .child(
                                Button::new("normal-end")
                                    .label("末尾")
                                    .on_click_in(cx, |this, _, _, cx| {
                                        this.normal.scroll_to_end();
                                        cx.notify();
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(18.))
                            .child("大数据场景：10,000,000 项生成式数据"),
                    )
                    .child(Self::metrics("大数据列表", &self.large))
                    .child(div().w_full().h(px(220.)).child(large))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                Button::new("large-start")
                                    .label("返回开头")
                                    .on_click_in(cx, |this, _, _, cx| {
                                        this.large.scroll_to_start();
                                        cx.notify();
                                    }),
                            )
                            .child(
                                Button::new("large-9000000")
                                    .label("跳到第 9,000,000 项")
                                    .on_click_in(cx, |this, _, _, cx| {
                                        this.large.scroll_to_index(9_000_000);
                                        cx.notify();
                                    }),
                            )
                            .child(
                                Button::new("large-end")
                                    .label("末尾")
                                    .on_click_in(cx, |this, _, _, cx| {
                                        this.large.scroll_to_end();
                                        cx.notify();
                                    }),
                            ),
                    ),
            )
    }
}

fn main() {
    application()
        .with_assets(vektra::assets::Assets)
        .run(|cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(840.), px(900.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Vektra VirtualList".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| VirtualListExample::new()),
            )
            .expect("VirtualList 示例窗口应能成功打开");
            cx.activate(true);
        });
}
