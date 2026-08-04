# Callback Model

Vektra builder callbacks have two forms:

1. Standard `on_click`, `on_focus`, and `on_blur` forms receive [`App`](./gpui-types#app)-level callbacks and suit logic that does not need host Entity state.
2. `on_click_in`, `on_focus_in`, and `on_blur_in` receive a `&Context<T>`. They use [`Context::listener`](./gpui-types#contextt) internally, so handlers can mutate `&mut T` and call `cx.notify()`.

`_in` is Vektra's host-Entity binding convention. It is neither DOM capture/bubble nor GPUI's subtree `on_focus_in` semantics. Component callbacks execute synchronously; they do not publish string events or enter a generic runtime event bus.

After the host Entity is destroyed, the weak reference held by `Context::listener` no longer upgrades, so the callback safely stops running. Start asynchronous work from an Entity-bound callback with GPUI task APIs and let the host own task lifecycle.
