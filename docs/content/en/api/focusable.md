# `Focusable`

`Focusable` registers real focus lifecycle callbacks. Rerendering, changing builder state, or changing Checkbox checked state does not fire them by itself.

```rust
pub trait Focusable: Sized {
    fn on_focus(
        self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self;
    fn on_blur(
        self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self;
    fn on_focus_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self;
    fn on_blur_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self;
}
```

Implementors: [`Checkbox`](/en/components/checkbox), [`Switch`](/en/components/switch), [`Button`](/en/components/button), and [`IconButton`](/en/components/icon-button). All expose inherent forwarding methods.

<<< ../../../preview/src/demos/button.rs#button-focus{rust}

The `_in` suffix on `on_focus_in`/`on_blur_in` means host Entity binding. It does not mean focus entering a subtree and is not DOM `focusin`. Callbacks run synchronously.

Enabled components remain in normal Tab order; disabled components do not. Under the pinned GPUI revision, making a focused component disabled produces one blur. Direct removal destroys its keyed state and subscriptions before GPUI later clears window focus, so the removed component receives no business blur. Tooltip and business handlers share one focus handle, so one transition does not duplicate business callbacks.
