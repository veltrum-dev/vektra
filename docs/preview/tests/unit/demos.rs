use super::{DemoSelection, PreviewLang, parse_demo_query, parse_lang_query, parse_theme_query};
use std::{collections::BTreeSet, fs, path::PathBuf};
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
fn every_registered_demo_id_round_trips_through_query_parsing() {
    for demo_id in DemoSelection::ALL_IDS {
        let selection = parse_demo_query(&format!("?demo={demo_id}"));
        assert_eq!(selection.id(), *demo_id, "demo ID 未正确注册：{demo_id}");
        assert!(!matches!(selection, DemoSelection::Unknown(_)));
    }
}

#[test]
fn input_demo_covers_types_password_security_events_and_existing_composition() {
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

    let search = source
        .split("// #region input-example-search")
        .nth(1)
        .unwrap()
        .split("// #endregion input-example-search")
        .next()
        .unwrap();
    assert!(search.contains(".input_type(InputType::Search)"));
    assert!(search.contains(".prefix(Icon::new(IconName::Search))"));
    assert!(search.contains(".clearable("));
    assert!(search.contains(".on_submit_in("));

    let password = source
        .split("// #region input-example-password")
        .nth(1)
        .unwrap()
        .split("// #endregion input-example-password")
        .next()
        .unwrap();
    assert!(password.contains(".input_type(InputType::Password)"));
    assert!(password.contains(".password_revealed(revealed)"));
    assert!(password.contains("IconName::Eye"));
    assert!(password.contains("IconName::EyeOff"));
    assert!(password.contains(".selected(revealed)"));
    assert!(password.contains(".aria_label(action_label)"));
    assert!(password.contains(".tooltip(action_label)"));
    assert!(password.contains("window.focus(&focus, cx)"));

    let types = source
        .split("// #region input-example-types")
        .nth(1)
        .unwrap()
        .split("// #endregion input-example-types")
        .next()
        .unwrap();
    for input_type in ["Email", "Phone", "Url"] {
        assert!(types.contains(&format!(".input_type(InputType::{input_type})")));
    }

    let events = source
        .split("// #region input-example-events")
        .nth(1)
        .unwrap()
        .split("// #endregion input-example-events")
        .next()
        .unwrap();
    for callback in ["on_change_in", "on_submit_in", "on_focus_in", "on_blur_in"] {
        assert!(events.contains(&format!(".{callback}(")));
    }

    let basic = source
        .split("// #region input-example-basic")
        .nth(1)
        .unwrap()
        .split("// #endregion input-example-basic")
        .next()
        .unwrap();
    assert!(basic.contains("Entity<InputState>"));
    assert!(basic.contains("InputState::new(\"\", cx)"));
    assert!(basic.contains("Input::new(\"name-input\""));
    assert!(!basic.contains(".prefix("));
    assert!(!basic.contains(".suffix("));
    assert!(!basic.contains(".attached_suffix("));
    assert!(!basic.contains(".clearable("));
    assert!(!basic.contains(".invalid("));

    let group = source
        .split("// #region input-example-group")
        .nth(1)
        .unwrap()
        .split("// #endregion input-example-group")
        .next()
        .unwrap();
    assert!(group.contains(".attached_suffix("));
    assert!(group.contains("Button::new(\"search-button\")"));
    assert!(group.contains(".size(ComponentSize::Md)"));

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
fn controlled_basic_examples_keep_only_their_required_state() {
    let checkbox = include_str!("../../src/demos/checkbox.rs")
        .split("// #region checkbox-example-basic")
        .nth(1)
        .unwrap()
        .split("// #endregion checkbox-example-basic")
        .next()
        .unwrap();
    assert_eq!(checkbox.matches("checked: bool").count(), 1);
    assert!(!checkbox.contains("indeterminate"));
    assert!(!checkbox.contains("batch_"));
    assert!(!checkbox.contains("IconSource"));

    let switch = include_str!("../../src/demos/switch.rs")
        .split("// #region switch-example-basic")
        .nth(1)
        .unwrap()
        .split("// #endregion switch-example-basic")
        .next()
        .unwrap();
    assert_eq!(switch.matches("checked: bool").count(), 1);
    assert!(!switch.contains("loading"));
    assert!(!switch.contains("SwitchContent"));

    let radio = include_str!("../../src/demos/radio.rs")
        .split("// #region radio-example-basic")
        .nth(1)
        .unwrap()
        .split("// #endregion radio-example-basic")
        .next()
        .unwrap();
    assert_eq!(radio.matches("Option<BasicPlan>").count(), 1);
    assert!(!radio.contains("pending"));
    assert!(!radio.contains("disabled"));
}

#[test]
fn focused_examples_cover_component_states_keyboard_sizes_and_tooltips() {
    let checkbox = include_str!("../../src/demos/checkbox.rs");
    for region in ["checkbox-example-states", "checkbox-example-sizes"] {
        assert!(checkbox.contains(&format!("// #region {region}")));
        assert!(checkbox.contains(&format!("// #endregion {region}")));
    }
    assert!(checkbox.contains(".indeterminate("));
    assert!(checkbox.contains(".disabled(true)"));

    let radio = include_str!("../../src/demos/radio.rs");
    for region in [
        "radio-example-disabled",
        "radio-example-keyboard",
        "radio-example-orientation",
    ] {
        assert!(radio.contains(&format!("// #region {region}")));
        assert!(radio.contains(&format!("// #endregion {region}")));
    }
    assert!(radio.contains(".orientation(Orientation::Horizontal)"));
    assert!(radio.contains(".disabled(true)"));

    let switch = include_str!("../../src/demos/switch.rs");
    for region in [
        "switch-example-states",
        "switch-example-sizes",
        "switch-example-content",
    ] {
        assert!(switch.contains(&format!("// #region {region}")));
        assert!(switch.contains(&format!("// #endregion {region}")));
    }
    assert!(switch.contains("SwitchContent::text"));
    assert!(switch.contains("SwitchContent::icon("));
    assert!(switch.contains("SwitchContent::icon_text("));

    let icon_button = include_str!("../../src/demos/icon_button.rs");
    assert!(icon_button.contains("// #region icon-button-example-states"));
    assert!(icon_button.contains(".selected(true)"));
    assert!(icon_button.contains("// #region icon-button-example-tooltip"));
    assert!(icon_button.contains(".aria_label(label)"));
    assert!(icon_button.contains(".tooltip(label)"));

    let tooltip = include_str!("../../src/demos/tooltip.rs");
    assert!(tooltip.contains("// #region tooltip-example-appearance"));
    assert!(tooltip.contains(".arrow(false)"));
    assert!(tooltip.contains(".bg_color("));
    assert!(tooltip.contains("// #region tooltip-example-lifecycle"));
    assert!(tooltip.contains("Escape dismisses without moving focus"));
}

#[test]
fn component_pages_pair_every_preview_with_compiled_source_and_registered_id() {
    let docs_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../content");
    let components = [
        "button",
        "checkbox",
        "radio",
        "switch",
        "icon-button",
        "input",
        "tooltip",
    ];
    let mut documented_ids = BTreeSet::new();

    for component in components {
        let chinese = fs::read_to_string(docs_root.join(format!("components/{component}.md")))
            .expect("中文组件文档必须可读取");
        let english = fs::read_to_string(docs_root.join(format!("en/components/{component}.md")))
            .expect("英文组件文档必须可读取");

        for source in [&chinese, &english] {
            assert!(!source.contains("<VektraPreview"));
            assert!(!source.contains("/comprehensive\""));
            assert_eq!(
                source.matches("<VektraExample").count(),
                source.matches("</VektraExample>").count()
            );
            assert_eq!(
                source.matches("<VektraExample").count(),
                source.matches("<<<").count()
            );
        }

        let chinese_ids = example_ids(&chinese);
        let english_ids = example_ids(&english);
        assert_eq!(chinese_ids, english_ids, "{component} 中英文示例必须一致");
        assert_eq!(
            chinese_ids.first().map(String::as_str),
            Some(format!("{component}/basic").as_str()),
            "{component} 的第一个示例必须是 Basic"
        );
        documented_ids.extend(chinese_ids);
    }

    let expected = DemoSelection::DOCUMENTED_IDS
        .iter()
        .map(|id| (*id).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(documented_ids, expected);

    for demo_id in documented_ids {
        assert_eq!(DemoSelection::from_demo_id(Some(&demo_id)).id(), demo_id);
    }
}

#[test]
fn getting_started_pages_use_public_git_dependencies_and_platform_application() {
    let docs_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../content");

    for relative_path in ["guide/getting-started.md", "en/guide/getting-started.md"] {
        let source = fs::read_to_string(docs_root.join(relative_path))
            .expect("中英文快速开始文档必须可读取");

        for dependency in [
            "gpui = { git = \"https://github.com/zed-industries/zed\", rev = \"82aef44308540b576e4e51fb379efa71614e5c91\" }",
            "vektra = { git = \"https://github.com/veltrum-dev/vektra.git\" }",
            "[target.'cfg(target_os = \"macos\")'.dependencies]",
            "features = [\"font-kit\"]",
            "[target.'cfg(any(target_os = \"linux\", target_os = \"freebsd\"))'.dependencies]",
            "features = [\"wayland\", \"x11\"]",
            "[target.'cfg(target_os = \"windows\")'.dependencies]",
        ] {
            assert!(
                source.contains(dependency),
                "{relative_path} 必须包含可直接使用的 Git 依赖：{dependency}"
            );
        }

        assert!(source.contains("gpui_platform::application()"));
        assert_eq!(source.matches("gpui_platform = { git =").count(), 3);
        assert!(source.contains("ParentElement"));
        assert!(!source.contains("gpui::Application::new()"));
        assert!(!source.contains("workspace = true"));
        assert!(!source.contains("vektra = { path ="));
    }
}

fn example_ids(source: &str) -> Vec<String> {
    source
        .match_indices("<VektraExample demo=\"")
        .map(|(index, marker)| {
            let remainder = &source[index + marker.len()..];
            remainder
                .split_once('"')
                .expect("VektraExample 必须提供闭合的 demo 属性")
                .0
                .to_owned()
        })
        .collect()
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
