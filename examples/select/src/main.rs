#[path = "../../shared.rs"]
mod shared;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, actions,
    div, px, size,
};
use gpui_platform::application;
use vektra::{Select, SelectGroup, SelectOption, current_theme};

actions!(vektra_select_example, [Tab, TabPrev]);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Plan {
    Free,
    Pro,
    Enterprise,
}

struct SelectExample {
    plan: Option<Plan>,
    region: Option<&'static str>,
    city: Option<usize>,
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
                            .child("打开后可使用滚轮或拖动滚动条，按 End 可跳到最后一项。")
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
                    ),
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
            let bounds = Bounds::centered(None, size(px(640.), px(680.)), cx);
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
