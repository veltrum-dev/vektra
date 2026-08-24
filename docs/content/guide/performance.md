# 为什么 Vektra 快

Vektra 的产品定位是“跨平台、高性能、可组合的纯 GPUI 组件库”。性能不是口号旁边的可选优化，而是公共组件的正确性和 API 契约。

```text
Vec / 数组 / builder / 生成式 / 分页 / 远程数据
                         ↓
                  统一惰性数据源
                         ↓
            统一状态、导航、焦点、AccessKit
                         ↓
              visible + bounded overdraw
                         ↓
                GPUI 绘制 + Vektra Scrollbar
```

## 集合架构

集合组件只保留一个内核。便利 API 使用 owned adapter，分页与远程数据实现相同协议；不会建立 eager/lazy 两套渲染、导航和无障碍逻辑。固定行高路径通过 `count × height` 推导总高度，只物化当前视口。大型数据源负责 key/value、enabled navigation 与 typeahead 索引，render 线程只做非阻塞读取和 range request。

`VirtualList` 的 Vektra 状态为 O(1)，当前行缓存硬上限为 0。Select Popup 使用同一虚拟列表内核；owned option/group 通过临时 option-ID `HashSet` 和最终 value 索引保留预期 O(n) first-canonical 语义，外部百万项数据不建立全量 catalog。

## 热路径与生命周期

render、布局、prepaint 和 paint 中禁止阻塞 I/O、主题/JSON 解析、正则编译、全量数据构建和重复 SVG 解析。缓存必须记录 key、失效条件与硬上限；Task、Subscription、Entity、timer 和历史必须有 owner 与释放路径。warm steady-state 不得持续净增长。

## 预算与边界

参考环境的 120fps 稳态目标是 8.33ms，普通交互到下一次绘制目标是 16.67ms。它们不是对所有机器、GPU 或完整宿主应用的无条件承诺。机器可读预算位于仓库根 `performance-budgets.json`，完整契约位于 `PERFORMANCE.md`。

Criterion 的 GPUI 场景测量 CPU 侧状态、布局、prepaint 和测试绘制，不代表真实 GPU FPS 或系统合成器延迟。数字必须注明机器、系统、Rust 和 GPUI revision；没有专用 runner、真实辅助技术或物理平台证据时明确标为未验证。
