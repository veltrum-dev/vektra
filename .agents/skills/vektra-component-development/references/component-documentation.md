# 组件文档、示例、测试与 WASM 预览

## VitePress 页面标准

每个公开可见组件必须拥有 VitePress 文档。缺少文档或预览时，组件不能标记为完成。

Vektra 文档站和组件预览默认使用简体中文。用户可见文案不得因为字体、
平台或渲染问题改成英文规避；中文字体缺失、缺字、方框或乱码属于阻塞问题。

每个组件页面至少包含：

- 用途、适用场景和非目标。
- 组件 anatomy。
- 基础使用和有状态使用。
- 构造函数和完整 API 表。
- 能力 trait 和事件说明。
- variants、sizes、states 和默认值。
- 键盘操作、焦点及无障碍说明。
- 浅色和深色主题。
- 响应式和跨平台行为。
- 可运行 Rust 示例。
- WASM 实时预览。
- 已知限制。

示例代码必须来自实际参与 Cargo/WASM 编译的 Rust 文件，再由 VitePress 导入展示。禁止在 Markdown 中维护无法编译的重复示例。

## 文档示例卡片

- Basic 必须是页面第一个示例，并提供最小、正确、聚焦的首次使用路径。
- 禁止 Basic 引用 omnibus/comprehensive demo 的完整状态；受控组件只保留正确交互必需的最小状态与回调。
- 每张文档示例卡片必须让 WASM 实时预览与展示源码一一对应，使用同一个稳定 `demo_id` 表达同一场景。
- 一个示例包含多项独立能力或代码过长时，按单一能力继续拆分，不把综合 Demo 重新包装成一张卡片。
- 展示源码必须从实际参与 Cargo/WASM 编译的 Rust 文件导入，不能在 Markdown 中维护副本或不可编译伪代码。
- 文档示例统一使用“预览、操作栏、可折叠源码”的卡片结构；操作栏至少提供代码展开、准确复制当前源码和独立页面打开。
- 示例卡片的展开与复制操作必须支持键盘、可访问名称和即时反馈；窄宽度下操作栏允许换行，不产生页面水平溢出。

## WASM 预览硬性标准

- 每个公开可见组件至少有一个实际可交互的 GPUI WASM 预览。
- 主要 variant、size、state 和交互也应覆盖。
- 文档构建必须能检查组件页面、示例源码和预览注册项是否齐全。
- GPUI WASM 预览运行时必须显式提供中文字体覆盖，不能把“Rust 字符串包含中文”当作字体渲染通过的证据。
- Web 字体注册属于文档预览宿主职责，不属于叶子组件职责；组件不得为了文档预览硬编码中文字体。
- 字体必须来自可重新获取的可信来源，字体许可证必须随静态资产保存。
- 字体加载失败必须显示可理解的中文错误状态，不能静默回退为英文或缺字 Canvas。
- 文档预览必须报告字体、WASM 和总体静态资源体积。
- 浏览器验收必须包含实际 Canvas 中文显示。
- 无法支持 WASM 的组件必须在设计阶段提出平台例外，不能静默省略。
- 如果组件依赖桌面原生能力，应优先设计 Web 适配层。

## 推荐预览架构

- VitePress 生成静态 HTML/CSS/JS。
- Rust 示例统一编译进共享 GPUI WASM 预览运行时。
- 使用稳定 `demo_id` 注册各组件示例。
- VitePress 通过 `VektraPreview` 组件引用 `demo_id`。
- 每个预览使用懒加载 iframe 隔离 GPUI、Canvas、焦点和状态。
- iframe 使用类似 `/previews/index.html?demo=button/basic` 的静态地址。
- 所有页面复用同一 WASM 产物和浏览器缓存。
- 不为每个组件重复编译完整 GPUI WASM。
- 默认使用 `gpui_platform::single_threaded_web()`，降低静态托管对 SharedArrayBuffer、COOP/COEP 响应头的要求。
- 必须包含加载状态、错误状态、尺寸约束和静态截图降级。
- WASM/JS/HTML/资源文件最终都能部署到普通静态托管或 CDN。
- 静态服务器必须正确提供 `application/wasm` MIME 类型。

## 锁定 GPUI Web 参考

当前仓库锁定 Zed revision `82aef44308540b576e4e51fb379efa71614e5c91`。参考本地路径：

- `/Users/coloxan/.cargo/git/checkouts/zed-a70e2ad075855582/82aef44/crates/gpui_platform/src/gpui_platform.rs`
- `/Users/coloxan/.cargo/git/checkouts/zed-a70e2ad075855582/82aef44/crates/gpui_web/`
- `/Users/coloxan/.cargo/git/checkouts/zed-a70e2ad075855582/82aef44/crates/gpui_web/examples/hello_web/main.rs`

已观察到 `gpui_platform::web_init()`、`gpui_platform::application()` 和 wasm-only `gpui_platform::single_threaded_web()`；实际预览运行时实现前必须通过最小 WASM 编译验证最终入口。

## 测试目录与覆盖

组件测试必须放在对应 crate 的独立 `tests/` 目录中：

```text
crates/<crate>/tests/*.rs
crates/<crate>/tests/unit/*.rs
crates/<crate>/tests/support/
```

不要在业务模块内堆积大型 `#[cfg(test)]` 测试实现。公共行为优先通过 crate 公共 API 在 `tests/` 中验证；确需白盒测试时按 `rust-code-style` 和 `vektra-ui-guardrails` 的最小 `#[path]` 声明方式处理。

测试至少覆盖：

- 公开构造函数、builder、默认值和导出路径。
- variant、size、state、slot 和主题 token 解析。
- 事件、禁用状态、焦点和键盘行为。
- 可访问名称、Role 和鼠标/键盘激活一致性。
- 必要的 WASM 构建和预览注册完整性检查。

## 完成检查

组件完成检查至少包括：

- Rust API 和中文 rustdoc。
- 导出路径。
- 独立 `tests/`。
- 格式化、Clippy 和测试。
- VitePress 页面。
- 编译通过的 Rust 示例。
- WASM demo 注册。
- 可交互预览。
- 浅色/深色/System 模式。
- 键盘和焦点。
- 响应式检查。
- 平台限制说明。

报告时按 `vektra-ui-guardrails` 的方式标记“通过 / 不适用 / 未验证”，不要把未运行的平台、视觉、性能或 WASM 检查写成通过。
