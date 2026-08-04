use gpui::{App, Context, Window};

/// 支持真实焦点生命周期回调的组件能力。
///
/// `Focusable` 是静态 consuming-builder 能力，不是运行时事件总线。回调只由 GPUI
/// 观察到的真实焦点转换触发；重新渲染、其他 builder 状态变化或 Checkbox 的 checked
/// 变化本身都不会触发焦点回调。各组件仍各自决定 disabled、Tab 顺序和具体焦点行为。
pub trait Focusable: Sized {
    /// 注册组件从未聚焦变为聚焦时调用的回调。
    ///
    /// 一次真实焦点转换只调用一次；单纯重新渲染不会重复调用。
    fn on_focus(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self;

    /// 注册组件从聚焦变为未聚焦时调用的回调。
    ///
    /// 焦点移向其他组件，或锁定 GPUI 因已聚焦元素变为 disabled 而清除焦点时，按
    /// GPUI 实际产生的焦点转换调用。组件从渲染树移除时，其 keyed state 与订阅会先
    /// 销毁，因此已移除组件不会再收到 blur 回调。
    fn on_blur(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self;

    /// 注册可访问宿主 Entity 状态的聚焦回调。
    ///
    /// `_in` 表示通过 [`Context::listener`] 绑定宿主 Entity；它不表示“焦点进入子树”，
    /// 也不等同于 DOM `focusin`。Entity 已销毁时保留 GPUI listener 的弱引用/no-op
    /// 生命周期语义。
    fn on_focus_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        let listener = cx.listener(move |this, _: &(), window, cx| handler(this, window, cx));
        self.on_focus(move |window, cx| listener(&(), window, cx))
    }

    /// 注册可访问宿主 Entity 状态的失焦回调。
    ///
    /// `_in` 只表示宿主 Entity 绑定，不表示 GPUI 的 `focus_out` 子树语义。Entity
    /// 已销毁后回调会按 [`Context::listener`] 的约定安全地变为 no-op。
    fn on_blur_in<T: 'static>(
        self,
        cx: &Context<T>,
        handler: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
    ) -> Self {
        let listener = cx.listener(move |this, _: &(), window, cx| handler(this, window, cx));
        self.on_blur(move |window, cx| listener(&(), window, cx))
    }
}
