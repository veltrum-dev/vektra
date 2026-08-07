# Button

Button represents an action such as saving, confirming, changing a mode, or starting a long-running task. It is a plain GPUI element and is exported as `vektra::Button`.

Do not use Button for status-only text, standalone progress panels, or composite list items that need arrow-key selection. Icon-only actions should use `IconButton` with an accessible name.

## Basic Usage

Set the label with `.label(...)`. Handle activation with `.on_click(...)` or `.on_click_in(...)`.

<VektraExample demo="button/basic" title="Button basic usage" :height="240">

<<< ../../../preview/src/demos/button.rs#button-example-basic{rust}

</VektraExample>

## Controlled States

`Button::on_click_in(cx, ...)` gives the handler access to the host `&mut T`, `ClickEvent`, `Window`, and `Context<T>`. Call `cx.notify()` after updating host state.

<VektraExample demo="button/states" title="Button controlled states" :height="280">

<<< ../../../preview/src/demos/button.rs#button-example-states{rust}

</VektraExample>

## Loading, Selected, and Progress

`.loading(bool)`, `.progress(f32)`, and `.selected(bool)` are controlled inputs. Button does not start asynchronous work, calculate progress, or toggle selected after a click; the host updates state and calls `cx.notify()`.

- Loading and progress share one mutually exclusive activity state. The later builder wins, and `.loading(false)` returns to idle.
- Loading replaces the start icon with a rotating indicator while preserving the original label and end icon. GPUI `AnimationExt` automatically respects reduced motion.
- Progress preserves both icon slots and the label, and draws a bottom bar without changing external dimensions.
- Selected is independent from activity. Toggle semantics are only exposed after `.selected(false|true)` is called. A persistent inner outline means selection is not conveyed by color alone.
- Disabled has the highest priority: it uses disabled styling and leaves the Tab order. An activity indicator remains visible, but activation stays blocked.

During loading/progress, Button keeps focus and `Role::Button`, but consumes mouse, Enter, and Space events to prevent duplicate submission and parent activation. Use a separate Button for cancellation.

## Variants and Sizes

Button provides 6 `ButtonVariant` values and 4 `ComponentSize` values. A disabled button uses the disabled token for its current variant.
The ordinary variant rows in the preview use the default `Md` size. The separate size comparison area intentionally shows `Xs`, `Sm`, `Md`, and `Lg`; the height differences are not rendering inconsistencies.

<VektraExample demo="button/variants" title="Button visual variants" :height="280">

<<< ../../../preview/src/demos/button.rs#button-example-variants{rust}

</VektraExample>

## Icons

Start icons, end icons, paired icons, fixed width, and full-width layout use the same `Button` API.

<VektraExample demo="button/icons" title="Button icon slots" :height="240">

<<< ../../../preview/src/demos/button.rs#button-example-icons{rust}

</VektraExample>

## Chinese Auto Spacing

Chinese auto spacing is enabled by default. It can be enabled or disabled explicitly. Long labels, mixed Chinese/English text, and mixed numeric text are not rewritten.

<VektraExample demo="button/auto-space" title="Button automatic CJK spacing" :height="240">

<<< ../../../preview/src/demos/button.rs#button-example-auto-space{rust}

</VektraExample>

## Width

Button sizes itself to its content by default. `.width(...)` sets a fixed width. `.full_width()` uses the full width offered by the parent layout. Both methods write the same width state, so the later call wins.

<VektraExample demo="button/width" title="Button width control" :height="280">

<<< ../../../preview/src/demos/button.rs#button-example-width{rust}

</VektraExample>

## Capability Traits

| Trait | Contract |
| --- | --- |
| [`Clickable`](/en/api/clickable) | Provides `on_click(...)` and `on_click_in(...)`. Mouse clicks, Enter, and Space enter the same callback contract. |
| [`Focusable`](/en/api/focusable) | Provides `on_focus`, `on_blur`, and Entity-bound forms for real focus transitions. |
| `Disableable` | Provides `disabled(bool)`. `disabled(true)` blocks mouse clicks and Enter/Space activation. |

## Constructor and API

| API | Description |
| --- | --- |
| `Button::new(id)` | Creates a Button with a stable `ElementId`. The `id` is used for GPUI interaction state, focus, and test targeting. |
| `.label(label)` | Sets visible text. The accessible name uses the original label. |
| `.variant(ButtonVariant)` | Sets visual semantics. Defaults to `Primary`. |
| `.size(ComponentSize)` | Sets size. Defaults to `Md`. |
| `.width(width)` | Sets a GPUI `DefiniteLength`, such as `gpui::px(200.)`. |
| `.full_width()` | Fills the width offered by the parent layout. Shares state with `.width(...)`; the later call wins. |
| `.start_icon(icon)` | Sets the leading decorative icon. A later call replaces the earlier icon. |
| `.end_icon(icon)` | Sets the trailing decorative icon. A later call replaces the earlier icon. |
| `.disabled(bool)` | Sets disabled state. |
| `.loading(bool)` | Sets indeterminate activity. `true` blocks activation; `false` returns to idle. The later loading/progress builder wins. |
| `.progress(value)` | Sets determinate progress and blocks activation. The range is `0.0..=1.0`; out-of-range and non-finite values are normalized safely. |
| `.selected(bool)` | Explicitly sets controlled toggle state. The component does not toggle itself. |
| `.auto_insert_space(bool)` | Controls visual spacing for two-Han-character labels. Enabled by default. |
| `.tooltip(text_or_tooltip)` | Accepts a string or `Tooltip` configuration. Strings keep the 500ms automatic behavior; configuration can set `open`, arrow, colors, and animation. |
| `.tooltip_placement(TooltipPlacement)` | Sets the preferred Tooltip placement; defaults to `Bottom` and still flips/shifts when needed. |
| `.aria_description(text)` | Sets a supplementary accessible description independently from the visual Tooltip. |
| `.on_click(handler)` | Registers a standard GPUI click callback: `Fn(&ClickEvent, &mut Window, &mut App)`. |
| `.on_click_in(cx, handler)` | Registers a callback that can access host Entity state. |
| `.on_focus(handler)` / `.on_blur(handler)` | Registers standard GPUI focus and blur callbacks. |
| `.on_focus_in(cx, handler)` / `.on_blur_in(cx, handler)` | Registers focus callbacks that can mutate the host Entity and call `cx.notify()`. |
| `.id()` | Returns the stable `ElementId`. |
| `.label_text()` | Returns the original label passed by the caller. |
| `.display_label()` | Returns the visual label. |

## ButtonVariant

| Variant | Use |
| --- | --- |
| `Primary` | Primary action. This is the default variant. |
| `Outline` | Secondary action with a border. |
| `Ghost` | Lightweight transparent button with hover feedback. |
| `Destructive` | Dangerous or irreversible action. |
| `Secondary` | Secondary filled button. |
| `Link` | Link appearance with Button semantics; hover draws an underline. |

## ComponentSize

| Size | Height |
| --- | --- |
| `Xs` | 24px |
| `Sm` | 32px |
| `Md` | 36px, default |
| `Lg` | 40px |

Icon size, content gap, horizontal padding, radius, font size, and state colors come from the theme tokens for the current size and variant.
When `.size(...)` is omitted, Button reads the global default from `component_size(cx)`. `set_component_size(size, cx)` refreshes windows and affects Button, IconButton, and Checkbox instances without explicit size overrides.

When text is too narrow, it truncates visually. The original label remains available as the accessible name.

## Icon Slots

`start_icon(...)` and `end_icon(...)` accept values that implement `IntoIconSource`. Icons are decorative and do not add accessible names. The Button accessible name always comes from the original label. Use `IconButton` for icon-only actions.

## Disabled

`disabled(true)` removes the focusable tab index, does not register the mouse click handler, and does not register Enter/Space keyboard activation. The visual state uses the disabled token for the current variant and shows a non-interactive cursor.

## Activity and Progress Values

`.loading(true)` represents indeterminate progress; `.progress(value)` represents determinate progress. Finite values are clamped to `0.0..=1.0`, positive infinity becomes `1.0`, and negative infinity and NaN become `0.0`. When activity builders are chained, the later call wins.

Activity only communicates state and prevents duplicate activation. Task completion, failure, retry, and cancellation remain host-application responsibilities.

## Chinese Auto Spacing

By default, when the label contains exactly two Unicode Han characters, Button inserts a regular space in the visual label. For example, `保存` is displayed as `保 存`. This does not change the original label or accessible name. Call `.auto_insert_space(false)` to disable the behavior. One-character labels, labels with three or more characters, labels with whitespace, English labels, and mixed labels are not rewritten.

## Mouse and Keyboard

When enabled, mouse, touch, and focused Enter/Space activation all enter the same GPUI `on_click` callback. Enter and Space each synthesize exactly one `ClickEvent::Keyboard` only after a valid KeyDown + KeyUp cycle, with `KeyboardButton::Enter` or `KeyboardButton::Space` as the source; KeyDown alone does not call the business handler. Selected buttons use the same activation path. Loading/progress consumes mouse and Enter/Space events (including Space's default scrolling behavior) without calling the business handler. Disabled buttons do not activate.

Button registers a GPUI Tab stop. With the pinned GPUI revision, the host window still maps real Tab/Shift+Tab keys to `window.focus_next(cx)`/`focus_prev(cx)`; the quick start and desktop example show the minimal wiring. A string Tooltip appears after 500ms of keyboard focus. `Tooltip::new(...).open(true)` displays without focus, while `open(false)` forces it closed. Escape dismisses without moving Button focus; a dismissed controlled `open(true)` requires a `false -> true` change to reopen.

## Focus and Accessibility

Focus callbacks run only for real transitions and are independent from `on_click`, selected, loading, and progress. Rerendering the same `ElementId` does not fire them and installs the latest handler. Tooltip and business callbacks share one focus handle. `_in` means host Entity binding, not DOM `focusin`; see [`Focusable`](/en/api/focusable).

Tab/Shift+Tab (wired by the host to GPUI traversal) and any programmatic transition targeting the same GPUI focus identity use the same lifecycle. Vektra intentionally adds no `focus()` or `focus_handle()` API. The current Button activation path prevents the default left-mouse-down behavior, so a click that runs an activation handler does not also force focus transfer; without an activation handler, GPUI's default pointer focus transfer remains active.

The Button root always uses `Role::Button` and sets `aria_label` from the original label. Enabled and busy buttons set `tab_index(0)`; `focus_visible` uses the theme focus token and focus width. Disabled buttons leave the Tab order.

After `.selected(false|true)`, the root reports False/True through `aria_toggled`; an ordinary Button does not report toggle state. Loading/progress uses a stable child ID derived from the Button `ElementId`, `Role::ProgressIndicator`, and the original label as its accessible name. Determinate progress reports a minimum of 0, maximum of 100, and the current percentage.

## Theme

Button normal, hover, pressed, focus-visible, disabled, and selected states come from Vektra theme tokens. The default Light/Dark themes define a complete selected matrix for every variant. Existing custom themes may omit the optional selected extension; runtime styling falls back to their pressed, focus-visible, and disabled tokens. Loading/progress colors derive from the currently visible foreground, with no render-time JSON parsing or file I/O.

Loading and Tooltip motion use GPUI `AnimationExt`. When the system or host enables reduced motion, they render static end states and stop requesting decorative animation frames. Fixed per-instance Tooltip colors do not adapt to Light/Dark/System; the caller owns contrast. The documentation preview follows the current VitePress Light/Dark theme. Standalone previews accept `theme=light|dark`; missing or invalid values use `ThemeMode::System`.

## Responsive Behavior

Button is a leaf component and does not manage layout wrapping for its parent. Its contents stay centered inside the button. The text area uses `min_w_0`, `overflow_hidden`, `whitespace_nowrap`, and `text_ellipsis` so a narrow button does not force horizontal overflow. Use `.full_width()` for row-level actions and let the parent layout provide the available width.

Button uses the same GPUI component implementation on macOS, Windows, Linux, and the Web preview. Platform differences are limited to host focus traversal, fonts, and system input mapping; callers do not switch Vektra APIs.

## Current Limits

- Button does not own asynchronous work, progress calculation, automatic selected toggling, or cancellation protocols.
- Loading/progress is a non-activating submission state. Provide a separate Button for cancellation.
- `Link` is link appearance with Button semantics; it does not become a navigation link.
- Icon slots do not accept per-slot pixel sizes. The icon size comes from `ComponentSize`.
- The preview requires browser WebGPU and the font asset provided by the docs preview host.
