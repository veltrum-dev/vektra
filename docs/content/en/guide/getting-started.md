# Quick Start

Vektra is a component library for [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui).

::: warning Pre-release status
Vektra is in early development. GPUI has published pre-1.0 releases, but its API is still moving quickly and may introduce breaking changes. Vektra currently pins a specific GPUI revision and has not committed to stable compatibility with crates.io GPUI, so there is no production-ready Vektra crate release yet. Use a Git workspace or path dependency, and expect Vektra public APIs to change in breaking ways.

The `vektra` 0.0.1 package on crates.io only reserves the project name. It does not contain the current component library implementation and must not be used as the formal dependency. The current real component crate lives in `crates/vektra` and remains marked `publish = false`.
:::

## Requirements

- The Rust workspace uses edition 2024 and `rust-version = "1.95"`.
- GPUI is pinned in the root `Cargo.toml` to Zed revision `82aef44308540b576e4e51fb379efa71614e5c91`.
- The documentation preview build needs the `wasm32-unknown-unknown` target and Trunk `0.21.14`.
- The documentation site uses Bun for frontend dependencies.

## Use Vektra in the same workspace

If your application package is in this workspace, inherit the workspace dependencies:

```toml
[dependencies]
gpui = { workspace = true }
vektra = { workspace = true }
```

For a local project outside this repository, point a path dependency at the Vektra crate:

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "82aef44308540b576e4e51fb379efa71614e5c91" }
vektra = { path = "../vektra/crates/vektra" }
```

## Minimal Example

Vektra components are plain GPUI elements. Your app still creates windows and views through GPUI. Vektra provides components, themes, and assets.

```rust
use gpui::{div, App, AppContext, IntoElement, Render, Window};
use vektra::Button;

struct Demo;

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div().child(
            Button::new("save")
                .label("保存")
                .on_click(|_, _, _| {
                    // Handle activation.
                }),
        )
    }
}

fn main() {
    gpui::Application::new()
        .with_assets(vektra::assets::Assets)
        .run(|cx: &mut App| {
            let _ = cx;
        });
}
```

An application still needs to create a view through GPUI's window model. Use `Button::on_click_in(cx, ...)` when the handler needs to read or update host Entity state.

## Relationship to GPUI

Vektra does not replace GPUI's application lifecycle, windows, actions, or focus system. It reuses GPUI elements, `Context<T>`, `Window`, and the asset pipeline. Button owns its visual states, mouse activation, Enter/Space activation, and disabled behavior.

## Tab / Shift+Tab Focus Traversal

Button and IconButton register GPUI Tab stops. The pinned GPUI revision does not automatically map real Tab keys to host focus traversal. A window root View holds a stable `FocusHandle`, receives initial focus, and handles local Actions by calling `window.focus_next(cx)`/`focus_prev(cx)`, with bindings for both `tab` and `shift-tab`. Keep this wiring in the host rather than a Vektra global initializer; the complete compiled pattern is in the Button/IconButton desktop examples and docs preview runtime.

After `.tooltip(...)` is configured, keyboard focus held for 500ms shows the Tooltip. Escape dismisses only the Tooltip and leaves Button focus unchanged.
