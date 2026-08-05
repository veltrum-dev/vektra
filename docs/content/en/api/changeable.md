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

Checkbox and Switch implement `Changeable<bool>`, RadioGroup implements `Changeable<T>`, and Input implements `Changeable<SharedString>`. Input calls `on_change` only when a user edit actually changes the value; programmatic `InputState::set_value`, `clear`, and `reset` do not call it. Changes may originate in keyboard, pointer, or platform text-input paths, so the callback carries no `ClickEvent`.

Each component keeps inherent builders with the same names, delegating to the same implementation as the trait.
