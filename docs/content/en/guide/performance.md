# Why Vektra is fast

Vektra is a cross-platform, high-performance, composable pure GPUI component library. Performance is part of public-component correctness and API contracts, not an optional afterthought.

```text
Vec / arrays / builders / generated / paged / remote data
                              ↓
                    unified lazy data source
                              ↓
              shared state, navigation, focus, AccessKit
                              ↓
                   visible + bounded overdraw
                              ↓
                    GPUI paint + Vektra Scrollbar
```

## Collection architecture

Each collection has one kernel. Convenience APIs use owned adapters while paged and remote data implement the same protocol; there are no separate eager/lazy render, navigation, or accessibility paths. Fixed-height paths derive total height from `count × height` and materialize only the viewport. Large sources own key/value, enabled-navigation, and typeahead indexes; the render thread performs non-blocking reads and range requests only.

`VirtualList` keeps O(1) Vektra state with a hard row-cache limit of zero. The Select popup uses the same virtual-list kernel. Owned options/groups preserve expected-O(n) first-canonical semantics with a temporary option-ID HashSet plus the final value index, while external million-item sources receive no full Vektra catalog.

## Hot paths and lifetime

Render, layout, prepaint, and paint prohibit blocking I/O, theme/JSON parsing, regex compilation, full-data construction, and repeated SVG parsing. Every cache declares its key, invalidation rule, and hard capacity. Tasks, subscriptions, entities, timers, and history have explicit owners and release paths. Warm steady state must not grow continuously.

## Budgets and boundaries

The reference-environment goal is 8.33ms for a 120fps steady frame and 16.67ms from ordinary interaction to the next draw. These are not unconditional promises for every machine, GPU, or host application. Machine-readable budgets live in `performance-budgets.json`, with the full contract in root `PERFORMANCE.md`.

Criterion GPUI scenarios measure CPU-side state, layout, prepaint, and test drawing—not real GPU FPS or compositor latency. Every number must name the machine, OS, Rust, and GPUI revision. Dedicated-runner, real-assistive-technology, and physical-platform gaps remain explicitly unverified.
