use super::{DemoSelection, PreviewLang, parse_demo_query, parse_lang_query, parse_theme_query};
use vektra::ThemeMode;

#[test]
fn missing_demo_defaults_to_button_basic() {
    assert_eq!(parse_demo_query(""), DemoSelection::ButtonBasic);
    assert_eq!(parse_demo_query("?other=value"), DemoSelection::ButtonBasic);
}

#[test]
fn button_basic_is_selected_by_stable_id() {
    assert_eq!(
        parse_demo_query("?demo=button/basic"),
        DemoSelection::ButtonBasic
    );
    assert_eq!(
        parse_demo_query("?demo=button%2Fbasic"),
        DemoSelection::ButtonBasic
    );
    assert_eq!(
        parse_demo_query("?x=1&demo=button/basic&y=2"),
        DemoSelection::ButtonBasic
    );
}

#[test]
fn button_showcase_is_selected_by_stable_id() {
    assert_eq!(
        parse_demo_query("?demo=button/showcase"),
        DemoSelection::ButtonShowcase
    );
    assert_eq!(
        parse_demo_query("?demo=button%2Fshowcase"),
        DemoSelection::ButtonShowcase
    );
}

#[test]
fn checkbox_radio_switch_input_icon_button_and_tooltip_are_selected_by_stable_ids() {
    assert_eq!(
        parse_demo_query("?demo=radio/basic"),
        DemoSelection::RadioBasic
    );
    assert_eq!(
        parse_demo_query("?demo=checkbox/basic"),
        DemoSelection::CheckboxBasic
    );
    assert_eq!(
        parse_demo_query("?demo=switch/basic"),
        DemoSelection::SwitchBasic
    );
    assert_eq!(
        parse_demo_query("?demo=icon-button/basic"),
        DemoSelection::IconButtonBasic
    );
    assert_eq!(
        parse_demo_query("?demo=input/basic"),
        DemoSelection::InputBasic
    );
    assert_eq!(
        parse_demo_query("?demo=tooltip/basic"),
        DemoSelection::TooltipBasic
    );
}

#[test]
fn input_demo_covers_search_actions_variants_sizes_slots_clear_and_states() {
    let source = include_str!("../../src/demos/input.rs");

    for variant in ["Outline", "Filled", "Borderless", "Underline"] {
        assert!(source.contains(&format!("InputVariant::{variant}")));
    }
    for size in ["Xs", "Sm", "Md", "Lg"] {
        assert!(source.contains(&format!("ComponentSize::{size}")));
    }
    assert!(source.contains(".prefix("));
    assert!(source.contains(".suffix("));
    assert!(source.contains(".attached_suffix("));
    assert!(source.contains(".clearable("));
    assert!(source.contains(".invalid(true)"));
    assert!(source.contains(".read_only(true)"));
    assert!(source.contains(".disabled(true)"));
    assert!(source.contains("InputClear::new"));
    assert!(source.contains("Tooltip::new"));
    assert!(source.contains("Input::new(\"search-icon-only\""));
    assert!(source.contains("IconButton::new(\"submit-search-icon\", IconName::Search)"));
    assert!(source.contains("Input::new(\"search-text-only\""));
    assert!(source.contains("Button::new(\"submit-search-text\")"));
    assert!(source.contains("Input::new(\"search-icon-text\""));
    assert!(source.contains("\"search-attached-icon-text\""));
    assert!(source.contains("Button::new(\"submit-search-attached-icon-text\")"));
    assert!(source.contains("\"search-attached-icon-only\""));
    assert!(source.contains("IconButton::new(\"submit-search-attached-icon\", IconName::Search)"));
    assert!(source.contains(".start_icon(IconName::Search)"));
    assert!(source.contains("中文 IME"));

    let basic = source
        .split("// #region input-basic")
        .nth(1)
        .unwrap()
        .split("// #endregion input-basic")
        .next()
        .unwrap();
    assert!(basic.contains(".attached_suffix("));
    assert!(basic.contains("Button::new(\"input-submit-search\")"));
    assert!(basic.contains(".size(ComponentSize::Md)"));

    let text_search = source
        .split("Input::new(\"search-text-only\"")
        .nth(1)
        .unwrap()
        .split("Input::new(\"search-icon-text\"")
        .next()
        .unwrap();
    assert!(text_search.contains(".attached_suffix("));
    assert!(text_search.contains(".size(ComponentSize::Md)"));

    let icon_search = source
        .split("Input::new(\"search-icon-only\"")
        .nth(1)
        .unwrap()
        .split("Input::new(\"search-text-only\"")
        .next()
        .unwrap();
    assert!(icon_search.contains(".suffix("));
    assert!(!icon_search.contains(".attached_suffix("));

    let icon_text_search = source
        .split("Input::new(\"search-icon-text\"")
        .nth(1)
        .unwrap()
        .split("\"search-attached-icon-text\"")
        .next()
        .unwrap();
    assert!(icon_text_search.contains(".suffix("));
    assert!(!icon_text_search.contains(".attached_suffix("));

    let attached_icon_text_search = source
        .split("\"search-attached-icon-text\"")
        .nth(1)
        .unwrap()
        .split("\"search-attached-icon-only\"")
        .next()
        .unwrap();
    assert!(attached_icon_text_search.contains(".attached_suffix("));
    assert!(attached_icon_text_search.contains(".start_icon(IconName::Search)"));
    assert!(attached_icon_text_search.contains(".size(ComponentSize::Md)"));

    let attached_icon_search = source
        .split("\"search-attached-icon-only\"")
        .nth(1)
        .unwrap()
        .split("// #endregion input-search-actions")
        .next()
        .unwrap();
    assert!(attached_icon_search.contains(".attached_suffix("));
    assert!(attached_icon_search.contains("IconName::Search"));
    assert!(attached_icon_search.contains(".size(ComponentSize::Md)"));
    assert!(attached_icon_search.contains(".aria_label(search_label)"));
    assert!(attached_icon_search.contains(".tooltip(search_label)"));
}

#[test]
fn switch_demo_covers_compact_and_all_state_content_forms() {
    let source = include_str!("../../src/demos/switch.rs");

    assert!(source.contains("Switch::new(\"switch-notifications\")"));
    assert!(source.contains("SwitchContent::text"));
    assert!(source.contains("SwitchContent::icon("));
    assert!(source.contains("SwitchContent::icon_text"));
    assert!(source.contains(".disabled(true)"));
    assert!(source.contains(".loading(self.loading)"));
    assert!(source.contains(".transition_duration(Duration::from_millis(100))"));
    assert!(source.contains(".transition_duration(Duration::from_millis(400))"));
    assert!(source.contains(".transition_duration(Duration::ZERO)"));
    assert!(source.contains("switch-disabled-loading"));
    for size in ["Xs", "Sm", "Md", "Lg"] {
        assert!(source.contains(&format!("ComponentSize::{size}")));
    }
}

#[test]
fn radio_demo_uses_controlled_group_and_disabled_item() {
    let source = include_str!("../../src/demos/radio.rs");

    assert!(source.contains("RadioGroup::new(\"preview-plan-group\")"));
    assert!(source.contains(".selected_value(self.plan)"));
    assert!(source.contains(".on_change_in"));
    assert!(source.contains("Radio::new"));
    assert!(source.contains(".disabled(true)"));
}

#[test]
fn unknown_demo_is_preserved_for_error_state() {
    assert_eq!(
        parse_demo_query("?demo=unknown"),
        DemoSelection::Unknown("unknown".to_owned())
    );
    assert_eq!(
        parse_demo_query("?demo=%E6%9C%AA%E7%9F%A5"),
        DemoSelection::Unknown("未知".to_owned())
    );
}

#[test]
fn valid_theme_query_selects_light_or_dark() {
    assert_eq!(parse_theme_query("?theme=light"), ThemeMode::Light);
    assert_eq!(parse_theme_query("?theme=dark"), ThemeMode::Dark);
}

#[test]
fn missing_or_invalid_theme_query_uses_system() {
    assert_eq!(parse_theme_query(""), ThemeMode::System);
    assert_eq!(parse_theme_query("?other=value"), ThemeMode::System);
    assert_eq!(parse_theme_query("?theme=system"), ThemeMode::System);
    assert_eq!(
        parse_theme_query("?theme=%E6%9A%97%E8%89%B2"),
        ThemeMode::System
    );
}

#[test]
fn theme_query_order_does_not_affect_demo_query() {
    assert_eq!(
        parse_demo_query("?theme=dark&demo=button/basic"),
        DemoSelection::ButtonBasic
    );
    assert_eq!(
        parse_demo_query("?demo=button/basic&theme=light"),
        DemoSelection::ButtonBasic
    );
    assert_eq!(
        parse_theme_query("?demo=button/basic&theme=dark"),
        ThemeMode::Dark
    );
}

#[test]
fn valid_language_query_selects_preview_language() {
    assert_eq!(parse_lang_query("?lang=zh-CN"), PreviewLang::ZhCn);
    assert_eq!(parse_lang_query("?lang=en-US"), PreviewLang::EnUs);
}

#[test]
fn missing_or_invalid_language_query_uses_chinese() {
    assert_eq!(parse_lang_query(""), PreviewLang::ZhCn);
    assert_eq!(parse_lang_query("?other=value"), PreviewLang::ZhCn);
    assert_eq!(parse_lang_query("?lang=en"), PreviewLang::ZhCn);
}

#[test]
fn query_order_does_not_affect_language_or_theme() {
    let query = "?demo=button/showcase&lang=en-US&theme=dark";

    assert_eq!(parse_demo_query(query), DemoSelection::ButtonShowcase);
    assert_eq!(parse_lang_query(query), PreviewLang::EnUs);
    assert_eq!(parse_theme_query(query), ThemeMode::Dark);
}
