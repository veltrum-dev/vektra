# `Sizable`

```rust
pub trait Sizable: Sized {
    fn size(self, size: ComponentSize) -> Self;
}
```

Implementors: [`Checkbox`](/en/components/checkbox), [`Switch`](/en/components/switch), [`Input`](/en/components/input), [`Button`](/en/components/button), and [`IconButton`](/en/components/icon-button). An explicit size overrides the global default returned by `component_size(cx)`. Each component maps `ComponentSize::{Xs, Sm, Md, Lg}` to its own theme tokens; Input uses heights of 24, 32, 36, and 40 px.

There is no `_in` form because size is a render input, not a lifecycle callback. `set_component_size(size, cx)` updates the global default and refreshes windows; explicitly sized instances remain unchanged.
