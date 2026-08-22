# 版本与发布政策

Vektra 使用 GitHub Project、Issue 和 Milestone 分别管理长期路线、可执行能力和发布承诺。本政策说明版本号表达的承诺，以及工作如何进入一次发布。

## Select 数据源迁移

性能架构治理后，`Select<T>` 的约束由 `Clone + PartialEq` 收紧为
`Clone + Eq + Hash + 'static`。业务 enum 通常只需增加 `#[derive(Eq, Hash)]`。逐项
`.option()`、`.group()` 和 `.items()` 仍可使用，但现在通过 `OwnedSelectDataSource` 进入同一
虚拟内核；百万级、分页或远程数据改用 `.data_source(Rc<dyn SelectDataSource<T>>)`，并实现
key/value 定位、enabled navigation、typeahead 与 range request。

## `0.1.0` 的含义

`0.1.0` 是 Vektra 首个功能完善、文档齐全并经过验证的公开版本。进入该版本的组件必须具备完整、可组合的公开 API，并覆盖适用的状态、键盘与焦点行为、无障碍语义、主题、文档、示例和确定性测试。

发布范围内不得遗留 P0/P1 级正确性、键盘、焦点、无障碍、主题或文档缺口。`0.1.0` 不要求实现路线图中的所有未来组件；没有成为发布承诺的能力继续由 Roadmap 追踪。

## `0.x` 与 `1.0.0`

`0.x` 表示公共 API 仍可能随 GPUI 演进而发生破坏性变化，不表示质量低、组件残缺或缺少验证。每个已发布组件仍须满足对应版本声明的质量标准。

`1.0.0` 表示项目开始承诺长期公共 API 稳定性。它不是基础质量首次达标的版本，也不意味着此后永远没有破坏性变更；需要破坏兼容性时将遵循相应的主版本升级规则。

## GPUI 兼容性

GPUI 尚未稳定，Vektra 在 workspace 根 `Cargo.toml` 中锁定具体 revision。该 revision 是当前 GPUI API 与兼容性判断的事实来源。升级 revision 可能要求同步调整 Vektra 的实现或公共 API，因此使用方应让 `gpui` 与 `gpui_platform` 跟随 Vektra 当前锁定的同一 revision。

任何面向使用方的破坏性变更都必须在 changelog 或 release notes 中明确记录，包括迁移影响和必要的替代方式。

## 规划对象的职责

- **Vektra Roadmap Project**：组织跨版本的长期方向、优先级、目标发布和执行状态；不替代 Issue。
- **Issue**：记录单项可执行、可验收的能力或缺口，并保存讨论、依赖和关联关系。
- **Milestone**：表示已经进入具体发布范围的承诺。没有确认日期时不设置截止日期，未来想法也不会仅因出现在 Roadmap 中自动进入 Milestone。

发布范围变化必须通过 Issue 与 Milestone 保持可追踪；尚未安排的工作保留在 Roadmap 的 Future/Unscheduled 视图中，不使用源码 TODO 或聊天记录作为唯一记录。
