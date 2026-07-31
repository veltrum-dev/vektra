# Button

Button 表达一次性操作，例如保存、确认、切换模式或进入下一步。它是普通 GPUI element，公开导出路径为 `vektra::Button`。

只展示状态的文本、长任务进度、需要方向键选择的复合列表项，不应使用 Button。没有可见文字的图标操作应使用 `IconButton` 并提供可访问名称。

## 实时预览

<VektraPreview demo="button/basic" title="Button 预览" :height="760" />

## 基础用法

使用 `.label(...)` 设置文字，通过 `.on_click(...)` 或 `.on_click_in(...)` 处理激活。

<<< ../../preview/src/demos/button.rs#button-basic{rust}

## 有状态组件

`Button::on_click_in(cx, ...)` 可以访问宿主 `&mut T`、`ClickEvent`、`Window` 和 `Context<T>`。修改宿主状态后调用 `cx.notify()` 触发渲染。

<<< ../../preview/src/demos/button.rs#button-states{rust}

## 变体与尺寸

Button 提供 6 种 `ButtonVariant` 和 4 种 `ButtonSize`。disabled 会使用当前 variant 的 disabled token。
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
| `Clickable` | 提供 `on_click(...)` 和 `on_click_in(...)`。鼠标点击、聚焦后的 Enter 和 Space 会进入同一回调契约。 |
| `Disableable` | 提供 `disabled(bool)`。`disabled(true)` 同时阻止鼠标点击和 Enter/Space 键盘激活。 |

## 构造函数与 API

| API | 说明 |
| --- | --- |
| `Button::new(id)` | 创建带稳定 `ElementId` 的 Button。`id` 用于 GPUI 交互状态、焦点和测试定位。 |
| `.label(label)` | 设置可见文本。无障碍名称使用原始 label。 |
| `.variant(ButtonVariant)` | 设置视觉语义，默认 `Primary`。 |
| `.size(ButtonSize)` | 设置尺寸，默认 `Md`。 |
| `.width(width)` | 设置 GPUI `DefiniteLength` 宽度，例如 `gpui::px(200.)`。 |
| `.full_width()` | 填满父布局提供的可用宽度。与 `.width(...)` 同属一个宽度状态，后调用者生效。 |
| `.start_icon(icon)` | 设置前置装饰图标，后一次调用覆盖前一次。 |
| `.end_icon(icon)` | 设置后置装饰图标，后一次调用覆盖前一次。 |
| `.disabled(bool)` | 设置禁用状态。 |
| `.auto_insert_space(bool)` | 控制两个汉字 label 的视觉自动空格，默认开启。 |
| `.on_click(handler)` | 注册标准 GPUI 点击回调：`Fn(&ClickEvent, &mut Window, &mut App)`。 |
| `.on_click_in(cx, handler)` | 注册可访问宿主 Entity 状态的回调。 |
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

## ButtonSize

| 尺寸 | 高度 |
| --- | --- |
| `Xs` | 24px |
| `Sm` | 32px |
| `Md` | 36px，默认尺寸 |
| `Lg` | 40px |

图标尺寸、内容间距、水平 padding、圆角、字号和状态颜色由主题中当前 size 与 variant 的 token 决定。

文本过窄时会截断，原始 label 仍保留为无障碍名称。

## 图标插槽

`start_icon(...)` 和 `end_icon(...)` 接受实现 `IntoIconSource` 的值。图标是装饰内容，不产生额外无障碍名称；Button 的可访问名称始终来自原始 label。只显示图标的操作请使用 `IconButton`。

## Disabled

`disabled(true)` 会移除可聚焦 tab index，不注册鼠标点击回调，也不注册 Enter/Space 键盘回调。视觉上使用当前 variant 的 disabled token，并显示不可操作的 cursor。

## 中文自动空格

默认情况下，label 恰好由两个 Unicode Han 字符组成时，Button 会在视觉显示文本中插入一个普通空格，例如 `保存` 显示为 `保 存`。该行为不改变原始 label，也不改变无障碍名称。调用 `.auto_insert_space(false)` 可关闭它；一个字、三个及以上字符、已有空白、英文或混合字符不会被改写。

## 鼠标与键盘

可用状态下，左键点击会阻止默认行为、停止传播并触发回调。聚焦 Button 后，Enter 在 keydown 触发，Space 在 keyup 触发；两者都会构造 `ClickEvent::Keyboard` 并进入同一点击回调。disabled 状态不会触发这些路径。

## 焦点与无障碍

Button 渲染为带 `Role::Button` 的 GPUI 交互元素，并用原始 label 设置 `aria_label`。可用状态设置 `tab_index(0)`，`focus_visible` 使用主题中的 focus token 和 focus width。

## 主题

Button 的 normal、hover、pressed、focus-visible 和 disabled 状态都来自 Vektra 主题 token。文档预览跟随 VitePress 当前 Light/Dark 主题；切换主题不会丢失点击状态。独立打开预览时，合法的 `theme=light|dark` 查询参数会强制对应主题；未提供或值非法时使用 `ThemeMode::System`。

## 响应式

Button 是叶子组件，默认不负责布局换行。它会保持内容在自身内部居中，文本区域使用 `min_w_0`、`overflow_hidden`、`whitespace_nowrap` 和 `text_ellipsis`，避免窄宽度下撑破父布局。需要整行操作时使用 `.full_width()` 并由父容器控制可用宽度。

## 当前限制

- Button 没有 loading、selected 或 progress 状态。
- `Link` 是按钮语义的链接外观，不会变成导航链接。
- 图标插槽不支持单独指定像素尺寸，尺寸由 ButtonSize token 决定。
- 预览运行依赖浏览器 WebGPU 和文档预览宿主提供的字体资源。
