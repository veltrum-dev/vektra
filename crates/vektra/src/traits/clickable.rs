use gpui::{App, ClickEvent, Context, CursorStyle, Window};

/// 支持标准激活回调的组件能力。
///
/// `Clickable` 统一 Button、IconButton、Switch 等组件的原始激活入口。它是静态
/// builder 能力，不是运行时事件总线；组件仍然通过自身渲染逻辑决定何时触发回调，
/// 受控组件也不会因此自行改变业务状态。
pub trait Clickable: Sized {
    /// 注册标准 GPUI 点击回调。
    ///
    /// 回调参数依次为 `ClickEvent`、当前 `Window` 和应用级 `App`。组件可以将鼠标
    /// 点击、Enter 或 Space 等语义激活统一转发到该回调。
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self;

    /// 设置可用状态下的鼠标光标。
    ///
    /// disabled、loading 或 progress 等组件自身状态仍应优先表达自己的交互反馈。
    fn cursor_style(self, cursor_style: CursorStyle) -> Self;

    /// 注册可访问宿主 Entity 状态的点击回调。
    ///
    /// 该方法内部使用 `Context::listener` 将 `handler` 转换为标准 GPUI 回调。handler
    /// 参数依次为宿主 Entity 的 `&mut T`、`ClickEvent`、当前 `Window` 和
    /// `Context<T>`，因此可以直接修改宿主状态并调用 `cx.notify()`。Entity 已销毁时
    /// 保持 GPUI listener 原有的弱引用/no-op 语义。
    fn on_click_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &ClickEvent, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        self.on_click(cx.listener(handler))
    }
}
