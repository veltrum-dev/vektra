# Tooltip

Tooltip provides short, supplementary plain-text help for Button and IconButton. Information required to finish a task must remain available elsewhere. Do not use Tooltip for errors, complex help, validation, or interactive content.

## Basic Usage

<VektraExample demo="tooltip/basic" title="Tooltip basic usage" :height="260">

<<< ../../../preview/src/demos/tooltip.rs#tooltip-example-basic{rust}

</VektraExample>

## Explicit Visibility

<VektraExample demo="tooltip/controlled" title="Tooltip explicit visibility" :height="280">

<<< ../../../preview/src/demos/tooltip.rs#tooltip-example-controlled{rust}

</VektraExample>

## Preferred Placements

<VektraExample demo="tooltip/placements" title="Tooltip preferred placements" :height="340">

<<< ../../../preview/src/demos/tooltip.rs#tooltip-example-placements{rust}

</VektraExample>

## Appearance

### Custom Colors

<VektraExample demo="tooltip/appearance" title="Tooltip custom colors" :height="260">

<<< ../../../preview/src/demos/tooltip.rs#tooltip-example-appearance{rust}

</VektraExample>

### No Arrow

<VektraExample demo="tooltip/no-arrow" title="Tooltip no arrow" :height="260">

<<< ../../../preview/src/demos/tooltip.rs#tooltip-example-no-arrow{rust}

</VektraExample>

## Lifecycle and Escape

<VektraExample demo="tooltip/lifecycle" title="Tooltip visibility lifecycle" :height="280">

<<< ../../../preview/src/demos/tooltip.rs#tooltip-example-lifecycle{rust}

</VektraExample>

## API and Semantics

`Tooltip::new("Settings")` creates a configuration object, while `Tooltip::text("Settings", cx)` creates a GPUI `AnyView` with defaults. Button/IconButton `.tooltip(...)` accepts `&str`, `String`, `SharedString`, and `Tooltip`, so existing `.tooltip("Settings")` calls need no migration. `.tooltip_placement(TooltipPlacement::TopStart)` sets the preferred placement. The default is centered `Bottom`; collision handling may still flip or shift it.

| API | Default and semantics |
| --- | --- |
| `Tooltip::new(text)` | Creates plain-text configuration with automatic triggering, an arrow, and animation enabled. |
| `.open(bool)` | Sets explicit visibility. Omitting it uses hover/keyboard-focus triggering. |
| `.arrow(bool)` | Defaults to `true`. `false` removes both arrow drawing and arrow-height reservation while keeping the anchor gap. |
| `.color(impl Into<Hsla>)` | Overrides the text color for this instance. |
| `.bg_color(impl Into<Hsla>)` | Overrides the bubble and arrow background for this instance. |
| `.animated(bool)` | Defaults to `true`. `false` moves immediately to each visibility end state. |

`color` and `bg_color` accept `gpui::rgb(...)` directly, without `.into()`. Border, shadow, radius, padding, font size, and placement tokens still come from the active theme.

`.open(true).color(...).bg_color(...)` and `.open(true).arrow(false)` are both valid combinations: the first explicitly shows a Tooltip with custom colors, while the second explicitly shows a Tooltip without an arrow.

`aria_label` is the name, `aria_description` is supplementary assistive information, and Tooltip is visual help. Vektra does not copy among them. An icon-only button still requires `aria_label`.

## Lifecycle and Interaction

- Without `open(...)`, hover or keyboard focus created by Tab/Shift+Tab shows after 500ms.
- `open(true)` means that the current trigger explicitly requests its own Tooltip to be visible, so it displays immediately after the trigger mounts without hover/focus. `open(false)` explicitly prevents display and ignores automatic eligibility. Runtime changes use the matching transition, but `open(true)` does not turn the window-level single-Tooltip slot into a multi-Tooltip container.
- Leaving during the initial 500ms delay, blur, or owner removal cancels the task. After display, leaving both the trigger and bubble starts a 500ms close grace period. Entering either region before it expires cancels closing; otherwise the exit transition starts afterward.
- Mouse-created focus does not start the keyboard path.
- Escape dismisses a visible or pending Tooltip without moving trigger focus. Automatic mode requires leaving and re-entering the hover/focus cycle. For `open(true)`, the caller must send `false` and then `true`; unrelated rerenders do not reopen it.
- Hover and focus share one trigger state, so one trigger never draws duplicate Tooltips. Due to the GPUI limit of at most one Tooltip actually drawn per window per frame, callers should not rely on multiple triggers with `open(true)` being visible together in the same window. To show multiple persistent Tooltip appearances, use separate preview windows. If a keyboard-focused trigger and a different hovered trigger are both eligible, pointer input ends the old keyboard eligibility and the hovered trigger takes over.
- A disabled trigger cannot focus or activate, but its hover Tooltip can explain the disabled reason.

The pointer can enter the Tooltip bubble to keep its lifecycle active, but the bubble remains unfocusable, untabbable, unclickable, and free of interactive content. Enter and Space continue to activate the trigger's business callback.

## Placement, Theme, and Performance

Tooltip anchors to the full union of the trigger's prepaint child bounds; it does not compress that rectangle into a mouse coordinate. Vektra measures and places the bubble in the same frame. It tries the preferred placement first, flips to the opposite main-axis side while preserving Start/Center/End when needed, chooses the roomier side when neither fits, and shifts along the cross axis. When enabled, the arrow follows the final side, is recomputed after shifting, and stays outside the rounded-corner safety area. Disabling it removes only arrow space; the themed anchor gap still separates the bubble from the trigger.

| Placement | Alignment semantics |
| --- | --- |
| `TopStart` / `BottomStart` | Bubble left edge aligns to the trigger left edge. |
| `Top` / `Bottom` | Bubble is horizontally centered on the trigger. |
| `TopEnd` / `BottomEnd` | Bubble right edge aligns to the trigger right edge. |
| `LeftStart` / `RightStart` | Bubble top edge aligns to the trigger top edge. |
| `Left` / `Right` | Bubble is vertically centered on the trigger. |
| `LeftEnd` / `RightEnd` | Bubble bottom edge aligns to the trigger bottom edge. |

The bubble, arrow, surface background, foreground, border, `radius.md` corners, and light shadow default to Tooltip, semantic, and foundation tokens. Light, Dark, and System use the active theme. Instance `color`/`bg_color` overrides take priority, but fixed colors do not adapt to theme changes; the caller owns contrast. Long Chinese or English text wraps within the maximum width. In an extremely small viewport that cannot fit the trigger, gap, complete bubble, and shadow safety area together, the algorithm prioritizes visible content and uses best-effort placement, so normal spacing may be impossible.

Tooltip uses the same GPUI placement and lifecycle implementation on macOS, Windows, Linux, and the Web preview. Platform differences mainly come from window bounds, system fonts, and host focus traversal; no alternate component API is required.

The default enter animation is about 120ms, fading in with roughly 2px of travel along the final placement direction; exit fades out over about 80ms. Animation does not affect measurement, final placement, or trigger hit testing. `.animated(false)` renders static end states immediately. GPUI `App::reduce_motion` also suppresses decorative frame scheduling. Only configured triggers create a small keyed state; generation guards and owner lifetime cancel stale show-delay, close-grace, and transition tasks, and invisible content is not laid out. Large lists should be virtualized so state exists only for mounted rows.

## Limits

- Plain text only: no rich text, links, buttons, or arbitrary children.
- Bubble hover only maintains the visibility lifecycle; it does not add clicks, focus, or interactive children. There are no custom animation-duration/easing/transition, border, shadow, radius, padding, offset, or child builders.
- No Root, Provider, global initialization, general Overlay, or public `Tooltipable` trait.
- Due to a GPUI constraint, a window draws at most one Tooltip per frame; Vektra adds no global arbitration layer.
- The host still maps real Tab/Shift+Tab keys to GPUI focus traversal.

## Performance contract

- Normal scale is 100 triggers; stress scale is 1K interactive triggers / 10K configured triggers.
- Per-trigger state, placement, and paint are O(1). Delay/close/animation tasks have one owner, replacement cancels the prior task, and owner removal releases them.
- 1K focus+delay+draw and 10K build coverage live in `coverage/tooltip_icon_focus` / `stress/coverage`; deterministic owner-removal tests gate lifetime release.
