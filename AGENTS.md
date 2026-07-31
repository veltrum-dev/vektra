# Vektra 智能体协作说明

本文件适用于整个仓库。子目录若存在更具体的 `AGENTS.md`，则其规则在对应目录内优先。

## 项目定位

- Vektra 是独立、易组合的纯 GPUI 组件库，不是应用框架，也不是
  `gpui-component` 的二次封装。
- 不要引入 Vektra Root、Provider 或强制初始化流程。宿主应用仍使用 GPUI 自身的
  `Application`、`Window` 和根 View。
- 根 `Cargo.toml` 中的 GPUI 与 `gpui_platform` revision 是 API 事实来源。GPUI 尚未稳定；
  不根据其他 revision、旧示例或记忆猜测 API，必要时检查锁定依赖源码或用最小编译验证。
- 默认使用中文沟通。Vektra 自有公开 Rust API 使用有意义的中文 rustdoc。

## 开始工作前

1. 阅读与任务直接相关的代码、测试、文档和 `Cargo.toml`，先确认仓库中的真实模式。
2. 检查 `.agents/skills/`，按任务类型使用适用的 skill：
   - Rust 或 Cargo：`rust-code-style`
   - GPUI API、状态、事件、焦点、布局或自定义 Element：`gpui`
   - GPUI 确定性测试：`gpui-test`
   - 新增或修改公开组件能力：`vektra-component-development`
   - 可见 UI、交互、无障碍、响应式或性能：`vektra-ui-guardrails`
   - 用户要求先讨论、确认方案或生成交接指令：`clarify-and-handoff`
   - 编写、审查或重构代码：`karpathy-guidelines`
3. 保留用户已有改动；不要顺手修改、格式化或清理任务范围之外的文件。
4. 优先复用现有组件、主题 token、资产和测试模式。没有明确复用价值时，不新增抽象或依赖。

## 实现约束

- 变更保持聚焦，解决已确认的问题；避免猜测性功能、兼容层和无关重构。
- 公开组件 API 应保持易组合、无需全局注册，并覆盖合理的事件、状态、尺寸、variant、
  键盘操作和无障碍语义。
- GPUI 一次只接收一个 `AssetSource`。资源扩展沿用现有组合与覆盖机制，不另建并行入口。
- 第三方依赖和 workspace 内部 crate 路径优先在根 `[workspace.dependencies]` 统一管理；
  成员 crate 使用 `{ workspace = true }` 继承。
- 禁止为了让检查通过而弱化断言、静默吞错、加入无边界等待，或把未验证行为写成已验证。

## 组件、文档与示例

- 新增公开可见组件或修改公开组件 API 时，同步评估：Rust API 与导出、中文 rustdoc、
  单元/交互测试、VitePress 页面、可编译 Rust 示例以及 GPUI WASM 预览。
- 文档示例应尽量参与真实 Cargo/WASM 编译，避免维护无法验证的复制代码。
- 修改主题、图标或其他资源时，检查根 `assets/`、对应资源 crate、默认 feature 与自定义资源
  回退行为是否一致。
- `docs/public/previews/`、`docs/preview/dist/` 和 VitePress 构建目录属于生成产物，
  除非任务明确要求，不要手工编辑或提交。

## 验证与完成报告

Rust 或 Cargo 变更的默认检查为：

```sh
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

根据改动范围补充必要的 feature、doctest、文档或 WASM 检查。文档站点在 `docs/` 中使用
锁定的 Bun/VitePress 工具链；相关改动至少运行：

```sh
cd docs
bun run build
```

只运行与风险相称的检查。若受环境、平台或时间限制无法执行某项检查，在完成报告中明确写为
“未验证”并说明原因；不要把未运行的平台、视觉、性能或交互检查声称为通过。
