//! 可复用的组件能力契约。
//!
//! 这些 trait 描述多个 Vektra 组件已经共享的静态能力，用于泛型约束和一致的
//! builder 签名。它们不是运行时事件系统，也不负责跨 Entity 发布领域事件。

mod clickable;
mod disableable;

pub use clickable::Clickable;
pub use disableable::Disableable;
