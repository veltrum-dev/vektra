# Button

Button 表达操作，例如保存、确认、切换模式或启动长任务。它是普通 GPUI element，公开导出路径为 `vektra::Button`。

只展示状态的文本、独立进度面板、需要方向键选择的复合列表项，不应使用 Button。没有可见文字的图标操作应使用 `IconButton` 并提供可访问名称。

## 实时预览

<VektraPreview demo="button/basic" title="Button 预览" :height="760" />

## 基础用法

使用 `.label(...)` 设置文字，通过 `.on_click(...)` 或 `.on_click_in(...)` 处理激活。

<<< ../../preview/src/demos/button.rs#button-basic{rust}

## 有状态组件

`Button::on_click_in(cx, ...)` 可以访问宿主 `&mut T`、`ClickEvent`、`Window` 和 `Context<T>`。修改宿主状态后调用 `cx.notify()` 触发渲染。

<<< ../../preview/src/demos/button.rs#button-states{rust}

## Loading、selected 与 progress

`.loading(bool)`、`.progress(f32)` 和 `.selected(bool)` 都是由宿主控制的状态输入。Button 不启动异步任务、不计算进度，也不会在点击后自行翻转 selected；宿主应更新状态并调用 `cx.notify()`。

- loading 与 progress 共用互斥 activity 状态，后调用者生效；`.loading(false)` 恢复空闲。
- loading 用旋转指示器替换 start icon，但保留原始 label 和 end icon。动画通过 GPUI `AnimationExt` 自动遵守 reduce-motion。
- progress 保留 label 和两侧图标，在按钮底部绘制不改变外部尺寸的进度条。
- selected 与 activity 独立；只有显式调用 `.selected(false|true)` 才会暴露 toggle 语义。selected 使用持久内侧描边，不只依赖颜色。
- disabled 优先级最高：使用 disabled 样式并退出 Tab 顺序；activity 指示仍可见，但不会激活。

loading/progress 期间 Button 保留焦点与 `Role::Button`，但会消费鼠标、Enter 和 Space，防止重复提交和向父元素冒泡。取消操作应使用独立 Button。

## 变体与尺寸

Button 提供 6 种 `ButtonVariant` 和 4 种 `ComponentSize`。disabled 会使用当前 variant 的 disabled token。
预览中的普通 variant 行统一使用默认 `Md` 尺寸；独立尺寸对比区域刻意展示 `Xs`、`Sm`、`Md`、`Lg`，高度差异并非渲染不一致。

<<< ../../preview/src/demos/button.rs#button-variants{rust}

## 图标

前置图标、后置图标、双图标、固定宽度和 full-width 使用同一个 `Button` API。

<<< ../../preview/src/demos/button.rs#button-icons{rust}

## 中文自动空格

中文自动空格默认开启，也可以显式开启或关闭。长文本、中英文混合文本和数字混合文本不会被强行改写。

<<< ../../preview/src/demos/button.rs#button-auto-space{rust}

## 宽度

Button 默认按内容收缩。`.width(...)` 设置固定宽度，`.full_width()` 使用父布局提供的完整宽度。两者写入同一个宽度状态，后调用者生效。

<<< ../../preview/src/demos/button.rs#button-width{rust}

## 能力 trait

| Trait | 契约 |
| --- | --- |
| [`Clickable`](/api/clickable) | 提供 `on_click(...)` 和 `on_click_in(...)`。鼠标点击、聚焦后的 Enter 和 Space 会进入同一回调契约。 |
| [`Focusable`](/api/focusable) | 提供真实焦点转换的 `on_focus`、`on_blur` 及 Entity 绑定版本。 |
| `Disableable` | 提供 `disabled(bool)`。`disabled(true)` 同时阻止鼠标点击和 Enter/Space 键盘激活。 |

## 构造函数与 API

| API | 说明 |
| --- | --- |
| `Button::new(id)` | 创建带稳定 `ElementId` 的 Button。`id` 用于 GPUI 交互状态、焦点和测试定位。 |
| `.label(label)` | 设置可见文本。无障碍名称使用原始 label。 |
| `.variant(ButtonVariant)` | 设置视觉语义，默认 `Primary`。 |
| `.size(ComponentSize)` | 设置尺寸，默认 `Md`。 |
| `.width(width)` | 设置 GPUI `DefiniteLength` 宽度，例如 `gpui::px(200.)`。 |
| `.full_width()` | 填满父布局提供的可用宽度。与 `.width(...)` 同属一个宽度状态，后调用者生效。 |
| `.start_icon(icon)` | 设置前置装饰图标，后一次调用覆盖前一次。 |
| `.end_icon(icon)` | 设置后置装饰图标，后一次调用覆盖前一次。 |
| `.disabled(bool)` | 设置禁用状态。 |
| `.loading(bool)` | 设置不确定 activity；`true` 阻止激活，`false` 恢复空闲。与 `.progress(...)` 后调用者生效。 |
| `.progress(value)` | 设置确定进度并阻止激活。值域为 `0.0..=1.0`，越界与非有限值会安全归一。 |
| `.selected(bool)` | 显式设置受控 toggle 状态；组件不会自动翻转。 |
| `.auto_insert_space(bool)` | 控制两个汉字 label 的视觉自动空格，默认开启。 |
| `.tooltip(text_or_tooltip)` | 接受字符串或 `Tooltip` 配置对象；字符串保持 500ms 自动触发，配置对象可设置 `open`、箭头、颜色和动画。 |
| `.tooltip_placement(TooltipPlacement)` | 设置 Tooltip 优先位置，默认 `Bottom`；空间不足时仍会自动 flip/shift。 |
| `.aria_description(text)` | 设置与视觉 Tooltip 相互独立的可访问补充描述。 |
| `.on_click(handler)` | 注册标准 GPUI 点击回调：`Fn(&ClickEvent, &mut Window, &mut App)`。 |
| `.on_click_in(cx, handler)` | 注册可访问宿主 Entity 状态的回调。 |
| `.on_focus(handler)` / `.on_blur(handler)` | 注册标准 GPUI 聚焦与失焦回调。 |
| `.on_focus_in(cx, handler)` / `.on_blur_in(cx, handler)` | 注册可修改宿主 Entity 并调用 `cx.notify()` 的焦点回调。 |
| `.id()` | 返回稳定 `ElementId`。 |
| `.label_text()` | 返回用户传入的原始 label。 |
| `.display_label()` | 返回视觉显示 label。 |

## ButtonVariant

| 变体 | 用途 |
| --- | --- |
| `Primary` | 主要操作，默认变体。 |
| `Outline` | 带边框的次要操作。 |
| `Ghost` | 背景透明、hover 时显示反馈的轻量按钮。 |
| `Destructive` | 危险或不可逆操作。 |
| `Secondary` | 次要实体按钮。 |
| `Link` | 文本链接外观，但无障碍角色仍是 Button；hover 时会绘制下划线。 |

## ComponentSize

| 尺寸 | 高度 |
| --- | --- |
| `Xs` | 24px |
| `Sm` | 32px |
| `Md` | 36px，默认尺寸 |
| `Lg` | 40px |

图标尺寸、内容间距、水平 padding、圆角、字号和状态颜色由主题中当前 size 与 variant 的 token 决定。
未显式调用 `.size(...)` 时，Button 会读取 `component_size(cx)` 的全局默认值；`set_component_size(size, cx)` 会刷新窗口并影响未显式覆盖尺寸的 Button、IconButton 和 Checkbox。

文本过窄时会截断，原始 label 仍保留为无障碍名称。

## 图标插槽

`start_icon(...)` 和 `end_icon(...)` 接受实现 `IntoIconSource` 的值。图标是装饰内容，不产生额外无障碍名称；Button 的可访问名称始终来自原始 label。只显示图标的操作请使用 `IconButton`。

## Disabled

`disabled(true)` 会移除可聚焦 tab index，不注册鼠标点击回调，也不注册 Enter/Space 键盘回调。视觉上使用当前 variant 的 disabled token，并显示不可操作的 cursor。

## Activity 与进度值

`.loading(true)` 表示不确定进度；`.progress(value)` 表示确定进度。有限 progress 值夹取到 `0.0..=1.0`，正无穷归一为 `1.0`，负无穷和 NaN 归一为 `0.0`。多个 activity builder 连用时后调用者生效。

activity 只表达状态与阻止重复激活，不拥有任务生命周期。任务完成、失败、重试和取消协议由宿主应用负责。

## 中文自动空格

默认情况下，label 恰好由两个 Unicode Han 字符组成时，Button 会在视觉显示文本中插入一个普通空格，例如 `保存` 显示为 `保 存`。该行为不改变原始 label，也不改变无障碍名称。调用 `.auto_insert_space(false)` 可关闭它；一个字、三个及以上字符、已有空白、英文或混合字符不会被改写。

## 鼠标与键盘

可用状态下，左键点击会阻止默认行为、停止传播并触发回调。聚焦 Button 后，Enter 在 keydown 触发，Space 在 keyup 触发；两者都会构造 `ClickEvent::Keyboard` 并进入同一点击回调。selected Button 仍使用相同激活路径。loading/progress 会消费鼠标和 Enter/Space（包括阻止 Space 默认滚动）但不触发业务回调；disabled 不触发这些路径。

Button 注册 GPUI Tab stop，但在当前锁定 GPUI revision 中，宿主窗口仍需把真实 Tab/Shift+Tab 绑定到 `window.focus_next(cx)`/`focus_prev(cx)`；快速开始和桌面 example 展示了最小接线。字符串 Tooltip 在键盘焦点停留 500ms 后显示；`Tooltip::new(...).open(true)` 无需焦点即可显示，`open(false)` 强制关闭。Escape 关闭提示但保留按钮焦点；受控 `open(true)` 被关闭后需经历 `false -> true` 才会恢复。

## 焦点与无障碍

焦点回调只由真实转换触发，与 `on_click`、selected、loading 和 progress 相互独立；同一 `ElementId` 重绘不会重复触发，并会使用最新 handler。Tooltip 与业务回调复用一个焦点句柄。`_in` 表示宿主 Entity 绑定，不是 DOM `focusin`；共享契约见 [`Focusable`](/api/focusable)。

Tab/Shift+Tab（由宿主接到 GPUI 遍历）与任何针对该 GPUI 焦点身份的程序化转换都会触发相同生命周期。Vektra 本次不暴露 `focus()` 或 `focus_handle()`。当前 Button 激活 handler 会在左键按下时阻止默认行为，因此“点击并激活”不会额外强制转移焦点；未注册激活 handler 时，GPUI 对可聚焦元素的默认鼠标转移仍生效。

Button 根节点始终使用 `Role::Button`，并用原始 label 设置 `aria_label`。可用及 busy 状态设置 `tab_index(0)`，`focus_visible` 使用主题中的 focus token 和 focus width；disabled 退出 Tab 顺序。

显式调用 `.selected(false|true)` 后，根节点通过 `aria_toggled` 报告 False/True；未调用时不报告 toggle 状态。loading/progress 使用从 Button `ElementId` 派生的稳定子 ID 和 `Role::ProgressIndicator`，可访问名称复用原始 label。确定进度报告最小值 0、最大值 100 和当前百分比。

## 主题

Button 的 normal、hover、pressed、focus-visible、disabled 及 selected 状态都来自 Vektra 主题 token。默认 Light/Dark 主题为每个 variant 提供完整 selected 状态矩阵；旧自定义主题可以省略 selected 扩展，运行时会用 pressed、focus-visible 和 disabled token 组合回退。loading/progress 颜色从当前可见前景色派生，不在渲染期解析 JSON 或读取文件。

loading 与 Tooltip 动画使用 GPUI `AnimationExt`；系统或宿主启用 reduce-motion 后显示静态帧且不再请求装饰动画帧。Tooltip 固定实例颜色不会自动适配 Light/Dark/System，对比度由调用方负责。文档预览跟随 VitePress 当前 Light/Dark 主题；独立预览的 `theme=light|dark` 查询参数可强制主题，未提供或非法时使用 `ThemeMode::System`。

## 响应式

Button 是叶子组件，默认不负责布局换行。它会保持内容在自身内部居中，文本区域使用 `min_w_0`、`overflow_hidden`、`whitespace_nowrap` 和 `text_ellipsis`，避免窄宽度下撑破父布局。需要整行操作时使用 `.full_width()` 并由父容器控制可用宽度。

## 当前限制

- Button 不拥有异步任务、进度计算、自动 selected 翻转或取消协议。
- loading/progress 是不可激活的提交状态；需要取消时应提供独立 Button。
- `Link` 是按钮语义的链接外观，不会变成导航链接。
- 图标插槽不支持单独指定像素尺寸，尺寸由 ComponentSize token 决定。
- 预览运行依赖浏览器 WebGPU 和文档预览宿主提供的字体资源。
