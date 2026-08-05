# `Disableable`

```rust
pub trait Disableable: Sized {
    fn disabled(self, disabled: bool) -> Self;
}
```

实现组件：[`Checkbox`](/components/checkbox)、[`Switch`](/components/switch)、[`Input`](/components/input)、[`Button`](/components/button)、[`IconButton`](/components/icon-button)。`disabled(true)` 同时阻止鼠标和键盘激活，并离开正常 Tab 顺序；具体颜色、光标、loading/progress 优先级由组件负责。Input 只拥有文本编辑器和内置 clear 的 disabled 状态，不会改写任意 prefix/suffix 子组件。

这是 consuming-builder 契约，没有默认 no-op，也没有 `_in` 版本，因为设置 disabled 是同步输入值而非回调。宿主 Entity 在 render 时把自己的布尔状态传给组件即可。
