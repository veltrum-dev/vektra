use crate::ComponentSize;

/// 支持 Vektra 共享语义尺寸的组件能力。
///
/// `Sizable` 是静态 builder 契约。组件仍负责把 [`ComponentSize`] 映射到自己的主题
/// token，而不是共享一套固定像素值。
pub trait Sizable: Sized {
    /// 设置组件级显式尺寸。
    ///
    /// 显式尺寸优先于全局默认尺寸；未调用时组件会在渲染阶段读取当前全局默认值。
    fn size(self, size: ComponentSize) -> Self;
}
