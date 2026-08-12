mod button;
mod checkbox;
mod icon_button;
mod input;
mod radio;
mod scrollbar;
mod select;
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
    ButtonStates,
    ButtonVariants,
    ButtonIcons,
    ButtonAutoSpace,
    ButtonWidth,
    ButtonShowcase,
    ButtonComprehensive,
    CheckboxBasic,
    CheckboxStates,
    CheckboxBulk,
    CheckboxIconOnly,
    CheckboxSizes,
    CheckboxComprehensive,
    RadioBasic,
    RadioDisabled,
    RadioKeyboard,
    RadioOrientation,
    RadioComprehensive,
    SelectBasic,
    SelectGroups,
    SelectStates,
    SelectKeyboard,
    SelectLongList,
    SwitchBasic,
    SwitchStates,
    SwitchFocus,
    SwitchLoading,
    SwitchSizes,
    SwitchContent,
    SwitchComprehensive,
    IconButtonBasic,
    IconButtonVariants,
    IconButtonSizes,
    IconButtonStates,
    IconButtonTooltip,
    IconButtonComprehensive,
    InputBasic,
    InputSearch,
    InputPassword,
    InputTypes,
    InputAffixes,
    InputGroup,
    InputVariants,
    InputSizes,
    InputStates,
    InputEvents,
    InputComprehensive,
    TooltipBasic,
    TooltipPlacements,
    TooltipControlled,
    TooltipAppearance,
    TooltipLifecycle,
    TooltipComprehensive,
    ScrollbarBasic,
    ScrollbarConfiguration,
    Unknown(String),
}

impl DemoSelection {
    pub(crate) const DEFAULT_ID: &'static str = "button/basic";
    pub(crate) const SHOWCASE_ID: &'static str = "button/showcase";
    #[cfg(test)]
    pub(crate) const DOCUMENTED_IDS: &'static [&'static str] = &[
        "button/basic",
        "button/states",
        "button/variants",
        "button/icons",
        "button/auto-space",
        "button/width",
        "checkbox/basic",
        "checkbox/states",
        "checkbox/bulk",
        "checkbox/icon-only",
        "checkbox/sizes",
        "radio/basic",
        "radio/disabled",
        "radio/keyboard",
        "radio/orientation",
        "select/basic",
        "select/groups",
        "select/states",
        "select/keyboard",
        "select/long-list",
        "switch/basic",
        "switch/states",
        "switch/focus",
        "switch/loading",
        "switch/sizes",
        "switch/content",
        "icon-button/basic",
        "icon-button/variants",
        "icon-button/sizes",
        "icon-button/states",
        "icon-button/tooltip",
        "input/basic",
        "input/search",
        "input/password",
        "input/types",
        "input/affixes",
        "input/group",
        "input/variants",
        "input/sizes",
        "input/states",
        "input/events",
        "tooltip/basic",
        "tooltip/placements",
        "tooltip/controlled",
        "tooltip/appearance",
        "tooltip/lifecycle",
        "scrollbar/basic",
        "scrollbar/configuration",
    ];
    pub(crate) const ALL_IDS: &'static [&'static str] = &[
        "button/basic",
        "button/states",
        "button/variants",
        "button/icons",
        "button/auto-space",
        "button/width",
        "button/showcase",
        "button/comprehensive",
        "checkbox/basic",
        "checkbox/states",
        "checkbox/bulk",
        "checkbox/icon-only",
        "checkbox/sizes",
        "checkbox/comprehensive",
        "radio/basic",
        "radio/disabled",
        "radio/keyboard",
        "radio/orientation",
        "radio/comprehensive",
        "select/basic",
        "select/groups",
        "select/states",
        "select/keyboard",
        "select/long-list",
        "switch/basic",
        "switch/states",
        "switch/focus",
        "switch/loading",
        "switch/sizes",
        "switch/content",
        "switch/comprehensive",
        "icon-button/basic",
        "icon-button/variants",
        "icon-button/sizes",
        "icon-button/states",
        "icon-button/tooltip",
        "icon-button/comprehensive",
        "input/basic",
        "input/search",
        "input/password",
        "input/types",
        "input/affixes",
        "input/group",
        "input/variants",
        "input/sizes",
        "input/states",
        "input/events",
        "input/comprehensive",
        "tooltip/basic",
        "tooltip/placements",
        "tooltip/controlled",
        "tooltip/appearance",
        "tooltip/lifecycle",
        "tooltip/comprehensive",
        "scrollbar/basic",
        "scrollbar/configuration",
    ];

    fn from_demo_id(demo_id: Option<&str>) -> Self {
        match demo_id {
            None => Self::ButtonBasic,
            Some(Self::DEFAULT_ID) => Self::ButtonBasic,
            Some("button/states") => Self::ButtonStates,
            Some("button/variants") => Self::ButtonVariants,
            Some("button/icons") => Self::ButtonIcons,
            Some("button/auto-space") => Self::ButtonAutoSpace,
            Some("button/width") => Self::ButtonWidth,
            Some(Self::SHOWCASE_ID) => Self::ButtonShowcase,
            Some("button/comprehensive") => Self::ButtonComprehensive,
            Some("checkbox/basic") => Self::CheckboxBasic,
            Some("checkbox/states") => Self::CheckboxStates,
            Some("checkbox/bulk") => Self::CheckboxBulk,
            Some("checkbox/icon-only") => Self::CheckboxIconOnly,
            Some("checkbox/sizes") => Self::CheckboxSizes,
            Some("checkbox/comprehensive") => Self::CheckboxComprehensive,
            Some("radio/basic") => Self::RadioBasic,
            Some("radio/disabled") => Self::RadioDisabled,
            Some("radio/keyboard") => Self::RadioKeyboard,
            Some("radio/orientation") => Self::RadioOrientation,
            Some("radio/comprehensive") => Self::RadioComprehensive,
            Some("select/basic") => Self::SelectBasic,
            Some("select/groups") => Self::SelectGroups,
            Some("select/states") => Self::SelectStates,
            Some("select/keyboard") => Self::SelectKeyboard,
            Some("select/long-list") => Self::SelectLongList,
            Some("switch/basic") => Self::SwitchBasic,
            Some("switch/states") => Self::SwitchStates,
            Some("switch/focus") => Self::SwitchFocus,
            Some("switch/loading") => Self::SwitchLoading,
            Some("switch/sizes") => Self::SwitchSizes,
            Some("switch/content") => Self::SwitchContent,
            Some("switch/comprehensive") => Self::SwitchComprehensive,
            Some("icon-button/basic") => Self::IconButtonBasic,
            Some("icon-button/variants") => Self::IconButtonVariants,
            Some("icon-button/sizes") => Self::IconButtonSizes,
            Some("icon-button/states") => Self::IconButtonStates,
            Some("icon-button/tooltip") => Self::IconButtonTooltip,
            Some("icon-button/comprehensive") => Self::IconButtonComprehensive,
            Some("input/basic") => Self::InputBasic,
            Some("input/search") => Self::InputSearch,
            Some("input/password") => Self::InputPassword,
            Some("input/types") => Self::InputTypes,
            Some("input/affixes") => Self::InputAffixes,
            Some("input/group") => Self::InputGroup,
            Some("input/variants") => Self::InputVariants,
            Some("input/sizes") => Self::InputSizes,
            Some("input/states") => Self::InputStates,
            Some("input/events") => Self::InputEvents,
            Some("input/comprehensive") => Self::InputComprehensive,
            Some("tooltip/basic") => Self::TooltipBasic,
            Some("tooltip/placements") => Self::TooltipPlacements,
            Some("tooltip/controlled") => Self::TooltipControlled,
            Some("tooltip/appearance") => Self::TooltipAppearance,
            Some("tooltip/lifecycle") => Self::TooltipLifecycle,
            Some("tooltip/comprehensive") => Self::TooltipComprehensive,
            Some("scrollbar/basic") => Self::ScrollbarBasic,
            Some("scrollbar/configuration") => Self::ScrollbarConfiguration,
            Some(value) => Self::Unknown(value.to_owned()),
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::ButtonBasic => Self::DEFAULT_ID,
            Self::ButtonStates => "button/states",
            Self::ButtonVariants => "button/variants",
            Self::ButtonIcons => "button/icons",
            Self::ButtonAutoSpace => "button/auto-space",
            Self::ButtonWidth => "button/width",
            Self::ButtonShowcase => Self::SHOWCASE_ID,
            Self::ButtonComprehensive => "button/comprehensive",
            Self::CheckboxBasic => "checkbox/basic",
            Self::CheckboxStates => "checkbox/states",
            Self::CheckboxBulk => "checkbox/bulk",
            Self::CheckboxIconOnly => "checkbox/icon-only",
            Self::CheckboxSizes => "checkbox/sizes",
            Self::CheckboxComprehensive => "checkbox/comprehensive",
            Self::RadioBasic => "radio/basic",
            Self::RadioDisabled => "radio/disabled",
            Self::RadioKeyboard => "radio/keyboard",
            Self::RadioOrientation => "radio/orientation",
            Self::RadioComprehensive => "radio/comprehensive",
            Self::SelectBasic => "select/basic",
            Self::SelectGroups => "select/groups",
            Self::SelectStates => "select/states",
            Self::SelectKeyboard => "select/keyboard",
            Self::SelectLongList => "select/long-list",
            Self::SwitchBasic => "switch/basic",
            Self::SwitchStates => "switch/states",
            Self::SwitchFocus => "switch/focus",
            Self::SwitchLoading => "switch/loading",
            Self::SwitchSizes => "switch/sizes",
            Self::SwitchContent => "switch/content",
            Self::SwitchComprehensive => "switch/comprehensive",
            Self::IconButtonBasic => "icon-button/basic",
            Self::IconButtonVariants => "icon-button/variants",
            Self::IconButtonSizes => "icon-button/sizes",
            Self::IconButtonStates => "icon-button/states",
            Self::IconButtonTooltip => "icon-button/tooltip",
            Self::IconButtonComprehensive => "icon-button/comprehensive",
            Self::InputBasic => "input/basic",
            Self::InputSearch => "input/search",
            Self::InputPassword => "input/password",
            Self::InputTypes => "input/types",
            Self::InputAffixes => "input/affixes",
            Self::InputGroup => "input/group",
            Self::InputVariants => "input/variants",
            Self::InputSizes => "input/sizes",
            Self::InputStates => "input/states",
            Self::InputEvents => "input/events",
            Self::InputComprehensive => "input/comprehensive",
            Self::TooltipBasic => "tooltip/basic",
            Self::TooltipPlacements => "tooltip/placements",
            Self::TooltipControlled => "tooltip/controlled",
            Self::TooltipAppearance => "tooltip/appearance",
            Self::TooltipLifecycle => "tooltip/lifecycle",
            Self::TooltipComprehensive => "tooltip/comprehensive",
            Self::ScrollbarBasic => "scrollbar/basic",
            Self::ScrollbarConfiguration => "scrollbar/configuration",
            Self::Unknown(value) => value,
        }
    }

    fn status(&self) -> &'static str {
        if matches!(self, Self::Unknown(_)) {
            "unknown-demo"
        } else {
            "ready"
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
        let separator = match self {
            Self::ZhCn => "、",
            Self::EnUs => ", ",
        };
        let supported = DemoSelection::ALL_IDS.join(separator);
        match self {
            Self::ZhCn => format!("不支持 demo_id `{demo_id}`。当前支持的预览：{supported}。"),
            Self::EnUs => {
                format!("Unsupported demo_id `{demo_id}`. Supported previews: {supported}.")
            }
        }
    }
}

pub(crate) struct PreviewApp {
    selection: DemoSelection,
    language: PreviewLang,
    font_family: &'static str,
    button_demo: button::ButtonDemo,
    checkbox_basic_demo: checkbox::CheckboxBasicDemo,
    checkbox_demo: checkbox::CheckboxDemo,
    radio_basic_demo: radio::RadioBasicDemo,
    radio_demo: radio::RadioDemo,
    select_demo: select::SelectDemo,
    switch_basic_demo: switch::SwitchBasicDemo,
    switch_demo: switch::SwitchDemo,
    input_basic_demo: input::InputBasicDemo,
    input_demo: input::InputDemo,
    scrollbar_demo: scrollbar::ScrollbarDemo,
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
            checkbox_basic_demo: checkbox::CheckboxBasicDemo::new(),
            checkbox_demo: checkbox::CheckboxDemo::new(),
            radio_basic_demo: radio::RadioBasicDemo::new(),
            radio_demo: radio::RadioDemo::new(),
            select_demo: select::SelectDemo::new(),
            switch_basic_demo: switch::SwitchBasicDemo::new(),
            switch_demo: switch::SwitchDemo::new(),
            input_basic_demo: input::InputBasicDemo::new(cx),
            input_demo: input::InputDemo::new(cx),
            scrollbar_demo: scrollbar::ScrollbarDemo::new(),
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
                .render_example_basic(self.language, window, cx)
                .into_any_element(),
            DemoSelection::ButtonStates => self
                .button_demo
                .render_example_states(self.language, window, cx)
                .into_any_element(),
            DemoSelection::ButtonVariants => self
                .button_demo
                .render_example_variants(self.language, window, cx)
                .into_any_element(),
            DemoSelection::ButtonIcons => self
                .button_demo
                .render_example_icons(self.language, window, cx)
                .into_any_element(),
            DemoSelection::ButtonAutoSpace => self
                .button_demo
                .render_example_auto_space(self.language, window, cx)
                .into_any_element(),
            DemoSelection::ButtonWidth => self
                .button_demo
                .render_example_width(self.language, window, cx)
                .into_any_element(),
            DemoSelection::ButtonShowcase => self
                .button_demo
                .render_showcase(self.language, window, cx)
                .into_any_element(),
            DemoSelection::ButtonComprehensive => self
                .button_demo
                .render_basic(self.language, focus_status, window, cx)
                .into_any_element(),
            DemoSelection::CheckboxBasic => self
                .checkbox_basic_demo
                .render(self.language, window, cx)
                .into_any_element(),
            DemoSelection::CheckboxStates => self
                .checkbox_demo
                .render_states(self.language, window, cx)
                .into_any_element(),
            DemoSelection::CheckboxBulk => self
                .checkbox_demo
                .render_bulk(self.language, window, cx)
                .into_any_element(),
            DemoSelection::CheckboxIconOnly => self
                .checkbox_demo
                .render_icon_only(self.language, window, cx)
                .into_any_element(),
            DemoSelection::CheckboxSizes => self
                .checkbox_demo
                .render_sizes(self.language, window, cx)
                .into_any_element(),
            DemoSelection::CheckboxComprehensive => self
                .checkbox_demo
                .render(self.language, focus_status, window, cx)
                .into_any_element(),
            DemoSelection::RadioBasic => self
                .radio_basic_demo
                .render(self.language, window, cx)
                .into_any_element(),
            DemoSelection::RadioDisabled => self
                .radio_demo
                .render_disabled(self.language, window, cx)
                .into_any_element(),
            DemoSelection::RadioKeyboard => self
                .radio_demo
                .render_keyboard(self.language, window, cx)
                .into_any_element(),
            DemoSelection::RadioOrientation => self
                .radio_demo
                .render_orientation(self.language, window, cx)
                .into_any_element(),
            DemoSelection::RadioComprehensive => self
                .radio_demo
                .render(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SelectBasic => self
                .select_demo
                .render_basic(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SelectGroups => self
                .select_demo
                .render_groups(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SelectStates => self
                .select_demo
                .render_states(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SelectKeyboard => self
                .select_demo
                .render_keyboard(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SelectLongList => self
                .select_demo
                .render_long_list(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SwitchBasic => self
                .switch_basic_demo
                .render(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SwitchStates => self
                .switch_demo
                .render_states(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SwitchFocus => self
                .switch_demo
                .render_focus(self.language, focus_status, window, cx)
                .into_any_element(),
            DemoSelection::SwitchLoading => self
                .switch_demo
                .render_loading(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SwitchSizes => self
                .switch_demo
                .render_sizes(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SwitchContent => self
                .switch_demo
                .render_content(self.language, window, cx)
                .into_any_element(),
            DemoSelection::SwitchComprehensive => self
                .switch_demo
                .render(self.language, focus_status, window, cx)
                .into_any_element(),
            DemoSelection::IconButtonBasic => {
                icon_button::render_basic(self.language, window, cx).into_any_element()
            }
            DemoSelection::IconButtonVariants => {
                icon_button::render_variants(self.language, window, cx).into_any_element()
            }
            DemoSelection::IconButtonSizes => {
                icon_button::render_sizes(self.language, window, cx).into_any_element()
            }
            DemoSelection::IconButtonStates => {
                icon_button::render_states(self.language, focus_status, window, cx)
                    .into_any_element()
            }
            DemoSelection::IconButtonTooltip => {
                icon_button::render_tooltip(self.language, window, cx).into_any_element()
            }
            DemoSelection::IconButtonComprehensive => {
                icon_button::render(self.language, focus_status, window, cx).into_any_element()
            }
            DemoSelection::InputBasic => self
                .input_basic_demo
                .render(self.language, window, cx)
                .into_any_element(),
            DemoSelection::InputSearch => self
                .input_demo
                .render_search(self.language, window, cx)
                .into_any_element(),
            DemoSelection::InputPassword => self
                .input_demo
                .render_password(self.language, window, cx)
                .into_any_element(),
            DemoSelection::InputTypes => self
                .input_demo
                .render_types(self.language, window, cx)
                .into_any_element(),
            DemoSelection::InputAffixes => self
                .input_demo
                .render_affixes(self.language, window, cx)
                .into_any_element(),
            DemoSelection::InputGroup => self
                .input_demo
                .render_group(self.language, window, cx)
                .into_any_element(),
            DemoSelection::InputVariants => self
                .input_demo
                .render_variants(self.language, window, cx)
                .into_any_element(),
            DemoSelection::InputSizes => self
                .input_demo
                .render_sizes(self.language, window, cx)
                .into_any_element(),
            DemoSelection::InputStates => self
                .input_demo
                .render_states(self.language, window, cx)
                .into_any_element(),
            DemoSelection::InputEvents => self
                .input_demo
                .render_events(self.language, window, cx)
                .into_any_element(),
            DemoSelection::InputComprehensive => self
                .input_demo
                .render(self.language, window, cx)
                .into_any_element(),
            DemoSelection::TooltipBasic => {
                tooltip::render_basic(self.language, window, cx).into_any_element()
            }
            DemoSelection::TooltipPlacements => {
                tooltip::render_placements(self.language, window, cx).into_any_element()
            }
            DemoSelection::TooltipControlled => {
                tooltip::render_controlled(self.language, window, cx).into_any_element()
            }
            DemoSelection::TooltipAppearance => {
                tooltip::render_appearance(self.language, window, cx).into_any_element()
            }
            DemoSelection::TooltipLifecycle => {
                tooltip::render_lifecycle(self.language, window, cx).into_any_element()
            }
            DemoSelection::TooltipComprehensive => {
                tooltip::render(self.language, window, cx).into_any_element()
            }
            DemoSelection::ScrollbarBasic => {
                scrollbar::render_basic(self.language, window, cx).into_any_element()
            }
            DemoSelection::ScrollbarConfiguration => self
                .scrollbar_demo
                .render_configuration(self.language, window, cx)
                .into_any_element(),
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
    Some(parse_demo_query(&query).id().to_owned())
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
