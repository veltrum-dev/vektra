# 快速开始

Vektra 是面向 [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) 的组件库。

::: warning 预发布状态
Vektra 当前处于早期开发阶段。GPUI 虽已发布 pre-1.0 版本，但 API 仍在快速演进，可能发生破坏性变更。Vektra 当前锁定特定 GPUI revision，尚未承诺稳定兼容 crates.io GPUI，因此暂不发布可供生产使用的正式 crate。请通过 Git dependency 使用，并预期 Vektra 公共 API 也可能发生破坏性变更。

crates.io 上的 `vektra` 0.0.1 只用于保留项目名称，不包含当前组件库实现，不能作为正式依赖。当前真实组件 crate 位于 `crates/vektra`，仍标记为 `publish = false`。
:::

## 环境要求

- Rust workspace 使用 edition 2024，`rust-version` 为 `1.98.0`。
- GPUI 依赖由仓库根 `Cargo.toml` 锁定到 Zed revision `fd82517a115d97a07835b52f0512b22b38e38ccf`。
- 文档预览的 Web 构建需要 `wasm32-unknown-unknown` target 和 Trunk `0.21.14`。
- 文档站使用 Bun 管理前端依赖。

## 添加依赖

在应用的 `Cargo.toml` 中添加 GPUI、平台启动 crate 与 Vektra：

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "fd82517a115d97a07835b52f0512b22b38e38ccf" }
vektra = { git = "https://github.com/veltrum-dev/vektra.git" }

[target.'cfg(target_os = "macos")'.dependencies]
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "fd82517a115d97a07835b52f0512b22b38e38ccf", features = ["font-kit"] }

[target.'cfg(any(target_os = "linux", target_os = "freebsd"))'.dependencies]
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "fd82517a115d97a07835b52f0512b22b38e38ccf", features = ["wayland", "x11"] }

[target.'cfg(target_os = "windows")'.dependencies]
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "fd82517a115d97a07835b52f0512b22b38e38ccf" }
```

`gpui` 与 `gpui_platform` 必须使用 Vektra 当前锁定的同一 revision。省略 `rev` 会让 Cargo 同时引入 Zed 最新提交与 Vektra 锁定提交，造成 GPUI 类型不兼容。

`gpui_platform` 的 feature 与目标平台相关：macOS 需要 `font-kit` 才能绘制文字；Linux 与 FreeBSD 至少启用 `wayland` 或 `x11` 中的一项；Windows 不需要额外 feature。只构建单一平台时，可以只保留对应的 target dependency。

## 最小示例

Vektra 组件是普通 GPUI element。应用仍由 GPUI 创建窗口并渲染 view，Vektra 只提供组件、主题和资源。

```rust
use gpui::{
    App, AppContext, Bounds, IntoElement, ParentElement, Render, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use vektra::Button;

struct Demo;

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div().child(
            Button::new("save")
                .label("保存")
                .on_click(|_, _, _| {
                    // 处理按钮激活。
                }),
        )
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(vektra::assets::Assets)
        .run(|cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(480.), px(320.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_window, cx| cx.new(|_| Demo),
            )
            .expect("应能创建 Vektra 示例窗口");
            cx.activate(true);
        });
}
```

应用需要按 GPUI 的窗口模型创建 view。需要读取或修改宿主 Entity 状态时，使用 `Button::on_click_in(cx, ...)`，示例见 Button 文档。

## 与 GPUI 的关系

Vektra 不替代 GPUI 的应用生命周期、窗口、Action 或焦点系统。它复用 GPUI element、`Context<T>`、`Window` 和主题资源机制。Button 负责自己的视觉状态、鼠标激活、Enter/Space 键盘激活与 disabled 行为。

## Tab / Shift+Tab 焦点遍历

Button 与 IconButton 会注册 GPUI Tab stop；当前锁定 GPUI revision 不会替宿主自动把真实 Tab 键映射为焦点遍历。窗口根 View 应持有稳定 `FocusHandle`、获得初始焦点，并在局部 Action handler 中调用 `window.focus_next(cx)`/`focus_prev(cx)`，同时绑定 `tab` 和 `shift-tab`。不要把这段接线放进 Vektra 全局初始化；完整、可编译模式见 Button、IconButton 桌面 example 和文档 preview runtime。

配置 `.tooltip(...)` 后，Tab 聚焦停留 500ms 会显示 Tooltip；Escape 只关闭 Tooltip，不移动按钮焦点。
