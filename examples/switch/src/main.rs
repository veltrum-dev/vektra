use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, Styled, Window, WindowBounds, WindowOptions, actions, div, px, size,
};
use gpui_platform::application;
use std::time::Duration;
use vektra::{
    ComponentSize, IconSource, Switch, SwitchContent, ThemeMode, current_theme, set_theme_mode,
};

actions!(vektra_switch_example, [Tab, TabPrev]);

struct SwitchExample {
    notifications: bool,
    analytics: bool,
    icon_status: bool,
    detailed_status: bool,
    instant_status: bool,
    focus_status: &'static str,
    focus_handle: FocusHandle,
}

impl SwitchExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            notifications: true,
            analytics: false,
            icon_status: true,
            detailed_status: false,
            instant_status: true,
            focus_status: "焦点尚未移动",
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

impl Render for SwitchExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);
        div()
            .id("vektra-switch-example")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .size_full()
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .p(px(20.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .max_w(px(720.))
                    .child(div().text_size(px(24.)).child("Vektra Switch"))
                    .child("受控设置：默认 180ms ease-out cubic；Space 切换，Enter 不切换。")
                    .child(self.focus_status)
                    .child(
                        Switch::new("notifications")
                            .checked(self.notifications)
                            .label("MD · 推送通知（紧凑，默认 180ms）")
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.notifications = next_checked;
                                cx.notify();
                            })
                            .on_focus_in(cx, |this, _, cx| {
                                this.focus_status = "已聚焦：推送通知";
                                cx.notify();
                            })
                            .on_blur_in(cx, |this, _, cx| {
                                this.focus_status = "已失焦：推送通知";
                                cx.notify();
                            }),
                    )
                    .child(
                        Switch::new("analytics")
                            .checked(self.analytics)
                            .checked_content(SwitchContent::text("开启"))
                            .unchecked_content(SwitchContent::text("关闭"))
                            .label("SM · 自动更新")
                            .size(ComponentSize::Sm)
                            .transition_duration(Duration::from_millis(100))
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.analytics = next_checked;
                                cx.notify();
                            }),
                    )
                    .child(
                        Switch::new("icon-status")
                            .checked(self.icon_status)
                            .checked_content(SwitchContent::icon(IconSource::asset(
                                "components/checkbox/check.svg",
                            )))
                            .unchecked_content(SwitchContent::icon(IconSource::asset(
                                "components/checkbox/minus.svg",
                            )))
                            .label("MD · 声音提醒（默认 180ms）")
                            .size(ComponentSize::Md)
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.icon_status = next_checked;
                                cx.notify();
                            }),
                    )
                    .child(
                        Switch::new("detailed-status")
                            .checked(self.detailed_status)
                            .checked_content(SwitchContent::icon_text(
                                IconSource::asset("components/checkbox/check.svg"),
                                "开启",
                            ))
                            .unchecked_content(SwitchContent::icon_text(
                                IconSource::asset("components/checkbox/minus.svg"),
                                "关闭",
                            ))
                            .label("LG · 在线状态")
                            .size(ComponentSize::Lg)
                            .transition_duration(Duration::from_millis(400))
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.detailed_status = next_checked;
                                cx.notify();
                            }),
                    )
                    .child(
                        Switch::new("instant-status")
                            .checked(self.instant_status)
                            .checked_content(SwitchContent::text("即时"))
                            .unchecked_content(SwitchContent::text("关闭"))
                            .label("MD · 无动画切换（Duration::ZERO）")
                            .transition_duration(Duration::ZERO)
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.instant_status = next_checked;
                                cx.notify();
                            }),
                    )
                    .child(
                        Switch::new("loading-checked")
                            .checked(true)
                            .checked_content(SwitchContent::text("开启"))
                            .unchecked_content(SwitchContent::text("关闭"))
                            .label("MD · Loading（checked，保留 Tab）")
                            .loading(true),
                    )
                    .child(
                        Switch::new("loading-unchecked")
                            .checked(false)
                            .checked_content(SwitchContent::text("开启"))
                            .unchecked_content(SwitchContent::text("关闭"))
                            .label("MD · Loading（unchecked，保留 Tab）")
                            .loading(true),
                    )
                    .child(
                        Switch::new("disabled")
                            .checked(true)
                            .checked_content(SwitchContent::text("开启"))
                            .unchecked_content(SwitchContent::text("关闭"))
                            .label("MD · 禁用设置")
                            .disabled(true),
                    )
                    .child(
                        Switch::new("disabled-loading")
                            .checked(true)
                            .checked_content(SwitchContent::icon_text(
                                IconSource::asset("components/checkbox/check.svg"),
                                "开启",
                            ))
                            .unchecked_content(SwitchContent::text("关闭"))
                            .label("LG · Disabled + loading")
                            .size(ComponentSize::Lg)
                            .disabled(true)
                            .loading(true),
                    ),
            )
    }
}

fn main() {
    application()
        .with_assets(vektra::assets::Assets)
        .run(|cx: &mut App| {
            set_theme_mode(ThemeMode::System, cx);
            cx.bind_keys([
                KeyBinding::new("tab", Tab, None),
                KeyBinding::new("shift-tab", TabPrev, None),
            ]);
            let bounds = Bounds::centered(None, size(px(720.), px(680.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| SwitchExample::new(window, cx)),
            )
            .expect("Switch 示例窗口应能成功打开");
            cx.activate(true);
        });
}
