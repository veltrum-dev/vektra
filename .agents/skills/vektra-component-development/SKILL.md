---
name: vektra-component-development
description: 用于在 Vektra 仓库中创建新的公开可见 GPUI 组件、修改公开组件 API、为组件增加事件、状态、variant、size 或交互能力、重构多个组件共享能力、审查组件设计是否符合 Vektra 规范，或为组件补充 VitePress 文档、Rust 示例和 GPUI WASM 预览。不用于纯后台逻辑、与 UI 无关的数据结构、单纯知识问答，或不涉及公开组件 API 的局部修复。
---

# Vektra 组件开发规范

## 使用边界

本 Skill 约束 Vektra 公开可见组件的设计、实现、测试、文档和 WASM 预览完成标准。它不替代下列项目级 Skill：

- 涉及 GPUI API、Entity、自定义 Element、Action、事件或焦点时，同时使用 `gpui`。
- 涉及 GPUI 测试、测试种子、`TestAppContext` 或 `VisualTestContext` 时，同时使用 `gpui-test`。
- 涉及 Rust 源码、Cargo、公开 API 或 rustdoc 时，同时使用 `rust-code-style`。
- 涉及视觉状态、无障碍、主题、响应式、跨平台或性能证据时，同时使用 `vektra-ui-guardrails`。

不要复制这些 Skill 的完整规则；只在组件开发任务中应用它们。

## 开始前检查

1. 读取仓库根 `Cargo.toml`、目标 crate 的 `Cargo.toml`、目标目录代码和相关测试，确认真实 workspace、feature、导出路径和局部风格。
2. 检查现有组件是否已覆盖需求。当前基础组件包括 `crates/vektra/src/button.rs`、`crates/vektra/src/icon_button.rs`、`crates/vektra/src/icon.rs` 和主题 token。
3. 确认锁定 GPUI revision。当前根 `Cargo.toml` 锁定 `gpui` 与 `gpui_platform` 到 Zed revision `82aef44308540b576e4e51fb379efa71614e5c91`；实现前必须用本地源码或最小编译验证确认 API。
4. 如任务涉及 Web/WASM，检查锁定 GPUI 的 `gpui_platform::web_init`、`gpui_platform::single_threaded_web`、`gpui_web::WebPlatform` 和 `gpui_web/examples/hello_web/`。不要从旧示例或未锁定版本推断。

## 组件设计流程

按顺序确认并记录结论，验收标准明确后再实施：

1. 判断是否能组合现有组件、slot、variant 或显式状态输入，而不是创建新组件。
2. 选择实现形态：
   - `RenderOnce`：无状态展示组件或完全由调用方控制的轻量交互组件。
   - `Render + Entity`：组件拥有内部状态、订阅、异步任务、焦点句柄或生命周期。
   - 自定义 `Element`：普通布局和现有控件无法满足绘制、性能、命中区域或特殊交互时。
3. 判断状态由调用者控制还是组件内部拥有；优先采用可测试的显式状态输入和语义回调。
4. 明确构造函数必填项、consuming builder、variant、size、tone、事件能力、slot 和导出路径；逐项审计组件是否应实现仓库现有的标准能力 trait。
5. 明确键盘、焦点、鼠标、主题、响应式、无障碍和跨平台语义。
6. 明确测试、VitePress 文档、可编译 Rust 示例和 GPUI WASM 预览计划。
7. 定义完成检查和验收命令。

不要为了统一外观而过度抽象；只有至少两个组件共享相同语义和签名后，才提取新的共享能力。

## 标准能力 trait

- 先读取 `crates/vektra/src/traits/` 中的真实定义；组件公开 builder 与现有 trait 的语义和签名一致时，必须实现该标准 trait，不得只提供同名 inherent 方法。
- 保留便于调用的 inherent forwarding builder，并让 trait 实现委托给同一实现，避免两套行为分叉。
- 按需审计 `Clickable`、`Focusable`、`Disableable`、`Sizable`；具体适用条件和语义边界见 [component-api-design.md](references/component-api-design.md)。
- 受控组件即使拥有 `on_change(next_value, ...)` 等语义回调，只要还需要供通用包装器、前置请求或中间件使用的原始激活入口，也应实现 `Clickable`；标准入口与语义入口必须复用同一激活路径，并明确组合优先级，禁止一次激活重复触发两套回调。
- 实现交互组件前，先检查根 `Cargo.toml` 锁定的 GPUI revision 及其真实源码是否已提供语义事件合成。Button 类组件若已由 GPUI `on_click` 统一鼠标、触摸、Enter 与 Space，只能注册一条语义激活路径；禁止同时注册 `on_click`，又在 `on_key_down` 或 `on_key_up` 中手动调用同一业务 handler。
- 原始键盘处理只用于 GPUI 未覆盖的组件专属语义，例如 Radio 方向键、Home、End，输入编辑、Escape 关闭或 busy 状态事件消费；不得借此重复触发语义回调。可重映射命令和应用快捷键使用 `Action`/`KeyBinding`，不要塞进组件 `on_click`。
- GPUI 原始 `Keystroke.key` 是 `String` 只属于底层实现事实；公共激活来源通过 `ClickEvent::Keyboard` 与 `KeyboardButton` 枚举判断。完整职责边界和测试规则见 [component-api-design.md](references/component-api-design.md#单一语义激活路径)。
- 为适用 trait 增加泛型能力测试和 inherent forwarding 测试，验证 trait 调用与直接 builder 调用遵循同一契约。

## 参考路由

- 设计或修改公共组件 API、builder、variant、size、状态、事件能力 trait 或 Clickable 目标形态时，读取 [component-api-design.md](references/component-api-design.md)。
- 编写或审查组件文档、示例、VitePress 页面、WASM 预览、demo 注册和完成检查时，读取 [component-documentation.md](references/component-documentation.md)。
- 借鉴 Web、MDN、WAI-ARIA、React controlled component 或 DOM 事件概念时，读取 [web-to-gpui.md](references/web-to-gpui.md)。

## 事件机制选择

先选择正确 GPUI/Vektra 机制，不要把它们合并成通用事件总线：

- 组件局部回调：使用 `on_click` 等 builder callback。
- Entity 对外发布领域事件：使用 `EventEmitter<E>` 与 `cx.emit`。
- 键盘命令和可重映射操作：使用 `Action`。
- 原始指针、键盘和布局交互：使用 GPUI `InteractiveElement`。
- 视觉状态：使用 GPUI `hover`、`active`、`focus_visible` 等样式机制。

## 完成要求

公开可见组件完成前至少具备：

- Rust API、中文 rustdoc、导出路径和稳定交互组件的 `ElementId` 支持。
- 所有适用的仓库标准能力 trait 均已实现；不适用项有明确的语义理由和测试边界。
- 对应 crate 独立 `tests/` 目录中的 API、事件、禁用状态、焦点和键盘行为测试。
- `cargo fmt --all --check`、必要范围的 `cargo check`、`cargo clippy` 和 `cargo test` 结果。
- VitePress 组件页面、实际参与 Cargo/WASM 编译的 Rust 示例、GPUI WASM demo 注册和可交互预览。
- Light/Dark/System、键盘与焦点、响应式、平台限制和已知限制说明。

缺少文档或 WASM 预览时，公开可见组件不能标记为完成。无法支持 WASM 的组件必须在设计阶段提出平台例外，不能静默省略。
