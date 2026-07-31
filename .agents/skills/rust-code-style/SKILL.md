---
name: rust-code-style
description: 用于在 Vektra 仓库中编写、修改、重构、审查或生成 Rust 源码与 Cargo 配置。覆盖中文 rustdoc、命名与导入、控制流、迭代器、类型转换、错误处理、异步代码、crate 与模块组织、依赖与 feature、测试以及提交前质量检查；处理 `.rs`、`Cargo.toml`、`rustfmt.toml`、`clippy.toml` 或 Rust workspace 结构时使用。
---

# Vektra Rust 代码规范

## 工作流程

1. 先读取仓库根目录和目标目录中的 `AGENTS.md`、`Cargo.toml`、格式化配置及相关代码，确认现有 workspace 结构、依赖边界和局部风格。不要假设 crate 或自定义命令已经存在。
2. 明确任务范围和可验证结果，只修改满足需求所必需的代码与配置。发现无关问题时说明，但不要顺手重构。
3. 编写代码时遵守本规范；现有代码、用户明确要求或更具体的项目级指令与本规范冲突时，以更具体的约束为准。
4. 为新增行为或缺陷修复增加与风险相称的测试；选择单元测试、集成测试或文档测试中最自然的层级。
5. 根据改动范围执行格式化、静态检查和测试。无法执行的检查要说明原因，不要声称通过。

涉及 GPUI API、可见交互或确定性 GPUI 测试时，同时加载仓库中对应的 `gpui` 或 `gpui-test` skill；本 skill 只负责通用 Rust 与 Cargo 规范。

## Rustdoc 与注释

- Vektra 自有的公开 API 必须提供有意义的中文 rustdoc，包括公开模块、类型、trait、函数、方法、关联项、字段和枚举变体。
- 模块职责使用 `//!`；API、字段和变体使用 `///`。内部实现说明使用 `//`，避免解释代码已经清楚表达的内容。
- 文档说明职责、适用场景、重要参数、返回值、状态变化和副作用，不要只复述名称。
- 新增公开 API 时同步新增文档；改变行为、错误条件或可见性时同步更新文档和引用处。
- 用法不直观时增加可运行的 `# Examples`；返回 `Result` 时按需增加 `# Errors`；存在非显然 panic 条件时增加 `# Panics`。
- 示例优先写成可执行 doctest。必须忽略时说明环境或平台限制，不能用 `ignore` 掩盖过期示例。
- trait 实现中继承的语义无需逐字复制；实现增加额外约束或副作用时补充说明。

```rust
/// 桌面应用的启动模式。
///
/// 运行器根据该模式决定启动后是否立即创建主窗口。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StartupMode {
    /// 后台启动，不主动打开主窗口。
    Background,

    /// 前台启动，并按窗口配置打开主窗口。
    #[default]
    Foreground,
}
```

## 命名、导入与可见性

- 遵循 Rust 命名约定：模块、函数和变量使用 `snake_case`，类型、trait 和枚举变体使用 `UpperCamelCase`，常量和静态量使用 `SCREAMING_SNAKE_CASE`。
- 名称表达业务或架构职责。不要机械地给每个 crate 或类型添加 `vektra_` 前缀，也不要使用含糊的 `utils`、`manager`、`handler`，除非它确实是仓库既有术语。
- 默认使用最小可见性。仅在跨模块或跨 crate 契约确实需要时使用 `pub`；crate 内共享优先考虑 `pub(crate)`。
- 合并同一 crate 或模块下的导入，并让 `rustfmt` 负责布局和排序。签名中优先使用已导入且无歧义的短类型名。
- 避免通配符导入；prelude、测试模块或明确约定的宏导入可以例外。
- 不用 `#[allow(...)]` 隐藏由当前改动引入的问题。确需豁免时缩小作用域，并写清不能消除警告的原因。

```rust
use crate::{
    error::AppError,
    store::{FileStore, Store},
};
```

## 实现与控制流

- 优先使用标准库和仓库已有抽象；不要为单次调用增加包装层、泛型参数、trait 或配置开关。
- 单模式分支使用 `if let`，连续匹配使用 `while let`；多分支穷举、需要守卫或强调状态机时使用 `match`。
- 使用提前返回和 `let ... else` 减少嵌套，但不要为了追求扁平化拆散紧密相关的逻辑。
- 对集合做无副作用的筛选、转换和收集时优先使用迭代器。闭包包含复杂分支、异步步骤或明显副作用时，使用具名函数或清晰的 `for` 循环。
- 能在一次遍历中完成时考虑 `filter_map`、`flat_map`、`fold`、`try_fold` 或 `collect::<Result<_, _>>()`，但不要用晦涩的长链牺牲可读性。
- 基于已有值只修改少数字段时使用结构体更新语法，并确认移动与借用语义。重复访问同一值的多个字段时考虑解构。
- 标准能力语义完全一致时优先 `#[derive(...)]`；需要校验、不平凡默认值或自定义语义时手写 trait 实现。

## 类型与转换

- 无损且不会失败的转换实现 `From`；可能失败的转换实现 `TryFrom`。不要用 panic 表达正常的转换失败。
- 只有参数确实需要接受多种借用表示时才使用 `AsRef` 或 `AsMut`；单一输入类型保持具体签名。
- 不为方便调用而无条件使用 `Into<T>`、`impl Trait` 或复杂泛型。泛型必须服务于真实的多类型调用或抽象边界。
- 避免无依据的 `as` 转换。整数窄化、符号变化和可能丢失精度的转换使用显式检查或 `TryFrom`。
- 优先借用而不是克隆。只有所有权边界、异步生命周期或缓存语义确实需要时才复制数据。

## 错误处理

- 可通过 `From` 自动转换并直接向上传播的错误使用 `?`，不要手写等价的 `match` 或 `map_err(Error::from)`。
- 只有需要增加上下文或改变错误语义时使用 `map_err`。错误信息要包含操作和相关对象，避免只有“失败了”。
- library crate 的公共边界优先使用稳定、可匹配的结构化错误类型；应用入口或任务编排层需要汇总异构错误时可使用上下文型错误。
- 沿用仓库已有错误方案。引入 `thiserror`、`anyhow` 或其他错误依赖前先确认 workspace 已有选择和使用边界。
- 使用 `ok_or_else`、`and_then`、`transpose` 等组合器减少机械分支，但在嵌套闭包降低可读性时改用明确控制流。
- 生产代码禁止使用 `unwrap()`、`expect()` 处理可恢复错误。只有不变量已被局部证明、测试代码或进程启动后不可恢复的配置错误才可使用，并让原因清晰可审查。
- 不吞掉错误。刻意忽略错误时明确记录、注释或通过类型表达该决策。

## 异步与并发

- 异步调用链中的文件、网络、进程和定时 I/O 使用异步 API，不阻塞执行器线程。
- 必须调用阻塞 API 或执行较长 CPU 工作时，使用当前运行时的阻塞任务机制，并处理任务 panic、取消和错误传播。
- 不持有同步锁跨越 `.await`；缩小锁作用域，或采用与执行模型匹配的异步同步原语。
- 生成任务时明确所有权和生命周期：谁等待、谁取消、谁记录错误。不要静默丢弃可能失败的任务句柄。
- 同步边界保持同步。不要仅为形式统一把简单、无并发需求的接口改成 async。
- GPUI 任务生成、Entity 生命周期和测试计时遵守 `gpui`、`gpui-test` skill 的专门规则。

## Cargo workspace 与依赖

- 先检查根 `Cargo.toml`。如果 Vektra 使用 Cargo workspace，第三方依赖版本以及 workspace 内部 crate 路径优先由根 `[workspace.dependencies]` 统一管理；成员 crate 使用 `{ workspace = true }` 继承。
- Cargo workspace 成员目录使用简短的职责名称，禁止为了强调归属而添加 `vektra-`、`vektra_` 等项目名前缀。Cargo package 名可以保留 `vektra-` 前缀以提供唯一身份，目录名与 package 名不要求机械一致。例如使用 `crates/assets` 对应 `vektra-assets`、`docs/preview` 对应 `vektra-docs-preview`、`examples/button` 对应 `vektra-button-example`；不要创建 `crates/vektra-assets`、`crates/vektra-docs-preview` 或 `examples/vektra-button`。
- 根据成员职责选择顶层目录：可复用 library 放入 `crates/<role>`，独立示例放入 `examples/<example>`，文档站及其构建、预览工具放入 `docs/<tool>`。不要仅因为目标由 Rust/Cargo 构建就把文档工具放进 `crates/`。
- 核心门面 crate 可以使用产品名本身，例如 `crates/vektra`；这表示该 crate 的真实职责，不视为附加前缀。现有 `crates/vektra-macros` 视为遗留路径，不在无关任务中顺手迁移；迁移时使用独立任务处理 workspace 路径、依赖和工具配置。
- 平台条件依赖可以位于成员 crate 的 target-specific dependency 表中，但已有 workspace 条目时仍使用 `{ workspace = true }`。
- 根配置只放多个使用方共享的版本、来源和最小公共 feature；单个 crate 专用 feature 在成员清单中按需追加。
- 引入依赖前检查标准库或现有依赖能否完成需求，并检查默认 feature。不要默认启用 `full`、`all` 等聚合 feature。
- 只有确认默认 feature 带来不需要的能力且关闭后兼容时，才设置 `default-features = false`。
- 新增或变更 feature 后按需运行 `cargo tree -e features`，检查 feature 合并是否意外扩大依赖树。
- crate 只依赖实际使用的契约和能力。共享类型至少被两个边界真实使用且稳定后再抽取公共 crate；不要为假想复用提前拆分。
- 保持依赖方向单向，避免基础模型、契约或领域 crate 反向依赖应用入口、UI 或基础设施实现。
- 成员 crate 的 package 名、目录名和代码引用保持可辨识的一致性；Cargo 名中的连字符在 Rust 路径中转换为下划线。

```toml
# 根 Cargo.toml
[workspace.dependencies]
serde = { version = "1", features = ["derive"] }

# 成员 Cargo.toml
[dependencies]
serde = { workspace = true }
```

若仓库尚未建立 workspace，不要仅为遵守上述形式创建多 crate 结构。先选择满足当前产品边界的最小 Cargo 结构。

## Crate 与模块组织

- 沿用仓库已经建立的 `apps/`、`crates/` 或其他布局；仓库尚无约定时，根据可执行入口与可复用库的真实边界选择最小结构。
- `main.rs` 保持为参数解析、配置加载、依赖装配和启动流程。业务逻辑增长到需要复用或独立测试时，再移入 library crate 或同 package 的 library target；不要提前拆 crate。
- 一个 package 同时包含 `src/main.rs` 和 `src/lib.rs` 是合法方案。是否拆成独立 package 取决于复用边界、依赖树和发布需求，不设绝对禁令。
- 新建 Rust 模块必须使用 Rust 2018 文件布局，禁止新建 `mod.rs`。模块根使用与模块同名的 `.rs` 文件，子模块放入同名目录：

  ```text
  src/
  ├── button.rs
  └── button/
      ├── event.rs
      └── render.rs
  ```

  `button.rs` 是 `button` 模块根，并在其中声明 `mod event;`、`mod render;` 和必要的 re-export。不要创建 `src/button/mod.rs`。
- 嵌套模块继续遵守同一规则。例如 `button/event.rs` 是 `button::event` 的模块根，其子模块应位于 `button/event/*.rs`，不得改用 `button/event/mod.rs`。
- `lib.rs`、`main.rs`、`build.rs` 和 Cargo `tests/*.rs` 集成测试入口属于 Rust/Cargo 约定的入口文件，不受“模块根必须与目录同名”规则影响；它们声明的普通子模块仍必须遵守上述布局。
- 已有 `mod.rs` 视为遗留结构。无关任务不要扩大迁移范围；当任务需要为该模块新增文件、拆分内容或实质修改模块结构时，应在同一任务中迁移为 `<module>.rs + <module>/`，并修正路径属性与引用。
- 模块围绕职责组织，入口文件声明子模块并提供必要 re-export。不要建立只转发一项内容的深层模块树。
- 跨 crate 契约不要直接暴露 UI 框架、数据库实体或运行时专用类型，除非这就是该 crate 明确承诺的公共边界。

## 测试

- 修复缺陷时先写能稳定复现问题的测试；新增行为时覆盖主要成功路径和直接相关的边界或错误路径。
- 每个 crate 采用统一测试目录结构：`crates/<crate>/src/` 只放生产代码和最小测试模块声明，`crates/<crate>/tests/*.rs` 放基于公共 API 的集成、交互和行为测试，`crates/<crate>/tests/unit/*.rs` 放必须访问私有实现的单元测试，`crates/<crate>/tests/support/` 放测试 fixture、模拟 renderer、辅助 View 和共享工具。
- 禁止在 `src/**/*.rs` 中直接编写完整的 `#[cfg(test)] mod tests { ... }`。
- 禁止把测试 fixture、模拟对象、测试 View 或 renderer 混入生产源码。
- 公共行为优先通过 crate 的公共 API 在 `tests/` 中验证。
- 不为了集成测试公开无业务意义的实现细节，也不得为了集成测试扩大生产 API 的可见性。
- 私有逻辑确实需要白盒测试时，允许生产文件只保留最小声明，例如：

  ```rust
  #[cfg(test)]
  #[path = "../tests/unit/button.rs"]
  mod tests;
  ```

- `#[path]` 路径必须根据实际源文件位置正确计算。
- `tests/unit/` 中的测试通过上述模块声明获得私有访问，不应被 Cargo 误作为独立集成测试入口。
- 新增或修改的测试必须遵守此结构；现有内联测试视为遗留债务，当前任务不顺手迁移。
- 后续不得继续向遗留内联测试模块添加测试；如果未来任务需要修改某个现有内联测试模块，应在该任务中把受影响模块的测试迁移到对应 `tests/` 目录。
- 测试文件名称应对应被测组件或行为。
- GPUI 测试仍须遵守 `gpui-test` skill 的确定性调度、测试上下文和种子复现规则。
- 文件系统测试使用独立临时目录；测试不得依赖执行顺序、用户主目录、固定端口或共享可变全局状态。
- CLI 集成测试启动真实二进制并断言退出码、标准输出和标准错误；沿用仓库已有测试库，引入新测试依赖前检查必要性。
- 异步和并发测试必须有确定的完成条件，不用任意时长 `sleep` 掩盖竞态。
- 测试名称描述行为和条件。断言失败信息应能定位输入与预期，不复制实现细节。
- 公开 API 示例适合表达契约时使用 doctest；需要环境、窗口或复杂 fixture 时使用普通测试。

## 宏与 unsafe

- 标准派生宏能准确表达语义时优先派生；不为减少几行清晰代码而引入自定义宏。
- 修改声明宏或过程宏时覆盖成功与失败路径。过程宏诊断应指向用户输入位置；适合时使用 `trybuild` 验证编译失败用例。
- 宏展开必须保持 hygiene，不依赖调用方的偶然导入，也不生成隐藏 I/O、全局状态或后台任务。
- `cargo expand`、基准、体积或汇编检查只在宏复杂度和性能风险需要时执行，不把它们作为每次宏改动的无条件门禁。
- 避免新增 `unsafe`。确需使用时保持最小作用域，在紧邻位置用 `// SAFETY:` 解释必须成立的不变量，并增加覆盖边界条件的测试。
- 封装 unsafe 的公共安全 API 必须在实现中维护并记录不变量；公开 unsafe API 使用 `# Safety` 说明调用者责任。

## 质量检查

先用 `cargo metadata --no-deps` 或根 `Cargo.toml` 确认真实 package、workspace 和 feature，再选择命令。不要运行不存在的旧项目自定义命令。

Rust 或 Cargo 改动的默认检查为：

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- 编写过程中可以先运行受影响 package 或测试过滤项；交付前在成本合理且环境允许时运行 workspace 范围。
- 仅当仓库存在相应脚本、工具链配置或 CI 门禁时，才运行额外的自定义 lint、`cargo nextest`、`cargo deny` 或平台检查。
- 修改 feature、条件编译、release-only 行为或构建脚本时，增加相应 feature 组合、target 或 `--release` 检查。
- 修改公开文档或 doctest 时运行相关 `cargo test --doc`，必要时运行 `cargo doc --no-deps`。
- `cargo fmt --all --check` 失败时先执行 `cargo fmt --all` 修复，再重新检查。
- 修复当前改动引入的编译器、Clippy 和测试问题。若 workspace 因无关既有故障或环境依赖无法完整通过，保留证据，执行覆盖改动的最小范围并在交付中说明。
