#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, AppContext, AssetSource, Bounds, Context, FocusHandle, InteractiveElement, IntoElement,
    KeyBinding, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window,
    WindowBounds, WindowOptions, actions, div, px, size,
};
use gpui_platform::application;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use vektra::{
    Button, Checkbox, ComponentSize, IconName, ThemeMode, component_size, current_theme,
    set_component_size, set_theme_mode,
};

actions!(vektra_checkbox_example, [Tab, TabPrev]);

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/heart-filled.svg"]
#[include = "icons/heart.svg"]
struct CheckboxExampleAssets;

impl AssetSource for CheckboxExampleAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(Self::get(path).map(|file| file.data))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter(|asset_path| asset_path.starts_with(path))
            .map(Into::into)
            .collect())
    }
}

#[derive(Clone, Copy)]
struct CheckboxState {
    checked: bool,
    indeterminate: bool,
}

impl CheckboxState {
    const fn unchecked() -> Self {
        Self {
            checked: false,
            indeterminate: false,
        }
    }

    const fn checked() -> Self {
        Self {
            checked: true,
            indeterminate: false,
        }
    }

    const fn indeterminate() -> Self {
        Self {
            checked: false,
            indeterminate: true,
        }
    }

    fn apply_change(&mut self, next_checked: bool) {
        self.checked = next_checked;
        self.indeterminate = false;
    }
}

struct CheckboxDemo {
    terms: CheckboxState,
    mixed: CheckboxState,
    no_label: CheckboxState,
    xs: CheckboxState,
    sm: CheckboxState,
    md: CheckboxState,
    lg: CheckboxState,
    custom_icon: CheckboxState,
    favorite: CheckboxState,
    batch_product: CheckboxState,
    batch_billing: CheckboxState,
    batch_security: CheckboxState,
}

impl CheckboxDemo {
    const fn new() -> Self {
        Self {
            terms: CheckboxState::unchecked(),
            mixed: CheckboxState::indeterminate(),
            no_label: CheckboxState::unchecked(),
            xs: CheckboxState::unchecked(),
            sm: CheckboxState::checked(),
            md: CheckboxState::checked(),
            lg: CheckboxState::indeterminate(),
            custom_icon: CheckboxState::checked(),
            favorite: CheckboxState::unchecked(),
            batch_product: CheckboxState::checked(),
            batch_billing: CheckboxState::unchecked(),
            batch_security: CheckboxState::checked(),
        }
    }

    fn batch_selected_count(&self) -> usize {
        [self.batch_product, self.batch_billing, self.batch_security]
            .into_iter()
            .filter(|state| state.checked)
            .count()
    }

    fn batch_all_selected(&self) -> bool {
        self.batch_selected_count() == 3
    }

    fn batch_indeterminate(&self) -> bool {
        matches!(self.batch_selected_count(), 1 | 2)
    }

    fn set_batch_checked(&mut self, checked: bool) {
        for item in [
            &mut self.batch_product,
            &mut self.batch_billing,
            &mut self.batch_security,
        ] {
            item.apply_change(checked);
        }
    }

    fn invert_batch_selection(&mut self) {
        self.batch_product.apply_change(!self.batch_product.checked);
        self.batch_billing.apply_change(!self.batch_billing.checked);
        self.batch_security
            .apply_change(!self.batch_security.checked);
    }
}

struct CheckboxExample {
    demo: CheckboxDemo,
    focus_status: SharedString,
    focus_handle: FocusHandle,
}

impl CheckboxExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            demo: CheckboxDemo::new(),
            focus_status: "焦点尚未移动".into(),
            focus_handle,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }

    fn record_focus(&mut self, focused: bool, cx: &mut Context<Self>) {
        self.focus_status = if focused {
            "已聚焦：接受服务条款"
        } else {
            "已失焦：接受服务条款"
        }
        .into();
        cx.notify();
    }
}

impl Render for CheckboxExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);

        div()
            .id("vektra-checkbox-example")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .size_full()
            .overflow_y_scroll()
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .p(px(20.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .max_w(px(720.))
                    .child(div().text_size(px(24.)).child("Vektra Checkbox"))
                    .child(format!("全局尺寸：{:?}", component_size(cx)))
                    .child(self.focus_status.clone())
                    .child(
                        Checkbox::new("terms")
                            .checked(self.demo.terms.checked)
                            .label("接受服务条款")
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.demo.terms.apply_change(next_checked);
                                cx.notify();
                            })
                            .on_focus_in(cx, |this, _, cx| this.record_focus(true, cx))
                            .on_blur_in(cx, |this, _, cx| this.record_focus(false, cx)),
                    )
                    .child(
                        Checkbox::new("mixed")
                            .checked(self.demo.mixed.checked)
                            .indeterminate(self.demo.mixed.indeterminate)
                            .label("部分选中项目")
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.demo.mixed.apply_change(next_checked);
                                cx.notify();
                            }),
                    )
                    .child(
                        Checkbox::new("no-label")
                            .checked(self.demo.no_label.checked)
                            .aria_label("无可见标签的复选框")
                            .aria_description("示例展示 aria_label 用法")
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.demo.no_label.apply_change(next_checked);
                                cx.notify();
                            }),
                    )
                    .child(
                        Checkbox::new("disabled")
                            .checked(true)
                            .label("禁用选项")
                            .disabled(true),
                    )
                    .child(
                        div().flex().gap(px(10.)).flex_wrap().children([
                            Checkbox::new("xs")
                                .checked(self.demo.xs.checked)
                                .label("XS")
                                .size(ComponentSize::Xs)
                                .on_change_in(cx, |this, next_checked, _, cx| {
                                    this.demo.xs.apply_change(next_checked);
                                    cx.notify();
                                }),
                            Checkbox::new("sm")
                                .checked(self.demo.sm.checked)
                                .label("SM")
                                .size(ComponentSize::Sm)
                                .on_change_in(cx, |this, next_checked, _, cx| {
                                    this.demo.sm.apply_change(next_checked);
                                    cx.notify();
                                }),
                            Checkbox::new("md")
                                .checked(self.demo.md.checked)
                                .label("MD")
                                .size(ComponentSize::Md)
                                .on_change_in(cx, |this, next_checked, _, cx| {
                                    this.demo.md.apply_change(next_checked);
                                    cx.notify();
                                }),
                            Checkbox::new("lg")
                                .checked(self.demo.lg.checked)
                                .indeterminate(self.demo.lg.indeterminate)
                                .label("LG")
                                .size(ComponentSize::Lg)
                                .on_change_in(cx, |this, next_checked, _, cx| {
                                    this.demo.lg.apply_change(next_checked);
                                    cx.notify();
                                }),
                        ]),
                    )
                    .child(
                        Checkbox::new("custom-icon")
                            .checked(self.demo.custom_icon.checked)
                            .label("自定义选中图标")
                            .checked_icon(IconName::Settings)
                            .on_change_in(cx, |this, next_checked, _, cx| {
                                this.demo.custom_icon.apply_change(next_checked);
                                cx.notify();
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child("纯图标状态切换")
                            .child(
                                Checkbox::new("favorite")
                                    .checked(self.demo.favorite.checked)
                                    .indicator_icons(
                                        vektra::IconSource::asset("icons/heart.svg"),
                                        vektra::IconSource::asset("icons/heart-filled.svg"),
                                    )
                                    .aria_label("收藏")
                                    .on_change_in(cx, |this, next_checked, _, cx| {
                                        this.demo.favorite.apply_change(next_checked);
                                        cx.notify();
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child("批量选择")
                            .child(
                                Checkbox::new("batch-all")
                                    .checked(self.demo.batch_all_selected())
                                    .indeterminate(self.demo.batch_indeterminate())
                                    .label("所有通知")
                                    .on_change_in(cx, |this, next_checked, _, cx| {
                                        this.demo.set_batch_checked(next_checked);
                                        cx.notify();
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(6.))
                                    .child(
                                        Checkbox::new("batch-product")
                                            .checked(self.demo.batch_product.checked)
                                            .label("产品更新")
                                            .on_change_in(cx, |this, next_checked, _, cx| {
                                                this.demo.batch_product.apply_change(next_checked);
                                                cx.notify();
                                            }),
                                    )
                                    .child(
                                        Checkbox::new("batch-billing")
                                            .checked(self.demo.batch_billing.checked)
                                            .label("账单提醒")
                                            .on_change_in(cx, |this, next_checked, _, cx| {
                                                this.demo.batch_billing.apply_change(next_checked);
                                                cx.notify();
                                            }),
                                    )
                                    .child(
                                        Checkbox::new("batch-security")
                                            .checked(self.demo.batch_security.checked)
                                            .label("安全警报")
                                            .on_change_in(cx, |this, next_checked, _, cx| {
                                                this.demo.batch_security.apply_change(next_checked);
                                                cx.notify();
                                            }),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap(px(8.))
                                    .flex_wrap()
                                    .child(
                                        Button::new("batch-select-all").label("全选").on_click_in(
                                            cx,
                                            |this, _, _, cx| {
                                                this.demo.set_batch_checked(true);
                                                cx.notify();
                                            },
                                        ),
                                    )
                                    .child(Button::new("batch-invert").label("反选").on_click_in(
                                        cx,
                                        |this, _, _, cx| {
                                            this.demo.invert_batch_selection();
                                            cx.notify();
                                        },
                                    ))
                                    .child(format!(
                                        "已选择：{}/3",
                                        self.demo.batch_selected_count()
                                    )),
                            ),
                    )
                    .child(
                        div().flex().gap(px(8.)).flex_wrap().children([
                            Button::new("size-sm")
                                .label("全局 Sm")
                                .on_click(|_, _, cx| {
                                    set_component_size(ComponentSize::Sm, cx);
                                }),
                            Button::new("size-md")
                                .label("全局 Md")
                                .on_click(|_, _, cx| {
                                    set_component_size(ComponentSize::Md, cx);
                                }),
                            Button::new("theme-light")
                                .label("浅色")
                                .on_click(|_, _, cx| {
                                    set_theme_mode(ThemeMode::Light, cx);
                                }),
                            Button::new("theme-dark")
                                .label("深色")
                                .on_click(|_, _, cx| {
                                    set_theme_mode(ThemeMode::Dark, cx);
                                }),
                        ]),
                    ),
            )
    }
}

fn main() {
    let assets = vektra::assets::Assets::with_overrides(CheckboxExampleAssets);
    application().with_assets(assets).run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("tab", Tab, None),
            KeyBinding::new("shift-tab", TabPrev, None),
        ]);

        let bounds = Bounds::centered(None, size(px(760.), px(520.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| CheckboxExample::new(window, cx)),
        )
        .expect("Checkbox 示例窗口应能成功打开");
        cx.activate(true);
    });
}

#[cfg(test)]
#[path = "../tests/assets.rs"]
mod tests;
