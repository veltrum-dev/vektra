# Input

`Input` 是纯 GPUI、支持 IME 的单行文本输入。编辑状态由调用方持有的 `Entity<InputState>` 保存；组件不引入 Root、Provider 或注册流程。

## 基础用法

Basic 只包含稳定 ID、状态、placeholder 与可访问名称。

<VektraExample demo="input/basic" title="Input 基础用法" :height="240">

<<< ../../preview/src/demos/input.rs#input-example-basic{rust}

</VektraExample>

## 输入类型

### Search

`InputType::Search` 提供搜索语义；搜索图标、clear 与 Enter 提交由现有组合能力显式组成。

<VektraExample demo="input/search" title="Search Input" :height="280">

<<< ../../preview/src/demos/input.rs#input-example-search{rust}

</VektraExample>

### Password

Password 默认按 grapheme 使用固定字符掩码。示例由宿主控制显隐状态，并用 Eye/EyeOff IconButton 提供随状态变化的可访问名称、Tooltip 与 selected/toggled 语义。

<VektraExample demo="input/password" title="Password 显隐" :height="260">

<<< ../../preview/src/demos/input.rs#input-example-password{rust}

</VektraExample>

隐藏态允许粘贴，但禁止复制和剪切；显示态恢复普通复制和剪切。显隐切换不会改变真实 value、选区、IME 或撤销历史，也不会发送 `Changed`。隐藏与显示都保持 `PasswordInput` 角色。

### Email、Phone 与 Url

<VektraExample demo="input/types" title="常用输入语义" :height="340">

<<< ../../preview/src/demos/input.rs#input-example-types{rust}

</VektraExample>

这些类型只提供正确语义，不会自动验证、格式化或过滤字符；业务校验仍由宿主负责。

| `InputType` | AccessKit 角色 | 额外行为 |
| --- | --- | --- |
| `Text` | `TextInput` | 默认普通单行文本。 |
| `Search` | `SearchInput` | 不自动添加图标、clear 或提交逻辑。 |
| `Password` | `PasswordInput` | 默认安全掩码；可受控显示。 |
| `Email` | `EmailInput` | 无自动邮箱校验。 |
| `Phone` | `PhoneNumberInput` | 无自动格式化或字符过滤。 |
| `Url` | `UrlInput` | 无自动 URL 校验。 |

## 组合能力

### Prefix、suffix 与 clear

三个能力保持独立：slot 子组件拥有自己的角色、焦点与事件；`InputClear` 复用 IconButton，并强制调用方提供可访问名称。

<VektraExample demo="input/affixes" title="Prefix、suffix 与 clear" :height="260">

<<< ../../preview/src/demos/input.rs#input-example-affixes{rust}

</VektraExample>

### Input Group

把相同尺寸的 Button 放入 `attached_suffix`，即可得到共用外框、全高操作区与主题分隔线。

<VektraExample demo="input/group" title="Input Group" :height="240">

<<< ../../preview/src/demos/input.rs#input-example-group{rust}

</VektraExample>

## 外观与状态

<VektraExample demo="input/variants" title="Input 外观变体" :height="360">

<<< ../../preview/src/demos/input.rs#input-example-variants{rust}

</VektraExample>

`Outline` 是默认完整边框；`Filled` 使用填充表面；`Borderless` 保留焦点与错误反馈；`Underline` 只显示底线。

<VektraExample demo="input/sizes" title="Input 语义尺寸" :height="360">

<<< ../../preview/src/demos/input.rs#input-example-sizes{rust}

</VektraExample>

`ComponentSize::{Xs, Sm, Md, Lg}` 的高度分别为 24、32、36、40 px。Input 默认填满父容器宽度，文本 viewport 可收缩并水平滚动。

<VektraExample demo="input/states" title="Input 状态" :height="320">

<<< ../../preview/src/demos/input.rs#input-example-states{rust}

</VektraExample>

`invalid`、`read_only` 与 `disabled` 均由宿主显式传入。disabled 离开普通 Tab 顺序并拒绝输入、选区和 SetValue；read-only 仍可聚焦、选择和复制普通文本，但拒绝修改。

## IME 与语义事件

<VektraExample demo="input/events" title="IME、Changed 与 Submitted" :height="300">

<<< ../../preview/src/demos/input.rs#input-example-events{rust}

</VektraExample>

`InputState::value()` 始终返回真实值。用户输入、删除、显示态 cut、paste、undo、redo、IME commit 与内置 clear 仅在值实际变化时发送一次 `Changed`；IME preedit 保持静默。非组合状态下按 Enter 发送 `Submitted`。`set_value`、`clear` 与 `reset` 是程序化操作，不发送用户语义事件。

`set_value` 表示宿主同步权威值：值实际变化时会结束组合并清空旧的 undo/redo 历史，之后撤销不会跨过这次外部同步边界。`clear` 沿用相同语义；`reset` 还会重置选区、组合、滚动与布局缓存。IME 返回的 UTF-16 选区会依据更新后的完整文本归一到 grapheme 边界。

## 键盘与无障碍

- 左右方向键按 grapheme 移动；平台单词修饰键按词移动。Home/End、Shift 选择、Backspace/Delete、Select All 与 Undo/Redo 均可用。
- macOS 使用 Option+Backspace/Delete 按词删除、Command+Backspace/Delete 删除到行首/行尾；Windows/Linux 使用各平台已有的 Control 修饰键规则。
- 只接受明确支持的修饰键组合；未识别组合继续冒泡。
- 实际 editor 节点使用对应 `InputType` 角色；prefix、suffix 与 attached suffix 保持独立无障碍子树。
- AccessKit 的可选择字符单位与编辑器一致，使用扩展 grapheme；ZWJ emoji 与组合字符不会暴露字素内部停点。
- Password 隐藏态的绘制文本、无障碍 value 与 synthetic text runs 只包含掩码，不包含明文。

## API

| API | 说明 |
| --- | --- |
| `Input::new(id, Entity<InputState>)` | 创建绑定稳定 ID 与调用方状态的 Input。 |
| `input_type(InputType)` | 设置 Text、Search、Password、Email、Phone 或 Url 语义。 |
| `password_revealed(bool)` | 受控 Password 显示状态；默认 `false`，其他类型忽略。 |
| `placeholder`, `aria_label`, `aria_description` | 文本与无障碍元数据。 |
| `variant`, `size`, `caret_color` | 视觉配置。 |
| `disabled`, `read_only`, `invalid` | 外部状态。 |
| `prefix`, `suffix`, `attached_suffix`, `clearable` | 可组合 slot 与内置清除。 |
| `on_change`, `on_submit`, `on_focus`, `on_blur` | 用户语义事件及 Entity 绑定 `_in` 版本。 |

Input 实现 [`Changeable<SharedString>`](/api/changeable)、[`Focusable`](/api/focusable)、[`Disableable`](/api/disableable) 和 [`Sizable`](/api/sizable)，不实现 `Clickable`。

## 主题、响应式与跨平台

Light、Dark 与 System 通过当前主题解析边框、表面、文字、placeholder、selection、caret 与状态颜色。Input 在可用宽度内收缩，slot 过多时编辑区会被压缩。组件使用 GPUI 跨平台文本与输入法接口；平台快捷键遵循 macOS 与 Windows/Linux 的各自约定。

自定义主题现在必须在 `ResolvedTheme::from_tokens` 构造阶段完整提供并验证所有 Input token；缺键、类型错误或无效引用会直接返回 `ThemeError`，不再从旧主题静默回退。迁移自定义主题时补齐四种 variant、七种 visual state 与四种 size，并把字符串访问改为不可失败的 `input_state(InputVariantKind, InputVisualState)` 和 `input_size(ThemeSize)`。

## 已知限制

- 仅支持单行纯文本，不提供多行、Number、Date、Time、格式化模板或内建校验消息。
- Email、Phone 与 Url 不承诺自动验证；Password 不提供自定义 mask 字符。
- slot 的业务状态、加载与错误处理由宿主负责。
- Web 预览依赖浏览器 WebGPU 与文档宿主提供的字体资源。
