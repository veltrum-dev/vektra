# GPUI 依赖类型

以下类型由 GPUI 定义，不是 Vektra 类型。Vektra 只在公共回调边界使用它们，不复制维护完整定义。当前链接固定到 commit `82aef44308540b576e4e51fb379efa71614e5c91`。

## `ClickEvent`

仅出现在 `Clickable::on_click` 等原始激活入口中，用于区分鼠标与键盘激活。`Changeable::on_change` 不携带它，因为变化也可能来自方向键、Home 或 End。[查看锁定源码](https://github.com/zed-industries/zed/blob/82aef44308540b576e4e51fb379efa71614e5c91/crates/gpui/src/interactive.rs#L281)。

## `Window`

当前窗口的焦点、输入、绘制和平台状态入口。Vektra 回调传入 `&mut Window`，让宿主可调用当前 revision 支持的窗口 API。[查看锁定源码](https://github.com/zed-industries/zed/blob/82aef44308540b576e4e51fb379efa71614e5c91/crates/gpui/src/window.rs#L1073)。

## `App`

GPUI 应用级上下文。普通 builder callback 接收 `&mut App`，适合不需要某个 Entity 状态的同步逻辑。[查看锁定源码](https://github.com/zed-industries/zed/blob/82aef44308540b576e4e51fb379efa71614e5c91/crates/gpui/src/app.rs#L679)。

## `Context<T>`

Entity `T` 的更新上下文。`*_in` 便利方法通过 `Context::listener` 把标准 GPUI 回调绑定到宿主 Entity；Entity 销毁后保持弱引用/no-op 语义。[查看锁定源码](https://github.com/zed-industries/zed/blob/82aef44308540b576e4e51fb379efa71614e5c91/crates/gpui/src/app/context.rs#L20)。

## `Entity<T>` 与 `SharedString`

Input 要求调用方持有 `Entity<InputState>`，使编辑状态跨 render 稳定存在。`SharedString` 是 Input 的公开 value/event 字符串类型；`InputState::value()` 仍以 `&str` 形式读取内容。

Input 内部实现锁定 revision 的 `EntityInputHandler`，并通过 `ElementInputHandler` 接入平台文本输入、IME、UTF-16 selection 和范围坐标。这些是实现细节，不要求宿主注册全局处理器。

可编译的 Entity 绑定示例来自实际 WASM preview：

<<< ../../preview/src/demos/checkbox.rs#checkbox-focus{rust}

将来 GPUI 提供与此 revision 对应的稳定官方 API 文档后，本站可改链接目标；在此之前源码是签名事实来源。
