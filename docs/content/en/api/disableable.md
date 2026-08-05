# `Disableable`

```rust
pub trait Disableable: Sized {
    fn disabled(self, disabled: bool) -> Self;
}
```

Implementors: [`Checkbox`](/en/components/checkbox), [`Switch`](/en/components/switch), [`Input`](/en/components/input), [`Button`](/en/components/button), and [`IconButton`](/en/components/icon-button). `disabled(true)` blocks mouse and keyboard activation and removes the component from normal Tab order. Input owns only its editor and built-in clear state; it does not rewrite arbitrary prefix or suffix children.

This consuming-builder contract has no default no-op and no `_in` form: disabled is a synchronous render input, not a callback. A host Entity passes its boolean state during render.
