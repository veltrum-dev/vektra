# Select

`Select<T>` 用于从结构化或惰性数据源中选择一个值，适合需要节省纵向空间的单选场景。`T` 需要 `Clone + Eq + Hash + 'static`。它是强类型受控组件：`selected_value(Option<T>)` 是宿主持有的权威值，`on_change(T, ...)` 只请求下一值；方向导航只改变私有 active option，不会提前改变业务选择。

如果少量互斥项需要始终可见，优先使用 [RadioGroup](/components/radio)。Select 不提供搜索、文本编辑或多选；这些能力分别属于未来的 [Combobox 路线图](https://github.com/veltrum-dev/vektra/issues/15) 与 [MultiSelect 路线图](https://github.com/veltrum-dev/vektra/issues/16)。

## 基础与受控用法

<VektraExample demo="select/basic" title="Select 基础用法" :height="300">

<<< ../../preview/src/demos/select.rs#select-example-basic{rust}

</VektraExample>

宿主可以立即接受请求，也可以在服务端确认后才更新 `selected_value`。若宿主拒绝请求，继续传入旧值即可；Select 不会维护第二份业务选择。再次提交当前权威值不会重复调用 `on_change`。

## 分组与禁用项

<VektraExample demo="select/groups" title="Select 分组与禁用项" :height="340">

<<< ../../preview/src/demos/select.rs#select-example-groups{rust}

</VektraExample>

group 标题只用于可见与可访问分组，不进入 active、选择或键盘索引。禁用项保持可见，但不能成为 active option，也不会产生变化请求。

## Loading、Empty 与 Error

<VektraExample demo="select/states" title="Select 宿主控制状态" :height="390">

<<< ../../preview/src/demos/select.rs#select-example-states{rust}

</VektraExample>

`SelectStatus` 是互斥状态。`Ready` 显示 option；`Loading`、`Empty`、`Error` 显示宿主提供的文案。Select 不发起网络请求、等待、重试或自动切换状态；状态内容也不是 option，不会触发选择。非 Ready 弹层仍可由键盘或鼠标打开并访问状态消息，但方向键、分页、typeahead 与 Enter 提交不会建立 active option 或请求值。

## 键盘导航

<VektraExample demo="select/keyboard" title="Select 键盘导航" :height="320">

<<< ../../preview/src/demos/select.rs#select-example-keyboard{rust}

</VektraExample>

| 按键 | Popup 关闭时 | Popup 打开时 |
| --- | --- | --- |
| Enter / Space | 打开并 active 可用选中项或首项 | 提交可用 active option 并关闭 |
| ArrowDown | 打开；从选中项或首个可用项开始 | 移到下一个可用项，不循环 |
| ArrowUp | 打开；从选中项或末个可用项开始 | 移到上一个可用项，不循环 |
| Home / End | 正常传播 | 移到首个 / 末个可用项 |
| PageUp / PageDown | 正常传播 | 按弹层当前实测可见页移动，并在首尾钳制 |
| 可打印字符 | Ready 时打开并从当前项后循环匹配可访问名称 | 累积短时前缀并循环匹配；重复字符循环同首字母项 |
| Escape | 正常传播 | 关闭，不改变业务值，焦点留在 Trigger |
| Tab / Shift+Tab | 正常遍历 | 关闭并继续正常焦点遍历 |

带不支持修饰键的组合和未识别按键继续传播。Typeahead 仅匹配 enabled canonical option，使用 Unicode 大小写不敏感前缀，短暂停顿后清空缓冲；无匹配时保持当前 active。Enter 与 Space 使用 GPUI 完整 KeyDown/KeyUp 激活周期，一次交互最多产生一次变化请求。

## 百万项惰性数据、窄窗口与 resize

<VektraExample demo="select/long-list" title="Select 百万项惰性数据源" :height="390">

<<< ../../preview/src/demos/select.rs#select-example-long-list{rust}

</VektraExample>

Popup 使用固定行高 `VirtualList`，只创建当前可见 option/group 行；外部百万项数据源不建立全量 catalog、HashSet 或 Element 树。Arrow、Home、End、PageUp、PageDown、typeahead 和 active reveal 都调用数据源索引，不依赖目标行已渲染。Popup 优先向下展开，下方不足时向上翻转，并受视口边距和最大高度限制；窄窗口会水平收敛。

`cargo run --example select` 在同一个 Select 示例入口中同时展示普通场景与明确标注的百万项生成式场景，并显示 visible range、item 读取次数和零行缓存上限。

## Anatomy

```text
Select Trigger（ComboBox、真实 Tab stop、expanded）
└─ 当前 label / placeholder + ChevronDown / ChevronUp 指示器
Select Popup（ListBox、私有视口约束浮层）
└─ VirtualList + Vektra Scrollbar
   ├─ SelectGroup（Group）
   │  ├─ group label（Label）
   │  └─ SelectOption（ListBoxOption）
   └─ loading / empty（Status）或 error（Alert）
```

选中项使用右侧 Check 图标；活动项使用柔和背景。Error 的 `!` 和 focus-visible 边框也提供非颜色线索。

## API

| API | 说明 |
| --- | --- |
| `Select::new(id)` | 创建带稳定根 `ElementId`、无选中值的 Select。 |
| `.selected_value(Option<T>)` | 设置宿主持有的权威业务值。 |
| `.option(SelectOption<T>)` | 添加顶层结构化 option。 |
| `.group(SelectGroup<T>)` | 添加带标题的结构化 option 组。 |
| `.items(Vec/array)` | 通过 owned adapter 添加多项，进入同一惰性内核。 |
| `.data_source(Rc<dyn SelectDataSource<T>>)` | 使用生成式、分页或远程惰性数据源。 |
| `.placeholder(text)` | 设置无有效选中项时的 Trigger 文案；默认“请选择”。 |
| `.status(SelectStatus)` | 设置 `Ready`、`Loading`、`Empty` 或 `Error`。 |
| `.on_change` / `.on_change_in` | 请求下一值；不会乐观修改选择。 |
| `.disabled(bool)` | 禁用 Trigger 并退出普通 Tab 顺序。 |
| `.size(ComponentSize)` | 设置 `Xs`、`Sm`、`Md` 或 `Lg`。 |
| `.on_focus` / `.on_blur` 及 `_in` | 观察 Trigger 的真实焦点转换。 |
| `.aria_label` / `.aria_description` | 设置 Trigger 的可访问名称与描述。 |
| `SelectOption::new(id, value, label)` | 创建稳定 ID、强类型值和可见 label。 |
| `.icon(IconSource)` | 设置可选装饰图标。 |
| `.description(text)` | 设置可见补充说明，并作为默认可访问描述。 |
| `.aria_label` / `.aria_description` | 覆盖 option 的可访问名称与描述。 |
| `.disabled(bool)` | 禁止该 option 进入 active 与提交路径。 |
| `SelectGroup::new(id, label)` | 创建稳定 ID 与可见标题的组。 |
| `.aria_label(text)` | 覆盖组的可访问名称。 |
| `.option(SelectOption<T>)` | 添加同一值类型的结构化 option。 |
| `OwnedSelectDataSource` | 将 owned option/entry 适配到统一协议。 |
| `SelectDataSource<T>` | 提供 count/revision/key/item、value/key 定位、enabled navigation、typeahead、加载状态和 range request。 |

Select 实现 [`Changeable<T>`](/api/changeable)、[`Disableable`](/api/disableable)、[`Sizable`](/api/sizable) 与 [`Focusable`](/api/focusable)。SelectOption 实现 `Disableable`。Select 不实现 `Clickable`：Trigger 的打开/关闭和 option 的变化请求是复合选择语义，不能等同于一个原始 click 回调。Select 也不实现接收任意 `Element` 的 `ParentElement`。

## 稳定身份与动态选项

- 同一个 Select 内，option ID 与业务值都应唯一；group ID 也应稳定。
- 重复 ID 或重复值采用输入顺序 first-match/canonical 规则：首个 canonical option 正常工作，后续冲突项按禁用项处理，不产生第二个选中视觉或回调。
- 选中值对应项被移除时，Trigger 显示 placeholder，不自动选择替代值，也不调用 `on_change`。
- 已选项后来被禁用时，Trigger 仍展示权威值，但该项不能再次提交或成为 active。
- active 项被移除时，优先取原位置之后的可用项，再取之前最近的可用项；重排时按稳定 ID 跟随。
- 全部 option 禁用时 Popup 仍可安全打开和关闭，active 为空。

## 焦点、关闭与无障碍

Trigger 是唯一真实焦点和普通 Tab stop；Popup 打开时焦点仍留在 Trigger，option 通过 GPUI/AccessKit 的 active-descendant 语义报告。点击可用 option 后关闭并恢复 Trigger 焦点；再次点击 Trigger、外部点击、Escape、Tab/Shift+Tab 或窗口失活都会关闭 Popup。Popup 内点击、滚轮与 Scrollbar 交互不会被当作外部点击。

Trigger 报告 `ComboBox`、名称、描述、expanded 与 disabled；未选中时只报告 placeholder，不把它重复为 value，选中后 value 使用选中项的可访问名称。Popup、Group 和 Option 分别报告 `ListBox`、`Group`、`ListBoxOption`，option 报告 selected、disabled 与全数据集 `posinset`/`setsize`。虚拟化只导出当前可见行的 AccessKit children，active option 会先滚入并物化。Loading/Empty 使用 `Status`，Error 使用 `Alert`。Popup 打开时，Trigger 通过 AccessKit `controls` 关联真实 `ListBox` 节点。

disabled、expanded、selected、名称、描述和值映射具有确定性 AccessKit 节点断言；角色、active-descendant 与焦点路径由锁定 GPUI API 的编译和交互测试覆盖。GPUI 的普通测试平台不会激活完整辅助技术树，因此真实 VoiceOver、NVDA、Narrator、Orca 以及各平台朗读体验尚未人工验证。

## 主题与跨平台状态

Light、Dark 与 System 都解析 Select 专用 Trigger、Popup、Option、group、status 和 `Xs/Sm/Md/Lg` token；Scrollbar 继续使用共享 Scrollbar token。组件不开放任意颜色、圆角或间距透传。

自定义主题现在必须由 `ResolvedTheme::from_tokens` 一次性完整验证 Select token；缺键、类型错误或无效引用返回 `ThemeError`，旧主题缺失扩展的 fallback 已移除。迁移时补齐六种 Trigger state、五种 Option state 与四种 size，并把字符串访问改为不可失败的 `select_trigger_state(SelectTriggerState)`、`select_option_state(SelectOptionState)` 和 `select_size(ThemeSize)`。

代码目标覆盖 GPUI 支持的 macOS、Windows、Linux 与 Web/WASM。当前完成了本机编译、确定性交互测试、1.25x/1.5x/2x 测试缩放约束、百万项外部数据源物化上限和共享 WASM 构建；Windows/Linux 专用性能、物理高 DPI 与屏幕阅读器仍未人工验证。

## 性能契约

- owned adapter 构造使用临时 HashSet 做预期 O(n) canonical 校验，随后释放；不复制搜索文本。
- 外部大数据源负责唯一性、key/value 定位、enabled navigation 和 typeahead 索引。
- Popup Element、布局、prepaint、paint 与 AccessKit 为 O(visible)，overdraw 0，行缓存硬上限 0。
- 正常基准：10K 完整行为；压力基准：1M 惰性数据源。目标和命令见[性能架构](/guide/performance)。

## 已知限制

- 单选且不可编辑；不提供搜索、过滤、IME、Combobox 或 MultiSelect。
- option 只接受 label、可选 `IconSource` 与 description，不接受任意 Element 或 slot。
- 状态完全由宿主驱动，Select 不拥有异步任务和重试逻辑。
- Popup 是 Select 私有实现；公共固定行高集合能力由 `VirtualList` 提供。
- owned group/description 行统一使用固定最大行高；不提供可变高度精确索引。
- 桌面示例：`cargo run --example select`。
