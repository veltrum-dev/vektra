use super::{PreviewApp, PreviewLang};
use gpui::{Context, InteractiveElement, IntoElement, ParentElement, Styled, Window, div, px};
use vektra::{ComponentSize, Radio, RadioGroup};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Plan {
    Free,
    Pro,
    Enterprise,
}

pub(crate) struct RadioDemo {
    plan: Option<Plan>,
    pending_plan: Option<Plan>,
}

impl RadioDemo {
    pub(crate) const fn new() -> Self {
        Self {
            plan: None,
            pending_plan: None,
        }
    }

    pub(crate) fn render(
        &self,
        language: PreviewLang,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> impl IntoElement {
        let theme = vektra::current_theme(window, cx);
        let (title, intro, group_label, free, free_description, pro, pro_description, enterprise) =
            match language {
                PreviewLang::ZhCn => (
                    "RadioGroup 预览",
                    "方向键循环选择，Home/End 跳转，Space 激活；企业版演示单项禁用。",
                    "订阅方案",
                    "免费版",
                    "适合个人体验与小型项目",
                    "专业版",
                    "适合持续交付的专业团队",
                    "企业版（不可用）",
                ),
                PreviewLang::EnUs => (
                    "RadioGroup preview",
                    "Arrow keys wrap, Home/End jump, and Space activates. Enterprise is disabled.",
                    "Subscription plan",
                    "Free",
                    "For personal evaluation and small projects",
                    "Pro",
                    "For professional teams shipping continuously",
                    "Enterprise (unavailable)",
                ),
            };

        div()
            .id("radio-basic-demo")
            .size_full()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(20.))
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(div().text_size(px(24.)).child(title))
            .child(intro)
            .child(
                // #region radio-basic
                RadioGroup::new("preview-plan-group")
                    .selected_value(self.plan)
                    .aria_label(group_label)
                    .aria_description(intro)
                    .size(ComponentSize::Md)
                    .on_change_in(cx, |this, requested_plan, _, cx| {
                        // 预览立即批准；真实应用也可以先保存到 pending_plan，接口成功后再提交。
                        this.radio_demo.pending_plan = Some(requested_plan);
                        this.radio_demo.plan = this.radio_demo.pending_plan.take();
                        cx.notify();
                    })
                    .child(
                        Radio::new("preview-plan-free", Plan::Free)
                            .label(free)
                            .description(free_description),
                    )
                    .child(
                        Radio::new("preview-plan-pro", Plan::Pro)
                            .label(pro)
                            .description(pro_description),
                    )
                    .child(
                        Radio::new("preview-plan-enterprise", Plan::Enterprise)
                            .label(enterprise)
                            .disabled(true),
                    ),
                // #endregion radio-basic
            )
    }
}
