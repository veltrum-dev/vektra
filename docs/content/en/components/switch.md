# Switch

`Switch` represents an immediately applied on/off setting, such as push notifications or analytics. Use [`Checkbox`](./checkbox) for selection, bulk selection, or mixed state. It is controlled, stores no business state internally, and is not a form framework.

## Basic Usage

<VektraExample demo="switch/basic" title="Switch basic usage" :height="220">

<<< ../../../preview/src/demos/switch.rs#switch-example-basic{rust}

</VektraExample>

`.checked(...)` is the current controlled value, not an initial value. One valid activation passes `!checked` to `on_change`; the host updates its state, calls `cx.notify()`, and supplies the new value during the next render. Callbacks are synchronous local callbacks, not a runtime event bus; the host starts and owns asynchronous work. `.loading(...)` is also controlled and neither starts a task nor changes checked on its own.

## Common states

<VektraExample demo="switch/states" title="Switch enabled and disabled" :height="260">

<<< ../../../preview/src/demos/switch.rs#switch-example-states{rust}

</VektraExample>

## Anatomy

```text
Root interaction and focus area
├─ track
│  ├─ thumb slot (logical start when off, logical end when on)
│  └─ state-content slot (current state, opposite the thumb)
└─ optional trailing label
```

Track, thumb, and label have one interaction target, Tab stop, and accessibility node. Labels can wrap in narrow layouts while the track does not shrink.

Without state content, Switch keeps its existing compact track. Configuring either side enables content mode, which consistently uses a 24px track and 20px thumb regardless of content kind or semantic size. The track consists of one thumb slot and one shared content slot. The content slot uses the larger width required by the checked and unchecked content kinds, so toggling does not resize the track. Theme spacing separates state content from the outer track edge. Icon-only content takes only icon width instead of being inflated to text width. Checked content occupies the logical-start side; unchecked content occupies the logical-end side.

## API

| API | Description |
| --- | --- |
| `Switch::new(id)` | Creates a stable `ElementId`; defaults to off and enabled. |
| `.checked(bool)` | Sets the current controlled value. |
| `SwitchContent::text(text)` | Creates text-only state content. |
| `SwitchContent::icon(icon)` | Creates a decorative icon through `IntoIconSource`. |
| `SwitchContent::icon_text(icon, text)` | Creates icon-first text content. |
| `.checked_content(content)` | Sets checked track content; the last call wins. |
| `.unchecked_content(content)` | Sets unchecked track content; the last call wins. |
| `.loading(bool)` | Shows a spinner in the thumb and blocks activation while retaining focus and Tab stopping. |
| `.disabled(bool)` | Blocks mouse, Enter, Space, and normal Tab stopping. |
| `.transition_duration(Duration)` | Sets the next checked transition duration; defaults to 180ms, while `Duration::ZERO` switches directly. |
| `.label(text)` | Sets the trailing visible label and default accessible name. |
| `.size(ComponentSize)` | Sets `Xs`, `Sm`, `Md`, or `Lg`. |
| `.cursor_style(CursorStyle)` | Sets the idle enabled cursor; loading uses Arrow and disabled always wins. |
| `.aria_label(text)` | Overrides or supplies the accessible name. |
| `.aria_description(text)` | Supplies supplementary accessibility text. |
| `.on_change(handler)` | Receives the next bool, `Window`, and `App`; it carries no `ClickEvent`. |
| `.on_change_in(cx, handler)` | Binds the change callback to a host Entity. |
| `.on_click(handler)` | Provides the standard raw activation entry for starting a request first. |
| `.on_click_in(cx, handler)` | Binds the standard activation entry to a host Entity. |
| `.on_focus` / `.on_blur` | Registers real focus transitions. |
| `.on_focus_in` / `.on_blur_in` | Registers Entity-bound focus callbacks. |

`Switch` implements [`Changeable<bool>`](/en/api/changeable), [`Clickable`](/en/api/clickable), [`Disableable`](/en/api/disableable), [`Focusable`](/en/api/focusable), and [`Sizable`](/en/api/sizable). `on_click` and `on_change` share one activation-handler slot. The later builder wins, so one activation never invokes two competing callbacks.

## Keyboard, Focus, And Accessibility

An enabled, non-loading Switch participates in normal Tab order. Space toggles on keyup; Enter does not. Ctrl-, Alt-, Shift-, and Meta-modified Space do not toggle. Clicking the track, thumb, or label calls the callback only once. Loading consumes mouse, Enter, and Space to prevent duplicate submission or parent activation while retaining Tab focus and focus-visible. Disabled also blocks activation and leaves normal Tab order. With `disabled + loading`, disabled visuals, cursor, and focus rules win while the spinner remains visible.

The root uses `Role::Switch`, with off mapped to `Toggled::False` and on mapped to `Toggled::True`; it never emits mixed. `.aria_label(...)` overrides the visible label, and is required without one. Disabled uses disabled visuals and an unavailable cursor.

Track state content is visual supplementation only. Icons are decorative and create no accessibility node or Tab stop; “On/Off” does not replace the business name. `.label("Notifications")` or `.aria_label("Notifications")` still provides the accessible name.

<VektraExample demo="switch/focus" title="Switch focus lifecycle" :height="260">

<<< ../../../preview/src/demos/switch.rs#switch-example-focus{rust}

</VektraExample>

Checked state and the focus lifecycle are independent. Renders, builder updates, and focus transitions do not call `on_change`; checked updates do not fabricate focus/blur. `_in` means Entity binding through `Context::listener`, which safely becomes a no-op after Entity destruction.

## Loading And Controlled Tasks

<VektraExample demo="switch/loading" title="Switch controlled loading" :height="280">

<<< ../../../preview/src/demos/switch.rs#switch-example-loading{rust}

</VektraExample>

## Sizes and state content

<VektraExample demo="switch/sizes" title="Switch semantic sizes" :height="260">

<<< ../../../preview/src/demos/switch.rs#switch-example-sizes{rust}

</VektraExample>

<VektraExample demo="switch/content" title="Switch text and icon content" :height="340">

<<< ../../../preview/src/demos/switch.rs#switch-example-content{rust}

</VektraExample>

The loading spinner stays inside the thumb without changing thumb or track geometry. The thumb remains at the current controlled checked position and track content continues to describe that state. The spinner has its own stable animation ID and fixed loop period, so `.transition_duration(...)` cannot change or restart it. Under reduced motion it renders a static frame without continuously requesting animation frames.

The host may optimistically update checked when a request begins, or keep checked unchanged until success. Error UI, rollback, cancellation, and task lifetime remain host responsibilities. `.loading(false)` restores normal mouse and Space activation.

When the server is authoritative, use `on_click_in` to read host state and start the request without immediately changing checked. The host supplies `loading(true)` while pending and writes the server-confirmed checked value only after success; failures preserve the prior value and show business-level error UI. `on_change_in` remains the convenient alternative when the suggested next boolean is needed immediately. These entries are alternatives, and the later builder wins.

## Theme, Sizes, And Limits

All four semantic sizes retain their own compact track, hit-target, icon, content-width, spinner, gap, and typography tokens. Content mode consistently uses a 24px track and 20px thumb, while compact dimensions remain unchanged. Light, Dark, and System modes resolve through the Vektra theme; normal, hover, pressed, focus-visible, and disabled use theme tokens. Loading suppresses misleading hover and pressed feedback. Older themes receive semantic fallbacks when new content or loading tokens are absent; once one group is extended, that group must be complete across both visual states or all four sizes.

Switch uses the same GPUI implementation on macOS, Windows, Linux, and the Web preview. The host layout controls wrapping and available width in narrow containers; state text stays on one line and truncates at the themed limit. Platform differences are limited to system fonts, focus traversal, and input mapping.

Controlled checked changes move the thumb and content with a default 180ms fixed ease-out cubic transition. Old content fades out during the first half and new content fades in during the second half, avoiding obvious overlap with the moving thumb. `.transition_duration(...)` accepts the supplied nonzero duration without silent clamping; 100–400ms is recommended. `Duration::ZERO` creates no state-transition animation. Initial render never plays an entrance transition, a duration-only change does not increment motion generation or restart motion, and changing checked plus duration in one render uses the new duration. GPUI reduced motion has higher priority and renders thumb, content, and spinner in their static final state.

- No uncontrolled state or `default_checked`.
- State text is single-line and theme-truncated; use short labels such as “On/Off”.
- No drag, `indeterminate`, custom easing/complex motion configuration, arbitrary `AnyElement` slot, or form validation.
- If two choices must remain visible and separately clickable, use a Segmented Control instead of extending Switch.
- The label is fixed after the track.
- Run the desktop example with `cargo run --example switch`.
