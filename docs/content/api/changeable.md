# Changeable

`Changeable<T>` 是受控组件请求下一值的静态 builder 能力。它不提交内部业务状态，也不发布运行时事件。

```rust
pub trait Changeable<T>: Sized {
    fn on_change(self, handler: impl Fn(T, &mut Window, &mut App) + 'static) -> Self;
    fn on_change_in<U: 'static>(
        self,
        cx: &Context<U>,
        handler: impl Fn(&mut U, T, &mut Window, &mut Context<U>) + 'static,
    ) -> Self;
}
```

Checkbox 与 Switch 实现 `Changeable<bool>`，RadioGroup 实现 `Changeable<T>`，Input 实现 `Changeable<SharedString>`。Input 的 `on_change` 只表示用户编辑实际改变了值；程序化 `InputState::set_value`、`clear` 和 `reset` 不调用它。变化可能来自键盘、鼠标或平台文本输入协议，因此回调不携带 `ClickEvent`。

每个组件保留同名固有 builder，并与 trait 实现委托到同一路径。
