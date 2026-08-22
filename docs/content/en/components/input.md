# Input

`Input` is a pure-GPUI, IME-capable single-line text input. Editing state lives in an `Entity<InputState>` owned by the caller; there is no Root, Provider, or registration step.

## Basic usage

Basic contains only a stable ID, state, placeholder, and accessible name.

<VektraExample demo="input/basic" title="Input basic usage" :height="240">

<<< ../../../preview/src/demos/input.rs#input-example-basic{rust}

</VektraExample>

## Input types

### Search

`InputType::Search` provides search semantics. The search icon, clear action, and Enter submission are composed explicitly from existing capabilities.

<VektraExample demo="input/search" title="Search Input" :height="280">

<<< ../../../preview/src/demos/input.rs#input-example-search{rust}

</VektraExample>

### Password

Password masks each grapheme with a fixed character by default. The host controls reveal state, while Eye/EyeOff IconButton supplies a state-dependent accessible name, Tooltip, and selected/toggled semantics.

<VektraExample demo="input/password" title="Password reveal" :height="260">

<<< ../../../preview/src/demos/input.rs#input-example-password{rust}

</VektraExample>

Hidden passwords allow paste but block copy and cut; revealing restores ordinary copy and cut. Toggling reveal never changes the real value, selection, IME state, or undo history, and never emits `Changed`. Both states retain `PasswordInput` semantics.

### Email, Phone, and Url

<VektraExample demo="input/types" title="Common input semantics" :height="340">

<<< ../../../preview/src/demos/input.rs#input-example-types{rust}

</VektraExample>

These types only provide the correct semantics. They do not validate, format, or filter characters; business validation remains a host responsibility.

| `InputType` | AccessKit role | Additional behavior |
| --- | --- | --- |
| `Text` | `TextInput` | Default plain single-line text. |
| `Search` | `SearchInput` | Adds no icon, clear action, or submit logic automatically. |
| `Password` | `PasswordInput` | Secure masking by default; controlled reveal is available. |
| `Email` | `EmailInput` | No automatic email validation. |
| `Phone` | `PhoneNumberInput` | No automatic formatting or filtering. |
| `Url` | `UrlInput` | No automatic URL validation. |

## Composition

### Prefix, suffix, and clear

The three capabilities remain independent. Slot children keep their own roles, focus, and events. `InputClear` reuses IconButton and requires an accessible name.

<VektraExample demo="input/affixes" title="Prefix, suffix, and clear" :height="260">

<<< ../../../preview/src/demos/input.rs#input-example-affixes{rust}

</VektraExample>

### Input group

Place a same-sized Button in `attached_suffix` to create one outer frame, a full-height action area, and a themed divider.

<VektraExample demo="input/group" title="Input group" :height="240">

<<< ../../../preview/src/demos/input.rs#input-example-group{rust}

</VektraExample>

## Appearance and state

<VektraExample demo="input/variants" title="Input visual variants" :height="360">

<<< ../../../preview/src/demos/input.rs#input-example-variants{rust}

</VektraExample>

`Outline` is the default full border, `Filled` uses a filled surface, `Borderless` retains focus and error feedback, and `Underline` only draws the bottom edge.

<VektraExample demo="input/sizes" title="Input semantic sizes" :height="360">

<<< ../../../preview/src/demos/input.rs#input-example-sizes{rust}

</VektraExample>

`ComponentSize::{Xs, Sm, Md, Lg}` uses heights of 24, 32, 36, and 40 px. Input fills available width by default while its text viewport can shrink and scroll horizontally.

<VektraExample demo="input/states" title="Input states" :height="320">

<<< ../../../preview/src/demos/input.rs#input-example-states{rust}

</VektraExample>

The host supplies `invalid`, `read_only`, and `disabled`. Disabled editors leave the normal Tab order and reject input, selection, and SetValue. Read-only editors remain focusable and allow ordinary text selection and copy, but reject edits.

## IME and semantic events

<VektraExample demo="input/events" title="IME, Changed, and Submitted" :height="300">

<<< ../../../preview/src/demos/input.rs#input-example-events{rust}

</VektraExample>

`InputState::value()` always returns the real value. User input, deletion, revealed cut, paste, undo, redo, IME commit, and built-in clear emit one `Changed` only when the value changes; IME preedit stays silent. Enter emits `Submitted` outside composition. Programmatic `set_value`, `clear`, and `reset` do not emit user semantic events.

`set_value` synchronizes a host-owned authoritative value. An actual value change ends composition and clears stale undo/redo history, so later undo cannot cross that external synchronization boundary. `clear` follows the same rule; `reset` additionally resets selection, composition, scrolling, and layout caches. UTF-16 selections returned by an IME are normalized against the complete updated value at grapheme boundaries.

## Keyboard and accessibility

- Arrow keys move by grapheme; platform word modifiers move by word. Home/End, Shift selection, Backspace/Delete, Select All, and Undo/Redo are supported.
- macOS uses Option+Backspace/Delete for word deletion and Command+Backspace/Delete for deletion to the start/end of the line. Windows and Linux retain their existing Control-modifier conventions.
- Only documented modifier combinations are consumed; unknown combinations continue bubbling.
- The actual editor node uses the matching `InputType` role. Prefix, suffix, and attached suffix keep separate accessibility subtrees.
- AccessKit exposes the same extended-grapheme selectable units as the editor, so ZWJ emoji and combining sequences have no internal caret stops.
- A hidden password's painted text, accessibility value, and synthetic text runs contain masks only, never plaintext.

## API

| API | Description |
| --- | --- |
| `Input::new(id, Entity<InputState>)` | Binds a stable ID to caller-owned editing state. |
| `input_type(InputType)` | Selects Text, Search, Password, Email, Phone, or Url semantics. |
| `password_revealed(bool)` | Controlled Password reveal state; defaults to `false` and is ignored by other types. |
| `placeholder`, `aria_label`, `aria_description` | Text and accessibility metadata. |
| `variant`, `size`, `caret_color` | Visual configuration. |
| `disabled`, `read_only`, `invalid` | Externally supplied state. |
| `prefix`, `suffix`, `attached_suffix`, `clearable` | Composition slots and built-in clear. |
| `on_change`, `on_submit`, `on_focus`, `on_blur` | Semantic callbacks and Entity-bound `_in` forms. |

Input implements [`Changeable<SharedString>`](/en/api/changeable), [`Focusable`](/en/api/focusable), [`Disableable`](/en/api/disableable), and [`Sizable`](/en/api/sizable), but not `Clickable`.

## Themes, responsive behavior, and platforms

Light, Dark, and System resolve borders, surfaces, text, placeholder, selection, caret, and state colors through the active theme. Input shrinks within available width, while excessive slot width reduces the editor area. GPUI supplies cross-platform text and IME behavior; command modifiers follow macOS and Windows/Linux conventions.

Custom themes must now provide and validate every Input token when `ResolvedTheme::from_tokens` constructs the theme. Missing keys, wrong types, and invalid references return `ThemeError`; legacy theme fallback has been removed. Migrate custom themes by supplying all four variants, seven visual states, and four sizes, then replace string access with infallible `input_state(InputVariantKind, InputVisualState)` and `input_size(ThemeSize)` calls.

## Performance contract

- Normal scales: 64KiB and 1MiB; 16MiB is stress scale.
- Goals: ≤4ms update+draw at 64KiB and ≤16.67ms at 1MiB. The 16MiB case must remain linear and avoid OOM, but need not complete in one frame.
- Equal-size programmatic replacement clears undo/redo, and display text uses revision-bound shared storage without retaining old values.
- Allocated bytes target at most 8× input size. When unmet, root `PERFORMANCE.md` records the measured gap rather than weakening the budget.
- Benchmarks: `input/state`, `input/render`, and `input/interaction_and_draw`; see the [Benchmark Guide](/en/guide/benchmarks).

## Known limitations

- Input is single-line plain text only; there is no multiline, Number, Date, Time, mask template, or built-in validation message.
- Email, Phone, and Url promise no automatic validation; Password has no custom mask-character API.
- The host owns slot business state, loading, and error handling.
- Web previews require browser WebGPU and the font asset supplied by the documentation host.
