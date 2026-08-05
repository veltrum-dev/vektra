# Input

`Input` is a pure-GPUI, IME-capable single-line text input. Editing state lives in an `Entity<InputState>` owned by the caller. The component adds no Root, Provider, or registration step, and `InputState` deliberately excludes required, regex, error messages, dirty, touched, and other form metadata.

## Basic usage

Create one `Entity<InputState>` and pass that same Entity to a stable Input ID on every render. The basic example contains only a placeholder and the required accessible name.

<VektraExample demo="input/basic" title="Input basic usage" :height="240">

<<< ../../../preview/src/demos/input.rs#input-example-basic{rust}

</VektraExample>

`InputState::value()` reads the current text. `set_value`, `clear`, and `reset` are programmatic operations: they do not emit `InputEvent::Changed` or call `on_change`. All safely end IME composition and repair selection; `reset` also clears undo/redo history and horizontal scroll. User typing, deletion, cut, paste, undo, redo, IME commit, and built-in clear emit exactly one `Changed` only when the value changes.

`InputState` implements `EventEmitter<InputEvent>`. `on_change`, `on_submit`, `on_focus`, `on_blur`, and their `_in` forms are fed by the same semantic event path. IME preedit is silent, Enter does not submit during composition, and commit emits one change. Enter outside composition emits `Submitted(value)`.

## Variants and state

<VektraExample demo="input/variants" title="Input visual variants" :height="360">

<<< ../../../preview/src/demos/input.rs#input-example-variants{rust}

</VektraExample>

- `Outline` is the default full border for ordinary forms.
- `Filled` groups fields with a filled surface and retains a transparent structural border to prevent geometry jumps.
- `Borderless` suits toolbars and compact surfaces. Normal and hover have no visible border; actual mouse or keyboard focus adds an accent structural border, while keyboard focus-visible uses the wider focus border. The invalid icon remains present.
- `Underline` suits low-density forms and strengthens its bottom line for focus and errors. Its shell radius is always zero, keeping both line ends square.

Priority is disabled, invalid + focus-visible, invalid, focus-visible, hover, then normal. `invalid(bool)` only consumes validation state supplied by the caller. Input neither knows why validation failed nor runs rules. A future `FormControl<T>` or label/help/error layout can own required, regex, async validation, and touched state without putting them into `InputState`.

<VektraExample demo="input/sizes" title="Input semantic sizes" :height="360">

<<< ../../../preview/src/demos/input.rs#input-example-sizes{rust}

</VektraExample>

`ComponentSize::{Xs, Sm, Md, Lg}` uses heights of 24, 32, 36, and 40 px. Input fills available parent width by default, its text viewport can shrink and scroll horizontally, and the parent remains responsible for final width.

While focused, editable, and collapsed, the insertion caret switches discretely between fully visible and fully hidden every 500ms. It requests a redraw only when the phase changes and registers no continuous animation frames. Typing, deletion, movement, pointer placement, clear, undo/redo, IME updates/commit, and focus restart it fully visible. It stays solid during IME preedit and reduced motion, and retains no blink task when blurred, selected, disabled, or read-only. Caret height comes from shaped-text `ascent + descent`, centered within line-height; the default width is 1px. `caret_color(Hsla)` overrides the theme caret for this instance in all editable states, including invalid, without changing other tokens.

## Prefix, suffix, attached suffix, and clear

The order is fixed as `prefix | editor | status | clear | suffix | divider | attached suffix`; omit status when there is no invalid marker. Clear is editor-owned and stays inward of the regular suffix. `suffix(...)` continues to mean compact trailing content inside Input's horizontal padding. `attached_suffix(...)` means a segmented action that shares Input's single shell, sits flush against the right edge, fills the Input height, and is separated by the theme border color. The editor shrinks first while the attached suffix retains its width.

All three slots accept any `IntoElement` and retain their IDs, roles, focus, Tab order, tooltips, aria, and events. Text pointer handlers cover only the editor viewport, so activating a slot does not move the caret or emit Input `Changed`/`Submitted` events. With clear, an interactive suffix, and an attached suffix, normal Tab order is editor, clear, suffix, then attached suffix. The shell clips to its own border only when an attached suffix is present; Button's focus-visible border is inset and remains clearly visible.

`disabled(true)` and `read_only(true)` own only the editor and built-in clear operation. Input cannot reliably mutate arbitrary children; pass the same state to interactive prefix, suffix, and attached-suffix components when the whole composition must be disabled or read-only. Keep regular slots compact. Give an attached suffix the same `ComponentSize` as its Input so the full-height segment remains aligned.

`InputClear::new(aria_label)` requires the icon button's accessible name. It reuses `IconButtonVariant::Ghost` and existing Tooltip/placement support with a 24×24 `ComponentSize::Xs` transparent hit area. At rest it looks like only the icon; IconButton still owns hover, pressed, focus-visible, Enter, and Space. Clear remains visible while the value is non-empty and editable. Activation emits one `Changed("")`, then restores editor focus. Tooltip text is visual help, is never copied automatically into the aria label, and does not replace an accessible name.

### Input group

Search needs no dedicated component. Place a Button with the same size in `attached_suffix` to create one outer border, a full-height action area, and a themed divider.

<VektraExample demo="input/group" title="Input group" :height="240">

<<< ../../../preview/src/demos/input.rs#input-example-group{rust}

</VektraExample>

Icon-only actions may use either regular `suffix` or `attached_suffix` with `IconButton + IconName::Search`. They require an `aria_label`; Tooltip does not replace the accessible name. `IconName::Search` is available through Vektra's `icons` feature.

### States

The host supplies `invalid`, `read_only`, and `disabled` explicitly, and each keeps its own semantics.

<VektraExample demo="input/states" title="Input states" :height="320">

<<< ../../../preview/src/demos/input.rs#input-example-states{rust}

</VektraExample>

## Editing, keyboard, and accessibility

Input supports grapheme-safe arrows, Shift selection, Home/End, platform word movement/deletion, click placement, Shift-click, drag selection, double-click word selection, triple-click-or-more Select All, Copy/Cut/Paste, Undo/Redo, and horizontal scrolling. Select All, Copy/Cut/Paste, and Undo/Redo use Cmd on macOS or Ctrl on Windows/Linux; the Windows key and Linux Super are not treated as generic command modifiers. Direction and deletion handlers consume only explicitly supported modifier combinations, allowing all other combinations to propagate to the host or system. macOS additionally maps Fn + Left/Right to line start/end. Double/triple click only changes selection and emits no `Changed`; CJK, emoji, ZWJ, and combining marks remain on valid UTF-8 boundaries. Pasted CR/LF becomes spaces without trimming. Tab follows normal GPUI focus navigation, and Escape propagates.

A disabled editor leaves normal Tab order and rejects input, selection, and AccessKit SetValue. A read-only editor remains focusable, selectable, and copyable but rejects modifications and SetValue.

Only the actual editor node has `Role::TextInput`; interactive slots remain separate accessibility subtrees. The node supplies value, placeholder, description, text runs, UTF-16 selection, invalid, read-only, and SetValue. Applications must call `aria_label(...)` or provide an explicit accessible name through surrounding semantics. Placeholder is only a hint and never becomes the accessible name automatically.

## API summary

| API | Purpose |
| --- | --- |
| `Input::new(id, state)` | Binds a stable `ElementId` to caller-owned `Entity<InputState>`. |
| `placeholder`, `aria_label`, `aria_description` | Hint and accessibility semantics; none substitutes for another. |
| `variant`, `size`, `disabled`, `read_only`, `invalid` | Appearance, shared size, and externally driven state. |
| `caret_color(Hsla)` | Overrides this instance's insertion-caret color; otherwise uses the theme caret token. |
| `prefix`, `suffix` | Inserts independent arbitrary elements inside Input's horizontal padding. |
| `attached_suffix` | Adds a full-height segmented trailing element with a theme divider, flush to the right edge. |
| `clearable(InputClear)` | Adds semantic clear through IconButton + Tooltip. |
| `on_change`, `on_submit`, `on_focus`, `on_blur` | User semantic events and Entity-bound `_in` forms. |

Input implements [`Changeable<SharedString>`](/en/api/changeable), [`Focusable`](/en/api/focusable), [`Disableable`](/en/api/disableable), and [`Sizable`](/en/api/sizable), but not `Clickable`. Theme paths are `input.border-width`, `input.focus-width`, `input.caret-width` (1px by default), `input.variant.<variant>.<state>.*`, and `input.size.<size>.*`. Existing custom themes with no Input extension fall back to semantic/foundation tokens. Once a theme starts supplying Input state or size extensions, that section must be complete.

## Themes, Responsive Behavior, and Platforms

Light, Dark, and System resolve borders, surfaces, text, placeholder, selection, caret, and state colors through the active Vektra theme. Input shrinks within the available width while editor content scrolls horizontally. Prefix, suffix, and attached suffix widths are caller choices, so too many slots reduce the editor area. macOS, Windows, Linux, and the Web preview share the same GPUI implementation, while command modifiers and system text input follow platform conventions.

## Known Limitations

- Input is single-line plain text only; there is no multiline editing, masking, formatting, or built-in validation message.
- Prefix, suffix, and attached suffix are composition slots; the host owns their business state, loading, and error handling.
- The preview requires browser WebGPU and the font asset supplied by the documentation host.
