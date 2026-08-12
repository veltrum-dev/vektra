use super::{PreviewApp, PreviewLang};
use gpui::{Context, InteractiveElement, IntoElement, ParentElement, Styled, Window, div, px};
use vektra::{ComponentSize, Select, SelectGroup, SelectOption, SelectStatus};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Plan {
    Free,
    Pro,
    Enterprise,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Region {
    East,
    South,
    Global,
}

pub(crate) struct SelectDemo {
    basic: Option<Plan>,
    grouped: Option<Region>,
    keyboard: Option<Plan>,
    long_list: Option<usize>,
}

impl SelectDemo {
    pub(crate) const fn new() -> Self {
        Self {
            basic: None,
            grouped: Some(Region::East),
            keyboard: Some(Plan::Pro),
            long_list: None,
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
                "Enter/Space 打开，方向键与 Home/End 导航，Escape 关闭",
                "免费版",
                "专业版",
                "企业版",
            ),
            PreviewLang::EnUs => (
                "Keyboard navigation",
                "Enter/Space opens; arrows and Home/End navigate; Escape closes",
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
        let (title, label) = match language {
            PreviewLang::ZhCn => ("长列表与滚动跟随", "选择城市"),
            PreviewLang::EnUs => ("Long list and scroll follow", "Choose a city"),
        };

        // #region select-example-long-list
        let mut example = Select::new("select-long-list")
            .selected_value(self.long_list)
            .aria_label(label)
            .on_change_in(cx, |this, next, _, cx| {
                this.select_demo.long_list = Some(next);
                cx.notify();
            });
        for index in 1..=30 {
            example = example.option(SelectOption::new(
                format!("select-city-{index}"),
                index,
                format!("{label} {index}"),
            ));
        }
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
