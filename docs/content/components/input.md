# Input

`Input` 是纯 GPUI、支持 IME 的单行文本输入。编辑状态由调用方持有的 `Entity<InputState>` 保存；组件不引入 Root、Provider 或注册流程，也不保存 required、regex、错误消息、dirty、touched 等表单元数据。

<VektraPreview demo="input/basic" title="Input 预览" :height="760" />

## 状态所有权

<<< ../../preview/src/demos/input.rs#input-state{rust}

创建状态后，每次 render 都把同一个 Entity 传给稳定 ID 的 Input：

<<< ../../preview/src/demos/input.rs#input-basic{rust}

顶部基础示例直接使用 `attached_suffix(Button)`：Input 与按钮共用外框，按钮点击和 editor Enter 都产生同一个搜索结果。普通内嵌 `suffix` 的两种 Search 形式仍在下方对照展示。

`InputState::value()` 读取当前文本。`set_value`、`clear` 与 `reset` 是程序化操作，不发送 `InputEvent::Changed`，也不调用 `on_change`；它们会安全结束 IME 并校正选区，其中 `reset` 还会清空撤销/重做历史和水平滚动。用户输入、删除、cut、paste、undo、redo、IME commit 和内置 clear 仅在值实际改变时发送一次 `Changed`。

`InputState` 实现 `EventEmitter<InputEvent>`；`on_change`、`on_submit`、`on_focus`、`on_blur` 及 `_in` 版本都从同一语义路径触发。IME preedit 更新不发送 `Changed`，组合期间 Enter 不提交，commit 后发送一次变化；非组合状态 Enter 发送 `Submitted(value)`。

## 外观与状态

<<< ../../preview/src/demos/input.rs#input-variants{rust}

- `Outline` 是默认完整边框，适合普通表单。
- `Filled` 用填充背景建立分组感，并保留透明结构边框避免状态跳动。
- `Borderless` 适合工具栏和紧凑表面；normal/hover 没有明显边框，鼠标或键盘实际聚焦后使用强调色结构边框，键盘 focus-visible 会进一步使用更宽的焦点边框。invalid 图标始终保留。
- `Underline` 适合低密度表单，以更明显的焦点/错误底线反馈状态；外壳圆角固定为 0，底线两端保持笔直。

状态优先级是 disabled、invalid + focus-visible、invalid、focus-visible、hover、normal。`invalid(bool)` 只消费外围校验结果：Input 不知道错误原因，也不运行规则。未来的 `FormControl<T>` 或 label/help/error 布局可以持有 required、regex、异步校验和 touched 等状态，不需要把它们塞进 `InputState`。

<<< ../../preview/src/demos/input.rs#input-sizes{rust}

`ComponentSize::{Xs, Sm, Md, Lg}` 的高度分别为 24、32、36、40 px。Input 默认填满父容器宽度，文本 viewport 使用可收缩布局并水平滚动；最终宽度始终由父容器控制。

聚焦、可编辑且选区为空时，插入光标每 500ms 在完整可见与完整隐藏之间离散切换；它只在相位切换时请求重绘，不注册连续动画帧。输入、删除、移动、鼠标定位、clear、undo/redo、IME 更新/提交和重新聚焦都会从完整可见阶段重启。IME preedit 与 reduced-motion 下光标保持常亮；失焦、有选区、disabled 或 read-only 时不绘制且不保留闪烁任务。光标高度来自 shaped text 的 `ascent + descent` 并在 line-height 内居中，默认宽度为 1px。`caret_color(Hsla)` 可覆盖当前实例在 normal/invalid 等可编辑状态下的主题 caret 颜色，不影响其他 token。

## Prefix、suffix、attached suffix 与 clear

布局顺序固定为 `prefix | editor | status | clear | suffix | divider | attached suffix`；没有 invalid 状态图标时可省略 status。clear 属于编辑器的内建操作并保持在普通 suffix 内侧。`suffix(...)` 仍表示位于 Input 水平 padding 内的紧凑尾部内容；`attached_suffix(...)` 表示与 Input 共用唯一外壳、贴合右边缘的分段操作，其左侧使用主题边框色分隔并占满 Input 高度。编辑区域会优先收缩，attached suffix 保持自身宽度。

三种槽位都接受任意 `IntoElement`，保持自己的 ID、role、焦点、Tab 顺序、Tooltip、aria 与事件；文本的鼠标处理只覆盖 editor viewport，因此激活槽位不会移动 caret 或产生 Input 的 `Changed`/`Submitted`。同时存在 clear、交互式 suffix 与 attached suffix 时，普通 Tab 顺序是 editor、clear、suffix、attached suffix。外壳只在使用 attached suffix 时裁切到自身边框内；Button 的 focus-visible 边框位于按钮内部，仍清晰可见。

`disabled(true)` 与 `read_only(true)` 只约束文本编辑器和内置 clear。Input 无法可靠修改任意传入子组件的状态；若整个组合都需要禁用或只读，调用方应把同一状态传给交互式 prefix、suffix 与 attached suffix。普通槽位宜保持紧凑；attached suffix 应使用与 Input 相同的 `ComponentSize`，以保持全高分段区域对齐。

`InputClear::new(aria_label)` 强制提供纯图标按钮的可访问名称。它内部复用 `IconButtonVariant::Ghost` 及其 `Tooltip`/placement 能力，使用 24×24 的 `ComponentSize::Xs` 透明命中区；静止时只有图标，hover、pressed、focus-visible、Enter 与 Space 仍完全由 IconButton 负责。clear 在值非空且可编辑时持续显示；激活只发送一次 `Changed("")`，随后焦点回到编辑器。Tooltip 是视觉帮助，不会自动复制成 aria label，也不能替代可访问名称。

### Search 组合

Search 不需要新的专用组件。以下五个可编译示例复用 `InputClear`，并把尾部操作点击和 editor Enter 连接到同一个业务搜索结果：

<<< ../../preview/src/demos/input.rs#input-search-actions{rust}

- 纯图标：继续放在普通 `suffix` 中，使用 `IconButton + IconName::Search`；必须提供 `aria_label`，建议同时显示 Tooltip。
- 纯文字：放在 `attached_suffix` 中，使用带可见“搜索”label 的 `Button`，形成单一外框、全高操作区和竖向分隔线。
- 图标加文字（内嵌）：继续放在普通 `suffix` 中，使用 `Button::start_icon(IconName::Search)` 与可见 label。
- 图标加文字（拼接）：把同一个 Button 组合放入 `attached_suffix`，证明 attached suffix 不限制内容形态。
- 纯图标（拼接）：把带 `aria_label` 与 Tooltip 的 `IconButton` 放入 `attached_suffix`，保持全高分段区域和完整命中区。

两个内嵌示例使用 `Ghost + ComponentSize::Xs` 保持紧凑；三个拼接示例让 Input 与 Button/IconButton 同时使用 `ComponentSize::Md`。五者都保留 Button/IconButton 完整的 hover、pressed、focus-visible、disabled、Enter 与 Space 行为。`IconName::Search` 随 `vektra` 的 `icons` feature 提供；宿主也可以通过自己的 `IntoIconSource` 类型替换图标来源。

<<< ../../preview/src/demos/input.rs#input-states{rust}

## 编辑、键盘与无障碍

Input 支持 grapheme 安全的左右移动、Shift 扩选、Home/End、平台单词移动与删除、单击定位、Shift + 单击、拖选、双击选词、三击及以上全选、Select All、Copy/Cut/Paste、Undo/Redo 和长文本水平滚动。Select All、Copy/Cut/Paste、Undo/Redo 使用 macOS 的 Cmd 或 Windows/Linux 的 Ctrl；Windows 键与 Linux Super 不会被当作通用命令键。方向与删除只处理明确支持的修饰键组合，其他组合继续向宿主或系统冒泡；macOS 额外支持 Fn + Left/Right 跳到行首/行尾。双击/三击只改变选区，不发送 `Changed`；中文、emoji、ZWJ 与 combining mark 不会产生非法 UTF-8 边界。粘贴中的 CR/LF 会转换为空格，不会 trim。Tab 使用 GPUI 正常焦点导航；Escape 不被 Input 吞掉。

disabled editor 离开普通 Tab 顺序，拒绝输入、选区和 AccessKit SetValue。read-only editor 仍可聚焦、选择和复制，但拒绝修改与 SetValue。

只有实际 editor 节点使用 `Role::TextInput`；交互式槽位仍是独立无障碍子树。节点提供 value、placeholder、description、文本 runs、UTF-16 selection、invalid、read-only 和 SetValue。业务侧必须调用 `aria_label(...)` 或通过外围语义提供明确可访问名称；placeholder 只是提示，不会自动成为 accessible name。

## API 摘要

| API | 说明 |
| --- | --- |
| `Input::new(id, state)` | 绑定稳定 `ElementId` 与调用方持有的 `Entity<InputState>`。 |
| `placeholder`, `aria_label`, `aria_description` | 文本提示与无障碍语义；三者互不替代。 |
| `variant`, `size`, `disabled`, `read_only`, `invalid` | 外观、共享尺寸和外部驱动状态。 |
| `caret_color(Hsla)` | 覆盖当前实例的插入光标颜色；未设置时使用主题 caret token。 |
| `prefix`, `suffix` | 在 Input 水平 padding 内插入独立的任意元素。 |
| `attached_suffix` | 添加贴合右边缘、全高且带主题分隔线的分段尾部元素。 |
| `clearable(InputClear)` | 添加基于 IconButton + Tooltip 的语义清除操作。 |
| `on_change`, `on_submit`, `on_focus`, `on_blur` | 用户语义事件及 Entity 绑定 `_in` 版本。 |

Input 实现 [`Changeable<SharedString>`](/api/changeable)、[`Focusable`](/api/focusable)、[`Disableable`](/api/disableable) 和 [`Sizable`](/api/sizable)，不实现 `Clickable`。主题 token 位于 `input.border-width`、`input.focus-width`、`input.caret-width`（默认 1px）、`input.variant.<variant>.<state>.*` 与 `input.size.<size>.*`。旧自定义主题完全没有 Input 扩展时会回退到 semantic/foundation token；一旦开始提供 Input state 或 size 扩展，就必须完整提供对应配置。
