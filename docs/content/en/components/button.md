# Button

Button represents an action such as saving, confirming, changing a mode, or starting a long-running task. It is a plain GPUI element and is exported as `vektra::Button`.

Do not use Button for status-only text, standalone progress panels, or composite list items that need arrow-key selection. Icon-only actions should use `IconButton` with an accessible name.

## Live Preview

<VektraPreview demo="button/basic" title="Button preview" :height="760" />

## Basic Usage

Set the label with `.label(...)`. Handle activation with `.on_click(...)` or `.on_click_in(...)`.

<<< ../../../preview/src/demos/button.rs#button-basic{rust}

## Stateful Views

`Button::on_click_in(cx, ...)` gives the handler access to the host `&mut T`, `ClickEvent`, `Window`, and `Context<T>`. Call `cx.notify()` after updating host state.

<<< ../../../preview/src/demos/button.rs#button-states{rust}

## Loading, Selected, and Progress

`.loading(bool)`, `.progress(f32)`, and `.selected(bool)` are controlled inputs. Button does not start asynchronous work, calculate progress, or toggle selected after a click; the host updates state and calls `cx.notify()`.

- Loading and progress share one mutually exclusive activity state. The later builder wins, and `.loading(false)` returns to idle.
- Loading replaces the start icon with a rotating indicator while preserving the original label and end icon. GPUI `AnimationExt` automatically respects reduced motion.
- Progress preserves both icon slots and the label, and draws a bottom bar without changing external dimensions.
- Selected is independent from activity. Toggle semantics are only exposed after `.selected(false|true)` is called. A persistent inner outline means selection is not conveyed by color alone.
- Disabled has the highest priority: it uses disabled styling and leaves the Tab order. An activity indicator remains visible, but activation stays blocked.

During loading/progress, Button keeps focus and `Role::Button`, but consumes mouse, Enter, and Space events to prevent duplicate submission and parent activation. Use a separate Button for cancellation.

## Variants and Sizes

Button provides 6 `ButtonVariant` values and 4 `ButtonSize` values. A disabled button uses the disabled token for its current variant.
The ordinary variant rows in the preview use the default `Md` size. The separate size comparison area intentionally shows `Xs`, `Sm`, `Md`, and `Lg`; the height differences are not rendering inconsistencies.

<<< ../../../preview/src/demos/button.rs#button-variants{rust}

## Icons

Start icons, end icons, paired icons, fixed width, and full-width layout use the same `Button` API.

<<< ../../../preview/src/demos/button.rs#button-icons{rust}

## Chinese Auto Spacing

Chinese auto spacing is enabled by default. It can be enabled or disabled explicitly. Long labels, mixed Chinese/English text, and mixed numeric text are not rewritten.

<<< ../../../preview/src/demos/button.rs#button-auto-space{rust}

## Width

Button sizes itself to its content by default. `.width(...)` sets a fixed width. `.full_width()` uses the full width offered by the parent layout. Both methods write the same width state, so the later call wins.

<<< ../../../preview/src/demos/button.rs#button-width{rust}

## Capability Traits

| Trait | Contract |
| --- | --- |
| `Clickable` | Provides `on_click(...)` and `on_click_in(...)`. Mouse clicks, Enter, and Space enter the same callback contract. |
| `Disableable` | Provides `disabled(bool)`. `disabled(true)` blocks mouse clicks and Enter/Space activation. |

## Constructor and API

| API | Description |
| --- | --- |
| `Button::new(id)` | Creates a Button with a stable `ElementId`. The `id` is used for GPUI interaction state, focus, and test targeting. |
| `.label(label)` | Sets visible text. The accessible name uses the original label. |
| `.variant(ButtonVariant)` | Sets visual semantics. Defaults to `Primary`. |
| `.size(ButtonSize)` | Sets size. Defaults to `Md`. |
| `.width(width)` | Sets a GPUI `DefiniteLength`, such as `gpui::px(200.)`. |
| `.full_width()` | Fills the width offered by the parent layout. Shares state with `.width(...)`; the later call wins. |
| `.start_icon(icon)` | Sets the leading decorative icon. A later call replaces the earlier icon. |
| `.end_icon(icon)` | Sets the trailing decorative icon. A later call replaces the earlier icon. |
| `.disabled(bool)` | Sets disabled state. |
| `.loading(bool)` | Sets indeterminate activity. `true` blocks activation; `false` returns to idle. The later loading/progress builder wins. |
| `.progress(value)` | Sets determinate progress and blocks activation. The range is `0.0..=1.0`; out-of-range and non-finite values are normalized safely. |
| `.selected(bool)` | Explicitly sets controlled toggle state. The component does not toggle itself. |
| `.auto_insert_space(bool)` | Controls visual spacing for two-Han-character labels. Enabled by default. |
| `.on_click(handler)` | Registers a standard GPUI click callback: `Fn(&ClickEvent, &mut Window, &mut App)`. |
| `.on_click_in(cx, handler)` | Registers a callback that can access host Entity state. |
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

## ButtonSize

| Size | Height |
| --- | --- |
| `Xs` | 24px |
| `Sm` | 32px |
| `Md` | 36px, default |
| `Lg` | 40px |

Icon size, content gap, horizontal padding, radius, font size, and state colors come from the theme tokens for the current size and variant.

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

When enabled, a left-click prevents the default event, stops propagation, and triggers the callback. When the Button is focused, Enter activates on keydown and Space activates on keyup. Both create `ClickEvent::Keyboard` and enter the same click callback. Selected buttons use the same activation path. Loading/progress consumes mouse and Enter/Space events (including Space's default scrolling behavior) without calling the business handler. Disabled buttons do not activate.

## Focus and Accessibility

The Button root always uses `Role::Button` and sets `aria_label` from the original label. Enabled and busy buttons set `tab_index(0)`; `focus_visible` uses the theme focus token and focus width. Disabled buttons leave the Tab order.

After `.selected(false|true)`, the root reports False/True through `aria_toggled`; an ordinary Button does not report toggle state. Loading/progress uses a stable child ID derived from the Button `ElementId`, `Role::ProgressIndicator`, and the original label as its accessible name. Determinate progress reports a minimum of 0, maximum of 100, and the current percentage.

## Theme

Button normal, hover, pressed, focus-visible, disabled, and selected states come from Vektra theme tokens. The default Light/Dark themes define a complete selected matrix for every variant. Existing custom themes may omit the optional selected extension; runtime styling falls back to their pressed, focus-visible, and disabled tokens. Loading/progress colors derive from the currently visible foreground, with no render-time JSON parsing or file I/O.

Loading uses GPUI `AnimationExt`. When the system or host enables reduced motion, it renders a static frame and stops requesting animation frames. The documentation preview follows the current VitePress Light/Dark theme. Standalone previews accept `theme=light|dark`; missing or invalid values use `ThemeMode::System`.

## Responsive Behavior

Button is a leaf component and does not manage layout wrapping for its parent. Its contents stay centered inside the button. The text area uses `min_w_0`, `overflow_hidden`, `whitespace_nowrap`, and `text_ellipsis` so a narrow button does not force horizontal overflow. Use `.full_width()` for row-level actions and let the parent layout provide the available width.

## Current Limits

- Button does not own asynchronous work, progress calculation, automatic selected toggling, or cancellation protocols.
- Loading/progress is a non-activating submission state. Provide a separate Button for cancellation.
- `Link` is link appearance with Button semantics; it does not become a navigation link.
- Icon slots do not accept per-slot pixel sizes. The icon size comes from `ButtonSize`.
- The preview requires browser WebGPU and the font asset provided by the docs preview host.
