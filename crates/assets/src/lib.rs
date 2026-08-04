//! Vektra 自带资源的 GPUI `AssetSource` 实现与组合工具。
//!
//! 该 crate 负责把 Vektra 框架资源嵌入二进制，并通过 GPUI 原生资源接口提供给
//! `Application::with_assets`。默认包含主题（含 Tooltip token）和 Button loading 指示器；启用 `icons`
//! feature 后才包含其余 Vektra 内置 SVG 图标。

use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use std::{borrow::Cow, collections::BTreeSet};

/// Vektra 默认资源集合。
///
/// 该资源源默认提供 `themes/default/**/*`（含 Checkbox、Radio、Switch 与 Tooltip token）和
/// Button 使用的 `components/button/loading.svg` 以及 Checkbox 默认状态图标。
/// 启用 `icons` feature 时，额外提供其他 `icons/**/*.svg` 内置图标。应用没有自定义
/// 资源时，可以把该类型直接传给 GPUI：
///
/// ```
/// let _assets = vektra_assets::Assets;
/// ```
pub struct Assets;

#[derive(RustEmbed)]
#[folder = "../../assets"]
#[include = "themes/default/**/*"]
#[include = "components/button/loading.svg"]
#[include = "components/checkbox/check.svg"]
#[include = "components/checkbox/heart-filled.svg"]
#[include = "components/checkbox/heart.svg"]
#[include = "components/checkbox/minus.svg"]
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
