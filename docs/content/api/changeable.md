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

Checkbox 与 Switch 实现 `Changeable<bool>`，RadioGroup 实现 `Changeable<T>`。变化可能来自鼠标、Space、方向键、Home 或 End，因此回调不携带 `ClickEvent`。宿主可以立即采用下一值，也可以先完成接口审批，再把最终权威值传回组件。

每个组件保留同名固有 builder，并与 trait 实现委托到同一路径。
