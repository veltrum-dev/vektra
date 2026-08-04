# Changeable

`Changeable<T>` is a static builder capability for requesting the next value of a controlled component. It neither commits internal business state nor publishes runtime events.

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

Checkbox and Switch implement `Changeable<bool>`; RadioGroup implements `Changeable<T>`. A change may come from a pointer, Space, arrow keys, Home, or End, so the callback carries no `ClickEvent`. Hosts may adopt the value immediately or wait for server approval before supplying the authoritative value again.

Each component keeps inherent builders with the same names, delegating to the same implementation as the trait.
