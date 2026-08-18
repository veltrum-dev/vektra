# Checkbox

Checkbox is a controlled checkbox component for single toggles, bulk selection, and parent mixed states. It is not a form framework, does not store business state internally, and does not provide validation or error-message APIs.

## Basic Usage

<VektraExample demo="checkbox/basic" title="Checkbox basic usage" :height="220">

<<< ../../../preview/src/demos/checkbox.rs#checkbox-example-basic{rust}

</VektraExample>

`Checkbox::new(id)` defaults to unchecked. `.checked(...)` is the current controlled value, not an initial value. Each activation computes the next checked value and calls `on_change`; the host updates state and calls `cx.notify()`.

## Common states

<VektraExample demo="checkbox/states" title="Checkbox common states" :height="300">

<<< ../../../preview/src/demos/checkbox.rs#checkbox-example-states{rust}

</VektraExample>

Checked, indeterminate, and disabled are explicit states. Indeterminate visual and accessibility semantics take priority over checked.

## Bulk Selection

<VektraExample demo="checkbox/bulk" title="Checkbox bulk selection" :height="340">

<<< ../../../preview/src/demos/checkbox.rs#checkbox-example-bulk{rust}

</VektraExample>

A parent Checkbox can derive `checked` and `indeterminate` from child items: checked when every item is selected, mixed when only some items are selected. Activating the parent writes one value to every child item; bulk actions such as invert selection live in host state and do not need extra Checkbox API.

## Icon-only State

<VektraExample demo="checkbox/icon-only" title="Checkbox icon-only state" :height="220">

<<< ../../../preview/src/demos/checkbox.rs#checkbox-example-icon-only{rust}

</VektraExample>

`indicator_icons(unchecked, checked)` replaces the default box with two state icons. An icon-only Checkbox omits the visible label but must provide an accessible name through `aria_label(...)`. The full hit area triggers hover and pressed feedback, while that feedback is still drawn only on the state icon, without a square container or border.

## Size

<VektraExample demo="checkbox/sizes" title="Checkbox semantic sizes" :height="240">

<<< ../../../preview/src/demos/checkbox.rs#checkbox-example-sizes{rust}

</VektraExample>

## API

| API | Description |
| --- | --- |
| `Checkbox::new(id)` | Creates a checkbox with a stable `ElementId`. |
| `.checked(bool)` | Sets the current controlled checked value. Defaults to `false`. |
| `.indeterminate(bool)` | Sets the mixed state; visual and accessibility state take priority over checked. |
| `.disabled(bool)` | Blocks mouse, touch, and keyboard activation. |
| `.label(text)` | Sets visible text and uses it as the default accessible name. |
| `.size(ComponentSize)` | Sets an explicit size; otherwise reads the global default size. |
| `.cursor_style(CursorStyle)` | Sets the enabled cursor; disabled state takes priority. |
| `.unchecked_icon(icon)` | Overrides the unchecked icon. Defaults to no icon. |
| `.checked_icon(icon)` | Overrides the checked icon. Defaults to a check mark. |
| `.indeterminate_icon(icon)` | Overrides the mixed icon. Defaults to a horizontal line. |
| `.indicator_icons(unchecked, checked)` | Replaces the default box indicator with unchecked/checked icons. |
| `.aria_label(text)` | Overrides or provides the accessible name. |
| `.aria_description(text)` | Provides supplementary accessibility text. |
| `.on_change(handler)` | Synchronous callback with the next checked value, `Window`, and `App`; it carries no `ClickEvent`. |
| `.on_change_in(cx, handler)` | Synchronous callback bound to a host Entity. |
| `.on_focus(handler)` / `.on_blur(handler)` | Registers callbacks for real focus and blur transitions. |
| `.on_focus_in(cx, handler)` / `.on_blur_in(cx, handler)` | Registers focus callbacks bound to a host Entity. |

## State

Unchecked activates to `true`; checked activates to `false`. `indeterminate(true)` always activates to `true`, and the host usually clears indeterminate at the same time.

The root node uses `Role::CheckBox` and maps to `Toggled::False`, `Toggled::True`, or `Toggled::Mixed`. If there is no visible label, provide `aria_label(...)`.

## Keyboard And Interaction

Enabled Checkbox can be focused with Tab. Space activates; Enter does not. The box, label, and root padding share one hit area; hovering or pressing anywhere in it drives the same internal indicator feedback, and one activation calls the callback once. Disabled Checkbox leaves the normal Tab order, does not call the handler, does not respond to hover or pressed states, and uses disabled visuals and cursor.

`on_change` and the focus lifecycle are independent: checked/indeterminate changes do not produce focus/blur, and focus transitions do not call `on_change`. Rerendering the same `ElementId` uses the latest focus handler. `_in` means Entity binding, not DOM `focusin`; see [`Focusable`](/en/api/focusable). The current mouse activation path with `on_change` prevents GPUI's default pointer focus transfer. Tab and real GPUI programmatic focus transitions still invoke the callbacks. This release adds no `focus()` or `focus_handle()` API.

`ComponentSize::{Xs, Sm, Md, Lg}` is the shared semantic size enum. `component_size(cx)` reads the global default; `set_component_size(size, cx)` updates it and refreshes windows. Explicit `.size(...)` takes priority over the global default.

## Async Work

`on_change` and `on_change_in` are synchronous. For HTTP requests or async validation, start work inside the callback with the host Entity's `cx.spawn` / `cx.spawn_in`, and store the returned `Task` when lifecycle management needs it.

`Checkbox` implements [`Changeable<bool>`](/en/api/changeable); its inherent builders and trait calls delegate to the same implementation.

## Themes, Responsive Behavior, and Platforms

Checkbox normal, hover, pressed, focus-visible, checked, mixed, and disabled states come from the active Light, Dark, or System theme tokens. The component keeps a compact single-row hit target; the host layout controls wrapping and column width in narrow containers, and icon-only mode still requires an `aria_label`. macOS, Windows, Linux, and the Web preview share the same GPUI implementation, with platform differences limited to system focus traversal, fonts, and input mapping.

## Known Limits

- The first label API accepts plain text only.
- There is no uncontrolled state, `default_checked`, validation, error message, or FormControl API.
- Custom icons are visual only and do not create extra accessible names.

## Performance contract

- The standard load is 100 visible Checkboxes; construction, state resolution, layout, and paint are O(1).
- No row cache, history, or background task is retained. Keyed focus/subscription count must remain stable across warm rerenders.
- Use [`VirtualList`](/en/components/virtual-list) for 10K/100K data; leaf-wall coverage lives in `component_wall`.
