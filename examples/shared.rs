use gpui::{App, Orientation, ParentElement, Styled, Window, div, px};
use vektra::{
    ComponentSize, Radio, RadioGroup, ResolvedThemeMode, ThemeMode, resolved_theme_mode,
    set_theme_mode, theme_mode,
};

pub(crate) fn theme_selector(id_prefix: &'static str, window: &Window, cx: &App) -> gpui::Div {
    let configured = theme_mode(cx);
    let resolved = resolved_theme_mode(window, cx);

    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(div().text_size(px(13.)).child(format!(
            "主题模式：{} · 当前主题：{}",
            configured_label(configured),
            resolved_label(resolved)
        )))
        .child(
            RadioGroup::new(format!("{id_prefix}-theme-group"))
                .selected_value(Some(configured))
                .orientation(Orientation::Horizontal)
                .size(ComponentSize::Sm)
                .aria_label("当前主题")
                .on_change(|mode, _, cx| set_theme_mode(mode, cx))
                .child(
                    Radio::new(format!("{id_prefix}-theme-system"), ThemeMode::System)
                        .label("System"),
                )
                .child(
                    Radio::new(format!("{id_prefix}-theme-light"), ThemeMode::Light).label("Light"),
                )
                .child(
                    Radio::new(format!("{id_prefix}-theme-dark"), ThemeMode::Dark).label("Dark"),
                ),
        )
}

fn configured_label(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::System => "System",
        ThemeMode::Light => "Light",
        ThemeMode::Dark => "Dark",
    }
}

fn resolved_label(mode: ResolvedThemeMode) -> &'static str {
    match mode {
        ResolvedThemeMode::Light => "Light",
        ResolvedThemeMode::Dark => "Dark",
    }
}
