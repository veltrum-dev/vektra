# `Clickable`

`Clickable` is the static builder capability shared by Button and IconButton. It is not a cross-Entity event bus.

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

Implementors: [`Button`](/en/components/button) and [`IconButton`](/en/components/icon-button). `on_click` accepts a standard GPUI callback. `on_click_in` binds the host Entity through `Context::listener`; after that Entity is destroyed, GPUI's weak-listener behavior safely becomes a no-op.

<<< ../../../preview/src/demos/button.rs#button-basic{rust}

Disabled components, and Button loading/progress, block activation. Component pages define exact mouse, Enter, and Space timing. [`ClickEvent`](./gpui-types#clickevent) belongs to GPUI.
