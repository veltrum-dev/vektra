use gpui::{AssetSource, Result, SharedString};
use std::{borrow::Cow, collections::BTreeMap, io};
use vektra_assets::Assets;

#[derive(Default)]
struct TestAssets {
    assets: BTreeMap<&'static str, Cow<'static, [u8]>>,
    fail_load: bool,
    fail_list: bool,
}

impl TestAssets {
    fn with_asset(mut self, path: &'static str, bytes: &'static [u8]) -> Self {
        self.assets.insert(path, Cow::Borrowed(bytes));
        self
    }
}

impl AssetSource for TestAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if self.fail_load {
            return Err(io::Error::other("load failed").into());
        }
        Ok(self.assets.get(path).cloned())
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        if self.fail_list {
            return Err(io::Error::other("list failed").into());
        }
        Ok(self
            .assets
            .keys()
            .filter(|asset_path| asset_path.starts_with(path))
            .map(|path| (*path).into())
            .collect())
    }
}

#[test]
fn loads_and_lists_default_theme_resources() {
    let text = Assets::load_text("themes/default/foundation.json")
        .unwrap()
        .unwrap();
    assert!(text.contains("foundation"));
    assert!(
        Assets::load_text("themes/default/tooltip.json")
            .unwrap()
            .unwrap()
            .contains("tooltip")
    );
    assert!(
        Assets::load_text("themes/default/checkbox.json")
            .unwrap()
            .unwrap()
            .contains("checkbox")
    );
    assert!(
        Assets::load_text("themes/default/switch.json")
            .unwrap()
            .unwrap()
            .contains("switch")
    );
    assert!(
        Assets::load_text("themes/default/scrollbar.json")
            .unwrap()
            .unwrap()
            .contains("scrollbar")
    );
    assert!(
        Assets::load_text("themes/default/input.json")
            .unwrap()
            .unwrap()
            .contains("input")
    );
    assert!(
        Assets::load_text("themes/default/select.json")
            .unwrap()
            .unwrap()
            .contains("select")
    );
    assert!(
        Assets
            .list("themes/default")
            .unwrap()
            .iter()
            .any(|path| path.as_ref() == "themes/default/foundation.json")
    );
}

#[test]
fn input_icons_are_core_resources() {
    for path in ["components/input/clear.svg", "components/input/invalid.svg"] {
        let bytes = Assets.load(path).unwrap().unwrap();
        let svg = std::str::from_utf8(bytes.as_ref()).unwrap();
        assert!(svg.contains("viewBox=\"0 0 16 16\""));
        assert!(svg.contains("currentColor"));
    }
}

#[test]
fn loading_indicator_is_a_core_resource() {
    let bytes = Assets
        .load("components/button/loading.svg")
        .unwrap()
        .unwrap();
    let svg = std::str::from_utf8(bytes.as_ref()).unwrap();
    assert!(svg.contains("viewBox=\"0 0 16 16\""));
    assert!(svg.contains("currentColor"));
    assert!(
        Assets
            .list("components/button")
            .unwrap()
            .iter()
            .any(|path| path.as_ref() == "components/button/loading.svg")
    );
}

#[test]
fn checkbox_icons_are_core_resources() {
    for path in [
        "components/checkbox/check.svg",
        "components/checkbox/heart-filled.svg",
        "components/checkbox/heart.svg",
        "components/checkbox/minus.svg",
    ] {
        let bytes = Assets.load(path).unwrap().unwrap();
        let svg = std::str::from_utf8(bytes.as_ref()).unwrap();
        assert!(svg.contains("viewBox=\"0 0 16 16\""));
        assert!(svg.contains("currentColor"));
    }
    assert!(
        Assets
            .list("components/checkbox")
            .unwrap()
            .iter()
            .any(|path| path.as_ref() == "components/checkbox/check.svg")
    );
}

#[test]
fn select_indicator_is_a_core_resource() {
    for path in [
        "components/select/chevron-down.svg",
        "components/select/chevron-up.svg",
    ] {
        let bytes = Assets.load(path).unwrap().unwrap();
        let svg = std::str::from_utf8(bytes.as_ref()).unwrap();
        assert!(svg.contains("viewBox=\"0 0 16 16\""));
        assert!(svg.contains("currentColor"));
    }
    assert!(
        Assets
            .list("components/select")
            .unwrap()
            .iter()
            .any(|path| path.as_ref() == "components/select/chevron-down.svg")
    );
    assert!(
        Assets
            .list("components/select")
            .unwrap()
            .iter()
            .any(|path| path.as_ref() == "components/select/chevron-up.svg")
    );
}

#[test]
fn overrides_win_and_missing_entries_fall_back() {
    let assets = Assets::with_overrides(
        TestAssets::default().with_asset("components/button/loading.svg", b"override"),
    );
    assert_eq!(
        assets
            .load("components/button/loading.svg")
            .unwrap()
            .unwrap()
            .as_ref(),
        b"override"
    );

    let assets = Assets::with_overrides(TestAssets::default());
    assert!(
        assets
            .load("components/button/loading.svg")
            .unwrap()
            .is_some()
    );
    assert!(assets.load("icons/missing.svg").unwrap().is_none());
}

#[test]
fn list_merges_deduplicates_and_sorts() {
    let assets = Assets::with_overrides(
        TestAssets::default()
            .with_asset("themes/default/foundation.json", b"override")
            .with_asset("themes/default/custom.json", b"custom"),
    );
    let paths = assets
        .list("themes/default")
        .unwrap()
        .into_iter()
        .map(|path| path.to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        paths,
        vec![
            "themes/default/button.json",
            "themes/default/checkbox.json",
            "themes/default/custom.json",
            "themes/default/dark.json",
            "themes/default/foundation.json",
            "themes/default/input.json",
            "themes/default/light.json",
            "themes/default/radio.json",
            "themes/default/scrollbar.json",
            "themes/default/select.json",
            "themes/default/switch.json",
            "themes/default/tooltip.json",
        ]
    );
}

#[test]
fn override_errors_are_propagated() {
    let assets = Assets::with_overrides(TestAssets {
        fail_load: true,
        ..Default::default()
    });
    assert!(assets.load("components/button/loading.svg").is_err());

    let assets = Assets::with_overrides(TestAssets {
        fail_list: true,
        ..Default::default()
    });
    assert!(assets.list("icons").is_err());
}

#[cfg(feature = "icons")]
#[test]
fn icons_feature_embeds_all_typed_icons() {
    for icon in vektra_icons::IconName::ALL {
        let bytes = Assets
            .load(icon.path())
            .unwrap()
            .unwrap_or_else(|| panic!("{:?} 未嵌入资源 {}", icon, icon.path()));
        let svg = std::str::from_utf8(bytes.as_ref()).unwrap();
        assert!(svg.contains("currentColor"), "{:?}", icon);
    }
}

#[cfg(not(feature = "icons"))]
#[test]
fn default_build_embeds_all_core_component_resources() {
    assert!(
        Assets
            .load("components/button/loading.svg")
            .unwrap()
            .is_some()
    );
    assert!(Assets.load("components/input/clear.svg").unwrap().is_some());
    assert!(
        Assets
            .load("components/checkbox/check.svg")
            .unwrap()
            .is_some()
    );
    assert!(
        Assets
            .load("components/select/chevron-down.svg")
            .unwrap()
            .is_some()
    );
    assert!(
        Assets
            .load("components/select/chevron-up.svg")
            .unwrap()
            .is_some()
    );
    assert!(Assets.load("icons/search.svg").unwrap().is_none());
    assert!(Assets.load("icons/settings.svg").unwrap().is_none());
}
