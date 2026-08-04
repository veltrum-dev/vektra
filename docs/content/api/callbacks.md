# 回调模型

Vektra 的 builder callback 分为两类：

1. `on_click`、`on_focus`、`on_blur` 等标准形式接收 [`App`](./gpui-types#app) 级回调，适合不需要宿主 Entity 状态的逻辑。
2. `on_click_in`、`on_focus_in`、`on_blur_in` 接收 `&Context<T>`，内部使用 [`Context::listener`](./gpui-types#contextt) 绑定宿主 Entity，因此 handler 可修改 `&mut T` 并调用 `cx.notify()`。

`_in` 是 Vektra 的 Entity 绑定命名约定。它不是 DOM capture/bubble，也不是 GPUI `on_focus_in` 的子树语义。组件 callback 同步执行，不会发布字符串事件或进入通用运行时事件总线。

Entity 销毁后，`Context::listener` 持有的弱引用无法升级，回调安全地不再执行。需要异步工作时，在 Entity 绑定回调中使用 GPUI 任务 API，并由宿主管理任务生命周期。
