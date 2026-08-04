# API Reference

This section documents the public surface of the root `vektra` crate. VitePress explains concepts, behavior, and compiled examples; <a href="../../api/rust/vektra/">the complete Rust API (rustdoc)</a> is the exhaustive symbol reference.

## API Ownership

- `vektra::Button`, `Checkbox`, `IconButton`, `Tooltip`, capability traits, theme APIs, and shared types are Vektra APIs documented here and in rustdoc.
- `gpui::ClickEvent`, `Window`, `App`, and `Context<T>` are GPUI APIs. Vektra uses them in callback signatures without copying their definitions; use the [GPUI dependency type index](./gpui-types) for pinned source links.
- `gpui` and `gpui_platform` are pinned to commit `82aef44308540b576e4e51fb379efa71614e5c91`. Links never target a drifting `main` branch.

## Navigation

- Capability traits: [Clickable](./clickable), [Focusable](./focusable), [Disableable](./disableable), [Sizable](./sizable)
- [Callback model and the `_in` convention](./callbacks)
- [Vektra public type index](./public-types)
- [GPUI dependency type index](./gpui-types)

Prefer the root `vektra` facade for application code. Internal workspace crates only need separate treatment for advanced direct-use cases such as asset composition.
