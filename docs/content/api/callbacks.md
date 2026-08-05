# 回调模型

Vektra 的 builder callback 分为两类：

1. `on_click`、`on_change`、`on_focus`、`on_blur` 等标准形式接收 [`App`](./gpui-types#app) 级回调，适合不需要宿主 Entity 状态的逻辑。
2. `on_click_in`、`on_change_in`、`on_focus_in`、`on_blur_in` 接收 `&Context<T>`，内部使用 [`Context::listener`](./gpui-types#contextt) 绑定宿主 Entity，因此 handler 可修改 `&mut T` 并调用 `cx.notify()`。

`_in` 是 Vektra 的 Entity 绑定命名约定。它不是 DOM capture/bubble，也不是 GPUI `on_focus_in` 的子树语义。组件 callback 同步执行。

Entity 销毁后，`Context::listener` 持有的弱引用无法升级，回调安全地不再执行。需要异步工作时，在 Entity 绑定回调中使用 GPUI 任务 API，并由宿主管理任务生命周期。

[`Changeable<T>`](./changeable) 的 `on_change` 表达用户请求的下一受控值，不携带 `ClickEvent`，也不表示组件已经提交状态。

Input 是明确的例外：调用方持有的 `InputState` 还实现 `EventEmitter<InputEvent>`。builder callbacks 和 `Changed`、`Focused`、`Blurred`、`Submitted` 事件由同一语义路径发布，不是两套编辑逻辑；程序化 `set_value`、`clear`、`reset` 均不会产生 `Changed`。
