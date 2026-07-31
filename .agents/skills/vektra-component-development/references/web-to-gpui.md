# Web/MDN 到 GPUI 的借鉴边界

可以借鉴 MDN、WAI-ARIA 和 Web 平台的语义契约，但必须翻译成 Rust、GPUI 和 Vektra 的静态 API。

## 概念映射

| Web / React 概念 | Vektra / GPUI 对应 |
| --- | --- |
| `onclick` | `Clickable::on_click` 或组件 builder callback |
| 依赖宿主状态的 handler | `on_click_in` 或调用方直接使用 `cx.listener` |
| `CustomEvent` | `EventEmitter<E>` 与类型化事件 |
| 命令、菜单、快捷键 | `Action` |
| CSS `:hover`、`:active`、`:focus-visible` | GPUI `hover`、`active`、`focus_visible` 样式机制 |
| ARIA button 语义 | GPUI `Role` / AccessKit、可访问名称、键盘行为 |
| controlled component | 显式状态输入和语义回调 |
| React slot / children | 明确 slot builder 或组合现有 `IntoElement` |

## 可以借鉴

- WAI-ARIA 的角色、键盘操作、禁用状态、焦点顺序和可访问名称要求。
- MDN 对控件状态、输入语义和用户预期的描述。
- controlled component 的显式状态和回调契约。
- CSS 状态伪类背后的视觉反馈语义。

## 不要复制

- 字符串事件名称。
- DOM `EventTarget`。
- 完整 capture/bubble 架构。
- JavaScript 类继承。
- 动态属性包。
- React 风格不受控 props 扩散。
- 为 hover、focus 或 arbitrary style 提供没有语义边界的自由透传。

## 转换原则

- 把动态事件名转换为 Rust 类型、trait 或具名 builder。
- 把 DOM 全局事件模型转换为组件局部 callback、Entity 领域事件、Action 或 GPUI 底层交互之一。
- 把 props 扩散转换为显式构造参数、consuming builder、slot 和语义 enum。
- 把 CSS 状态转换为主题 token 和 GPUI 状态样式。
- 把 ARIA 语义转换为 GPUI `Role`、AccessKit、键盘行为和可访问名称。

不要为了接近 Web API 形状而牺牲 Rust 可发现性、编译期检查、主题一致性或 GPUI 的事件模型。
