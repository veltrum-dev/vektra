# Vektra GPUI 文档预览

`docs/preview` 是文档 iframe 使用的静态 GPUI WASM 预览入口。目录名是 `docs/preview`，Cargo package 名是 `vektra-docs-preview`，Rust crate 路径是 `vektra_docs_preview`。

## 环境

- Rust target：`wasm32-unknown-unknown`。
- Trunk：当前使用 `0.21.14` 验证，不使用未固定版本。
- 浏览器：需要 WebGPU。当前入口直接创建 `gpui_web::WebPlatform::new(false)`，不要求 COOP/COEP 或 SharedArrayBuffer。
- 字体：预览启动前会下载并注册 Noto Sans SC，否则显示 DOM 错误层而不启动 Canvas。

## 开发

```bash
rustup target add wasm32-unknown-unknown
cd docs/preview
trunk serve
```

打开 `http://127.0.0.1:8080/?demo=button/basic`。未提供 `demo` 参数时默认使用 `button/basic`。独立页面可额外带 `theme=light|dark` 和 `lang=zh-CN|en-US`；缺失或非法 `theme` 会回退到 `ThemeMode::System`，缺失或非法 `lang` 会使用中文。

## Release 构建

```bash
cd docs/preview
trunk build --release --public-url ./
```

静态产物输出到 `docs/preview/dist/`，可由普通静态服务器或 CDN 托管。不要把 `dist/`、Trunk 缓存或浏览器缓存提交进仓库。

## Demo

- `button/basic`：组件页预览，覆盖 6 种 variant、4 种 size、disabled、图标、默认状态、中文自动空格、自动/固定/窄/full-width 宽度以及点击次数和最近点击项。
- `button/showcase`：首页预览，展示少量代表性 variant、size、图标、disabled 和点击反馈。

未知 `demo` 值会显示错误状态，不会 panic，也不会静默回退。

## 主题同步

嵌入 VitePress 时，父页面是唯一主题控制源。iframe 初始 URL 会包含 `theme=light|dark`，随后 Light/Dark 变化通过同源 `postMessage` 同步，不改变 iframe `src`，因此不会重载 WASM 或丢失点击与焦点状态。

主题消息格式：

```js
{
  type: "vektra-preview:theme",
  value: "light" // 或 "dark"
}
```

iframe 会校验 `event.origin`、`event.source`、消息类型和值。合法消息早于字体或 GPUI 初始化到达时，只缓存最后一个主题；应用启动后通过 `ApplicationHandle::update` 调用 `vektra::set_theme_mode`。独立打开且没有父页面消息时，只使用 URL 参数或 `ThemeMode::System`。

可观察状态会写入 `window.__vektraPreviewState.theme` 和 `body[data-vektra-preview-theme]`，同时保留 demo id、状态、点击次数、最近点击项和字体状态。

## 语言

VitePress 页面通过 iframe URL 传递 `lang=zh-CN|en-US`。preview 启动时读取一次语言参数并选择静态文案。语言切换由 VitePress locale 导航完成，不使用运行时语言消息协议。

## 中文字体

GPUI Web Canvas 不使用 HTML/CSS 的 `system-ui`，锁定的 GPUI Web 平台也不会加载浏览器系统字体。预览宿主必须在打开 GPUI 窗口前显式下载、注册并验证中文字体；字体加载失败时只显示 DOM 中文错误层，不启动 Canvas 预览。

- 字体名称：Noto Sans SC。
- 版本：Noto Sans CJK Sans 2.004。
- 格式：官方 Region-specific Subset Variable Simplified Chinese TTF。
- 文件：`assets/fonts/noto-sans-sc/NotoSansSC-VF.ttf`。
- Family：`Noto Sans SC`。
- 字体大小：17,773,132 bytes。
- SHA-256：`d68bafcb48a2707749396aa12bbbd833cb70401f3a9a689fd2902c7e0d295964`。
- 官方来源：`https://github.com/notofonts/noto-cjk/releases/download/Sans2.004/02_NotoSansCJK-TTF-VF.zip` 中的 `Variable/TTF/Subset/NotoSansSC-VF.ttf`。
- 官方 release：`https://github.com/notofonts/noto-cjk/releases/tag/Sans2.004`。
- 许可证：SIL Open Font License 1.1，随静态资产保存为 `assets/fonts/noto-sans-sc/LICENSE`。

Trunk 通过 `<link data-trunk rel="copy-dir" href="assets" />` 将字体作为独立静态资源复制到 `dist/assets/...`，不通过 `include_bytes!` 嵌入 WASM。字体 URL 使用相对路径 `assets/fonts/noto-sans-sc/NotoSansSC-VF.ttf`，因此可在 `/previews/`、`/docs/previews/` 或 `/vektra/previews/` 等非根路径下解析。

## 非根路径

构建到相对 public URL：

```bash
cd docs/preview
trunk build --release --public-url ./
python3 -m http.server 9000 --directory dist
```

可通过 `http://127.0.0.1:9000/?demo=button/basic` 验证。最终文档站可以把同一目录挂到 `/previews/` 或 `/vektra/previews/`，iframe 地址继续使用 `index.html?demo=button/basic&theme=light&lang=zh-CN`。

## 已知限制

- 需要支持 WebGPU 的浏览器。
- 默认使用锁定 GPUI Web 的单线程 `WebPlatform::new(false)`，不要求 COOP/COEP。
