# 组件 API 与事件能力规范

## 公共 API

- 将必需信息放入构造函数。示例：稳定交互组件应让 `ElementId` 成为构造必填项，类似当前 `Button::new(id)` 与 `IconButton::new(id, icon)`。
- 将可选配置设计为 consuming builder，返回 `Self`，保持 GPUI/Rust fluent 风格。
- 使用语义 enum 表达 `variant`、`size`、`tone`、`state` 等稳定产品语义；避免大量布尔参数。
- Tooltip、Popover、Menu、Dialog 等具有明确打开/关闭语义的浮层，表示当前受控状态时统一使用
  `open(bool)`，不要为同一语义分别引入 `show`、`visible`、`is_open` 等 builder。只设置初始状态时
  使用 `default_open(bool)`，不得用 `open` 伪装成受控 API。普通可见性以及 `expanded`、
  `selected`、`active` 等不同产品语义不机械改名为 `open`。
- 避免任意样式透传，除非能证明不会破坏主题、焦点、禁用、无障碍和组件结构。
- 优先组合、slot 和显式状态输入。slot 图标、label、leading/trailing 内容等必须有明确尺寸、无障碍和主题语义。
- 稳定交互组件应支持 `ElementId`，便于 GPUI 交互状态、焦点和测试定位。
- Vektra 自有公开 API 必须有有意义的中文 rustdoc，包括公开模块、类型、trait、函数、方法、字段和 enum 变体。
- API 命名遵循 Rust 与 GPUI 风格，不机械复制 JavaScript、DOM 或 React props 形态。

## 实现形态

- `RenderOnce` 适合无状态展示组件、由调用者完全控制状态的组件，以及当前 `Button`、`IconButton` 这类轻量交互组件。
- `Render + Entity` 适合组件拥有内部状态、订阅、异步任务、焦点句柄、生命周期或需要 `Context<T>` 的场景。
- 自定义 `Element` 只在普通 GPUI element 组合无法满足绘制、性能、布局、命中区域或特殊交互时使用；使用前读取 `gpui` 的 Element reference。

不要为了统一外观而抽取宽泛基类、宏或复杂泛型。共享能力必须来自实际重复和稳定语义。

## 小型能力 trait

采用小型能力 trait，不创建包含大量可选空方法的统一 `Events` trait。

建议能力包括：

- `Clickable`
- `Disableable`
- `Toggleable`
- `Focusable`，仅在组件语义真正一致时

其他能力必须有至少两个组件共享相同语义和签名后再提取。trait 的核心能力方法必须是必需实现，禁止默认 no-op；可以提供具有真实代理行为的默认便利方法。

不要使用字符串事件名称，不复刻 DOM `EventTarget`、运行时事件表、capture/bubble 架构或 JavaScript 类继承，不为了“未来可能使用”提前建立复杂事件层。

### Vektra 标准能力映射

实现或审查公开组件时，逐项检查 `crates/vektra/src/traits/` 中当前存在的标准能力：

- `Clickable`：组件支持标准原始激活入口，并提供语义一致的 `on_click` 与 `cursor_style` 时实现。Button、IconButton 直接使用该入口；Switch 等受控组件可将它作为后台请求、权限检查或通用包装器的前置入口。
- `Focusable`：组件公开真实 `on_focus`、`on_blur` 生命周期 builder 时实现；builder 状态变化不得伪造焦点事件。
- `Disableable`：组件公开 `disabled(bool)`，并同时阻止自身鼠标与语义键盘激活时实现。
- `Sizable`：组件公开 `size(ComponentSize)`，并通过组件主题 token 解析共享语义尺寸时实现。

适用能力必须同时提供：

1. 用户可直接调用的 inherent forwarding builder；
2. 委托到同一行为实现的标准 trait impl；
3. 使用泛型 trait bound 的编译/行为测试；
4. 直接 builder 与 trait 调用的契约一致性测试。

受控组件可以同时保留语义回调与标准原始激活入口。例如 Switch 的
`on_change(next_checked, ClickEvent, ...)` 便于立即采用下一值，`Clickable::on_click` 则便于
宿主先请求后台接口，成功后再更新 checked。两者必须适配到同一内部激活路径，并明确采用
“后调用者生效”等单一优先级；不得在一次激活中无条件同时调用两者。组件仍不应自行等待请求
或提前修改受控状态。不要新增公开 `Loadable`、`Toggleable` 等 trait，除非至少两个组件已经
共享完全一致且稳定的公开契约。

## Clickable 目标 API

`Clickable` 统一组件激活能力。目标调用形式为：

```rust
Button::new("save")
    .on_click(|event, window, cx| {
        // 不依赖宿主 Entity 状态
    })
```

需要访问宿主 Entity 时，提供类似：

```rust
Button::new("save")
    .on_click_in(cx, |this, event, window, cx| {
        this.save(window, cx);
    })
```

目标 trait 形态可以参考：

```rust
pub trait Clickable: Sized {
    fn on_click(
        self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self;

    fn on_click_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(
            &mut T,
            &ClickEvent,
            &mut Window,
            &mut Context<T>,
        ) + 'static,
    ) -> Self {
        self.on_click(cx.listener(handler))
    }
}
```

这是规范目标，不是已编译事实。未来真正实现前，必须针对仓库锁定的 GPUI revision 做最小编译验证。

设计意图：

- `on_click` 接受标准 GPUI callback：`Fn(&ClickEvent, &mut Window, &mut App)`。
- `on_click_in` 封装 `cx.listener` 的 Entity 弱引用绑定。
- 高级调用者仍可直接传入标准 GPUI callback。
- Rust 不支持按参数列表重载同名方法，因此不要强行让两种形式都叫 `on_click`。

## 悬停规则

- 纯视觉 hover 优先使用 GPUI 的 `hover` 样式。
- 不要提前创建通用 `on_hover`。
- 业务确实需要观察悬停变化时，再考虑明确的 `on_hover_change(bool, ...)` 或 enter/leave API。
- 不允许只能通过 hover 才能访问的功能。

## 事件层级

- 局部组件激活、输入或选择：builder callback。
- 可被其他 Entity 订阅的领域事件：`EventEmitter<E>`。
- 快捷键、命令面板或可重映射操作：`Action`。
- 底层鼠标、键盘、滚动、命中区域或布局交互：GPUI `InteractiveElement` 和必要的自定义 `Element`。
- 视觉反馈：GPUI 状态样式。

当前锁定 GPUI 已提供 `Context::listener`、`Context::emit`、`EventEmitter<E>`、`Render`、`RenderOnce` 和 `InteractiveElement::on_click` 等能力；实现前仍应读取本地源码或用最小代码编译确认签名。
