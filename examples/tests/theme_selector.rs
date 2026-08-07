use std::{fs, path::PathBuf};

#[test]
fn every_desktop_example_exposes_the_shared_theme_selector() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for example in [
        "button",
        "checkbox",
        "switch",
        "radio",
        "custom_assets",
        "icon_button",
        "tooltip",
        "input",
        "scrollbar",
    ] {
        let source = fs::read_to_string(root.join(example).join("src/main.rs"))
            .unwrap_or_else(|error| panic!("无法读取 {example} example：{error}"));
        assert!(
            source.contains("mod shared;"),
            "{example} 必须导入共享 example 工具"
        );
        assert!(
            source.contains("shared::theme_selector("),
            "{example} 必须渲染主题选择器"
        );
    }
}

#[test]
fn shared_theme_selector_covers_system_light_and_dark() {
    let source = include_str!("../shared.rs");
    for mode in ["ThemeMode::System", "ThemeMode::Light", "ThemeMode::Dark"] {
        assert!(source.contains(mode), "共享主题选择器缺少 {mode}");
    }
    assert!(source.contains("resolved_theme_mode(window, cx)"));
}
