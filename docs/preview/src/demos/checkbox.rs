use super::{PreviewApp, PreviewLang};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use vektra::{Button, Checkbox, ComponentSize, IconName, IconSource};

// #region checkbox-state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

pub(super) struct CheckboxDemo {
    terms: CheckboxState,
    mixed: CheckboxState,
    no_label: CheckboxState,
    xs: CheckboxState,
    sm: CheckboxState,
    md: CheckboxState,
    lg: CheckboxState,
    custom_unchecked: CheckboxState,
    custom_checked: CheckboxState,
    custom_mixed: CheckboxState,
    favorite: CheckboxState,
    batch_product: CheckboxState,
    batch_billing: CheckboxState,
    batch_security: CheckboxState,
    global_size: CheckboxState,
    explicit_size: CheckboxState,
}

impl CheckboxDemo {
    pub(super) const fn new() -> Self {
        Self {
            terms: CheckboxState::unchecked(),
            mixed: CheckboxState::indeterminate(),
            no_label: CheckboxState::unchecked(),
            xs: CheckboxState::unchecked(),
            sm: CheckboxState::checked(),
            md: CheckboxState::checked(),
            lg: CheckboxState::indeterminate(),
            custom_unchecked: CheckboxState::unchecked(),
            custom_checked: CheckboxState::checked(),
            custom_mixed: CheckboxState::indeterminate(),
            favorite: CheckboxState::unchecked(),
            batch_product: CheckboxState::checked(),
            batch_billing: CheckboxState::unchecked(),
            batch_security: CheckboxState::checked(),
            global_size: CheckboxState::unchecked(),
            explicit_size: CheckboxState::unchecked(),
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
// #endregion checkbox-state

impl CheckboxDemo {
    pub(super) fn render(
        &self,
        language: PreviewLang,
        focus_status: gpui::SharedString,
        window: &mut Window,
        cx: &mut Context<PreviewApp>,
    ) -> AnyElement {
        let theme = vektra::current_theme(window, cx);
        let (
            title,
            intro,
            controlled,
            mixed,
            disabled,
            no_label,
            custom_icons,
            icon_only,
            favorite,
            batch_title,
            batch_all,
            batch_product,
            batch_billing,
            batch_security,
            batch_selected,
            select_all,
            invert,
            global_size,
            explicit,
        ) = match language {
            PreviewLang::ZhCn => (
                "Checkbox 预览",
                "受控状态由宿主保存；Space 激活，Enter 不激活。",
                "接受服务条款",
                "部分选中项目",
                "禁用选项",
                "无可见 label",
                "自定义状态图标",
                "纯图标状态切换",
                "收藏",
                "批量选择",
                "所有通知",
                "产品更新",
                "账单提醒",
                "安全警报",
                "已选择",
                "全选",
                "反选",
                "全局尺寸",
                "显式 Md 覆盖",
            ),
            PreviewLang::EnUs => (
                "Checkbox preview",
                "State is controlled by the host; Space activates while Enter does not.",
                "Accept terms of service",
                "Partially selected items",
                "Disabled option",
                "No visible label",
                "Custom state icons",
                "Icon-only state toggle",
                "Favorite",
                "Bulk selection",
                "All notifications",
                "Product updates",
                "Billing reminders",
                "Security alerts",
                "Selected",
                "Select all",
                "Invert",
                "Global size",
                "Explicit Md override",
            ),
        };

        div()
            .id("checkbox-basic-demo")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(20.))
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(div().text_size(px(24.)).child(title))
            .child(intro)
            .child(focus_status)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    // #region checkbox-basic
                    .child(
                        Checkbox::new("checkbox-controlled")
                            .checked(self.terms.checked)
                            .label(controlled)
                            .on_change_in(cx, |this, next_checked, _window, cx| {
                                this.checkbox_demo.terms.apply_change(next_checked);
                                cx.notify();
                            })
                            // #region checkbox-focus
                            .on_focus_in(cx, move |this, _, cx| {
                                this.record_focus(controlled, true, cx);
                            })
                            .on_blur_in(cx, move |this, _, cx| {
                                this.record_focus(controlled, false, cx);
                            }),
                        // #endregion checkbox-focus
                    )
                    .child(
                        Checkbox::new("checkbox-mixed")
                            .checked(self.mixed.checked)
                            .indeterminate(self.mixed.indeterminate)
                            .label(mixed)
                            .on_change_in(cx, |this, next_checked, _window, cx| {
                                this.checkbox_demo.mixed.apply_change(next_checked);
                                cx.notify();
                            }),
                    )
                    .child(
                        Checkbox::new("checkbox-disabled")
                            .checked(true)
                            .label(disabled)
                            .disabled(true),
                    )
                    .child(
                        Checkbox::new("checkbox-no-label")
                            .checked(self.no_label.checked)
                            .aria_label(no_label)
                            .aria_description("Standalone checkbox")
                            .on_change_in(cx, |this, next_checked, _window, cx| {
                                this.checkbox_demo.no_label.apply_change(next_checked);
                                cx.notify();
                            }),
                    ),
                // #endregion checkbox-basic
            )
            .child(
                div().flex().gap(px(10.)).flex_wrap().children([
                    Checkbox::new("checkbox-xs")
                        .checked(self.xs.checked)
                        .label("XS")
                        .size(ComponentSize::Xs)
                        .on_change_in(cx, |this, next_checked, _window, cx| {
                            this.checkbox_demo.xs.apply_change(next_checked);
                            cx.notify();
                        }),
                    Checkbox::new("checkbox-sm")
                        .checked(self.sm.checked)
                        .label("SM")
                        .size(ComponentSize::Sm)
                        .on_change_in(cx, |this, next_checked, _window, cx| {
                            this.checkbox_demo.sm.apply_change(next_checked);
                            cx.notify();
                        }),
                    Checkbox::new("checkbox-md")
                        .checked(self.md.checked)
                        .label("MD")
                        .size(ComponentSize::Md)
                        .on_change_in(cx, |this, next_checked, _window, cx| {
                            this.checkbox_demo.md.apply_change(next_checked);
                            cx.notify();
                        }),
                    Checkbox::new("checkbox-lg")
                        .checked(self.lg.checked)
                        .indeterminate(self.lg.indeterminate)
                        .label("LG")
                        .size(ComponentSize::Lg)
                        .on_change_in(cx, |this, next_checked, _window, cx| {
                            this.checkbox_demo.lg.apply_change(next_checked);
                            cx.notify();
                        }),
                ]),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(custom_icons)
                    .child(
                        Checkbox::new("checkbox-custom-unchecked")
                            .checked(self.custom_unchecked.checked)
                            .label("unchecked")
                            .unchecked_icon(IconSource::asset("icons/settings.svg"))
                            .on_change_in(cx, |this, next_checked, _window, cx| {
                                this.checkbox_demo
                                    .custom_unchecked
                                    .apply_change(next_checked);
                                cx.notify();
                            }),
                    )
                    .child(
                        Checkbox::new("checkbox-custom-checked")
                            .checked(self.custom_checked.checked)
                            .label("checked")
                            .checked_icon(IconName::Settings)
                            .on_change_in(cx, |this, next_checked, _window, cx| {
                                this.checkbox_demo.custom_checked.apply_change(next_checked);
                                cx.notify();
                            }),
                    )
                    .child(
                        Checkbox::new("checkbox-custom-mixed")
                            .checked(self.custom_mixed.checked)
                            .indeterminate(self.custom_mixed.indeterminate)
                            .label("mixed")
                            .indeterminate_icon(IconName::Settings)
                            .on_change_in(cx, |this, next_checked, _window, cx| {
                                this.checkbox_demo.custom_mixed.apply_change(next_checked);
                                cx.notify();
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(icon_only)
                    // #region checkbox-icon-only
                    .child(
                        Checkbox::new("checkbox-favorite")
                            .checked(self.favorite.checked)
                            .indicator_icons(
                                IconSource::asset("components/checkbox/heart.svg"),
                                IconSource::asset("components/checkbox/heart-filled.svg"),
                            )
                            .aria_label(favorite)
                            .on_change_in(cx, |this, next_checked, _window, cx| {
                                this.checkbox_demo.favorite.apply_change(next_checked);
                                cx.notify();
                            }),
                    ),
                // #endregion checkbox-icon-only
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(batch_title)
                    // #region checkbox-bulk
                    .child(
                        Checkbox::new("checkbox-batch-all")
                            .checked(self.batch_all_selected())
                            .indeterminate(self.batch_indeterminate())
                            .label(batch_all)
                            .on_change_in(cx, |this, next_checked, _window, cx| {
                                this.checkbox_demo.set_batch_checked(next_checked);
                                cx.notify();
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .child(
                                Checkbox::new("checkbox-batch-product")
                                    .checked(self.batch_product.checked)
                                    .label(batch_product)
                                    .on_change_in(cx, |this, next_checked, _window, cx| {
                                        this.checkbox_demo.batch_product.apply_change(next_checked);
                                        cx.notify();
                                    }),
                            )
                            .child(
                                Checkbox::new("checkbox-batch-billing")
                                    .checked(self.batch_billing.checked)
                                    .label(batch_billing)
                                    .on_change_in(cx, |this, next_checked, _window, cx| {
                                        this.checkbox_demo.batch_billing.apply_change(next_checked);
                                        cx.notify();
                                    }),
                            )
                            .child(
                                Checkbox::new("checkbox-batch-security")
                                    .checked(self.batch_security.checked)
                                    .label(batch_security)
                                    .on_change_in(cx, |this, next_checked, _window, cx| {
                                        this.checkbox_demo
                                            .batch_security
                                            .apply_change(next_checked);
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
                                Button::new("checkbox-batch-select-all")
                                    .label(select_all)
                                    .on_click_in(cx, |this, _, _window, cx| {
                                        this.checkbox_demo.set_batch_checked(true);
                                        cx.notify();
                                    }),
                            )
                            .child(
                                Button::new("checkbox-batch-invert")
                                    .label(invert)
                                    .on_click_in(cx, |this, _, _window, cx| {
                                        this.checkbox_demo.invert_batch_selection();
                                        cx.notify();
                                    }),
                            )
                            .child(format!(
                                "{batch_selected}: {}/3",
                                self.batch_selected_count()
                            )),
                    ),
                // #endregion checkbox-bulk
            )
            .child(
                div()
                    .flex()
                    .gap(px(10.))
                    .flex_wrap()
                    .child(
                        Checkbox::new("checkbox-global")
                            .checked(self.global_size.checked)
                            .label(global_size)
                            .on_change_in(cx, |this, next_checked, _window, cx| {
                                this.checkbox_demo.global_size.apply_change(next_checked);
                                cx.notify();
                            }),
                    )
                    .child(
                        Checkbox::new("checkbox-explicit")
                            .checked(self.explicit_size.checked)
                            .label(explicit)
                            .size(ComponentSize::Md)
                            .on_change_in(cx, |this, next_checked, _window, cx| {
                                this.checkbox_demo.explicit_size.apply_change(next_checked);
                                cx.notify();
                            }),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/checkbox.rs"]
mod tests;
