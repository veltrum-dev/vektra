# Benchmark guide

Regular targets cover Select, VirtualList, Input, component walls, Scrollbar, and Theme. The `stress-bench` feature isolates 1M/10M collections, 16MiB Input, and 1M Theme cases.

```sh
cargo bench -p vektra --bench component_scalability
cargo bench -p vektra-theme --bench theme_scalability
cargo check -p vektra --bench scalability_stress --features stress-bench
cargo check -p vektra-theme --bench theme_stress --features stress-bench
```

Benchmarks containing `allocation_observed` report allocations/op, allocated bytes/op, net allocation, and peak live bytes. The resource wrapper completes compilation first and then runs only the exact filtered benchmark executable:

```sh
./scripts/bench-resource-usage.sh vektra component_scalability \
  'input/render/allocation_observed_equal_size_update_and_draw/1048576'
```

Output includes human-readable `VEKTRA_PROCESS_METRICS` and unified `VEKTRA_PROCESS_METRICS_JSON`. Windows uses `scripts/bench-resource-usage.ps1`. Fields cover wall time, user/system or total CPU, CPU percent, peak memory, exit status, platform, package, target, and filter; Cargo/rustc resources are excluded.

Compare timing and RSS only on the same dedicated machine with the same toolchain and GPUI revision. Statistically significant timing regressions above 10%, allocation/byte regressions above 5%, and RSS regressions above 10% fail. Algorithm, materialization, or cache-bound failures fail directly. Shared CI does not gate on absolute milliseconds. The 100K eager-child Scrollbar case remains a public-tree build diagnostic only; render stress uses a same-scale VirtualList so the full regular target does not execute a known resource-exhaustion anti-pattern.
