# Assets and Icons

Vektra runtime assets live under the repository root `assets/` directory. `vektra-assets` provides a GPUI `AssetSource` and includes `themes/default/**/*` by default. With the `icons` feature enabled, it also includes the built-in SVG icons.

```toml
[dependencies]
vektra = { path = "crates/vektra" }
```

If your app does not provide custom assets, mount the Vektra assets directly:

```rust
application()
    .with_assets(vektra::assets::Assets)
    .run(|cx| {
        // ...
    });
```

## Custom Icons

You can use your app's own icon assets without enabling the `icons` feature. The host `AssetSource` only needs to serve the referenced path:

```rust
use vektra::{Button, Icon, IconButton, IconSource};

Icon::new(IconSource::asset("icons/logo.svg"));

Button::new("settings")
    .label("设置")
    .start_icon(IconSource::asset("icons/logo.svg"));

IconButton::new("settings", IconSource::asset("icons/logo.svg"))
    .aria_label("设置");
```

Use `Assets::with_overrides` when the application has its own asset source. App assets are checked first, and Vektra assets are used as fallback.

```rust
application()
    .with_assets(vektra::assets::Assets::with_overrides(AppAssets))
    .run(|cx| {
        // ...
    });
```

## Built-in Icons

Enable the `icons` feature to use `vektra::IconName`:

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

Applications can also derive `IntoIconSource` for their own enum. By default, `PascalCase` variants map to `icons/<snake_case>.svg`. Use `#[icon(path = "...")]` when a variant should point to a different file.

```rust
#[derive(Debug, Clone, Copy, vektra::IntoIconSource)]
enum AppIconName {
    Logo,
    FavoriteFilled,

    #[icon(path = "icons/heart.svg")]
    Favorite,
}
```

`examples/custom_assets` shows custom assets, default enum mapping, explicit path overrides, and fallback to `IconName::Settings`.
