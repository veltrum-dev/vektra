# Vektra 智能体协作说明

本文件适用于整个仓库。子目录若存在更具体的 `AGENTS.md`，则其规则在对应目录内优先。

## 项目定位

- Vektra 是面向 Windows、Linux 和 macOS 的跨平台、高性能、可组合纯 GPUI 组件库，
  不是应用框架，也不是
  `gpui-component` 的二次封装。
- 性能是公共组件正确性和 API 契约的一部分，不是可选的事后优化。
- 不要引入 Vektra Root、Provider 或强制初始化流程。宿主应用仍使用 GPUI 自身的
  `Application`、`Window` 和根 View。
- 根 `Cargo.toml` 中的 GPUI 与 `gpui_platform` revision 是 API 事实来源。GPUI 尚未稳定；
  不根据其他 revision、旧示例或记忆猜测 API，必要时检查锁定依赖源码或用最小编译验证。
- 默认使用中文沟通。Vektra 自有公开 Rust API 使用有意义的中文 rustdoc。

## 性能架构硬规则

- 每个公开组件在设计阶段必须声明正常规模、较大规模、压力规模、时间与内存复杂度、
  实际物化数量上界、缓存上限和已验证平台；答案不完整时不能标记完成。
- 不得以方便实现为理由引入全量渲染、无界缓存、无界任务、意外 O(n²) 或普通稳态帧中的
  O(n) 全量处理。render、布局、prepaint、paint 热路径禁止阻塞 I/O、JSON/主题解析、正则
  编译、全量数据构建和重复 SVG 解析。
- Select、List、Table、Tree、Menu、Command Palette、Autocomplete、Combobox 及未来类似
  集合组件默认使用统一惰性数据源。Vec、数组、逐项 builder、生成式、分页和远程数据只能
  作为同一协议的 adapter，不得建立 eager/lazy 两套状态、导航、焦点或 AccessKit 实现。
- 集合组件的 Element、布局、prepaint、paint 与 AccessKit 物化必须为
  O(visible + bounded overdraw)。不得声称“虚拟化”却保存全量 Element、metadata、逐行 Entity
  或无界行状态；严格大数据路径不得复制全部业务数据或建立第二份全量 catalog/search text。
- 固定高度大数据路径优先以 `count × height` 推导总高度并保持 O(viewport) 或更小的 Vektra
  附加内存。可变高度、高度索引、精确随机跳转和内存成本必须显式记录，不能把 O(n)
  `ListState` 宣传为固定内存。
- 大数据 typeahead、disabled navigation、key/value 定位和随机跳转必须由数据源索引支持；
  数据源方法不得在 render 线程阻塞，缺失页只能发出非阻塞 range request。
- 所有缓存必须记录 key、失效条件和硬容量；Task、Subscription、Entity、timer 和历史必须有
  owner 与释放路径。warm steady-state 不得持续净增长。测量时区分调用方业务数据、Vektra
  索引/缓存、GPUI Element 与 renderer 内存。
- 公开组件完成前必须有正常/压力规模、首次绘制、稳态重绘、交互+绘制、allocation、allocated
  bytes、CPU/RSS 和长期净内存证据，并文档化复杂度、缓存上限与平台状态。算法/物化/缓存上限
  失败直接失败；不得降低数据规模、弱化断言、吞错或无证据放宽预算。
- 120fps 稳态目标为 8.33ms，普通交互到下一次绘制目标为 16.67ms；它们只是在明确参考环境
  中的工程目标，不是对所有机器和完整宿主应用的无条件保证。预算和证据见 `PERFORMANCE.md`
  与 `performance-budgets.json`。

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
- 本次或未来任何支持虚拟渲染的组件，都必须在该组件已有的 `examples/` 运行入口中同时展示
  普通场景和明确标注的大数据场景，不得另拆一个难以发现的 `*-large` example。例如
  `cargo run --example select` 必须同时看到普通 Select 与“百万项惰性数据源”场景。界面和说明
  必须明确展示数据规模、数据是否生成式/分页、visible range、实际物化数量和缓存上限；百万级
  示例不得先构建同规模 `Vec`。
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
