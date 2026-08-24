#[path = "../../shared.rs"]
mod shared;

use gpui::{
    App, AppContext, Bounds, Context, ElementId, FocusHandle, InteractiveElement, IntoElement,
    KeyBinding, ParentElement, Render, Styled, TitlebarOptions, Window, WindowBounds,
    WindowOptions, actions, div, px, size,
};
use gpui_platform::application;
use std::{cell::Cell, ops::Range, rc::Rc};
use vektra::{
    Button, LazyDataSource, Select, SelectDataSource, SelectEntry, SelectGroup, SelectOption,
    current_theme,
};

actions!(vektra_select_example, [Tab, TabPrev]);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Plan {
    Free,
    Pro,
    Enterprise,
}

const LARGE_SELECT_ITEMS: usize = 1_000_000;

struct MillionSelectSource {
    visible_start: Cell<usize>,
    visible_end: Cell<usize>,
    item_reads: Cell<u64>,
}

impl MillionSelectSource {
    fn new() -> Self {
        Self {
            visible_start: Cell::new(0),
            visible_end: Cell::new(0),
            item_reads: Cell::new(0),
        }
    }

    fn enabled(index: usize) -> bool {
        !index.is_multiple_of(10)
    }
}

impl LazyDataSource for MillionSelectSource {
    type Item = SelectEntry<usize>;
    type Key = ElementId;

    fn item_count(&self) -> usize {
        LARGE_SELECT_ITEMS
    }

    fn revision(&self) -> u64 {
        1
    }

    fn key(&self, index: usize) -> Self::Key {
        ElementId::named_usize("large-select-option", index)
    }

    fn item(&self, index: usize) -> Option<Self::Item> {
        if index >= LARGE_SELECT_ITEMS {
            return None;
        }
        self.item_reads.set(self.item_reads.get() + 1);
        Some(SelectEntry::Option(
            SelectOption::new(self.key(index), index, format!("大数据选项 {index:07}"))
                .disabled(!Self::enabled(index)),
        ))
    }

    fn request_range(&self, range: Range<usize>, _: &mut Window, _: &mut App) {
        self.visible_start.set(range.start);
        self.visible_end.set(range.end);
    }
}

impl SelectDataSource<usize> for MillionSelectSource {
    fn index_of_key(&self, key: &ElementId) -> Option<usize> {
        match key {
            ElementId::NamedInteger(name, index) if name.as_ref() == "large-select-option" => {
                usize::try_from(*index)
                    .ok()
                    .filter(|index| *index < LARGE_SELECT_ITEMS)
            }
            _ => None,
        }
    }

    fn index_of_value(&self, value: &usize) -> Option<usize> {
        (*value < LARGE_SELECT_ITEMS).then_some(*value)
    }

    fn first_enabled(&self) -> Option<usize> {
        Some(1)
    }

    fn is_enabled(&self, index: usize) -> bool {
        index < LARGE_SELECT_ITEMS && Self::enabled(index)
    }

    fn last_enabled(&self) -> Option<usize> {
        Some(LARGE_SELECT_ITEMS - 1)
    }

    fn next_enabled(&self, index: usize, forward: bool, wrap: bool) -> Option<usize> {
        let mut candidate = index;
        for _ in 0..=10 {
            candidate = if forward {
                match candidate
                    .checked_add(1)
                    .filter(|candidate| *candidate < LARGE_SELECT_ITEMS)
                {
                    Some(candidate) => candidate,
                    None if wrap => 0,
                    None => return None,
                }
            } else {
                match candidate.checked_sub(1) {
                    Some(candidate) => candidate,
                    None if wrap => LARGE_SELECT_ITEMS - 1,
                    None => return None,
                }
            };
            if Self::enabled(candidate) {
                return Some(candidate);
            }
        }
        None
    }

    fn search_prefix(&self, query: &str, _: Option<usize>) -> Option<usize> {
        query
            .strip_prefix("大数据选项 ")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|index| self.is_enabled(*index))
    }

    fn option_count(&self) -> usize {
        LARGE_SELECT_ITEMS
    }

    fn option_position(&self, index: usize) -> Option<usize> {
        (index < LARGE_SELECT_ITEMS).then_some(index)
    }
}

struct SelectExample {
    plan: Option<Plan>,
    region: Option<&'static str>,
    city: Option<usize>,
    large_selected: Option<usize>,
    large_source: Rc<MillionSelectSource>,
    focus_handle: FocusHandle,
}

impl SelectExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            plan: None,
            region: Some("华东"),
            city: None,
            large_selected: Some(1),
            large_source: Rc::new(MillionSelectSource::new()),
            focus_handle,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}

impl Render for SelectExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);
        div()
            .id("vektra-select-example")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .size_full()
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .p(px(24.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .max_w(px(520.))
                    .child(div().text_size(px(24.)).child("Vektra Select"))
                    .child("受控单值选择：导航只移动 active option，提交后才请求下一值。")
                    .child(shared::theme_selector("select-example", window, cx))
                    .child(
                        Select::new("plan-select")
                            .selected_value(self.plan)
                            .placeholder("选择订阅方案")
                            .aria_label("订阅方案")
                            .on_change_in(cx, |this, plan, _, cx| {
                                this.plan = Some(plan);
                                cx.notify();
                            })
                            .option(
                                SelectOption::new("plan-free", Plan::Free, "免费版")
                                    .description("适合个人体验与小型项目"),
                            )
                            .group(
                                SelectGroup::new("paid-plans", "付费方案")
                                    .option(
                                        SelectOption::new("plan-pro", Plan::Pro, "专业版")
                                            .description("适合持续交付的专业团队"),
                                    )
                                    .option(
                                        SelectOption::new(
                                            "plan-enterprise",
                                            Plan::Enterprise,
                                            "企业版",
                                        )
                                        .description("请联系销售获取报价")
                                        .disabled(true),
                                    ),
                            ),
                    )
                    .child(
                        Select::new("region-select")
                            .selected_value(self.region)
                            .aria_label("部署区域")
                            .on_change_in(cx, |this, region, _, cx| {
                                this.region = Some(region);
                                cx.notify();
                            })
                            .option(SelectOption::new("region-east", "华东", "华东"))
                            .option(SelectOption::new("region-south", "华南", "华南"))
                            .option(SelectOption::new("region-global", "海外", "海外")),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(div().text_size(px(18.)).child("长列表滚动"))
                            .child(
                                "打开后可输入名称定位，使用 PageUp/PageDown 实测分页，按 End 跳到最后一项。",
                            )
                            .child({
                                let mut city_select = Select::new("city-select")
                                    .selected_value(self.city)
                                    .placeholder("选择城市")
                                    .aria_label("城市")
                                    .on_change_in(cx, |this, city, _, cx| {
                                        this.city = Some(city);
                                        cx.notify();
                                    });

                                for index in 1..=40 {
                                    city_select = city_select.option(SelectOption::new(
                                        format!("city-{index}"),
                                        index,
                                        format!("城市 {index:02}"),
                                    ));
                                }

                                city_select
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_size(px(18.))
                                    .child("大数据 Select：1,000,000 项惰性生成"),
                            )
                            .child(
                                "该场景不构建百万项 Vec；外部数据源负责定位、禁用导航和搜索索引，Popup 只物化视口行。",
                            )
                            .child(
                                div()
                                    .debug_selector(|| "large-select-diagnostics".into())
                                    .flex()
                                    .flex_col()
                                    .child(format!(
                                        "visible range: {:07}..{:07}",
                                        self.large_source.visible_start.get(),
                                        self.large_source.visible_end.get(),
                                    ))
                                    .child(format!(
                                        "本次进程累计 item() 读取: {:010} · Vektra 行缓存: 0",
                                        self.large_source.item_reads.get(),
                                    )),
                            )
                            .child(
                                Button::new("large-select-jump")
                                    .label("将受控值跳到第 900,000 项")
                                    .on_click_in(cx, |this, _, _, cx| {
                                        this.large_selected = Some(900_001);
                                        cx.notify();
                                    }),
                            )
                            .child({
                                let source: Rc<dyn SelectDataSource<usize>> =
                                    self.large_source.clone();
                                Select::new("large-data-select")
                                    .selected_value(self.large_selected)
                                    .placeholder("选择百万项数据")
                                    .aria_label("百万项大数据 Select")
                                    .data_source(source)
                                    .on_change_in(cx, |this, value, _, cx| {
                                        this.large_selected = Some(value);
                                        cx.notify();
                                    })
                            }),
                    )
            )
    }
}

fn main() {
    application()
        .with_assets(vektra::assets::Assets)
        .run(|cx: &mut App| {
            cx.bind_keys([
                KeyBinding::new("tab", Tab, None),
                KeyBinding::new("shift-tab", TabPrev, None),
            ]);
            let bounds = Bounds::centered(None, size(px(720.), px(900.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Vektra Select".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| SelectExample::new(window, cx)),
            )
            .expect("Select 示例窗口应能成功打开");
            cx.activate(true);
        });
}
