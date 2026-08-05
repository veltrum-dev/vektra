//! Vektra 内置图标名称与资源路径。
//!
//! 该 crate 只维护 Vektra 自带图标的稳定名称、资源相对路径和一致性测试。
//! 图标 SVG 的嵌入由 `vektra-assets` 负责，GPUI 渲染由 `vektra` 组件负责。

/// Vektra 内置图标名称。
///
/// 每个变体都对应 `assets/icons/*.svg` 下的一个内置 SVG 文件。应用自己的图标
/// 不应尝试扩展该 enum，而是用自己的类型实现 `IntoIconSource`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconName {
    /// 搜索图标，对应 `icons/search.svg`。
    Search,

    /// 设置图标，对应 `icons/settings.svg`。
    Settings,
}

impl IconName {
    /// 所有 Vektra 内置图标名称。
    pub const ALL: &'static [Self] = &[Self::Search, Self::Settings];

    /// 返回图标在 GPUI `AssetSource` 中使用的稳定相对路径。
    pub const fn path(self) -> &'static str {
        match self {
            Self::Search => "icons/search.svg",
            Self::Settings => "icons/settings.svg",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::BTreeSet,
        fs,
        path::{Path, PathBuf},
    };

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crates/icons 应位于 workspace 的 crates 目录下")
            .to_path_buf()
    }

    fn svg_paths() -> BTreeSet<String> {
        let icons_dir = workspace_root().join("assets/icons");
        fs::read_dir(&icons_dir)
            .unwrap_or_else(|error| panic!("无法读取图标目录 `{}`：{error}", icons_dir.display()))
            .map(|entry| {
                let entry = entry.expect("图标目录项应可读取");
                let file_type = entry.file_type().expect("图标目录项类型应可读取");
                assert!(file_type.is_file(), "图标目录中只应包含 SVG 文件");
                let file_name = entry
                    .file_name()
                    .into_string()
                    .expect("图标文件名应为 UTF-8");
                assert!(
                    file_name.ends_with(".svg"),
                    "图标文件必须使用 .svg 后缀：{file_name}"
                );
                format!("icons/{file_name}")
            })
            .collect()
    }

    #[test]
    fn every_icon_name_has_svg_asset() {
        let svg_paths = svg_paths();
        for icon in IconName::ALL {
            assert!(
                svg_paths.contains(icon.path()),
                "`{:?}` 缺少对应 SVG `{}`",
                icon,
                icon.path()
            );
        }
    }

    #[test]
    fn every_svg_asset_has_icon_name() {
        let known_paths = IconName::ALL
            .iter()
            .map(|icon| icon.path())
            .collect::<BTreeSet<_>>();
        for path in svg_paths() {
            assert!(
                known_paths.contains(path.as_str()),
                "`{path}` 缺少 IconName 变体"
            );
        }
    }
}
