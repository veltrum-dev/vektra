# Checkbox

Checkbox 是受控复选框组件，适合单项开关、批量选择和父级部分选中状态。它不是表单框架，不保存内部业务状态，也不提供验证或错误消息 API。

## 基础用法

<VektraExample demo="checkbox/basic" title="Checkbox 基础用法" :height="220">

<<< ../../preview/src/demos/checkbox.rs#checkbox-example-basic{rust}

</VektraExample>

`Checkbox::new(id)` 默认未选中。`.checked(...)` 表示当前受控值，不是初始值；每次激活只计算下一 checked 值并调用 `on_change`，由宿主更新状态并 `cx.notify()`。

## 批量选择

<VektraExample demo="checkbox/bulk" title="Checkbox 批量选择" :height="340">

<<< ../../preview/src/demos/checkbox.rs#checkbox-example-bulk{rust}

</VektraExample>

父级 Checkbox 可以从子项推导 `checked` 和 `indeterminate`：全部子项选中时父级 checked，部分选中时父级 mixed。父级激活后统一写入所有子项；“反选”等批量操作由宿主状态实现，不需要 Checkbox 增加额外 API。

## 纯图标状态

<VektraExample demo="checkbox/icon-only" title="Checkbox 纯图标状态" :height="220">

<<< ../../preview/src/demos/checkbox.rs#checkbox-example-icon-only{rust}

</VektraExample>

`indicator_icons(unchecked, checked)` 使用两张状态图标替代默认方框。纯图标 Checkbox 不设置可见 label，但必须通过 `aria_label(...)` 提供可访问名称；整个命中区域仍可点击，hover 和 pressed 视觉只作用于状态图标。

## API

| API | 说明 |
| --- | --- |
| `Checkbox::new(id)` | 创建带稳定 `ElementId` 的复选框。 |
| `.checked(bool)` | 设置当前受控 checked 值，默认 `false`。 |
| `.indeterminate(bool)` | 设置部分选中状态；视觉和无障碍优先于 checked。 |
| `.disabled(bool)` | 禁用鼠标、触摸和键盘激活。 |
| `.label(text)` | 设置可见文本 label，并默认作为可访问名称。 |
| `.size(ComponentSize)` | 设置显式尺寸；未设置时读取全局默认尺寸。 |
| `.cursor_style(CursorStyle)` | 设置可用状态光标；disabled 优先。 |
| `.unchecked_icon(icon)` | 覆盖未选中状态图标，默认无图标。 |
| `.checked_icon(icon)` | 覆盖选中状态图标，默认对勾。 |
| `.indeterminate_icon(icon)` | 覆盖部分选中状态图标，默认横线。 |
| `.indicator_icons(unchecked, checked)` | 使用未选中/选中图标替代默认方框指示器。 |
| `.aria_label(text)` | 覆盖或提供可访问名称。 |
| `.aria_description(text)` | 提供补充无障碍描述。 |
| `.on_change(handler)` | 同步回调，参数为下一 checked 值、`Window` 和 `App`，不携带 `ClickEvent`。 |
| `.on_change_in(cx, handler)` | 绑定宿主 Entity 状态的同步回调。 |
| `.on_focus(handler)` / `.on_blur(handler)` | 注册真实聚焦与失焦转换回调。 |
| `.on_focus_in(cx, handler)` / `.on_blur_in(cx, handler)` | 注册绑定宿主 Entity 的焦点回调。 |

## 状态

未选中激活后回调 `true`，已选中激活后回调 `false`。`indeterminate(true)` 激活后统一回调 `true`，宿主通常同时清除 indeterminate。

根节点使用 `Role::CheckBox`，并映射为 `Toggled::False`、`Toggled::True` 或 `Toggled::Mixed`。没有可见 label 时必须提供 `aria_label(...)`。

## 键盘与交互

可用 Checkbox 可通过 Tab 聚焦。Space 激活；Enter 不激活。label 和方框共享同一个点击目标，单次激活只触发一次回调。disabled 状态不进入正常 Tab 顺序，不触发回调，并使用 disabled 视觉和光标。

`on_change` 与焦点生命周期相互独立：checked/indeterminate 变化不产生 focus/blur，焦点转换也不调用 `on_change`。同一 `ElementId` 重绘使用最新焦点 handler；`_in` 只表示 Entity 绑定，不是 DOM `focusin`。完整契约见 [`Focusable`](/api/focusable)。当前带 `on_change` 的鼠标激活会阻止 GPUI 默认鼠标焦点转移；Tab 与 GPUI 实际程序化焦点转换仍会触发回调。本次不新增 `focus()` 或 `focus_handle()`。

## 尺寸

`ComponentSize::{Xs, Sm, Md, Lg}` 是所有组件共享的语义尺寸。`component_size(cx)` 读取全局默认值，`set_component_size(size, cx)` 修改全局默认值并刷新窗口。显式 `.size(...)` 优先于全局默认值。

## 异步任务

`on_change` 和 `on_change_in` 保持同步。如果需要发起 HTTP 请求或异步校验，请在回调体内使用宿主 Entity 的 `cx.spawn` / `cx.spawn_in`，并按生命周期需要保存返回的 `Task`。

`Checkbox` 实现 [`Changeable<bool>`](/api/changeable)；固有 builder 与 trait 调用委托到同一实现。

## 主题、响应式与跨平台

Checkbox 的 normal、hover、pressed、focus-visible、checked、mixed 与 disabled 状态都来自当前 Light、Dark 或 System 主题 token。组件本身保持紧凑的单行命中区域；窄容器中的换行与列宽由宿主布局控制，纯图标模式仍须提供 `aria_label`。macOS、Windows、Linux 与 Web 预览共用同一 GPUI 实现，平台只影响系统焦点遍历、字体和输入映射。

## 已知限制

- 首版 label 只支持纯文本。
- 不提供非受控状态、`default_checked`、验证、错误消息或 FormControl。
- 自定义图标只作为视觉图形，不创建额外可访问名称。
