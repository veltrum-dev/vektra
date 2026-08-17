# Select

`Select<T>` chooses one value from structured options while conserving vertical space. It is strongly typed and controlled: `selected_value(Option<T>)` is the host-owned authoritative value, while `on_change(T, ...)` only requests the next value. Arrow navigation changes a private active option and never changes business selection early.

Use [RadioGroup](/en/components/radio) when a small set of mutually exclusive choices should stay visible. Select has no search, text editing, or multiple selection; those belong to the future [Combobox roadmap](https://github.com/veltrum-dev/vektra/issues/15) and [MultiSelect roadmap](https://github.com/veltrum-dev/vektra/issues/16).

## Basic controlled usage

<VektraExample demo="select/basic" title="Select basic usage" :height="300">

<<< ../../../preview/src/demos/select.rs#select-example-basic{rust}

</VektraExample>

The host may accept a request immediately or wait for server approval before updating `selected_value`. If rejected, keep passing the old value; Select does not retain a second business selection. Submitting the authoritative value again emits no redundant change.

## Groups and disabled options

<VektraExample demo="select/groups" title="Select groups and disabled options" :height="340">

<<< ../../../preview/src/demos/select.rs#select-example-groups{rust}

</VektraExample>

Group labels provide visible and accessible grouping only. They never enter active, selection, or keyboard indexing. Disabled options remain visible but cannot become active or request a value.

## Loading, empty, and error

<VektraExample demo="select/states" title="Select host-controlled states" :height="390">

<<< ../../../preview/src/demos/select.rs#select-example-states{rust}

</VektraExample>

`SelectStatus` is mutually exclusive. `Ready` shows options, while `Loading`, `Empty`, and `Error` show host-supplied text. Select never starts a request, waits, retries, or changes status itself. Status content is not an option and cannot emit selection. A non-Ready popup can still open by keyboard or pointer so its status message remains reachable, but arrows, paging, typeahead, and Enter submission cannot establish an active option or request a value.

## Keyboard navigation

<VektraExample demo="select/keyboard" title="Select keyboard navigation" :height="320">

<<< ../../../preview/src/demos/select.rs#select-example-keyboard{rust}

</VektraExample>

| Key | Closed | Open |
| --- | --- | --- |
| Enter / Space | Open and activate the enabled selected option or first option | Submit an enabled active option and close |
| ArrowDown | Open at the enabled selected option or first enabled option | Move to the next enabled option without wrapping |
| ArrowUp | Open at the enabled selected option or last enabled option | Move to the previous enabled option without wrapping |
| Home / End | Propagate normally | Move to the first / last enabled option |
| PageUp / PageDown | Propagate normally | Move by the popup's currently measured visible page and clamp at the ends |
| Printable text | In Ready, open and cycle from after the current option by accessible name | Accumulate a short-lived prefix; repeated characters cycle options with that initial |
| Escape | Propagate normally | Close without changing value; focus stays on the trigger |
| Tab / Shift+Tab | Traverse normally | Close and continue normal focus traversal |

Unsupported modifiers and unknown keys propagate. Typeahead considers enabled canonical options only, applies a Unicode case-insensitive prefix match, and clears its buffer after a short pause; no match preserves the current active option. Enter and Space use GPUI's complete KeyDown/KeyUp activation cycle; one interaction emits at most one value request.

## Long lists, narrow windows, and resize

<VektraExample demo="select/long-list" title="Select long list" :height="330">

<<< ../../../preview/src/demos/select.rs#select-example-long-list{rust}

</VektraExample>

The popup prefers opening below, flips above when needed, and is constrained by viewport padding and maximum height. Overflow reuses the Vektra Scrollbar. Arrow, Home, End, PageUp, PageDown, typeahead, and initial opening keep the active option visible. Paging uses the current ScrollArea viewport and measured option bounds rather than a fixed item count. Trigger and viewport geometry are measured again after layout and window resize; narrow windows constrain the popup horizontally.

The implementation is not virtualized and makes no unbounded option-count performance promise. Large-data work is tracked in [Issue #6](https://github.com/veltrum-dev/vektra/issues/6).

## Anatomy

```text
Select trigger (ComboBox, real Tab stop, expanded)
└─ current label / placeholder + ChevronDown / ChevronUp indicator
Select popup (ListBox, private viewport-constrained overlay)
└─ ScrollArea
   ├─ SelectGroup (Group)
   │  ├─ group label (Label)
   │  └─ SelectOption (ListBoxOption)
   └─ loading / empty (Status) or error (Alert)
```

Selection uses a trailing Check icon while the active option uses a subtle background. The error `!` and focus-visible outline also provide non-color cues.

## API

| API | Description |
| --- | --- |
| `Select::new(id)` | Creates an unselected Select with a stable root `ElementId`. |
| `.selected_value(Option<T>)` | Supplies the host-owned authoritative value. |
| `.option(SelectOption<T>)` | Adds a top-level structured option. |
| `.group(SelectGroup<T>)` | Adds a titled structured option group. |
| `.placeholder(text)` | Sets trigger text when no valid selection exists; defaults to “请选择”. |
| `.status(SelectStatus)` | Sets `Ready`, `Loading`, `Empty`, or `Error`. |
| `.on_change` / `.on_change_in` | Requests the next value without optimistic selection. |
| `.disabled(bool)` | Disables the trigger and removes it from normal Tab order. |
| `.size(ComponentSize)` | Applies `Xs`, `Sm`, `Md`, or `Lg`. |
| `.on_focus` / `.on_blur` and `_in` | Observes real trigger focus transitions. |
| `.aria_label` / `.aria_description` | Sets the trigger's accessible name and description. |
| `SelectOption::new(id, value, label)` | Creates a stable ID, typed value, and visible label. |
| `.icon(IconSource)` | Adds an optional decorative icon. |
| `.description(text)` | Adds visible detail and its fallback accessible description. |
| `.aria_label` / `.aria_description` | Overrides option accessibility text. |
| `.disabled(bool)` | Removes the option from active and submission paths. |
| `SelectGroup::new(id, label)` | Creates a group with a stable ID and visible heading. |
| `.aria_label(text)` | Overrides the group's accessible name. |
| `.option(SelectOption<T>)` | Adds a structured option of the same value type. |

Select implements [`Changeable<T>`](/en/api/changeable), [`Disableable`](/en/api/disableable), [`Sizable`](/en/api/sizable), and [`Focusable`](/en/api/focusable). SelectOption implements `Disableable`. Select deliberately does not implement `Clickable`: opening/closing the trigger and requesting an option are composite selection semantics, not one raw click contract. It also does not implement arbitrary-Element `ParentElement`.

## Stable identity and dynamic options

- Option IDs and business values should each be unique within a Select; group IDs should also remain stable.
- Duplicate IDs or values use input-order first-match/canonical behavior. The first canonical option works normally; later conflicts act disabled and cannot create a second selected visual or callback.
- Removing the selected option shows the placeholder without selecting a replacement or calling `on_change`.
- If a selected option becomes disabled, the trigger still shows the authoritative value, but that option cannot become active or submit again.
- Removing the active option prefers the next enabled option at its old position, then the nearest previous option. Reordering follows stable IDs.
- An all-disabled popup still opens and closes safely with no active option.

## Focus, closing, and accessibility

The trigger is the only real focus target and normal Tab stop. While open, focus stays on it and the active option is reported through GPUI/AccessKit active-descendant semantics. Submitting an enabled option closes and restores trigger focus. Clicking the trigger again, clicking outside, Escape, Tab/Shift+Tab, or window deactivation closes the popup. Internal clicks, wheel input, and Scrollbar interaction are not treated as outside clicks.

The trigger reports `ComboBox`, name, description, expanded, and disabled. When unselected it reports only a placeholder rather than duplicating it as a value; once selected, its value is the option's accessible name. Popup, group, label, and option report `ListBox`, `Group`, `Label`, and `ListBoxOption`; options report selected, disabled, and `posinset`/`setsize` across every rendered option, including disabled and duplicate-conflict options. Loading/empty use `Status`, and error uses `Alert`. While the popup is open, the trigger uses AccessKit `controls` to reference the real `ListBox` node in the same first frame. The keyboard-active option remains exposed through active-descendant and is not conflated with the business selection.

Disabled, expanded, selected, name, description, and value mappings have deterministic AccessKit node assertions. Roles, active-descendant, and focus paths are covered by locked-GPUI compilation and interaction tests. GPUI's regular test platform does not activate a complete assistive-technology tree, so VoiceOver, NVDA, Narrator, Orca, and platform announcement behavior have not been manually verified.

## Themes and cross-platform status

Light, Dark, and System resolve dedicated Select trigger, popup, option, group, status, and `Xs/Sm/Md/Lg` tokens. The scroll area keeps using shared Scrollbar tokens. No arbitrary color, radius, or spacing overrides are exposed.

Custom themes must now pass complete Select tokens through `ResolvedTheme::from_tokens`. Missing keys, wrong types, or invalid references return `ThemeError`; the legacy missing-extension fallback is gone. Migrate by supplying all six trigger states, five option states, and four sizes, then replace string access with infallible `select_trigger_state(SelectTriggerState)`, `select_option_state(SelectOptionState)`, and `select_size(ThemeSize)` calls.

The code targets GPUI-supported macOS, Windows, Linux, and Web/WASM. Local compilation, deterministic interaction tests, popup constraints under 1.25x/1.5x/2x test scaling, and the shared WASM build are covered. Linux behavior, physical-display high-DPI pixel parity, screen readers, and large-list performance are not manually verified.

## Known limitations

- Single selection and non-editable: no search, filtering, IME, Combobox, or MultiSelect.
- Options accept a label, optional `IconSource`, and description, not arbitrary Elements or slots.
- Status is entirely host-driven; Select owns no async task or retry logic.
- The popup remains a private Select implementation; no public Popover, Menu, List, or VirtualList is added.
- No virtualization; see [performance Issue #6](https://github.com/veltrum-dev/vektra/issues/6).
- Desktop example: `cargo run --example select`.
