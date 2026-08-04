# Vektra

Vektra 是一套独立、易组合的纯 GPUI 组件库。它不是应用框架，也不需要 Vektra Root、Provider 或 `vektra::init(cx)`；应用仍按 GPUI 的 `Application`、`Window` 和 view 模型组织。

Vektra 当前处于早期开发阶段。GPUI 虽已发布 pre-1.0 版本，但 API 仍在快速演进，可能发生破坏性变更。Vektra 当前锁定特定 GPUI revision，尚未承诺稳定兼容 crates.io GPUI，因此暂不发布可供生产使用的正式 crate。请通过 Git workspace 或 path dependency 使用，并预期 Vektra 公共 API 也可能发生破坏性变更。

crates.io 上的 `vektra` 0.0.1 只用于保留项目名称，不包含当前组件库实现，不能作为正式依赖。当前真实组件 crate 位于 `crates/vektra`，仍标记为 `publish = false`。

## 文档

- 中文文档：<https://veltrum-dev.github.io/vektra/>
- English docs: <https://veltrum-dev.github.io/vektra/en/>
- 本地文档开发：[docs/README.md](docs/README.md)
- 资源与图标：[docs/content/guide/assets-and-icons.md](docs/content/guide/assets-and-icons.md)
- Button API：[docs/content/components/button.md](docs/content/components/button.md)
- Switch API：[docs/content/components/switch.md](docs/content/components/switch.md)
- IconButton API：[docs/content/components/icon-button.md](docs/content/components/icon-button.md)
- Tooltip API：[docs/content/components/tooltip.md](docs/content/components/tooltip.md)

## 许可证

Vektra 使用 [MIT License](LICENSE)。

## 最小示例

```rust
use vektra::{Button, IconButton, IconSource, TooltipPlacement};

Button::new("save")
    .label("保存")
    .tooltip("保存当前修改")
    .tooltip_placement(TooltipPlacement::TopStart)
    .aria_description("保存当前修改")
    .on_click(|_, _, _| {
        // 鼠标、Enter 和 Space 激活共享这个回调契约。
    });

Button::new("settings")
    .label("设置")
    .start_icon(IconSource::asset("icons/settings.svg"));

IconButton::new("settings", IconSource::asset("icons/settings.svg"))
    .aria_label("设置")
    .tooltip("设置");
```

启用内置图标：

```toml
vektra = { path = "crates/vektra", features = ["icons"] }
```

## 示例

```bash
cargo run --example button
cargo run --example checkbox
cargo run --example switch
cargo run --example icon_button
cargo run --example custom_assets
cargo run --example tooltip
```

## 常用开发命令

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

文档：

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
cd docs
bun install --frozen-lockfile
bun run dev
VEKTRA_DOCS_BASE=/vektra/ bun run build
```

## Workspace

```text
.
├── assets/              # 默认主题与可选内置图标资源
├── crates/
│   ├── assets/          # vektra-assets
│   ├── icons/           # vektra-icons
│   ├── theme/           # vektra-theme
│   ├── vektra/          # 组件门面 crate
│   └── vektra-macros/   # 派生宏
├── docs/                # VitePress 文档站与 GPUI WASM preview
└── examples/            # 桌面示例
```

GPUI 锁定到 Zed revision `82aef44308540b576e4e51fb379efa71614e5c91`。仓库不使用 crates.io 的浮动 GPUI，也不使用 `branch = "main"`。
