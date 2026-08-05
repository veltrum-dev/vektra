# Radio 与 RadioGroup

`RadioGroup<T>` 用于从互斥选项中选择一个值。它是受控组件：`selected_value(Option<T>)` 输入当前权威值，`on_change(T, ...)` 只请求下一值。`Radio<T>` 只能通过组的强类型 `child(Radio<T>)` 添加，不能独立渲染。

## 基础与受控用法

<VektraExample demo="radio/basic" title="RadioGroup 基础用法" :height="280">

<<< ../../preview/src/demos/radio.rs#radio-example-basic{rust}

</VektraExample>

宿主可以立即更新 `selected_value`，也可以先保存待审批值，接口成功后再提交。请求失败时继续传入旧值，已选视觉不会被组件提前改变。

## 禁用项与禁用组

<VektraExample demo="radio/disabled" title="Radio 禁用能力" :height="360">

<<< ../../preview/src/demos/radio.rs#radio-example-disabled{rust}

</VektraExample>

单项 disabled 会被方向导航跳过；组级 disabled 优先于所有单项状态，并让整组退出普通 Tab 顺序。

## 键盘导航

<VektraExample demo="radio/keyboard" title="RadioGroup 键盘导航" :height="320">

<<< ../../preview/src/demos/radio.rs#radio-example-keyboard{rust}

</VektraExample>

## 布局方向

<VektraExample demo="radio/orientation" title="RadioGroup 水平布局" :height="260">

<<< ../../preview/src/demos/radio.rs#radio-example-orientation{rust}

</VektraExample>

## Anatomy

```text
RadioGroup（Role::RadioGroup、方向、组名与描述）
└─ Radio（Role::RadioButton、选中状态、唯一 roving focus）
   ├─ 圆形指示器与选中圆点
   └─ label + 可选 description
```

每个 Radio 是一个完整命中目标；label 与 description 不创建额外焦点。selected 除颜色外还由实心圆点与 `Toggled::True` 表达，focus-visible 使用独立边框。

## API

| API | 说明 |
| --- | --- |
| `RadioGroup::new(id)` | 创建默认无选中项、垂直方向的组。 |
| `.selected_value(Option<T>)` | 设置当前权威选中值。 |
| `.child(Radio<T>)` | 添加同值类型的强类型子项；不接受任意 `IntoElement`。 |
| `.on_change(handler)` / `.on_change_in(cx, handler)` | 请求下一值，不携带 `ClickEvent`。 |
| `.disabled(bool)` | 禁用整组并覆盖所有单项配置。 |
| `.size(ComponentSize)` | 为整组统一设置 `Xs`、`Sm`、`Md`、`Lg`。 |
| `.orientation(Orientation)` | 设置水平/垂直布局和可访问方向；默认垂直。 |
| `.aria_label` / `.aria_description` | 设置组级名称和描述。 |
| `Radio::new(id, value)` | 创建不可独立渲染的强类型单项。 |
| `.label` / `.description` | 设置可见语义文本及默认可访问名称/描述。 |
| `.aria_label` / `.aria_description` | 覆盖单项可访问名称/描述。 |
| `.disabled(bool)` | 禁止该项选择并从方向导航中跳过。 |
| `.on_focus` / `.on_blur` 及 `_in` | 观察真实单项焦点转换。 |

RadioGroup 实现 [`Changeable<T>`](/api/changeable)、[`Disableable`](/api/disableable)、[`Sizable`](/api/sizable)。Radio 实现 [`Focusable`](/api/focusable)、[`Disableable`](/api/disableable)。二者都不实现 `Clickable`；Radio 不实现 `IntoElement` 或 `Sizable`，RadioGroup 不实现 `ParentElement`、`Focusable` 或 `Clickable`。

## 键盘、焦点与禁用

- 整组最多一个 Tab stop：优先当前已选且可用项，否则首个可用项；全部禁用时整组退出 Tab 顺序。
- Up/Left 选择前一可用项，Down/Right 选择后一可用项；首尾循环并跳过 disabled。
- Home/End 请求首个/末个可用项，Space 请求当前聚焦项。
- 鼠标点击先聚焦单项，再走同一变化请求路径。
- 再次激活当前权威选中项不会取消，也不会重复调用 `on_change`。
- 组级 disabled 优先于单项 disabled。

## 主题、响应式与平台

Light、Dark 与 System 都解析 Radio 专用 normal、hover、pressed、focus-visible、selected、disabled 语义 token。尺寸、间距、边框和排版同样来自主题；渲染代码不硬编码颜色。文本在窄约束中换行，指示器保持固定逻辑尺寸。实现只依赖 GPUI 跨平台焦点、输入与 AccessKit API，目标覆盖 macOS、Windows、Linux 和 Web/WASM；未进行真实平台像素一致性承诺。

## 已知限制

- 第一版 label/description 只接受文本，不开放任意 child 或图标 slot。
- 不提供非受控值、默认值、验证消息、异步任务或 loading 状态。
- 方向只影响布局与可访问方向；为兼顾平台习惯，四个方向键始终可用。
- 桌面示例：`cargo run --example radio`。
