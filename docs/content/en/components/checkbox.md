# Checkbox

Checkbox is a controlled checkbox component for single toggles, bulk selection, and parent mixed states. It is not a form framework, does not store business state internally, and does not provide validation or error-message APIs.

<VektraPreview demo="checkbox/basic" title="Checkbox preview" :height="620" />

## Basic Usage

<<< ../../../preview/src/demos/checkbox.rs#checkbox-state{rust}

<<< ../../../preview/src/demos/checkbox.rs#checkbox-basic{rust}

`Checkbox::new(id)` defaults to unchecked. `.checked(...)` is the current controlled value, not an initial value. Each activation computes the next checked value and calls `on_change`; the host updates state and calls `cx.notify()`.

## Bulk Selection

<<< ../../../preview/src/demos/checkbox.rs#checkbox-bulk{rust}

A parent Checkbox can derive `checked` and `indeterminate` from child items: checked when every item is selected, mixed when only some items are selected. Activating the parent writes one value to every child item; bulk actions such as invert selection live in host state and do not need extra Checkbox API.

## Icon-only State

<<< ../../../preview/src/demos/checkbox.rs#checkbox-icon-only{rust}

`indicator_icons(unchecked, checked)` replaces the default box with two state icons. An icon-only Checkbox omits the visible label but must provide an accessible name through `aria_label(...)`. The full hit area remains clickable, while hover and pressed visuals apply only to the state icon.

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
| `.on_change(handler)` | Synchronous callback with the next checked value and GPUI [`ClickEvent`](/en/api/gpui-types#clickevent). |
| `.on_change_in(cx, handler)` | Synchronous callback bound to a host Entity. |
| `.on_focus(handler)` / `.on_blur(handler)` | Registers callbacks for real focus and blur transitions. |
| `.on_focus_in(cx, handler)` / `.on_blur_in(cx, handler)` | Registers focus callbacks bound to a host Entity. |

## State

Unchecked activates to `true`; checked activates to `false`. `indeterminate(true)` always activates to `true`, and the host usually clears indeterminate at the same time.

The root node uses `Role::CheckBox` and maps to `Toggled::False`, `Toggled::True`, or `Toggled::Mixed`. If there is no visible label, provide `aria_label(...)`.

## Keyboard And Interaction

Enabled Checkbox can be focused with Tab. Space activates; Enter does not. The label and box share one hit target, so one activation calls the callback once. Disabled Checkbox leaves the normal Tab order, does not call the handler, and uses disabled visuals and cursor.

`on_change` and the focus lifecycle are independent: checked/indeterminate changes do not produce focus/blur, and focus transitions do not call `on_change`. Rerendering the same `ElementId` uses the latest focus handler. `_in` means Entity binding, not DOM `focusin`; see [`Focusable`](/en/api/focusable). The current mouse activation path with `on_change` prevents GPUI's default pointer focus transfer. Tab and real GPUI programmatic focus transitions still invoke the callbacks. This release adds no `focus()` or `focus_handle()` API.

## Size

`ComponentSize::{Xs, Sm, Md, Lg}` is the shared semantic size enum. `component_size(cx)` reads the global default; `set_component_size(size, cx)` updates it and refreshes windows. Explicit `.size(...)` takes priority over the global default.

## Async Work

`on_change` and `on_change_in` are synchronous. For HTTP requests or async validation, start work inside the callback with the host Entity's `cx.spawn` / `cx.spawn_in`, and store the returned `Task` when lifecycle management needs it.

## Known Limits

- The first label API accepts plain text only.
- There is no uncontrolled state, `default_checked`, validation, error message, or FormControl API.
- Custom icons are visual only and do not create extra accessible names.
