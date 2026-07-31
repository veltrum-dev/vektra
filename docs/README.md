# Vektra 文档开发

本文说明如何在本地开发 VitePress 文档站和 GPUI WASM preview。

## 环境要求

- Rust `1.95`，workspace 使用 edition 2024。
- Bun `1.3.14`。
- Rust target：`wasm32-unknown-unknown`。
- Trunk `0.21.14`。
- 支持 WebGPU 的浏览器。

首次准备：

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
cd docs
bun install --frozen-lockfile
```

## 开发文档站

```bash
cd docs
bun run dev
```

`dev` 会先构建一次 `docs/preview`，把静态产物同步到 `docs/public/previews/`，再启动 VitePress。打开终端输出的本地地址即可看到包含 WASM preview 的完整文档站。

如果只想启动 VitePress 并复用已有 preview 产物：

```bash
cd docs
bun run dev:vitepress
```

## 只调试 GPUI preview

```bash
cd docs/preview
trunk serve
```

常用地址：

- `http://127.0.0.1:8080/?demo=button/basic`
- `http://127.0.0.1:8080/?demo=button/showcase&theme=dark&lang=en-US`

缺少或非法 `theme` 参数时，独立 preview 会使用 `ThemeMode::System`。缺少或非法 `lang` 参数时，preview 使用中文。

## 生产构建

```bash
cd docs
VEKTRA_DOCS_BASE=/vektra/ bun run build
```

构建流程会先运行 `bun run preview:build`，再运行 `vitepress build`。最终站点位于 `docs/.vitepress/dist/`，不要提交该目录。

本地预览构建结果时，默认用根路径构建再启动 VitePress preview：

```bash
cd docs
VEKTRA_DOCS_BASE=/ bun run build
bun run serve
```

如果刚刚用 `/vektra/` 构建，`docs/.vitepress/dist` 中的 HTML 会引用 `/vektra/` 绝对资源路径；请把 dist 挂载到 `/vektra/` 子路径，或改用下面的 base 验证命令检查产物。

## `VEKTRA_DOCS_BASE`

`VEKTRA_DOCS_BASE` 控制 VitePress `base`。GitHub 项目页应设置为 `/vektra/`：

```bash
cd docs
VEKTRA_DOCS_BASE=/vektra/ bun run build
```

验证时检查 `docs/.vitepress/dist/index.html` 中的静态资源路径是否以 `/vektra/` 开头，并确认 `docs/.vitepress/dist/previews/index.html`、preview JS/WASM 和字体都存在。

```bash
rg '"/vektra/' .vitepress/dist/index.html
test -f .vitepress/dist/previews/index.html
test -f .vitepress/dist/previews/assets/fonts/noto-sans-sc/NotoSansSC-VF.ttf
find .vitepress/dist/previews -name '*.wasm' -print
```

## 常见问题

WebGPU：GPUI Web preview 需要浏览器启用 WebGPU。若 Canvas 无法启动，请换用支持 WebGPU 的浏览器并查看开发者工具 console。

中文字体：preview 会在启动 GPUI 前加载 `docs/preview/assets/fonts/noto-sans-sc/NotoSansSC-VF.ttf`。字体失败时页面会显示 DOM 错误层，Canvas preview 不会启动。

独立 preview：直接打开 `/previews/index.html?demo=button/basic` 时没有 VitePress 父页面消息，主题只由 URL 参数或系统外观决定。`demo=button/showcase` 用于首页实验台，`demo=button/basic` 用于组件页。

主题同步：文档页通过 VitePress `useData().isDark` 解析当前 Light/Dark，并以同源 `postMessage` 通知 iframe。主题变化不会重载 iframe。

语言：VitePress 使用原生 locale。中文为默认语言 `/`，英文位于 `/en/`。新增或调整公开页面时，同步维护 `docs/content/...` 和 `docs/content/en/...` 的对应页面；Rust 标识符和 API 名不翻译。`VektraPreview.vue` 会把当前 locale 写入 iframe 的 `lang=zh-CN|en-US` 查询参数。

## GitHub Pages

`.github/workflows/docs.yml` 会在 pull request 中构建验证，在 `main` push 或手动触发时构建并部署 `docs/.vitepress/dist`。

仓库管理员需要在 GitHub 仓库 Settings -> Pages 中一次性将 Source 设置为 GitHub Actions。
