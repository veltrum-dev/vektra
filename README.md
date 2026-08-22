# Vektra

Vektra 是面向 Windows、Linux 和 macOS 的跨平台、高性能、可组合纯 GPUI 组件库。它不是应用框架，也不需要 Vektra Root、Provider 或 `vektra::init(cx)`；应用仍按 GPUI 的 `Application`、`Window` 和 view 模型组织。

集合组件通过统一惰性数据源、视窗级渲染和有界缓存控制大数据成本；Vec、数组、逐项 builder、生成式、分页与远程数据进入同一组件内核。性能证据、预算和平台边界见 [`PERFORMANCE.md`](PERFORMANCE.md)。

Vektra 当前处于早期开发阶段。GPUI 虽已发布 pre-1.0 版本，但 API 仍在快速演进，可能发生破坏性变更。Vektra 当前锁定特定 GPUI revision，尚未承诺稳定兼容 crates.io GPUI，因此暂不发布可供生产使用的正式 crate。请通过 Git workspace 或 path dependency 使用，并预期 Vektra 公共 API 也可能发生破坏性变更。

crates.io 上的 `vektra` 0.0.1 只用于保留项目名称，不包含当前组件库实现，不能作为正式依赖。当前真实组件 crate 位于 `crates/vektra`，仍标记为 `publish = false`。

## 文档

- 中文文档：<https://veltrum-dev.github.io/vektra/>
- English docs: <https://veltrum-dev.github.io/vektra/en/>
- 本地文档开发：[docs/README.md](docs/README.md)
- 版本与发布政策：[docs/content/guide/versioning-and-releases.md](docs/content/guide/versioning-and-releases.md)
- 资源与图标：[docs/content/guide/assets-and-icons.md](docs/content/guide/assets-and-icons.md)
- Button API：[docs/content/components/button.md](docs/content/components/button.md)
- Select API：[docs/content/components/select.md](docs/content/components/select.md)
- Switch API：[docs/content/components/switch.md](docs/content/components/switch.md)
- IconButton API：[docs/content/components/icon-button.md](docs/content/components/icon-button.md)
- Tooltip API：[docs/content/components/tooltip.md](docs/content/components/tooltip.md)
- Scrollbar API：[docs/content/components/scrollbar.md](docs/content/components/scrollbar.md)
- VirtualList API：[docs/content/components/virtual-list.md](docs/content/components/virtual-list.md)
- 性能架构与预算：[PERFORMANCE.md](PERFORMANCE.md)

## 许可证

Vektra 使用 [MIT License](LICENSE)。

## 最小示例

```rust
use vektra::{Button, IconButton, IconSource, TooltipPlacement};

Button::new("save")
    .label("保存")
    .tooltip("保存当前修改")
    .tooltip_placement(TooltipPlacement::TopStart)
    .aria_description("保存当前修改")
    .on_click(|_, _, _| {
        // 鼠标、Enter 和 Space 激活共享这个回调契约。
    });

Button::new("settings")
    .label("设置")
    .start_icon(IconSource::asset("icons/settings.svg"));

IconButton::new("settings", IconSource::asset("icons/settings.svg"))
    .aria_label("设置")
    .tooltip("设置");
```

启用内置图标：

```toml
vektra = { path = "crates/vektra", features = ["icons"] }
```

## 示例

```bash
cargo run --example button
cargo run --example checkbox
cargo run --example switch
cargo run --example radio
cargo run --example select
cargo run --example icon_button
cargo run --example custom_assets
cargo run --example tooltip
cargo run --example input
cargo run --example scrollbar
cargo run --example virtual-list
```

所有桌面示例都提供统一的 `System / Light / Dark` 主题选择器，并同时显示配置模式与当前解析到的实际主题。支持虚拟渲染的组件把普通与大数据场景放在同一个入口：`cargo run --example select` 同时展示普通 Select 和百万项惰性 Select，`cargo run --example virtual-list` 同时展示普通列表和一千万项生成式列表。

## 常用开发命令

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 性能基准

第一阶段基准用于建立同机相对基线、观察扩展趋势并发现潜在回归，不设置阻断合并的绝对
耗时门槛。GPUI 场景使用 `TestAppContext` 的测试平台，覆盖 CPU 侧组件构建、状态处理、
布局、prepaint 和测试绘制成本；这些数字不是实际 GPU FPS、物理显示器帧时间、系统合成
延迟或跨平台绝对性能。同一结果只应在相同机器、Rust 工具链和 GPUI revision 下比较。

常规规模如下：Select 公共树构建到 100K options、完整绘制到 10K；Input 到 1MiB；基础
组件墙公共构建到 100K、完整绘制到 10K；Scrollbar 到 100K children；Theme 合成
token 到 100K。显式压力 target 另外覆盖 1M Select 数据、100K Select 完整绘制、
16MiB Input、100K 可见组件墙、1M Scrollbar children 和 1M Theme tokens。

运行全部常规 target：

```bash
cargo bench -p vektra --bench component_scalability
cargo bench -p vektra-theme --bench theme_scalability
```

Criterion 接受名称过滤器，可只运行一个场景：

```bash
cargo bench -p vektra --bench component_scalability -- 'select/build/public_tree/100000'
cargo bench -p vektra-theme --bench theme_scalability -- 'theme/token_sets/single_set_parse_resolve/100000'
```

先在 main 保存同机基线，再在分支比较：

```bash
cargo bench -p vektra --bench component_scalability -- --save-baseline main
cargo bench -p vektra --bench component_scalability -- --baseline main
```

名称含 `allocation_observed` 的场景使用 dev-only `allocation-counter` 在当前线程统计每次
操作的 allocation 次数、allocated bytes、净分配和峰值在途字节，并输出
`VEKTRA_ALLOCATION` 记录。其 Criterion 时间包含统计开销，只应使用同名普通场景比较耗时：

```bash
cargo bench -p vektra --bench component_scalability -- allocation_observed
cargo bench -p vektra-theme --bench theme_scalability -- allocation_observed
```

进程级 CPU 与内存占用通过独立 wrapper 采集。wrapper 会先完成编译，再只包住一个经过
名称过滤的 bench 进程，避免把 rustc/Cargo 的资源占用混进结果。macOS 输出 user/system
CPU time 和 byte 单位的 maximum resident set size；Linux 使用 GNU `time -v`，其 maximum
resident set size 单位为 KiB；Windows 输出 wall time、总 CPU time、CPU/墙钟比和 byte
单位的 peak working set。所有平台同时输出统一的 `VEKTRA_PROCESS_METRICS_JSON`，包含
wall/CPU/peak memory/exit status/platform/package/target/filter：

```bash
# macOS / Linux
./scripts/bench-resource-usage.sh \
  vektra component_scalability 'select/build/public_tree/100000'

# 压力场景将 feature 作为第四个参数
./scripts/bench-resource-usage.sh \
  vektra scalability_stress 'stress/select/public_tree_build/1000000' stress-bench
```

```powershell
# Windows PowerShell 7+
./scripts/bench-resource-usage.ps1 `
  -Package vektra `
  -Target component_scalability `
  -Filter 'select/build/public_tree/100000'

./scripts/bench-resource-usage.ps1 `
  -Package vektra `
  -Target scalability_stress `
  -Filter 'stress/select/public_tree_build/1000000' `
  -Features stress-bench
```

进程级 CPU 百分比可因多线程而超过 100%；峰值 RSS/working set 是整个筛选场景进程的
高水位，不是 allocations/op，也不能在不同操作系统间直接比较。为减少噪声，一次只过滤
一个场景，并在相同机器、空闲负载和相同工具链下比较。

压力 target 需要显式 feature，支持同样的名称过滤。它们可能长时间占用大量 CPU 和内存，
运行前应先在较小常规规模探测资源余量，避免系统 OOM：

```bash
cargo bench -p vektra --bench scalability_stress --features stress-bench -- 'stress/select/public_tree_build'
cargo bench -p vektra-theme --bench theme_stress --features stress-bench -- 'stress/theme/json_parse'
```

CI 只通过 `cargo test --workspace --benches` 执行轻量 harness 冒烟，并编译压力 target；
不会运行 Criterion 统计、压力场景或基于托管 runner 绝对耗时做性能判定。Criterion 的估计、
置信区间、异常值和吞吐量保存在本地 `target/criterion/`，该目录及生成的图表不应提交。

文档：

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
cd docs
bun install --frozen-lockfile
bun run dev
VEKTRA_DOCS_BASE=/vektra/ bun run build
```

## Workspace

```text
.
├── assets/              # 默认主题与可选内置图标资源
├── crates/
│   ├── assets/          # vektra-assets
│   ├── icons/           # vektra-icons
│   ├── theme/           # vektra-theme
│   ├── vektra/          # 组件门面 crate
│   └── macros/          # vektra-macros 派生宏
├── docs/                # VitePress 文档站与 GPUI WASM preview
└── examples/            # 桌面示例
```

GPUI 锁定到 Zed revision `fd82517a115d97a07835b52f0512b22b38e38ccf`。仓库不使用 crates.io 的浮动 GPUI，也不使用 `branch = "main"`。
