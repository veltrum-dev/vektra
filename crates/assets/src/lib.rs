//! Vektra 自带资源的 GPUI `AssetSource` 实现与组合工具。
//!
//! 该 crate 负责把 Vektra 框架资源嵌入二进制，并通过 GPUI 原生资源接口提供给
//! `Application::with_assets`。默认包含主题等非图标资源；启用 `icons` feature 后
//! 才包含 Vektra 内置 SVG 图标。

use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use std::{borrow::Cow, collections::BTreeSet};

/// Vektra 默认资源集合。
///
/// 该资源源默认提供 `themes/default/**/*`。启用 `icons` feature 时，额外提供
/// `icons/**/*.svg` 内置图标。应用没有自定义资源时，可以把该类型直接传给 GPUI：
///
/// ```
/// let _assets = vektra_assets::Assets;
/// ```
pub struct Assets;

#[derive(RustEmbed)]
#[folder = "../../assets"]
#[include = "themes/default/**/*"]
struct CoreAssets;

#[cfg(feature = "icons")]
#[derive(RustEmbed)]
#[folder = "../../assets"]
#[include = "icons/**/*.svg"]
struct IconAssets;

impl Assets {
    /// 用应用资源覆盖或扩展 Vektra 内置资源。
    ///
    /// 组合后的资源源会先查询 `overrides`，命中时直接返回用户资源；用户资源缺失时
    /// 再回退到 Vektra 内置资源。该顺序允许应用添加新路径，也允许用同名路径明确
    /// 覆盖 Vektra 内置资源。
    pub fn with_overrides<A>(overrides: A) -> AssetsWithOverrides<A>
    where
        A: AssetSource,
    {
        AssetsWithOverrides { overrides }
    }

    /// 从 Vektra 内置资源中读取 UTF-8 文本。
    ///
    /// 返回 `Ok(None)` 表示资源路径不存在；资源存在但不是合法 UTF-8 时返回错误。
    /// 该方法只读取 Vektra 内置资源，不读取应用 override，适合加载框架默认主题。
    pub fn load_text(path: &str) -> Result<Option<Cow<'static, str>>> {
        let Some(bytes) = Assets.load(path)? else {
            return Ok(None);
        };

        match bytes {
            Cow::Borrowed(bytes) => Ok(Some(Cow::Borrowed(std::str::from_utf8(bytes)?))),
            Cow::Owned(bytes) => Ok(Some(Cow::Owned(String::from_utf8(bytes)?))),
        }
    }
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(file) = CoreAssets::get(path) {
            return Ok(Some(file.data));
        }

        #[cfg(feature = "icons")]
        if let Some(file) = IconAssets::get(path) {
            return Ok(Some(file.data));
        }

        Ok(None)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = BTreeSet::new();
        paths.extend(
            CoreAssets::iter()
                .filter(|asset_path| asset_path.starts_with(path))
                .map(|path| path.to_string()),
        );

        #[cfg(feature = "icons")]
        paths.extend(
            IconAssets::iter()
                .filter(|asset_path| asset_path.starts_with(path))
                .map(|path| path.to_string()),
        );

        Ok(paths.into_iter().map(Into::into).collect())
    }
}

/// 用户资源与 Vektra 内置资源的组合 `AssetSource`。
///
/// GPUI 当前一次只接受一个 `AssetSource`。该类型让应用保留自己的资源源，同时复用
/// Vektra 默认主题和可选内置图标。加载顺序固定为用户资源优先、Vektra 资源兜底。
pub struct AssetsWithOverrides<A>
where
    A: AssetSource,
{
    overrides: A,
}

impl<A> AssetSource for AssetsWithOverrides<A>
where
    A: AssetSource,
{
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = self.overrides.load(path)? {
            return Ok(Some(bytes));
        }

        Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = BTreeSet::new();
        paths.extend(
            self.overrides
                .list(path)?
                .into_iter()
                .map(|path| path.to_string()),
        );
        paths.extend(Assets.list(path)?.into_iter().map(|path| path.to_string()));
        Ok(paths.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, io};

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
    fn loads_default_theme_text() {
        let text = Assets::load_text("themes/default/foundation.json")
            .unwrap()
            .unwrap();
        assert!(text.contains("foundation"));
        assert!(text.contains("icon"));
    }

    #[test]
    fn lists_default_theme_path() {
        let assets = Assets.list("themes/default").unwrap();
        assert!(
            assets
                .iter()
                .any(|path| path.as_ref() == "themes/default/foundation.json")
        );
    }

    #[test]
    fn missing_asset_returns_none() {
        assert!(Assets.load("icons/missing.svg").unwrap().is_none());
    }

    #[test]
    fn missing_override_and_builtin_returns_none() {
        let assets = Assets::with_overrides(TestAssets::default());
        assert!(assets.load("icons/missing.svg").unwrap().is_none());
    }

    #[test]
    fn override_unique_asset_can_load() {
        let assets = Assets::with_overrides(
            TestAssets::default().with_asset("icons/custom.svg", b"<svg></svg>"),
        );
        let bytes = assets.load("icons/custom.svg").unwrap().unwrap();
        assert_eq!(bytes.as_ref(), b"<svg></svg>");
    }

    #[test]
    fn missing_override_falls_back_to_vektra_asset() {
        let assets = Assets::with_overrides(TestAssets::default());
        let bytes = assets
            .load("themes/default/foundation.json")
            .unwrap()
            .unwrap();
        let text = std::str::from_utf8(bytes.as_ref()).unwrap();
        assert!(text.contains("foundation"));
        assert!(text.contains("icon"));
    }

    #[test]
    fn override_wins_for_same_path() {
        let assets = Assets::with_overrides(
            TestAssets::default().with_asset("themes/default/foundation.json", b"override"),
        );
        let bytes = assets
            .load("themes/default/foundation.json")
            .unwrap()
            .unwrap();
        assert_eq!(bytes.as_ref(), b"override");
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
                "themes/default/custom.json",
                "themes/default/dark.json",
                "themes/default/foundation.json",
                "themes/default/light.json",
            ]
        );
    }

    #[test]
    fn override_errors_are_propagated() {
        let assets = Assets::with_overrides(TestAssets {
            fail_load: true,
            ..Default::default()
        });
        assert!(assets.load("themes/default/foundation.json").is_err());

        let assets = Assets::with_overrides(TestAssets {
            fail_list: true,
            ..Default::default()
        });
        assert!(assets.list("themes/default").is_err());
    }

    #[cfg(feature = "icons")]
    #[test]
    fn icons_feature_embeds_settings_icon() {
        let assets = Assets::with_overrides(TestAssets::default());
        assert!(
            assets
                .overrides
                .load("icons/settings.svg")
                .unwrap()
                .is_none()
        );

        let bytes = assets
            .load(vektra_icons::IconName::Settings.path())
            .unwrap()
            .unwrap();
        let svg = std::str::from_utf8(&bytes).unwrap();
        assert!(svg.contains("viewBox=\"0 0 16 16\""));
        assert!(svg.contains("stroke-width=\"1.2\""));
        assert!(svg.contains("currentColor"));
    }

    #[cfg(not(feature = "icons"))]
    #[test]
    fn default_build_does_not_embed_icons() {
        assert!(Assets.load("icons/settings.svg").unwrap().is_none());
        assert!(Assets.list("icons").unwrap().is_empty());
    }
}
