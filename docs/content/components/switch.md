# Switch

`Switch` 表达会立即生效的开启/关闭设置，例如推送通知或使用分析。批量选择、选择列表与部分选中状态请使用 [`Checkbox`](./checkbox)。它是受控组件，不保存内部业务状态，也不是表单框架。

<VektraPreview demo="switch/basic" title="Switch 预览" :height="500" />

## 基础用法

<<< ../../preview/src/demos/switch.rs#switch-state{rust}

<<< ../../preview/src/demos/switch.rs#switch-basic{rust}

`.checked(...)` 是当前受控值而非初始值。一次有效激活只将 `!checked` 传给 `on_change`；宿主在回调中更新状态并调用 `cx.notify()`，下一次 render 再传回新值。回调是同步本地 callback，不是运行时事件总线；异步工作由宿主在回调中启动和管理。`.loading(...)` 同样是受控输入，不会自行启动任务或改变 checked。

## Anatomy

```text
根交互与焦点区域
├─ track
│  ├─ thumb 槽（关闭时逻辑起始侧，开启时逻辑末端侧）
│  └─ 状态内容槽（显示当前状态，位于 thumb 腾出的另一侧）
└─ 可选 trailing label
```

track、thumb 与 label 只有一个交互目标、一个 Tab stop 和一个可访问节点。label 可在窄容器中换行，track 不会压缩。

没有配置状态内容时，Switch 保持原有紧凑 track。配置任意一侧后进入内容模式，统一使用 24px track 和 20px thumb，不因内容形式或语义尺寸改变高度。轨道由一个 thumb 槽和一个共享内容槽组成；内容槽宽度取 checked 与 unchecked 两种内容类型所需宽度的较大值，因此切换时轨道不会跳动。状态内容与外侧轨道边缘之间保留主题化间距；纯图标只占图标宽度，不会被强行撑成文字宽度。checked 内容显示在逻辑起始侧，unchecked 内容显示在逻辑末端侧。

## API

| API | 说明 |
| --- | --- |
| `Switch::new(id)` | 创建稳定 `ElementId` 的 Switch，默认关闭且可用。 |
| `.checked(bool)` | 设置当前受控值。 |
| `SwitchContent::text(text)` | 创建纯文字状态内容。 |
| `SwitchContent::icon(icon)` | 通过 `IntoIconSource` 创建纯装饰图标内容。 |
| `SwitchContent::icon_text(icon, text)` | 创建图标在前、文字在后的状态内容。 |
| `.checked_content(content)` | 设置开启状态的轨道内容，重复调用时最后一次生效。 |
| `.unchecked_content(content)` | 设置关闭状态的轨道内容，重复调用时最后一次生效。 |
| `.loading(bool)` | 在 thumb 内显示 spinner，阻止激活但保留焦点与 Tab 停靠。 |
| `.disabled(bool)` | 禁用鼠标、Enter、Space 和正常 Tab 停靠。 |
| `.transition_duration(Duration)` | 设置下一次 checked 切换的时长；默认 180ms，`Duration::ZERO` 直接切换。 |
| `.label(text)` | 设置 trailing 可见 label，也是默认可访问名称。 |
| `.size(ComponentSize)` | 显式 `Xs`、`Sm`、`Md` 或 `Lg` 尺寸。 |
| `.cursor_style(CursorStyle)` | 设置空闲可用光标；loading 使用 Arrow，disabled 始终优先。 |
| `.aria_label(text)` | 覆盖或提供可访问名称。 |
| `.aria_description(text)` | 提供补充无障碍描述。 |
| `.on_change(handler)` | 接收下一 bool 值、`Window` 和 `App`，不携带 `ClickEvent`。 |
| `.on_change_in(cx, handler)` | 绑定宿主 Entity 的状态变化回调。 |
| `.on_click(handler)` | 标准原始激活入口，适合先启动后台请求。 |
| `.on_click_in(cx, handler)` | 将标准激活入口绑定到宿主 Entity。 |
| `.on_focus` / `.on_blur` | 注册真实焦点转换回调。 |
| `.on_focus_in` / `.on_blur_in` | 注册 Entity 绑定的焦点回调。 |

`Switch` 实现 [`Changeable<bool>`](/api/changeable)、[`Clickable`](/api/clickable)、[`Disableable`](/api/disableable)、[`Focusable`](/api/focusable) 和 [`Sizable`](/api/sizable)。`on_click` 与 `on_change` 共用一个激活 handler 槽，连续配置时后调用者生效，不会在一次激活中重复调用两套回调。

## 键盘、焦点与无障碍

可用且非 loading 的 Switch 进入正常 Tab 顺序，Space 在 keyup 时切换，Enter 不激活；带 Ctrl、Alt、Shift 或 Meta 的 Space 不切换。鼠标点击 track、thumb 或 label 都只调用一次回调。loading 会消费鼠标、Enter 和 Space，避免重复提交或冒泡到父元素，但仍可通过 Tab 聚焦并保留 focus-visible。disabled 同样不激活，并退出正常 Tab 顺序；`disabled + loading` 使用 disabled 的颜色、光标和焦点规则，同时继续显示 spinner。

根节点使用 `Role::Switch`，并将关闭映射为 `Toggled::False`、开启映射为 `Toggled::True`，从不产生 mixed。`.aria_label(...)` 覆盖可见 label；没有可见 label 时必须提供它。disabled 状态使用 disabled 视觉与不可操作光标。

轨道状态内容只补充视觉状态：图标是装饰性的，不创建新的可访问节点或 Tab stop；“开启/关闭”也不会替代业务名称。`.label("通知")` 或 `.aria_label("通知")` 仍负责提供 Switch 的可访问名称。

<<< ../../preview/src/demos/switch.rs#switch-focus{rust}

checked 状态和焦点生命周期彼此独立：重绘、builder 值变化和焦点转换都不会自行触发 `on_change`；checked 改变也不会伪造 focus/blur。`_in` 表示 `Context::listener` 的 Entity 绑定，Entity 销毁后会安全 no-op。

## Loading 与受控任务

<<< ../../preview/src/demos/switch.rs#switch-motion-loading{rust}

loading spinner 固定在 thumb 内，不改变 thumb 或 track 尺寸；thumb 仍位于当前 checked 对应的位置，轨道内容继续表达该状态。spinner 使用独立的稳定动画 ID 和固定循环周期，因此 `.transition_duration(...)` 不会改变或重启它。reduced-motion 下显示静态帧，不持续请求动画帧。

宿主可以在请求开始时先乐观更新 checked，也可以保持 checked，等请求成功后再更新；失败提示、回滚、取消与任务生命周期都由宿主负责。`.loading(false)` 后恢复正常鼠标与 Space 激活。

需要以后端结果为准时，使用 `on_click_in` 读取宿主当前值并启动请求，不要立即修改 checked；请求期间由宿主传入 `loading(true)`，成功后再写入服务器确认的 checked，失败时保持原值并显示业务错误。若直接使用 `on_change_in`，回调会同时收到建议的下一布尔值。两种入口是替代关系，后调用者生效。

## 主题、尺寸与限制

四种语义尺寸保留各自的紧凑 track、命中区域、图标、内容宽度、spinner、间距和排版 token；进入内容模式后统一为 24px track 与 20px thumb。紧凑模式尺寸不变。Light、Dark 与 System 模式通过 Vektra 主题解析；normal、hover、pressed、focus-visible 和 disabled 均使用主题 token。loading 不显示误导性的 hover/pressed 反馈。旧主题未提供新增内容或 loading token 时使用语义 fallback；一旦开始提供其中一组新增 token，就必须完整覆盖该组的两种视觉状态或四种尺寸。

受控 checked 值变化时，thumb 与内容默认使用 180ms、固定 ease-out cubic；旧内容在前半程淡出，新内容在后半程淡入，避免与移动中的 thumb 明显重叠。`.transition_duration(...)` 接受调用方传入的非零时长且不静默夹取，建议 100–400ms；`Duration::ZERO` 不创建状态切换动画。初次 render 不播放入场动画，只改变 duration 不会增加 motion generation 或重启动画，同一次 render 同时改变 checked 与 duration 时使用新时长。GPUI reduced-motion 的优先级更高，会让 thumb、内容与 spinner 直接显示静态终态。

- 没有非受控状态或 `default_checked`。
- 状态文字保持单行并按主题上限截断，只适合“开启/关闭”等短文本。
- 没有拖拽、`indeterminate`、自定义 easing/复杂动画配置、任意 `AnyElement` slot 或表单校验。
- 如果两个选项需要始终同时可见并可分别点击，应使用 Segmented Control，而不是扩展 Switch。
- label 固定在 track 后方。
- 桌面示例可运行：`cargo run --example switch`。
