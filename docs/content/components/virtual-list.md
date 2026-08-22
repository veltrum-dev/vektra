# VirtualList

`VirtualList` 是固定行高、视窗级物化的公开集合组件。它接收项目总数、固定行高、稳定 key 和惰性 renderer；总高度由 `count × item height` 推导，不为每项创建 Entity、metadata、高度记录或 Element。

## 基础与百万项用法

<VektraExample demo="virtual-list/basic" title="VirtualList：1,000,000 项生成式数据" :height="430">

<<< ../../preview/src/demos/virtual_list.rs#virtual-list-example-basic{rust}

</VektraExample>

桌面示例 `cargo run --example virtual-list` 在同一窗口同时展示 100 项普通场景和 10,000,000 项生成式大数据场景，并显示 visible range、物化行数与缓存上限。

## API

| API | 说明 |
| --- | --- |
| `VirtualListState::new()` | 创建 O(1) 滚动状态；应跨帧复用。 |
| `VirtualList::new(id, state, count, item_height, key, renderer)` | 创建生成式固定行高列表。 |
| `VirtualList::from_data_source(...)` | 从统一 `LazyDataSource` 读取已加载项，并请求可见范围。 |
| `.revision(u64)` | 标记数据顺序、内容或加载状态 revision。 |
| `.request_visible_range(callback)` | 非阻塞通知宿主可见范围。 |
| `.scrollbar(ScrollbarConfig)` | 设置 Always/Auto/Never 与 Overlay/Stable；轴固定为垂直。 |
| `.aria_label(text)` | 设置 ScrollView 的可访问名称。 |
| `scroll_to_index` / `reveal_index` | 精确跳转或按需滚入索引。 |
| `scroll_to_start/middle/end` | 跳到开头、中间或末尾。 |
| `metrics()` | 读取 visible range、物化行数、renderer 调用和缓存上限。 |

`LazyDataSource` 提供 count、revision、稳定 key、按索引读取、loaded 状态和可见范围请求。`OwnedDataSource` 让 Vec/数组进入同一协议；大型生成式、分页或远程数据应直接实现 trait，且方法不得阻塞 UI 线程。

## 键盘、滚动与无障碍

列表报告 `ScrollView` 和可访问名称。滚轮、Arrow、PageUp/PageDown、Home/End、轨道点击与 thumb drag 使用同一个底层 `ScrollHandle`。Scrollbar 支持 `Always/Auto/Never` 和 `Overlay/Stable`。当前只把可见行导出到 Element/AccessKit 子树；真实 VoiceOver、NVDA 和 Orca 尚未人工验证。

## 性能契约

- 正常规模：100–100K；压力规模：1M 与 10M 生成式数据。
- request-layout、prepaint、paint、renderer 与 AccessKit：O(visible)，当前 overdraw 为 0。
- Vektra 附加状态：O(1)；逐行缓存硬上限为 0。
- item count 从 100K 增长到 1M 时，物化行数不得按十倍增长。
- 固定参考机器目标：1M 首绘 ≤50ms、稳态滚动/绘制 ≤8.33ms、生成式数据源附加 maximum RSS <128MB。
- Benchmark 与平台证据见[性能架构](/guide/performance)和[基准指南](/guide/benchmarks)。

## 适用与限制

推荐固定行高的大型日志、搜索结果、命令列表和分页数据。不支持可变高度、VirtualGrid、Table、Tree 或瀑布流；需要可变高度时必须使用明确记录 O(n) 高度索引成本的独立能力，不能把它宣传为严格 O(viewport) 内存。

Light、Dark、System 复用 Scrollbar 语义 token。代码目标覆盖 macOS、Windows、Linux 与 Web/WASM；当前本机和确定性测试已验证，其他平台性能和真实 GPU 仍为未验证。
