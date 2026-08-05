mod button;
mod checkbox;
mod icon_button;
mod input;
mod radio;
mod switch;
mod tooltip;

use gpui::{
    AnyElement, App, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, Styled, Window, actions, div, px,
};
use vektra::{ResolvedThemeMode, ThemeMode};

actions!(vektra_docs_preview, [Tab, TabPrev]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DemoSelection {
    ButtonBasic,
    ButtonShowcase,
    CheckboxBasic,
    RadioBasic,
    SwitchBasic,
    IconButtonBasic,
    InputBasic,
    TooltipBasic,
    Unknown(String),
}

impl DemoSelection {
    pub(crate) const DEFAULT_ID: &'static str = "button/basic";
    pub(crate) const SHOWCASE_ID: &'static str = "button/showcase";
    pub(crate) const CHECKBOX_ID: &'static str = "checkbox/basic";
    pub(crate) const RADIO_ID: &'static str = "radio/basic";
    pub(crate) const SWITCH_ID: &'static str = "switch/basic";
    pub(crate) const ICON_BUTTON_ID: &'static str = "icon-button/basic";
    pub(crate) const INPUT_ID: &'static str = "input/basic";
    pub(crate) const TOOLTIP_ID: &'static str = "tooltip/basic";

    fn from_demo_id(demo_id: Option<&str>) -> Self {
        match demo_id {
            None => Self::ButtonBasic,
            Some(Self::DEFAULT_ID) => Self::ButtonBasic,
            Some(Self::SHOWCASE_ID) => Self::ButtonShowcase,
            Some(Self::CHECKBOX_ID) => Self::CheckboxBasic,
            Some(Self::RADIO_ID) => Self::RadioBasic,
            Some(Self::SWITCH_ID) => Self::SwitchBasic,
            Some(Self::ICON_BUTTON_ID) => Self::IconButtonBasic,
            Some(Self::INPUT_ID) => Self::InputBasic,
            Some(Self::TOOLTIP_ID) => Self::TooltipBasic,
            Some(value) => Self::Unknown(value.to_owned()),
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::ButtonBasic => Self::DEFAULT_ID,
            Self::ButtonShowcase => Self::SHOWCASE_ID,
            Self::CheckboxBasic => Self::CHECKBOX_ID,
            Self::RadioBasic => Self::RADIO_ID,
            Self::SwitchBasic => Self::SWITCH_ID,
            Self::IconButtonBasic => Self::ICON_BUTTON_ID,
            Self::InputBasic => Self::INPUT_ID,
            Self::TooltipBasic => Self::TOOLTIP_ID,
            Self::Unknown(value) => value,
        }
    }

    fn status(&self) -> &'static str {
        match self {
            Self::ButtonBasic
            | Self::ButtonShowcase
            | Self::CheckboxBasic
            | Self::RadioBasic
            | Self::SwitchBasic
            | Self::IconButtonBasic
            | Self::InputBasic
            | Self::TooltipBasic => "ready",
            Self::Unknown(_) => "unknown-demo",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreviewLang {
    ZhCn,
    EnUs,
}

impl PreviewLang {
    fn from_lang(value: Option<&str>) -> Self {
        match value {
            Some("en-US") => Self::EnUs,
            _ => Self::ZhCn,
        }
    }

    pub(crate) fn no_recent_click(self) -> &'static str {
        match self {
            Self::ZhCn => "暂无",
            Self::EnUs => "None",
        }
    }

    fn no_recent_focus(self) -> &'static str {
        match self {
            Self::ZhCn => "焦点尚未移动",
            Self::EnUs => "Focus has not moved yet",
        }
    }

    fn unknown_title(self) -> &'static str {
        match self {
            Self::ZhCn => "未知预览",
            Self::EnUs => "Unknown preview",
        }
    }

    fn unknown_body(self, demo_id: &str) -> String {
        match self {
            Self::ZhCn => format!(
                "不支持 demo_id `{demo_id}`。当前支持的预览：`{}`、`{}`、`{}`、`{}`、`{}`、`{}`、`{}`、`{}`。",
                DemoSelection::DEFAULT_ID,
                DemoSelection::SHOWCASE_ID,
                DemoSelection::CHECKBOX_ID,
                DemoSelection::RADIO_ID,
                DemoSelection::SWITCH_ID,
                DemoSelection::ICON_BUTTON_ID,
                DemoSelection::INPUT_ID,
                DemoSelection::TOOLTIP_ID
            ),
            Self::EnUs => format!(
                "Unsupported demo_id `{demo_id}`. Supported previews: `{}`, `{}`, `{}`, `{}`, `{}`, `{}`, `{}`, and `{}`.",
                DemoSelection::DEFAULT_ID,
                DemoSelection::SHOWCASE_ID,
                DemoSelection::CHECKBOX_ID,
                DemoSelection::RADIO_ID,
                DemoSelection::SWITCH_ID,
                DemoSelection::ICON_BUTTON_ID,
                DemoSelection::INPUT_ID,
                DemoSelection::TOOLTIP_ID
            ),
        }
    }
}

pub(crate) struct PreviewApp {
    selection: DemoSelection,
    language: PreviewLang,
    font_family: &'static str,
    button_demo: button::ButtonDemo,
    checkbox_demo: checkbox::CheckboxDemo,
    radio_demo: radio::RadioDemo,
    switch_demo: switch::SwitchDemo,
    input_demo: input::InputDemo,
    focus_status: gpui::SharedString,
    focus_handle: FocusHandle,
}

impl PreviewApp {
    pub(crate) fn new(
        selection: DemoSelection,
        language: PreviewLang,
        font_family: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        set_browser_state(PreviewBrowserState {
            demo_id: selection.id(),
            status: selection.status(),
            clicks: 0,
            last_clicked: language.no_recent_click(),
            theme: resolved_theme_mode_label(vektra::resolved_theme_mode(window, cx)),
        });
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);

        Self {
            selection,
            language,
            font_family,
            button_demo: button::ButtonDemo::new(language),
            checkbox_demo: checkbox::CheckboxDemo::new(),
            radio_demo: radio::RadioDemo::new(),
            switch_demo: switch::SwitchDemo::new(),
            input_demo: input::InputDemo::new(cx),
            focus_status: language.no_recent_focus().into(),
            focus_handle,
        }
    }

    pub(crate) fn record_button_click(
        &mut self,
        label: gpui::SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.button_demo.record_click(label);
        set_browser_state(PreviewBrowserState {
            demo_id: self.selection.id(),
            status: self.selection.status(),
            clicks: self.button_demo.clicks(),
            last_clicked: self.button_demo.last_clicked(),
            theme: resolved_theme_mode_label(vektra::resolved_theme_mode(window, cx)),
        });
        cx.notify();
    }

    pub(crate) fn record_focus(
        &mut self,
        label: &'static str,
        focused: bool,
        cx: &mut Context<Self>,
    ) {
        self.focus_status = match (self.language, focused) {
            (PreviewLang::ZhCn, true) => format!("已聚焦：{label}"),
            (PreviewLang::ZhCn, false) => format!("已失焦：{label}"),
            (PreviewLang::EnUs, true) => format!("Focused: {label}"),
            (PreviewLang::EnUs, false) => format!("Blurred: {label}"),
        }
        .into();
        cx.notify();
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}

impl Render for PreviewApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_status = self.focus_status.clone();
        let child = match &self.selection {
            DemoSelection::ButtonBasic => self
                .button_demo
                .render_basic(self.language, focus_status, window, cx)
                .into_any_element(),
            DemoSelection::ButtonShowcase => self
                .button_demo
                .render_showcase(self.language, window, cx)
                .into_any_element(),
            DemoSelection::CheckboxBasic => self
                .checkbox_demo
                .render(self.language, focus_status, window, cx)
                .into_any_element(),
            DemoSelection::RadioBasic => self
                .radio_demo
                .render(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SwitchBasic => self
                .switch_demo
                .render(self.language, focus_status, window, cx)
                .into_any_element(),
            DemoSelection::IconButtonBasic => {
                icon_button::render(self.language, focus_status, window, cx).into_any_element()
            }
            DemoSelection::InputBasic => self
                .input_demo
                .render(self.language, window, cx)
                .into_any_element(),
            DemoSelection::TooltipBasic => {
                tooltip::render(self.language, window, cx).into_any_element()
            }
            DemoSelection::Unknown(demo_id) => {
                render_unknown_demo(demo_id, self.language, window, cx).into_any_element()
            }
        };

        div()
            .id("preview-root")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .font_family(self.font_family)
            .font_weight(gpui::FontWeight::MEDIUM)
            .size_full()
            .child(child)
    }
}

pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", Tab, None),
        KeyBinding::new("shift-tab", TabPrev, None),
    ]);
}

fn render_unknown_demo(
    demo_id: &str,
    language: PreviewLang,
    window: &mut Window,
    cx: &mut Context<PreviewApp>,
) -> AnyElement {
    let theme = vektra::current_theme(window, cx);

    div()
        .id("unknown-demo")
        .size_full()
        .bg(theme.semantic.background)
        .text_color(theme.semantic.foreground)
        .flex()
        .items_center()
        .justify_center()
        .p(px(20.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(10.))
                .max_w(px(520.))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(language.unknown_title()),
                )
                .child(
                    div()
                        .text_size(px(14.))
                        .line_height(px(22.))
                        .child(language.unknown_body(demo_id)),
                ),
        )
        .into_any_element()
}

pub(crate) fn current_selection() -> DemoSelection {
    #[cfg(target_family = "wasm")]
    {
        DemoSelection::from_demo_id(current_demo_id().as_deref())
    }

    #[cfg(not(target_family = "wasm"))]
    {
        let demo_id = std::env::var("VEKTRA_PREVIEW_DEMO").ok();
        DemoSelection::from_demo_id(demo_id.as_deref())
    }
}

pub(crate) fn current_theme_mode() -> ThemeMode {
    #[cfg(target_family = "wasm")]
    {
        let query = web_sys::window()
            .and_then(|window| window.location().search().ok())
            .unwrap_or_default();
        parse_theme_query(&query)
    }

    #[cfg(not(target_family = "wasm"))]
    {
        ThemeMode::System
    }
}

pub(crate) fn current_language() -> PreviewLang {
    #[cfg(target_family = "wasm")]
    {
        let query = web_sys::window()
            .and_then(|window| window.location().search().ok())
            .unwrap_or_default();
        parse_lang_query(&query)
    }

    #[cfg(not(target_family = "wasm"))]
    {
        let lang = std::env::var("VEKTRA_PREVIEW_LANG").ok();
        PreviewLang::from_lang(lang.as_deref())
    }
}

pub(crate) fn theme_mode_label(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::System => "跟随系统",
        ThemeMode::Light => "浅色",
        ThemeMode::Dark => "深色",
    }
}

pub(crate) fn theme_mode_label_for(mode: ThemeMode, language: PreviewLang) -> &'static str {
    match language {
        PreviewLang::ZhCn => theme_mode_label(mode),
        PreviewLang::EnUs => match mode {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        },
    }
}

pub(crate) fn resolved_theme_mode_label(mode: ResolvedThemeMode) -> &'static str {
    match mode {
        ResolvedThemeMode::Light => "light",
        ResolvedThemeMode::Dark => "dark",
    }
}

pub(crate) fn resolved_theme_mode_label_for(
    mode: ResolvedThemeMode,
    language: PreviewLang,
) -> &'static str {
    match language {
        PreviewLang::ZhCn => match mode {
            ResolvedThemeMode::Light => "浅色",
            ResolvedThemeMode::Dark => "深色",
        },
        PreviewLang::EnUs => match mode {
            ResolvedThemeMode::Light => "Light",
            ResolvedThemeMode::Dark => "Dark",
        },
    }
}

#[cfg(any(test, target_family = "wasm"))]
fn parse_demo_query(query: &str) -> DemoSelection {
    let query = query.strip_prefix('?').unwrap_or(query);
    let demo_id = query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == "demo").then(|| decode_query_value(value))
    });

    DemoSelection::from_demo_id(demo_id.as_deref())
}

#[cfg(any(test, target_family = "wasm"))]
fn parse_theme_query(query: &str) -> ThemeMode {
    let theme = query_value(query, "theme");

    match theme.as_deref() {
        Some("light") => ThemeMode::Light,
        Some("dark") => ThemeMode::Dark,
        _ => ThemeMode::System,
    }
}

#[cfg(any(test, target_family = "wasm"))]
fn parse_lang_query(query: &str) -> PreviewLang {
    PreviewLang::from_lang(query_value(query, "lang").as_deref())
}

#[cfg(any(test, target_family = "wasm"))]
fn query_value(query: &str, expected_key: &str) -> Option<String> {
    let query = query.strip_prefix('?').unwrap_or(query);
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == expected_key).then(|| decode_query_value(value))
    })
}

#[cfg(any(test, target_family = "wasm"))]
fn decode_query_value(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while let Some(&byte) = bytes.get(index) {
        if byte == b'%'
            && let (Some(high), Some(low)) = (bytes.get(index + 1), bytes.get(index + 2))
            && let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low))
        {
            decoded.push((high << 4) | low);
            index += 3;
            continue;
        }

        decoded.push(if byte == b'+' { b' ' } else { byte });
        index += 1;
    }

    String::from_utf8(decoded).unwrap_or_else(|_| value.to_owned())
}

#[cfg(any(test, target_family = "wasm"))]
fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(target_family = "wasm")]
fn current_demo_id() -> Option<String> {
    let query = web_sys::window()?.location().search().ok()?;
    match parse_demo_query(&query) {
        DemoSelection::ButtonBasic => Some(DemoSelection::DEFAULT_ID.to_owned()),
        DemoSelection::ButtonShowcase => Some(DemoSelection::SHOWCASE_ID.to_owned()),
        DemoSelection::CheckboxBasic => Some(DemoSelection::CHECKBOX_ID.to_owned()),
        DemoSelection::RadioBasic => Some(DemoSelection::RADIO_ID.to_owned()),
        DemoSelection::SwitchBasic => Some(DemoSelection::SWITCH_ID.to_owned()),
        DemoSelection::IconButtonBasic => Some(DemoSelection::ICON_BUTTON_ID.to_owned()),
        DemoSelection::InputBasic => Some(DemoSelection::INPUT_ID.to_owned()),
        DemoSelection::TooltipBasic => Some(DemoSelection::TOOLTIP_ID.to_owned()),
        DemoSelection::Unknown(value) => Some(value),
    }
}

struct PreviewBrowserState<'a> {
    demo_id: &'a str,
    status: &'a str,
    clicks: usize,
    last_clicked: &'a str,
    theme: &'a str,
}

#[cfg(target_family = "wasm")]
fn set_browser_state(state: PreviewBrowserState<'_>) {
    let Some(window) = web_sys::window() else {
        return;
    };

    let js_state = js_sys::Object::new();
    js_sys::Reflect::set(&js_state, &"demoId".into(), &state.demo_id.into()).ok();
    js_sys::Reflect::set(&js_state, &"status".into(), &state.status.into()).ok();
    js_sys::Reflect::set(&js_state, &"clicks".into(), &(state.clicks as f64).into()).ok();
    js_sys::Reflect::set(&js_state, &"lastClicked".into(), &state.last_clicked.into()).ok();
    js_sys::Reflect::set(&js_state, &"theme".into(), &state.theme.into()).ok();

    let Some(document) = window.document() else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    if let Some(font_status) = body.get_attribute("data-vektra-preview-font-status") {
        js_sys::Reflect::set(&js_state, &"fontStatus".into(), &font_status.into()).ok();
    }
    if let Some(font_family) = body.get_attribute("data-vektra-preview-font-family") {
        js_sys::Reflect::set(&js_state, &"fontFamily".into(), &font_family.into()).ok();
    }

    set_window_preview_state(&js_state);

    body.set_attribute("data-vektra-preview-demo-id", state.demo_id)
        .ok();
    body.set_attribute("data-vektra-preview-status", state.status)
        .ok();
    body.set_attribute("data-vektra-preview-clicks", &state.clicks.to_string())
        .ok();
    body.set_attribute("data-vektra-preview-last-clicked", state.last_clicked)
        .ok();
    body.set_attribute("data-vektra-preview-theme", state.theme)
        .ok();
}

#[cfg(not(target_family = "wasm"))]
fn set_browser_state(state: PreviewBrowserState<'_>) {
    let _ = (
        state.demo_id,
        state.status,
        state.clicks,
        state.last_clicked,
        state.theme,
    );
}

#[cfg(target_family = "wasm")]
pub(crate) fn set_window_preview_state(state: &js_sys::Object) {
    use wasm_bindgen::JsCast as _;

    let descriptor = js_sys::Object::new();
    js_sys::Reflect::set(&descriptor, &"value".into(), state).ok();
    js_sys::Reflect::set(&descriptor, &"configurable".into(), &true.into()).ok();
    js_sys::Reflect::set(&descriptor, &"enumerable".into(), &false.into()).ok();
    js_sys::Reflect::set(&descriptor, &"writable".into(), &true.into()).ok();

    let Ok(global) = js_sys::global().dyn_into::<js_sys::Object>() else {
        return;
    };
    js_sys::Object::define_property(&global, &"__vektraPreviewState".into(), &descriptor);
}

#[cfg(test)]
#[path = "../tests/unit/demos.rs"]
mod tests;
