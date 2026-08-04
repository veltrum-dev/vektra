# GPUI Dependency Types

These are GPUI types, not Vektra types. Vektra uses them at public callback boundaries without copying their full definitions. Every link is pinned to commit `82aef44308540b576e4e51fb379efa71614e5c91`.

## `ClickEvent`

Used by `Clickable::on_click` and `Checkbox::on_change` to distinguish mouse and keyboard activation. [Pinned source](https://github.com/zed-industries/zed/blob/82aef44308540b576e4e51fb379efa71614e5c91/crates/gpui/src/interactive.rs#L281).

## `Window`

The current window's focus, input, drawing, and platform-state entry point. Vektra callbacks receive `&mut Window`. [Pinned source](https://github.com/zed-industries/zed/blob/82aef44308540b576e4e51fb379efa71614e5c91/crates/gpui/src/window.rs#L1073).

## `App`

GPUI's application-level context. Standard builder callbacks receive `&mut App`. [Pinned source](https://github.com/zed-industries/zed/blob/82aef44308540b576e4e51fb379efa71614e5c91/crates/gpui/src/app.rs#L679).

## `Context<T>`

The update context for Entity `T`. `*_in` methods use `Context::listener` to bind standard callbacks to the host Entity and retain GPUI's weak-reference/no-op behavior after destruction. [Pinned source](https://github.com/zed-industries/zed/blob/82aef44308540b576e4e51fb379efa71614e5c91/crates/gpui/src/app/context.rs#L20).

This Entity-bound example is compiled into the real WASM preview:

<<< ../../../preview/src/demos/checkbox.rs#checkbox-focus{rust}

If GPUI later publishes stable official docs that match this revision, these links can move there. Until then, pinned source remains the signature authority.
