# `Focusable`

`Focusable` 注册真实焦点生命周期回调。重新渲染、builder 状态变化或 Checkbox checked 变化不会自行触发回调。

```rust
pub trait Focusable: Sized {
    fn on_focus(
        self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self;

    fn on_blur(
        self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self;

    fn on_focus_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self;

    fn on_blur_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self;
}
```

实现组件：[`Checkbox`](/components/checkbox)、[`Switch`](/components/switch)、[`Input`](/components/input)、[`Button`](/components/button)、[`IconButton`](/components/icon-button)。这些组件也提供同名 inherent forwarding 方法。

<<< ../../preview/src/demos/button.rs#button-focus{rust}

`on_focus_in`/`on_blur_in` 的 `_in` 表示宿主 Entity 绑定，不是“焦点进入子树”，也不等同于 DOM `focusin`。回调同步执行；需要异步工作时由宿主在回调中生成并管理任务。

enabled 组件进入正常 Tab 顺序；disabled 组件不进入。已聚焦组件在当前锁定 GPUI 下变为 disabled 时产生一次 blur。若组件直接从渲染树移除，它的 keyed state 与订阅会先销毁，随后 GPUI 才清除窗口焦点，因此已移除组件不会收到业务 blur。Tooltip 与业务回调复用同一焦点句柄，因此一次转换不会重复调用业务 handler。
