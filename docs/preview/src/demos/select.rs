use super::{PreviewApp, PreviewLang};
use gpui::{
    App, Context, ElementId, InteractiveElement, IntoElement, ParentElement, Styled, Window, div,
    px,
};
use std::{cell::Cell, ops::Range, rc::Rc};
use vektra::{
    Button, ComponentSize, LazyDataSource, Select, SelectDataSource, SelectEntry, SelectGroup,
    SelectOption, SelectStatus,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Plan {
    Free,
    Pro,
    Enterprise,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Region {
    East,
    South,
    Global,
}

const LARGE_SELECT_ITEMS: usize = 1_000_000;

struct LargeSelectSource {
    visible_start: Cell<usize>,
    visible_end: Cell<usize>,
}

impl LargeSelectSource {
    fn new() -> Self {
        Self {
            visible_start: Cell::new(0),
            visible_end: Cell::new(0),
        }
    }
}

impl LazyDataSource for LargeSelectSource {
    type Item = SelectEntry<usize>;
    type Key = ElementId;

    fn item_count(&self) -> usize {
        LARGE_SELECT_ITEMS
    }

    fn revision(&self) -> u64 {
        1
    }

    fn key(&self, index: usize) -> Self::Key {
        ElementId::named_usize("preview-large-select", index)
    }

    fn item(&self, index: usize) -> Option<Self::Item> {
        (index < LARGE_SELECT_ITEMS).then(|| {
            SelectEntry::Option(SelectOption::new(
                self.key(index),
                index,
                format!("Item {index:07}"),
            ))
        })
    }

    fn request_range(&self, range: Range<usize>, _: &mut Window, _: &mut App) {
        self.visible_start.set(range.start);
        self.visible_end.set(range.end);
    }
}

impl SelectDataSource<usize> for LargeSelectSource {
    fn index_of_key(&self, key: &ElementId) -> Option<usize> {
        match key {
            ElementId::NamedInteger(name, index) if name.as_ref() == "preview-large-select" => {
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
        Some(0)
    }

    fn is_enabled(&self, index: usize) -> bool {
        index < LARGE_SELECT_ITEMS
    }

    fn last_enabled(&self) -> Option<usize> {
        Some(LARGE_SELECT_ITEMS - 1)
    }

    fn next_enabled(&self, index: usize, forward: bool, wrap: bool) -> Option<usize> {
        if forward {
            index
                .checked_add(1)
                .filter(|index| *index < LARGE_SELECT_ITEMS)
                .or_else(|| wrap.then_some(0))
        } else {
            index
                .checked_sub(1)
                .or_else(|| wrap.then_some(LARGE_SELECT_ITEMS - 1))
        }
    }

    fn search_prefix(&self, query: &str, _: Option<usize>) -> Option<usize> {
        query
            .strip_prefix("item ")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|index| *index < LARGE_SELECT_ITEMS)
    }

    fn option_count(&self) -> usize {
        LARGE_SELECT_ITEMS
    }

    fn option_position(&self, index: usize) -> Option<usize> {
        (index < LARGE_SELECT_ITEMS).then_some(index)
    }
}

pub(crate) struct SelectDemo {
    basic: Option<Plan>,
    grouped: Option<Region>,
    keyboard: Option<Plan>,
    long_list: Option<usize>,
    large_source: Rc<LargeSelectSource>,
}

impl SelectDemo {
    pub(crate) fn new() -> Self {
        Self {
            basic: None,
            grouped: Some(Region::East),
            keyboard: Some(Plan::Pro),
            long_list: None,
            large_source: Rc::new(LargeSelectSource::new()),
        }
    }

    pub(crate) fn render_basic(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> impl IntoElement {
        let (title, label, placeholder, free, pro) = match language {
            PreviewLang::ZhCn => ("选择订阅方案", "订阅方案", "请选择方案", "免费版", "专业版"),
            PreviewLang::EnUs => (
                "Choose a subscription plan",
                "Subscription plan",
                "Choose a plan",
                "Free",
                "Pro",
            ),
        };

        // #region select-example-basic
        let example = Select::new("select-basic")
            .selected_value(self.basic)
            .placeholder(placeholder)
            .aria_label(label)
            .on_change_in(cx, |this, next, _, cx| {
                this.select_demo.basic = Some(next);
                cx.notify();
            })
            .option(SelectOption::new("select-basic-free", Plan::Free, free))
            .option(SelectOption::new("select-basic-pro", Plan::Pro, pro));
        // #endregion select-example-basic

        self.example_page("select-example-basic", title, example, window, cx)
    }

    pub(crate) fn render_groups(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> impl IntoElement {
        let (title, label, domestic, global, east, south, overseas) = match language {
            PreviewLang::ZhCn => (
                "分组与禁用项",
                "部署区域",
                "中国大陆",
                "全球区域",
                "华东",
                "华南（维护中）",
                "海外",
            ),
            PreviewLang::EnUs => (
                "Groups and disabled options",
                "Deployment region",
                "Mainland China",
                "Global regions",
                "East",
                "South (maintenance)",
                "Global",
            ),
        };

        // #region select-example-groups
        let example = Select::new("select-groups")
            .selected_value(self.grouped)
            .aria_label(label)
            .on_change_in(cx, |this, next, _, cx| {
                this.select_demo.grouped = Some(next);
                cx.notify();
            })
            .group(
                SelectGroup::new("select-domestic", domestic)
                    .option(SelectOption::new("select-east", Region::East, east))
                    .option(SelectOption::new("select-south", Region::South, south).disabled(true)),
            )
            .group(
                SelectGroup::new("select-global", global).option(SelectOption::new(
                    "select-overseas",
                    Region::Global,
                    overseas,
                )),
            );
        // #endregion select-example-groups

        self.example_page("select-example-groups", title, example, window, cx)
    }

    pub(crate) fn render_states(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> impl IntoElement {
        let (title, loading, empty, error, disabled) = match language {
            PreviewLang::ZhCn => (
                "宿主控制状态",
                "正在加载方案",
                "暂无可用方案",
                "方案加载失败",
                "已禁用",
            ),
            PreviewLang::EnUs => (
                "Host-controlled states",
                "Loading plans",
                "No plans available",
                "Failed to load plans",
                "Disabled",
            ),
        };

        // #region select-example-states
        let example = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                Select::<Plan>::new("select-loading")
                    .aria_label(loading)
                    .status(SelectStatus::loading(loading)),
            )
            .child(
                Select::<Plan>::new("select-empty")
                    .aria_label(empty)
                    .status(SelectStatus::empty(empty)),
            )
            .child(
                Select::<Plan>::new("select-error")
                    .aria_label(error)
                    .status(SelectStatus::error(error)),
            )
            .child(
                Select::new("select-disabled")
                    .selected_value(Some(Plan::Free))
                    .aria_label(disabled)
                    .disabled(true)
                    .option(SelectOption::new(
                        "select-disabled-free",
                        Plan::Free,
                        disabled,
                    )),
            );
        // #endregion select-example-states

        self.example_page("select-example-states", title, example, window, cx)
    }

    pub(crate) fn render_keyboard(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> impl IntoElement {
        let (title, hint, free, pro, enterprise) = match language {
            PreviewLang::ZhCn => (
                "键盘导航",
                "Enter/Space 打开；输入名称、方向键、Home/End、PageUp/PageDown 导航",
                "免费版",
                "专业版",
                "企业版",
            ),
            PreviewLang::EnUs => (
                "Keyboard navigation",
                "Enter/Space opens; type a name or use arrows, Home/End, PageUp/PageDown",
                "Free",
                "Pro",
                "Enterprise",
            ),
        };

        // #region select-example-keyboard
        let example = div().flex().flex_col().gap(px(8.)).child(hint).child(
            Select::new("select-keyboard")
                .selected_value(self.keyboard)
                .aria_label(title)
                .size(ComponentSize::Lg)
                .on_change_in(cx, |this, next, _, cx| {
                    this.select_demo.keyboard = Some(next);
                    cx.notify();
                })
                .option(SelectOption::new("select-keyboard-free", Plan::Free, free))
                .option(SelectOption::new("select-keyboard-pro", Plan::Pro, pro))
                .option(SelectOption::new(
                    "select-keyboard-enterprise",
                    Plan::Enterprise,
                    enterprise,
                )),
        );
        // #endregion select-example-keyboard

        self.example_page("select-example-keyboard", title, example, window, cx)
    }

    pub(crate) fn render_long_list(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> impl IntoElement {
        let (title, label, hint, jump) = match language {
            PreviewLang::ZhCn => (
                "大数据 Select：1,000,000 项",
                "选择百万项数据",
                "生成式数据源；不构建百万项 Vec；Popup 只物化视口行，缓存上限为 0。",
                "跳到第 900,000 项",
            ),
            PreviewLang::EnUs => (
                "Large-data Select: 1,000,000 items",
                "Choose from one million items",
                "Generated source; no million-item Vec; the popup materializes viewport rows only with a zero-row cache.",
                "Jump to item 900,000",
            ),
        };

        // #region select-example-long-list
        let source: Rc<dyn SelectDataSource<usize>> = self.large_source.clone();
        let example = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(hint)
            .child(format!(
                "visible range: {}..{} · cache: 0",
                self.large_source.visible_start.get(),
                self.large_source.visible_end.get(),
            ))
            .child(Button::new("select-large-jump").label(jump).on_click_in(
                cx,
                |this, _, _, cx| {
                    this.select_demo.long_list = Some(900_000);
                    cx.notify();
                },
            ))
            .child(
                Select::new("select-long-list")
                    .selected_value(self.long_list)
                    .aria_label(label)
                    .data_source(source)
                    .on_change_in(cx, |this, next, _, cx| {
                        this.select_demo.long_list = Some(next);
                        cx.notify();
                    }),
            );
        // #endregion select-example-long-list

        self.example_page("select-example-long-list", title, example, window, cx)
    }

    fn example_page(
        &self,
        id: &'static str,
        title: &'static str,
        example: impl IntoElement,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> impl IntoElement {
        let theme = vektra::current_theme(window, cx);
        div()
            .id(id)
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(16.))
            .p(px(20.))
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(div().text_size(px(18.)).child(title))
            .child(div().w_full().max_w(px(360.)).child(example))
    }
}
