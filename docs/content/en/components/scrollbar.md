# Scrollbar

Scrollbar turns any GPUI `Div` into a scroll area with Vektra-drawn scrollbars. It needs no public `window`, `cx`, Root, or Provider. Configure layout and children first, then call `.scrollbar()`.

## Basic Usage

<VektraExample demo="scrollbar/basic" title="Scrollbar basic usage" :height="390">

<<< ../../../preview/src/demos/scrollbar.rs#scrollbar-example-basic{rust}

</VektraExample>

`scrollbar()` defaults to `Both + Auto + Overlay`. Both enables scrolling on X and Y, while a track and thumb are drawn only for an axis that actually overflows.

## Visibility, Track, and Gutter

<VektraExample demo="scrollbar/configuration" title="Configure Axis, Visibility, and Gutter" :height="390">

<<< ../../../preview/src/demos/scrollbar.rs#scrollbar-example-configuration{rust}

</VektraExample>

The three Radio groups update one scroll area in place. The demo uses a 1160px-wide canvas that overflows on both X and Y, with a colored header explicitly marking the horizontal direction. It starts at `Both + Always + Overlay`, so both thumbs are immediately visible.

`Auto`, `Always`, and `Never` mean visible during interaction, an always-visible thumb, and no painted scrollbar respectively. Moving the pointer into the scrollbar hit area reveals the full track. Leaving hides the track while the thumb continues to follow its visibility mode. Only hovering the thumb itself switches to the hover color and expands it from the default 8px to 10px.

The live diagram below the Gutter controls enlarges the distinction: `Stable` always reserves the themed 14px `hit-thickness`, giving content and scrollbar separate slots. `Overlay` reserves 0px and paints directly over the content edge.

## API

Import the extension trait first:

```rust
use vektra::ScrollableExt;
```

| API | Semantics |
| --- | --- |
| `.scrollbar()` | `Both + Auto + Overlay`. |
| `.vertical_scrollbar()` / `.horizontal_scrollbar()` | Single-axis shortcuts. |
| `.scrollbar_for(&handle)` | Uses a caller-owned `gpui::ScrollHandle`; both single-axis `*_scrollbar_for` variants are also available. |
| `.scrollbar_with(config)` | Supplies a complete `ScrollbarConfig`. |
| `.scrollbar_with_axis(...)` | Starts with defaults and overrides only the axis. |
| `.scrollbar_with_visibility(...)` | Starts with defaults and overrides only visibility. |
| `.scrollbar_with_gutter(...)` | Starts with defaults and overrides only the gutter. |

`.scrollbar()` returns `ScrollArea`. Its follow-up methods deliberately use the names `.scrollbar_axis(...)`, `.scrollbar_visibility(...)`, `.scrollbar_gutter(...)`, `.scrollbar_id(...)`, and `.scrollbar_aria_label(...)`, avoiding broad names such as `.axis()` or `.visibility()`.

```rust
use vektra::{
    ScrollAxis, ScrollGutter, ScrollVisibility, ScrollableExt,
};

let area = div()
    .h(px(240.))
    .child(content)
    .scrollbar()
    .scrollbar_axis(ScrollAxis::Vertical)
    .scrollbar_visibility(ScrollVisibility::Always)
    .scrollbar_gutter(ScrollGutter::Stable)
    .scrollbar_aria_label("Notifications");
```

`ScrollbarConfig` itself uses short builders because its namespace is already explicit:

```rust
let config = ScrollbarConfig::new()
    .axis(ScrollAxis::Both)
    .visibility(ScrollVisibility::Auto)
    .gutter(ScrollGutter::Overlay);
```

Use `.scrollbar_id(...)` when a loop or shared call site creates multiple scroll areas.

## Interaction and Accessibility

- Mouse-wheel, trackpad, and native GPUI scrolling update the same `ScrollHandle`.
- The thumb is draggable. Clicking a track moves the thumb center to the pointer and begins dragging.
- The track is shown only while its axis is hovered or dragged. Hovering the thumb itself highlights and widens it. Leaving hides the track without hiding the thumb early.
- Once focused, arrow keys move 40px, PageUp/PageDown move roughly 90% of the viewport, and Home/End move to the main-axis edges.
- `Auto` fades in on pointer movement, wheel input, and track interaction, then fades out after roughly 900ms of inactivity. It remains visible during drag and track hover. Transitions are skipped when reduced motion is enabled.
- `.scrollbar_aria_label(...)` names the `ScrollView`; keyboard focus uses the themed focus ring.

Vektra does not expose `window` or `cx` parameters. Keyed state inside the GPUI Element lifecycle owns the internal `ScrollHandle` and short-lived visibility/drag state. `.scrollbar_for(...)` instead reuses a caller-owned handle for virtualized content, external navigation, or state synchronization.

## Theme and Limits

`ResolvedTheme::scrollbar` contains track, default/hover/pressed thumb, focus-ring, default/hover visual-thickness, hit-thickness, minimum-thumb-length, and radius tokens. The thumb always uses a capsule radius derived from its short edge. Defaults are 8px visual thickness, 10px on hover, 14px hit thickness, and a 24px minimum thumb.

- `Never` hides only Vektra's painted scrollbar. Native input and an external `ScrollHandle` can still scroll the content.
- V1 intentionally has no `System` visibility mode. GPUI desktop backends do not currently expose one consistent, dependable system scrollbar preference, so that name would promise behavior the component cannot guarantee.
- Treat `scrollbar()` as the final structural conversion after layout, size, children, and existing interactions. The returned `ScrollArea` exposes only explicit Scrollbar configuration.
