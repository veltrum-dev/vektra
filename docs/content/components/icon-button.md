# IconButton

IconButton 是固定正方形、只显示图标的操作控件，适合工具栏、紧凑标题栏和已有清晰上下文的常用操作。需要可见文字、复杂内容或导航链接时应使用其他组件。

## 实时预览

<VektraPreview demo="icon-button/basic" title="IconButton 预览" :height="360" />

预览源码实际编译进共享 GPUI WASM runtime：

<<< ../../preview/src/demos/icon_button.rs{rust}

## Anatomy 与 API

根节点提供 Button role、正方形命中区域、主题状态和唯一 Tab stop；内部 `Icon` 是装饰图形，不产生第二个名称或焦点目标。

| API | 说明 |
| --- | --- |
| `IconButton::new(id, icon)` | 创建稳定 `ElementId` 的纯图标按钮。 |
| `.aria_label(text)` | 设置必需的可访问名称；视觉 Tooltip 不能替代它。 |
| `.aria_description(text)` | 设置辅助技术朗读的补充说明。 |
| `.tooltip(text_or_tooltip)` | 接受字符串或 `Tooltip` 配置；可设置 `open`、箭头、颜色和动画。 |
| `.tooltip_placement(TooltipPlacement)` | 设置 Tooltip 优先位置，默认 `Bottom`；空间不足时自动 flip/shift。 |
| `.variant(...)` | `Primary`、`Outline`、`Ghost`、`Destructive`、`Secondary`。 |
| `.size(...)` | `Xs` 24px、`Sm` 32px、`Md` 36px（默认）、`Lg` 40px。 |
| `.icon_color(color)` | 只覆盖 enabled 图标颜色；disabled token 仍优先。 |
| `.disabled(bool)` | 阻止鼠标/键盘激活并退出 Tab 顺序。 |
| `.on_click(...)` / `.on_click_in(...)` | 注册鼠标、Enter 和 Space 共用的激活契约。 |

## 状态、键盘与无障碍

normal、hover、pressed、focus-visible 和 disabled 使用 Button 主题状态矩阵。Tab/Shift+Tab 由宿主的 GPUI 焦点遍历接线；Enter 在 keydown 激活，Space 在 keyup 激活。字符串 Tooltip 在 hover 或键盘焦点停留 500ms 后出现；配置对象支持立即 `open(true)` 或强制 `open(false)`。焦点移出会启动退出过渡，Escape 关闭但不移动焦点；受控 true 必须经历 `false -> true` 才能恢复。鼠标点击产生的焦点不会启动键盘 Tooltip。

纯图标按钮必须显式提供 `aria_label`。`aria_description` 是补充语义；Tooltip 是视觉提示，二者互不自动复制，避免重复朗读。disabled 按钮不可聚焦和激活，但仍允许 hover Tooltip 解释禁用原因。

## 主题、响应式与平台

Light、Dark、System 均从语义 token 解析；Tooltip 固定实例颜色的主题适配与对比度由调用方负责。Tooltip 动画默认开启并遵循 GPUI reduce-motion。IconButton 使用逻辑像素与 SVG，适应高 DPI；窄父容器不会改变其正方形尺寸。桌面和 WASM 使用同一 GPUI 组件路径。宿主负责应用级快捷键、Tab/Shift+Tab Action 和平台窗口生命周期。

## 当前限制

- 不显示 label，也没有 `Link` variant。
- Tooltip 只支持文本，不是可访问名称的替代品。
- 不公开任意 padding、圆角、背景或命中区域样式透传。
- hover/pressed 需在真实指针环境人工确认；预览需要 WebGPU 与宿主中文字体。
