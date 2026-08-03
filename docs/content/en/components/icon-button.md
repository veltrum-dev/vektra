# IconButton

IconButton is a fixed square, icon-only action for toolbars, compact title bars, and familiar actions with clear surrounding context. Use another component when visible text, complex content, or link navigation is required.

## Live Preview

<VektraPreview demo="icon-button/basic" title="IconButton preview" :height="360" />

The source below is compiled into the shared GPUI WASM runtime:

<<< ../../../preview/src/demos/icon_button.rs{rust}

## Anatomy and API

The root supplies Button role, a square hit area, themed states, and one Tab stop. The inner `Icon` is decorative and does not create another name or focus target.

| API | Description |
| --- | --- |
| `IconButton::new(id, icon)` | Creates an icon-only button with a stable `ElementId`. |
| `.aria_label(text)` | Sets the required accessible name; a visual Tooltip cannot replace it. |
| `.aria_description(text)` | Sets supplementary information for assistive technology. |
| `.tooltip(text_or_tooltip)` | Accepts a string or `Tooltip` configuration with `open`, arrow, color, and animation options. |
| `.tooltip_placement(TooltipPlacement)` | Sets the preferred Tooltip placement; defaults to `Bottom` with automatic flip/shift. |
| `.variant(...)` | `Primary`, `Outline`, `Ghost`, `Destructive`, or `Secondary`. |
| `.size(...)` | `Xs` 24px, `Sm` 32px, `Md` 36px (default), or `Lg` 40px. |
| `.icon_color(color)` | Overrides enabled icon color only; disabled tokens still win. |
| `.disabled(bool)` | Blocks mouse/keyboard activation and leaves the Tab order. |
| `.on_click(...)` / `.on_click_in(...)` | Registers the shared mouse, Enter, and Space activation contract. |

## States, Keyboard, and Accessibility

Normal, hover, pressed, focus-visible, and disabled use the Button theme matrix. The host wires Tab/Shift+Tab to GPUI focus traversal. Enter activates on keydown and Space on keyup. A string Tooltip appears after 500ms of hover or keyboard focus; configuration supports immediate `open(true)` or forced `open(false)`. Blur starts the exit transition. Escape dismisses without moving focus, and a controlled true value must change `false -> true` to reopen. Mouse-created focus does not start the keyboard Tooltip path.

Every icon-only button must have an `aria_label`. `aria_description` is supplementary semantics and Tooltip is visual help; Vektra does not copy between them. A disabled IconButton cannot focus or activate, but hover Tooltip remains available to explain why it is disabled.

## Theme, Responsive Behavior, and Platforms

Light, Dark, and System resolve semantic tokens. Fixed Tooltip instance colors remain the caller's theme and contrast responsibility, while default Tooltip motion respects GPUI reduced motion. IconButton uses logical pixels and SVG for high-DPI output and keeps its square size in narrow parents. Desktop and WASM use the same GPUI component path. The host owns application shortcuts, Tab Actions, and platform window lifecycle.

## Current Limits

- No visible label or `Link` variant.
- Tooltip is plain text and does not replace the accessible name.
- No arbitrary padding, radius, background, or hit-area styling pass-through.
- Pointer visuals still need real pointer verification; the preview requires WebGPU and host-provided Chinese fonts.
