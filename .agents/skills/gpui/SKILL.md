---
name: gpui
description: 用于在 Vektra 中编写、修改、审查或调试 GPUI 代码，涵盖状态与 Entity 生命周期、异步任务、事件、Action、焦点、布局、底层绘制和测试。处理 GPUI API、窗口交互、自定义 Element 或相关 `.rs` 文件时使用；可见控件应先检查当前仓库和已锁定依赖中实际存在的组件，不假设额外组件库可用。
---

# Vektra GPUI 开发

## 职责边界

- `gpui` 负责状态、生命周期、异步、事件、焦点、布局和底层绘制机制。
- 开始实现前检查 Vektra 的 `Cargo.toml`、锁定的 GPUI 版本、现有 import 和组件目录；只使用当前依赖版本真实提供的 API。
- 按钮、输入、选择器、Tabs、Menu、Dialog、Table、List 等语义控件优先复用仓库已有组件或当前 GPUI 依赖提供的组件。仓库没有对应抽象时，使用最小、清晰的 GPUI 实现，不凭旧示例引入或虚构组件库。
- 不要因为底层 `Element` 更熟悉就绕过已经存在的可复用控件；也不要为了一个局部控件提前建立完整组件系统。
- `div()`、`h_flex()`、`v_flex()` 等 Element 适用于页面结构、间距和无控件语义的内容组合，不适合重写标准控件的 hover、focus、selected、disabled、loading 或键盘导航。
- `h_flex()`、`v_flex()` 等辅助函数只有在当前代码或依赖确实提供时才能使用；否则使用 GPUI 核心布局 API。
- `references/element*.md` 只在普通布局和现有控件无法满足绘制、性能或特殊交互要求时加载。

## 导航

根据任务加载相关参考文件：

| 主题 | 文件 | 加载时机 |
|------|------|----------|
| Action 与快捷键 | [action.md](references/action.md) | 使用 `actions!`、`bind_keys`、`on_action`、`key_context` 时 |
| 异步与后台任务 | [async.md](references/async.md) | 使用 `cx.spawn`、`background_spawn`、`Task`、异步 I/O 时 |
| 上下文管理 | [context.md](references/context.md) | 使用 `App`、`Window`、`Context<T>`、`AsyncApp` 时 |
| 自定义元素（底层） | [element.md](references/element.md) | 普通布局和已有控件无法满足需求，并需要实现 `Element` trait、`request_layout`、`prepaint`、`paint` 时 |
| Entity 状态 | [entity.md](references/entity.md) | 使用 `Entity<T>`、`WeakEntity` 或管理状态时 |
| 事件与订阅 | [event.md](references/event.md) | 使用 `cx.emit`、`cx.subscribe`、`cx.observe` 时 |
| 焦点与键盘导航 | [focus-handle.md](references/focus-handle.md) | 使用 `FocusHandle`、`track_focus` 或 Tab 导航时 |
| 全局状态 | [global.md](references/global.md) | 使用 `Global` trait、`cx.global`、`cx.update_global` 或应用级配置时 |
| 布局与样式 | [layout-style.md](references/layout-style.md) | 使用 `div()`、`h_flex()`、`v_flex()`、Flexbox、溢出或定位做普通布局时；不要用它重写标准控件 |
| ElementId | [element-id.md](references/element-id.md) | 使用 `ElementId`、`.id()`、唯一性规则或有状态元素时 |
| 测试 | [test.md](references/test.md) | 使用 `#[gpui::test]`、`TestAppContext`、`VisualTestContext` 或 `VisualTestAppContext` 时 |

## 扩展参考

深入研究以下主题时，可加载对应的扩展参考文件。Element 扩展参考只在纯 GPUI 自定义已经有能力缺口证据时加载。

**Element trait：**

- [element-api.md](references/element-api.md) — 完整 API、命中区域系统和事件处理
- [element-patterns.md](references/element-patterns.md) — 文本、交互、容器和复合元素模式
- [element-examples.md](references/element-examples.md) — 文本、交互和复杂元素的完整示例
- [element-best-practices.md](references/element-best-practices.md) — 性能、状态和常见陷阱
- [element-advanced.md](references/element-advanced.md) — 瀑布流/环形布局、异步更新和虚拟列表

**Entity 管理：**

- [entity-api.md](references/entity-api.md) — 完整 Entity API、方法和生命周期
- [entity-patterns.md](references/entity-patterns.md) — 模型-视图、跨 Entity 通信和观察者模式
- [entity-best-practices.md](references/entity-best-practices.md) — 内存、性能和生命周期
- [entity-advanced.md](references/entity-advanced.md) — 集合、注册表、防抖和状态机

**测试：**

- [test-examples.md](references/test-examples.md) — 测试示例与模式
- [test-reference.md](references/test-reference.md) — 完整测试 API 参考
