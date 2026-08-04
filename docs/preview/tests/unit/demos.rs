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
fn checkbox_radio_switch_icon_button_and_tooltip_are_selected_by_stable_ids() {
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
        parse_demo_query("?demo=tooltip/basic"),
        DemoSelection::TooltipBasic
    );
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
