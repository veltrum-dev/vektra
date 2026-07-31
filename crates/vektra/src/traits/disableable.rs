/// 支持禁用状态的组件能力。
///
/// `Disableable` 是静态 builder 契约，不提供默认 no-op。实现该能力的交互组件在
/// `disabled(true)` 时应同时阻止鼠标和键盘激活，并在 `disabled(false)` 时保持与
/// 默认可用状态一致的交互语义。
pub trait Disableable: Sized {
    /// 设置组件禁用状态。
    ///
    /// `true` 会禁止鼠标点击以及 Enter、Space 等键盘激活；`false` 恢复默认可用
    /// 状态。具体视觉反馈由组件自身和主题 token 决定。
    fn disabled(self, disabled: bool) -> Self;
}
