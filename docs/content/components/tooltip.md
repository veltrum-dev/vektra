# Tooltip

Tooltip 为 Button/IconButton 提供简短、补充性的纯文本说明。完成任务所必需的信息必须在其他位置可见；错误信息、复杂帮助、表单校验和交互内容不应放进 Tooltip。

## 基础用法

<VektraExample demo="tooltip/basic" title="Tooltip 基础用法" :height="260">

<<< ../../preview/src/demos/tooltip.rs#tooltip-example-basic{rust}

</VektraExample>

## 受控显示

<VektraExample demo="tooltip/controlled" title="Tooltip 受控显示" :height="280">

<<< ../../preview/src/demos/tooltip.rs#tooltip-example-controlled{rust}

</VektraExample>

## 优先位置

<VektraExample demo="tooltip/placements" title="Tooltip 优先位置" :height="340">

<<< ../../preview/src/demos/tooltip.rs#tooltip-example-placements{rust}

</VektraExample>

## API 与语义

`Tooltip::new("设置")` 创建配置对象；`Tooltip::text("设置", cx)` 生成使用默认配置的 GPUI `AnyView` factory。Button/IconButton 的 `.tooltip(...)` 同时接受 `&str`、`String`、`SharedString` 和 `Tooltip`，所以已有 `.tooltip("设置")` 无需迁移。`.tooltip_placement(TooltipPlacement::TopStart)` 指定优先位置；默认是 `Bottom` 居中，视口碰撞处理仍可能翻转或平移。

| API | 默认值与语义 |
| --- | --- |
| `Tooltip::new(text)` | 创建纯文本配置，默认自动触发、显示箭头并启用动画。 |
| `.open(bool)` | 设置显式受控状态；未调用时使用 hover/键盘焦点自动触发。 |
| `.arrow(bool)` | 默认 `true`；`false` 不绘制箭头，也不预留箭头高度，但保留 anchor gap。 |
| `.color(impl Into<Hsla>)` | 覆盖当前实例的文字色。 |
| `.bg_color(impl Into<Hsla>)` | 覆盖当前实例的气泡和箭头背景。 |
| `.animated(bool)` | 默认 `true`；`false` 立即进入显隐终态。 |

`color` 与 `bg_color` 可直接接收 `gpui::rgb(...)`，无需调用 `.into()`。未覆盖的边框、阴影、圆角、padding、字号和定位 token 继续来自当前主题。

`aria_label` 是名称，`aria_description` 是辅助技术的补充说明，Tooltip 是视觉补充；Vektra 不在三者间自动复制。纯图标按钮的 Tooltip 不能替代 `aria_label`。

## 生命周期与交互

- 未调用 `open(...)` 时，hover 或 Tab/Shift+Tab 产生的键盘焦点保持 500ms 后显示。
- `open(true)` 在 trigger 挂载后立即显示，不要求 hover/focus；`open(false)` 强制关闭并忽略自动资格。运行时切换会执行对应显隐过渡。
- 首次显示的 500ms 内离开、焦点移出或 owner 卸载会取消任务。显示后，指针离开 trigger 与气泡会进入 500ms 关闭宽限期，期限内移入任一区域会取消关闭；期限结束后才开始退出过渡。
- 鼠标点击产生的 focus 不启动键盘路径。
- Escape 关闭可见/等待中的 Tooltip，保留 trigger 焦点。自动模式必须离开并重新进入当前 hover/focus 周期；`open(true)` 则必须由调用方先传入 `false`、再传回 `true`，普通重渲染不会重新打开。
- hover 与 focus 共用一份 trigger 状态，同一 trigger 不绘制两个 Tooltip；窗口每帧只绘制一个 Tooltip。键盘聚焦 trigger 与另一个 hovered trigger 同时具备资格时，指针输入会结束旧的键盘资格，由 hovered trigger 接管。
- disabled trigger 不进入 Tab 顺序且不能激活，但 hover Tooltip 仍可解释禁用原因。

指针可以移入 Tooltip 气泡并维持其生命周期，但气泡仍不获取焦点、不进入 Tab 顺序、不接受鼠标点击，也不包含交互内容。Enter/Space 继续由 trigger 的业务回调处理。

## 定位、主题与性能

Tooltip 使用 trigger 实际 prepaint 子边界作为完整矩形锚点，不把它压缩成鼠标坐标。Vektra 在同一帧测量文本气泡并完成定位：先尝试 preferred placement；主轴空间不足时翻转到相反方向并保留 Start/Center/End；两侧都不足时选择空间较多的一侧；交叉轴越界时 shift。启用箭头时，它依据最终 side 绘制，在 shift 后重新指向 trigger，并避开圆角危险区；关闭箭头只移除箭头空间，主题 anchor gap 仍将气泡与 trigger 分开。

| Placement | 对齐语义 |
| --- | --- |
| `TopStart` / `BottomStart` | 气泡左边与 trigger 左边对齐。 |
| `Top` / `Bottom` | 气泡在 trigger 水平方向居中。 |
| `TopEnd` / `BottomEnd` | 气泡右边与 trigger 右边对齐。 |
| `LeftStart` / `RightStart` | 气泡顶部与 trigger 顶部对齐。 |
| `Left` / `Right` | 气泡在 trigger 垂直方向居中。 |
| `LeftEnd` / `RightEnd` | 气泡底部与 trigger 底部对齐。 |

气泡、箭头、surface 背景、foreground、边框、`radius.md` 圆角和轻量阴影默认由 Tooltip/语义/foundation token 管理。Light、Dark、System 使用当前主题；实例 `color`/`bg_color` 的优先级高于主题，但固定颜色不会自动适配主题，对比度由调用方负责。长中文/英文会在最大宽度内换行。极小视口无法同时容纳 trigger、间距、完整气泡和阴影安全区时，算法优先让内容保持可见，并采用 best-effort 定位，此时可能无法维持正常间距。

Tooltip 在 macOS、Windows、Linux 与 Web 预览中复用同一 GPUI 定位与生命周期实现；平台差异主要来自窗口边界、系统字体与宿主焦点遍历，不需要不同的组件 API。

默认进入动画约 120ms：淡入并沿最终 placement 方向移动约 2px；退出约 80ms 淡出。动画不改变测量、最终定位或 trigger 命中区域。`.animated(false)` 直接呈现终态；GPUI `App::reduce_motion` 开启时同样不请求装饰动画帧。只有配置 Tooltip 的 trigger 才创建小型 keyed state，显示延迟、关闭宽限期与过渡任务均有代次防护并随 owner 卸载取消；不可见时不布局 Tooltip。大量列表应配合虚拟化，只为实际挂载项创建状态。

## 限制

- 只支持纯文本，无富文本、链接、按钮或任意 child。
- 气泡 hover 仅维持显隐生命周期，不提供点击、焦点或交互子项；不提供自定义动画时长/easing/transition，也不提供任意边框、阴影、圆角、padding、offset 或 child builder。
- 不提供 Root、Provider、全局初始化、通用 Overlay 或公开 `Tooltipable` trait。
- 受 GPUI 限制，每个窗口同一帧最多绘制一个 Tooltip；Vektra 不提供额外全局仲裁。
- 宿主仍负责把真实 Tab/Shift+Tab 映射到 GPUI 焦点遍历。
