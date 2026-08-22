# GPUI Dependency Types

These are GPUI types, not Vektra types. Vektra uses them at public callback boundaries without copying their full definitions. Every link is pinned to commit `fd82517a115d97a07835b52f0512b22b38e38ccf`.

## `ClickEvent`

Used only by raw activation entries such as `Clickable::on_click`. `Changeable::on_change` carries no click event because changes may also originate from arrow keys, Home, or End. [Pinned source](https://github.com/zed-industries/zed/blob/fd82517a115d97a07835b52f0512b22b38e38ccf/crates/gpui/src/interactive.rs#L281).

## `Window`

The current window's focus, input, drawing, and platform-state entry point. Vektra callbacks receive `&mut Window`. [Pinned source](https://github.com/zed-industries/zed/blob/fd82517a115d97a07835b52f0512b22b38e38ccf/crates/gpui/src/window.rs#L1132).

## `App`

GPUI's application-level context. Standard builder callbacks receive `&mut App`. [Pinned source](https://github.com/zed-industries/zed/blob/fd82517a115d97a07835b52f0512b22b38e38ccf/crates/gpui/src/app.rs#L692).

## `Context<T>`

The update context for Entity `T`. `*_in` methods use `Context::listener` to bind standard callbacks to the host Entity and retain GPUI's weak-reference/no-op behavior after destruction. [Pinned source](https://github.com/zed-industries/zed/blob/fd82517a115d97a07835b52f0512b22b38e38ccf/crates/gpui/src/app/context.rs#L20).

## `Entity<T>` and `SharedString`

Input requires the caller to own an `Entity<InputState>`, keeping editing state stable across renders. `SharedString` is Input's public value/event string type, while `InputState::value()` exposes content as `&str`.

Internally, Input implements the pinned revision's `EntityInputHandler` and connects it through `ElementInputHandler` for platform text input, IME, UTF-16 selection, and range bounds. These are implementation details and require no global handler registration by the host.

This Entity-bound example is compiled into the real WASM preview:

<<< ../../../preview/src/demos/checkbox.rs#checkbox-focus{rust}

If GPUI later publishes stable official docs that match this revision, these links can move there. Until then, pinned source remains the signature authority.
