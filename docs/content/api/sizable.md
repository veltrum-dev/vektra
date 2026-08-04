# `Sizable`

```rust
pub trait Sizable: Sized {
    fn size(self, size: ComponentSize) -> Self;
}
```

实现组件：[`Checkbox`](/components/checkbox)、[`Switch`](/components/switch)、[`Button`](/components/button)、[`IconButton`](/components/icon-button)。显式尺寸优先于 `component_size(cx)` 返回的全局默认值；每个组件负责把 `ComponentSize::{Xs, Sm, Md, Lg}` 映射到自己的主题 token。

`Sizable` 没有 `_in` 版本：尺寸是 render 输入，不是生命周期回调。调用 `set_component_size(size, cx)` 会修改全局默认并刷新窗口，已显式设置 `.size(...)` 的实例不受影响。
