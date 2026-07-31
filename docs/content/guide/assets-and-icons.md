# 资源与图标

Vektra 的运行时资源统一位于仓库根 `assets/`。`vektra-assets` 提供 GPUI `AssetSource`，默认包含 `themes/default/**/*`；启用 `icons` feature 后会额外包含内置 SVG 图标。

```toml
[dependencies]
vektra = { path = "crates/vektra" }
```

应用没有自定义资源时，可以直接装配 Vektra 资源：

```rust
application()
    .with_assets(vektra::assets::Assets)
    .run(|cx| {
        // ...
    });
```

## 自定义图标

不启用 `icons` feature 也可以使用应用自己的图标资源，只要宿主 `AssetSource` 能加载对应路径：

```rust
use vektra::{Button, Icon, IconButton, IconSource};

Icon::new(IconSource::asset("icons/logo.svg"));

Button::new("settings")
    .label("设置")
    .start_icon(IconSource::asset("icons/logo.svg"));

IconButton::new("settings", IconSource::asset("icons/logo.svg"))
    .aria_label("设置");
```

如果应用有自己的资源源，用 `Assets::with_overrides` 合并。查询顺序是用户资源优先、Vektra 内置资源兜底，因此同名路径会被用户资源覆盖。

```rust
application()
    .with_assets(vektra::assets::Assets::with_overrides(AppAssets))
    .run(|cx| {
        // ...
    });
```

## 内置图标

启用 `icons` feature 后可以使用 `vektra::IconName`：

```toml
[dependencies]
vektra = { path = "crates/vektra", features = ["icons"] }
```

```rust
use vektra::{Button, Icon, IconButton, IconName};

Icon::new(IconName::Settings);

Button::new("settings")
    .label("设置")
    .start_icon(IconName::Settings);

IconButton::new("settings", IconName::Settings)
    .aria_label("设置");
```

也可以为应用 enum 派生 `IntoIconSource`，默认会把 `PascalCase` 变体转换为 `icons/<snake_case>.svg`。名称与文件不一致时使用 `#[icon(path = "...")]` 覆盖。

```rust
#[derive(Debug, Clone, Copy, vektra::IntoIconSource)]
enum AppIconName {
    Logo,
    FavoriteFilled,

    #[icon(path = "icons/heart.svg")]
    Favorite,
}
```

`examples/custom_assets` 展示了自定义资源、默认映射、显式路径覆盖和内置 `IconName::Settings` 回退。
