#[path = "../../shared.rs"]
mod shared;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, Styled, Window, WindowBounds, WindowOptions, actions, div, px, size,
};
use gpui_platform::application;
use vektra::{Radio, RadioGroup, current_theme};

actions!(vektra_radio_example, [Tab, TabPrev]);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Plan {
    Free,
    Pro,
    Enterprise,
}

struct RadioExample {
    plan: Option<Plan>,
    focus_handle: FocusHandle,
}

impl RadioExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            plan: None,
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

impl Render for RadioExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);
        div()
            .id("vektra-radio-example")
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
                    .child(div().text_size(px(24.)).child("Vektra RadioGroup"))
                    .child("受控单选：方向键、Home、End 和 Space 都请求下一值。")
                    .child(shared::theme_selector("radio-example", window, cx))
                    .child(
                        RadioGroup::new("plan-group")
                            .selected_value(self.plan)
                            .aria_label("订阅方案")
                            .aria_description("请选择一个适合你的方案")
                            .on_change_in(cx, |this, requested_plan, _, cx| {
                                this.plan = Some(requested_plan);
                                cx.notify();
                            })
                            .child(
                                Radio::new("plan-free", Plan::Free)
                                    .label("免费版")
                                    .description("适合个人体验与小型项目"),
                            )
                            .child(
                                Radio::new("plan-pro", Plan::Pro)
                                    .label("专业版")
                                    .description("适合持续交付的专业团队"),
                            )
                            .child(
                                Radio::new("plan-enterprise", Plan::Enterprise)
                                    .label("企业版")
                                    .description("请联系销售获取报价")
                                    .disabled(true),
                            ),
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
            let bounds = Bounds::centered(None, size(px(640.), px(500.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| RadioExample::new(window, cx)),
            )
            .expect("Radio 示例窗口应能成功打开");
            cx.activate(true);
        });
}
