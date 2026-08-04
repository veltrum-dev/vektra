# `Clickable`

`Clickable` 是 Button、IconButton 与 Switch 共享的静态 builder 能力，不是跨 Entity 事件总线。

```rust
pub trait Clickable: Sized {
    fn on_click(
        self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self;

    fn cursor_style(self, cursor_style: CursorStyle) -> Self;

    fn on_click_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(
            &mut T,
            &ClickEvent,
            &mut Window,
            &mut Context<T>,
        ) + 'static,
    ) -> Self;
}
```

实现组件：[`Button`](/components/button)、[`IconButton`](/components/icon-button) 与 [`Switch`](/components/switch)。`on_click` 接收标准 GPUI 回调；`on_click_in` 通过 `Context::listener` 绑定宿主 Entity。Entity 销毁后 listener 按 GPUI 弱引用语义变为 no-op。Switch 可用该入口先请求后台接口，成功后再更新受控 checked；它不会自动切换状态。

<<< ../../preview/src/demos/button.rs#button-basic{rust}

disabled、Button loading/progress 会阻止激活。鼠标、Enter 与 Space 的精确触发时机由组件页说明；[`ClickEvent`](./gpui-types#clickevent) 来自 GPUI。
