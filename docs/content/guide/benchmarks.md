# 性能基准指南

常规基准覆盖 Select、VirtualList、Input、基础组件墙、Scrollbar 与 Theme；压力 target 通过 `stress-bench` feature 隔离 1M/10M 集合、16MiB Input 和 1M Theme。

```sh
cargo bench -p vektra --bench component_scalability
cargo bench -p vektra-theme --bench theme_scalability
cargo check -p vektra --bench scalability_stress --features stress-bench
cargo check -p vektra-theme --bench theme_stress --features stress-bench
```

名称含 `allocation_observed` 的场景输出 allocations/op、allocated bytes/op、净分配和峰值在途字节。资源 wrapper 先完成编译，再只运行精确筛选的 bench executable：

```sh
./scripts/bench-resource-usage.sh vektra component_scalability \
  'input/render/allocation_observed_equal_size_update_and_draw/1048576'
```

输出同时包含人类可读 `VEKTRA_PROCESS_METRICS` 和统一 JSON `VEKTRA_PROCESS_METRICS_JSON`。Windows 使用 `scripts/bench-resource-usage.ps1`。字段包括 wall time、user/system 或 total CPU、CPU percent、peak memory、exit status、platform、package、target 和 filter；Cargo/rustc 资源不计入目标进程。

只在同一固定机器、相同工具链和 GPUI revision 间比较时间/RSS。时间显著回归 >10%、allocation/bytes >5%、RSS >10% 时失败；算法、物化数量或缓存上界失败直接失败。共享 CI 不使用绝对毫秒阻断。Scrollbar 的 100K eager children 只保留公共树构建诊断；渲染压力由同规模 VirtualList 覆盖，避免把已确认的反模式放进完整常规 target 导致进程资源耗尽。
