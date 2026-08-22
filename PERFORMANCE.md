# Vektra 性能契约

Vektra 是面向 Windows、Linux 和 macOS 的跨平台、高性能、可组合纯 GPUI 组件库。性能是公共组件正确性和 API 契约的一部分：集合组件必须惰性读取、视窗级物化并使用有界缓存；普通稳态帧不得隐藏全量扫描、阻塞 I/O 或意外 O(n²)。

## 证据边界

当前 Criterion 基准使用 GPUI `TestAppContext` / `VisualTestContext`，测量 CPU 侧状态、布局、prepaint 和测试绘制。结果不代表真实 GPU FPS、窗口合成器延迟或辅助技术人工体验。所有对外数字都必须注明机器、操作系统、Rust 1.98.0 与 GPUI revision `fd82517a115d97a07835b52f0512b22b38e38ccf`。

时间数字只在同一台固定 runner、相同工具链和相同 GPUI revision 间比较。共享 CI 只门禁算法不变量、物化数量、缓存上限和稳定 allocation 指标，不使用绝对毫秒作为跨机器门禁。

## 预算

机器可读预算位于 [`performance-budgets.json`](performance-budgets.json)。每项记录 benchmark 名称、场景规模、正常/压力分类、最大时间、allocation、allocated bytes、RSS、renderer 调用数、缓存行数和硬门禁状态。

- 120fps 稳态目标：8.33ms。
- 普通交互到下一次绘制：16.67ms。
- 正常规模 UI 线程不得出现超过 50ms 的单个同步任务。
- 固定 runner 时间显著回归超过 10%、allocation 超过 5%、maximum RSS 超过 10% 时失败。
- 算法复杂度、实际物化数量或缓存硬上限失败时直接失败。
- 预算调整必须附带 profiler、Criterion、allocation 与 CPU/RSS 证据，不能直接放宽。

尚未达到的绝对预算必须保留为未达标，不得静默改大。优先消除 O(n²)、全量 Element 树、无界缓存和生命周期泄漏。

## 组件契约

### VirtualList

- 固定行高模式的 Vektra 附加状态为 O(1)，总高度由 `count × item height` 推导。
- Element、布局、prepaint、paint 与 AccessKit 子树为 O(visible)，当前 overdraw 为 0。
- 不保存逐行 metadata、Entity、高度或 Element；缓存行硬上限为 0。
- 数据源的 `item` / `request_range` 不得阻塞。分页缺失项由宿主在后台加载。
- 1M/10M 场景必须使用生成式数据源，禁止先构建同规模 `Vec`。

### Select

- `T` 的公共约束为 `Clone + Eq + Hash + 'static`。
- option、group、`items` 与外部数据源进入同一状态、导航、焦点、AccessKit 和虚拟 Popup 内核。
- owned adapter 以临时 `HashSet` 做一次预期 O(n) first-canonical 校验，随后释放；不复制标签、描述或搜索文本。
- 外部大数据源负责 key/value 定位、enabled navigation 与 typeahead 索引；Vektra 不建立百万级 catalog。
- Popup 只物化可见行，Vektra 行缓存上限为 0。

### Input

- 64KiB 更新加绘制目标不超过 4ms；1MiB 不超过 16.67ms。
- 等规模替换的 allocated bytes 目标不超过输入大小 8 倍。
- 16MiB 属于压力规模，要求线性、无 OOM 和无异常历史/缓存增长，不承诺单帧完成。
- 显示缓存与值/Password 可见性绑定；程序化同步会清空 undo/redo，旧文本不得留存在历史中。

### Theme

- render 路径只读取缓存主题；默认主题缓存读取应为零分配或有等价证据。
- 默认主题冷构建目标不超过 5ms；100K token 解析目标不超过 100ms。
- alias 解析为 O(tokens + edges)，循环、类型错误和 profile 校验不得弱化。

### ScrollArea 与叶子组件

`ScrollArea` 的 Scrollbar geometry 为 O(1)，但它不会把任意已构建 `Div` 自动变成虚拟集合。大集合必须使用 `VirtualList`。Button、Checkbox、Switch、Radio、IconButton、Icon 与 Tooltip 的标准负载是 100 个可见组件；无限可见叶子树不属于组件承诺。

## 运行与资源采集

常规基准：

```sh
cargo bench -p vektra --bench component_scalability
cargo bench -p vektra-theme --bench theme_scalability
```

压力 target：

```sh
cargo bench -p vektra --bench scalability_stress --features stress-bench
cargo bench -p vektra-theme --bench theme_stress --features stress-bench
```

资源 wrapper 会先完成编译，再只包住精确筛选的 bench executable，并同时输出人类可读行与 `VEKTRA_PROCESS_METRICS_JSON`：

```sh
./scripts/bench-resource-usage.sh vektra component_scalability \
  'input/render/equal_size_update_and_draw/1048576'
```

Windows 使用 `scripts/bench-resource-usage.ps1`。统一 JSON 字段包含 wall time、user/system 或 total CPU、CPU percent、peak memory、exit status、platform、package、target 和 filter；rustc/Cargo 资源不计入目标进程。

## 平台状态

- macOS：当前开发机器已执行本页列出的代表性 CPU/RSS 基准。
- Windows：CI 编译/行为覆盖以实际 workflow 结果为准；没有专用固定性能 runner 时标为性能未验证。
- Linux：CI 编译/行为覆盖以实际 workflow 结果为准；没有专用固定性能 runner 时标为性能未验证。
- 真实 GPU、VoiceOver、NVDA、Orca、fractional scale 人工视觉与高 DPI 跨平台证据必须分别记录；未运行即为未验证。

## 2026-08-22 本机治理结果

参考环境：MacBookPro18,3 / Apple M1 Pro，macOS 26.5.1，Rust 1.98.0，GPUI
`fd82517a115d97a07835b52f0512b22b38e38ccf`。以下为 Criterion `--quick` 代表值；RSS 是整个精确
筛选 bench 进程高水位，不是单次操作增量。

| 场景 | before | after | 状态 |
| --- | ---: | ---: | --- |
| Select 10K 首次打开+绘制 | 3.0647s / 341.6MB RSS | 17.12ms / 41.1MB RSS | O(n²) 与全量 Popup 已消除；略高于 16.67ms 普通交互目标 |
| 1M 惰性 Select 首次打开 | 无有界路径 | 0.360ms / 23.2MB RSS | 通过 50ms 与 128MB 目标；真实 GPU 未验证 |
| VirtualList 1M 首绘 | 无 | 72.3µs / 26.7MB RSS | 通过；物化 ≤16，缓存 0 |
| VirtualList 10M 首绘 | 无 | 72.4µs / 24.1MB RSS | 通过；规模增长未增加物化行 |
| ScrollArea 100K eager children 首绘 | 824.7ms / 1.585GB RSS | 大集合改用 VirtualList：100K 72.8µs / 25.7MB RSS | ScrollArea 仍不自动虚拟化任意 Div |
| 100 个 mixed 叶子组件 | 原有约1–4ms范围 | 稳态 2.30ms；10% 更新 5.00ms；100% 更新 5.48ms | 通过 4/8.33/16.67ms 预算 |
| Input 1MiB 等规模更新+绘制 | 26.96ms | 5.29ms | 通过 16.67ms 时间目标 |
| Input 1MiB set_value allocation | 145.1 alloc / 49.56MB | 108.1 alloc / 26.25MB | 改善但未达到 ≤8MiB，仍未达标 |
| Input 16MiB set_value | 约249ms / 约900MB RSS | 53.55ms / 675.2MB RSS | 线性、无 OOM；压力场景仍不承诺单帧 |
| Input 16MiB 完整首绘 | 约192ms / 约758MB RSS | 33.29ms / 447.2MB RSS | 明显改善；真实 GPU 未验证 |
| Theme 100K token parse/resolve | 206.9ms | 90.24ms | 通过 100ms 时间目标 |
| Theme 100K allocation | 1,413,381 alloc / 182.76MB | 700,058 alloc / 146.50MB | 明显改善；allocated bytes 线性倍率目标仍未达标 |
| Theme 1M parse/resolve | 约3.19s / 约1.52GB RSS | 1.620s / 1.395GB RSS | O(n+edges)；压力成本仍高 |
| Tooltip 1K trigger focus+delay+draw | 无 | 76.43ms / 72.0MB RSS | 压力覆盖；owner/task 释放由确定性生命周期测试通过 |
| Icon 100K 同路径公共构建 | 无 | 18.84ms / 643.1MB 进程 RSS | 构建覆盖；SVG 缓存命中真实绘制仍依赖 GPUI |

Windows/Linux 专用 runner、真实 GPU FPS、系统合成器、VoiceOver/NVDA/Orca 和物理高 DPI 结果均
未验证，不得从本表推断跨平台绝对数字。
