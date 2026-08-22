# VirtualList

`VirtualList` is the public fixed-row-height collection component with viewport-level materialization. It takes a count, fixed item height, stable key, and lazy renderer. Total height is derived from `count × item height`; no per-item Entity, metadata, height record, or Element is retained.

## Basic and million-item usage

<VektraExample demo="virtual-list/basic" title="VirtualList: 1,000,000 generated items" :height="430">

<<< ../../../preview/src/demos/virtual_list.rs#virtual-list-example-basic{rust}

</VektraExample>

The desktop entry `cargo run --example virtual-list` shows a normal 100-item case and a generated 10,000,000-item case in the same window, including visible range, materialized rows, and the cache limit.

## API

| API | Description |
| --- | --- |
| `VirtualListState::new()` | Creates O(1) scroll state to reuse across frames. |
| `VirtualList::new(id, state, count, item_height, key, renderer)` | Creates a generated fixed-height list. |
| `VirtualList::from_data_source(...)` | Reads loaded items from `LazyDataSource` and requests visible ranges. |
| `.revision(u64)` | Marks a data order, content, or loading-state revision. |
| `.request_visible_range(callback)` | Notifies the host of the visible range without blocking. |
| `.scrollbar(ScrollbarConfig)` | Configures Always/Auto/Never and Overlay/Stable; the axis stays vertical. |
| `.aria_label(text)` | Names the ScrollView. |
| `scroll_to_index` / `reveal_index` | Jumps to or reveals an index. |
| `scroll_to_start/middle/end` | Jumps to the start, middle, or end. |
| `metrics()` | Reads the visible range, materialized rows, renderer calls, and cache limit. |

`LazyDataSource` supplies count, revision, stable keys, indexed reads, loaded state, and visible-range requests. `OwnedDataSource` adapts Vec and arrays into the same protocol. Large generated, paged, or remote sources should implement the trait directly and must not block the UI thread.

## Keyboard, scrolling, and accessibility

The component reports `ScrollView` with an accessible name. Wheel input, arrows, PageUp/PageDown, Home/End, track clicks, and thumb dragging share one underlying `ScrollHandle`. Scrollbars support `Always/Auto/Never` and `Overlay/Stable`. Only visible rows enter the Element and AccessKit subtrees; real VoiceOver, NVDA, and Orca testing remains unverified.

## Performance contract

- Normal scale: 100–100K; stress scale: 1M and 10M generated items.
- Request layout, prepaint, paint, renderer, and AccessKit: O(visible), with zero overdraw today.
- Vektra state: O(1); hard row-cache limit: 0.
- Materialized row count must not grow tenfold when item count grows from 100K to 1M.
- Fixed-reference goals: ≤50ms first draw at 1M, ≤8.33ms steady scroll/draw, and <128MB additional maximum RSS with a generated source.
- See [Performance Architecture](/en/guide/performance) and [Benchmark Guide](/en/guide/benchmarks).

## Fit and limitations

Recommended for fixed-height logs, search results, command lists, and paged data. It does not provide variable heights, VirtualGrid, Table, Tree, or masonry. A variable-height mode must explicitly document its O(n) height-index cost and cannot be described as strict O(viewport) memory.

Light, Dark, and System reuse semantic Scrollbar tokens. The code targets macOS, Windows, Linux, and Web/WASM. Local and deterministic-test coverage exists; other-platform performance and real GPU behavior remain unverified.
