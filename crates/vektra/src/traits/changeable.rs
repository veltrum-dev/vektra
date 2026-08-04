use gpui::{App, Context, Window};

/// 支持受控值变化请求的组件能力。
///
/// `Changeable<T>` 表达用户交互产生的“下一受控值”，组件不会因此自行提交或持久化
/// 业务状态。变化可能来自鼠标、Space、方向键、Home 或 End，因此回调不携带
/// `ClickEvent`。
pub trait Changeable<T>: Sized {
    /// 注册下一受控值的变化回调。
    ///
    /// 宿主可以立即采用该值，也可以先完成异步审批，再通过组件的受控值 builder
    /// 传回最终权威状态。
    fn on_change(self, handler: impl Fn(T, &mut Window, &mut App) + 'static) -> Self;

    /// 注册可访问宿主 Entity 状态的变化回调。
    ///
    /// `_in` 表示通过 [`Context::listener`] 绑定宿主 Entity。Entity 已销毁后保留
    /// GPUI listener 的弱引用/no-op 生命周期语义。
    fn on_change_in<U: 'static>(
        self,
        cx: &Context<U>,
        handler: impl Fn(&mut U, T, &mut Window, &mut Context<U>) + 'static,
    ) -> Self;
}
