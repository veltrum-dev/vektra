# Radio and RadioGroup

`RadioGroup<T>` selects one value from mutually exclusive options. It is controlled: `selected_value(Option<T>)` supplies the authoritative value and `on_change(T, ...)` only requests the next value. `Radio<T>` can only be added through the group's strongly typed `child(Radio<T>)` and cannot render independently.

## Basic controlled usage

<VektraExample demo="radio/basic" title="RadioGroup basic usage" :height="280">

<<< ../../../preview/src/demos/radio.rs#radio-example-basic{rust}

</VektraExample>

The host may update `selected_value` immediately or wait for server approval. On failure, keep supplying the old value; the component never changes selected visuals optimistically.

## Disabled items and groups

<VektraExample demo="radio/disabled" title="Radio disabled behavior" :height="360">

<<< ../../../preview/src/demos/radio.rs#radio-example-disabled{rust}

</VektraExample>

Arrow navigation skips disabled items. Group-level disabled state overrides every item and removes the group from normal Tab order.

## Keyboard navigation

<VektraExample demo="radio/keyboard" title="RadioGroup keyboard navigation" :height="320">

<<< ../../../preview/src/demos/radio.rs#radio-example-keyboard{rust}

</VektraExample>

## Layout orientation

<VektraExample demo="radio/orientation" title="RadioGroup horizontal layout" :height="260">

<<< ../../../preview/src/demos/radio.rs#radio-example-orientation{rust}

</VektraExample>

## Anatomy

```text
RadioGroup (Role::RadioGroup, orientation, group name and description)
└─ Radio (Role::RadioButton, selected state, one roving focus target)
   ├─ circular indicator and selected dot
   └─ label + optional description
```

Each Radio is one hit target. Labels and descriptions add no extra focus targets. Selection is conveyed by the inner dot and `Toggled::True`, not color alone; focus-visible has a distinct outline.

## API

| API | Description |
| --- | --- |
| `RadioGroup::new(id)` | Creates a vertical group with no selection. |
| `.selected_value(Option<T>)` | Supplies the authoritative selected value. |
| `.child(Radio<T>)` | Adds a strongly typed item; arbitrary `IntoElement` children are rejected. |
| `.on_change(handler)` / `.on_change_in(cx, handler)` | Requests the next value without a `ClickEvent`. |
| `.disabled(bool)` | Disables the whole group and overrides item configuration. |
| `.size(ComponentSize)` | Applies one `Xs`, `Sm`, `Md`, or `Lg` size to the group. |
| `.orientation(Orientation)` | Sets horizontal/vertical layout and accessible orientation. |
| `.aria_label` / `.aria_description` | Sets the group name and description. |
| `Radio::new(id, value)` | Creates a strongly typed item that cannot render alone. |
| `.label` / `.description` | Sets visible text and fallback accessible text. |
| `.aria_label` / `.aria_description` | Overrides item accessibility text. |
| `.disabled(bool)` | Blocks selection and removes the item from arrow navigation. |
| `.on_focus` / `.on_blur` and `_in` | Observes real item focus transitions. |

RadioGroup implements [`Changeable<T>`](/en/api/changeable), [`Disableable`](/en/api/disableable), and [`Sizable`](/en/api/sizable). Radio implements [`Focusable`](/en/api/focusable) and [`Disableable`](/en/api/disableable). Neither is `Clickable`; Radio is not `IntoElement` or `Sizable`, while RadioGroup is not `ParentElement`, `Focusable`, or `Clickable`.

## Keyboard, focus, and disabled behavior

- The group has at most one Tab stop: the enabled selected item, otherwise the first enabled item. An all-disabled group leaves Tab order.
- Up/Left request the previous enabled item; Down/Right request the next, wrapping and skipping disabled items.
- Home/End request the first/last enabled item. Space requests the focused item.
- Pointer activation focuses the item and then uses the same change-request path.
- Reactivating the authoritative selected value neither clears it nor emits a redundant change.
- Group disabled state overrides item state.

## Themes, responsive behavior, and platforms

Light, Dark, and System resolve dedicated Radio tokens for normal, hover, pressed, focus-visible, selected, and disabled states. Dimensions, spacing, borders, and typography also come from theme tokens. Text wraps under narrow constraints while indicators retain logical size. The implementation uses GPUI's cross-platform focus, input, and AccessKit APIs and targets macOS, Windows, Linux, and Web/WASM; no unmeasured pixel-equivalence claim is made.

## Known limitations

- The first version accepts text labels/descriptions only, with no arbitrary child or icon slot.
- There is no uncontrolled value, default value, validation message, async task, or loading state.
- Orientation changes layout and accessibility metadata; all four arrow keys remain active for platform compatibility.
- Desktop example: `cargo run --example radio`.

## Performance contract

- The standard load is 100 visible Radios. Per-item construction/layout/paint is O(1); directional navigation is O(group size).
- RadioGroup retains no history, Element cache, or background task. Large-data single selection should use lazy Select/VirtualList rather than a huge RadioGroup.
- First, steady, and update leaf-wall coverage lives in `component_wall`.
