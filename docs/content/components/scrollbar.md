# Scrollbar

`ScrollArea` 只负责滚动模型、O(1) Scrollbar geometry 和交互，不会把已经完整构建的任意 `Div` 自动虚拟化。100K/1M 集合必须使用 [`VirtualList`](/components/virtual-list)，再复用同一套 Always/Auto/Never、Overlay/Stable、滚轮、键盘、轨道与 thumb 交互。

Scrollbar 把任意 GPUI `Div` 变成带 Vektra 自绘滚动条的滚动区域。它不要求 `window`、`cx`、Root 或 Provider；布局与子项先配置，最后调用 `.scrollbar()`。

## 基础用法

<VektraExample demo="scrollbar/basic" title="Scrollbar 基础用法" :height="390">

<<< ../../preview/src/demos/scrollbar.rs#scrollbar-example-basic{rust}

</VektraExample>

`scrollbar()` 默认等价于 `Both + Auto + Overlay`。Both 表示 X/Y 两轴都具备滚动能力，但只有实际发生溢出的轴才会绘制轨道和 Thumb。

## 显隐、Track 与 Gutter

<VektraExample demo="scrollbar/configuration" title="动态配置 Axis、Visibility 与 Gutter" :height="390">

<<< ../../preview/src/demos/scrollbar.rs#scrollbar-example-configuration{rust}

</VektraExample>

上方三组 Radio 会实时改变同一个滚动区域。示例使用宽 1160px 的内容画布并在 X/Y 两轴溢出，顶部色带也标明了横向内容方向；默认使用 `Both + Always + Overlay`，因此无需先滚动就能看到水平和垂直 Thumb。

`Auto`、`Always` 和 `Never` 分别表示交互时显示、Thumb 始终显示和完全不绘制滚动条。鼠标进入滚动栏命中区时，组件会显示完整 Track；移出后 Track 消失，Thumb 仍按照 visibility 策略保留。只有指针真正位于 Thumb 上时，它才会切换 hover 颜色并从默认 8px 加宽到 10px。

Gutter 控件下方的实时示意会把差异放大：`Stable` 始终预留主题的 14px `hit-thickness`，内容和滚动条各有独立槽位；`Overlay` 预留 0px，滚动条直接覆盖在内容边缘。

## API

先导入扩展 trait：

```rust
use vektra::ScrollableExt;
```

| API | 语义 |
| --- | --- |
| `.scrollbar()` | `Both + Auto + Overlay`。 |
| `.vertical_scrollbar()` / `.horizontal_scrollbar()` | 单轴快捷入口。 |
| `.scrollbar_for(&handle)` | 使用调用方持有的 `gpui::ScrollHandle`；也提供两个单轴 `*_scrollbar_for`。 |
| `.scrollbar_with(config)` | 一次传入完整 `ScrollbarConfig`。 |
| `.scrollbar_with_axis(...)` | 默认配置只覆盖 axis。 |
| `.scrollbar_with_visibility(...)` | 默认配置只覆盖 visibility。 |
| `.scrollbar_with_gutter(...)` | 默认配置只覆盖 gutter。 |

`.scrollbar()` 返回 `ScrollArea`。后续配置刻意使用 `.scrollbar_axis(...)`、`.scrollbar_visibility(...)`、`.scrollbar_gutter(...)`、`.scrollbar_id(...)` 和 `.scrollbar_aria_label(...)`，不会占用含义宽泛的 `.axis()` 或 `.visibility()` 名称。

```rust
use vektra::{
    ScrollAxis, ScrollGutter, ScrollVisibility, ScrollableExt,
};

let area = div()
    .h(px(240.))
    .child(content)
    .scrollbar()
    .scrollbar_axis(ScrollAxis::Vertical)
    .scrollbar_visibility(ScrollVisibility::Always)
    .scrollbar_gutter(ScrollGutter::Stable)
    .scrollbar_aria_label("通知列表");
```

`ScrollbarConfig` 自身使用简短 builder，因为它的命名空间已经明确：

```rust
let config = ScrollbarConfig::new()
    .axis(ScrollAxis::Both)
    .visibility(ScrollVisibility::Auto)
    .gutter(ScrollGutter::Overlay);
```

在循环或同一调用位置生成多个区域时，使用 `.scrollbar_id(...)` 提供稳定且唯一的 ID。

## 交互与无障碍

- 鼠标滚轮、触控板和 GPUI 原生滚动都直接更新同一个 `ScrollHandle`。
- Thumb 可拖动；点击轨道会把 Thumb 中心移动到指针位置并继续进入拖动。
- Track 仅在对应滚动轴 hover 或拖动时显示；Thumb 自身 hover 时会高亮并加宽。鼠标移出后只隐藏 Track，不会提前隐藏 Thumb。
- 聚焦滚动区域后，方向键按 40px 滚动；PageUp/PageDown 按约 90% 视口滚动；Home/End 到达主轴首尾。
- `Auto` 在指针移动、滚轮、轨道交互时平滑淡入，停止交互约 900ms 后淡出；拖动与轨道 hover 期间保持显示。系统启用“减少动态效果”时会跳过过渡。
- `.scrollbar_aria_label(...)` 为 `ScrollView` 提供名称，并配合主题 focus ring 显示键盘焦点。

Vektra 不公开要求调用方传入的 `window` 或 `cx`。组件在 GPUI Element 生命周期中用 keyed state 保留内部 `ScrollHandle` 与短生命周期显隐/拖动状态；`.scrollbar_for(...)` 则让虚拟列表、外部跳转或状态同步复用调用方的 handle。

## 主题与限制

`ResolvedTheme::scrollbar` 提供 track、默认/hover/pressed Thumb、focus ring、默认/hover 视觉宽度、命中宽度、最小 Thumb 长度和圆角 token。Thumb 始终按自身短边生成胶囊圆角；视觉宽度默认 8px、hover 时 10px，命中宽度默认 14px，最小 Thumb 长度默认 24px。

- `Never` 只隐藏 Vektra 自绘滚动条，内容仍可通过原生滚动和外部 `ScrollHandle` 滚动。
- V1 不提供 `System` 显隐策略：GPUI 各桌面后端当前没有一致且可依赖的系统 scrollbar 偏好接口，使用该名称会制造错误承诺。
- `scrollbar()` 应作为布局、尺寸、子项和原有交互配置之后的最后一个结构转换调用；返回的 `ScrollArea` 只开放明确的 Scrollbar 配置。

## 性能契约

- Scrollbar geometry、轨道点击、thumb drag 和键盘步进为 O(1)，交互状态和 Task 数量有常数上界。
- ScrollArea 不虚拟化调用方 children；100K/1M eager children 的成本属于完整 Element 树。大集合必须使用 [`VirtualList`](/components/virtual-list)。
- geometry、首次/稳态布局绘制和滚动更新由 `scrollbar` benchmark 覆盖。
