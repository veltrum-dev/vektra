//! 主题解析和转换错误。

use thiserror::Error;

/// Vektra 主题处理过程中可能返回的结构化错误。
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ThemeError {
    /// 内置主题资源不存在。
    #[error("缺少 Vektra 内置主题资源 `{path}`")]
    MissingResource {
        /// 缺失资源的相对路径。
        path: String,
    },

    /// 内置主题资源读取失败。
    #[error("读取 Vektra 内置主题资源 `{path}` 失败：{message}")]
    ResourceRead {
        /// 出错资源的相对路径。
        path: String,
        /// 具体错误信息。
        message: String,
    },

    /// 内置主题资源不是合法 UTF-8 文本。
    #[error("Vektra 内置主题资源 `{path}` 不是合法 UTF-8：{message}")]
    ResourceUtf8 {
        /// 出错资源的相对路径。
        path: String,
        /// 具体错误信息。
        message: String,
    },

    /// JSON 文本不是合法对象或无法反序列化。
    #[error("无法解析主题 JSON：{0}")]
    Json(String),

    /// Token 缺少 DTCG `$type`，且没有可继承的 group `$type`。
    #[error("Token `{path}` 缺少 $type")]
    MissingType {
        /// 出错 token 的完整路径。
        path: String,
    },

    /// Token 引用了不存在的别名。
    #[error("Token `{path}` 引用缺失的别名 `{reference}`")]
    MissingReference {
        /// 出错 token 的完整路径。
        path: String,
        /// 缺失的目标路径。
        reference: String,
    },

    /// Token 别名链中存在循环。
    #[error("Token `{path}` 存在循环引用：{cycle}")]
    CircularReference {
        /// 出错 token 的完整路径。
        path: String,
        /// 循环链路。
        cycle: String,
    },

    /// Token 的 DTCG 类型不符合当前位置需要的类型。
    #[error("Token `{path}` 类型不匹配：期望 `{expected}`，实际 `{found}`")]
    TypeMismatch {
        /// 出错 token 的完整路径。
        path: String,
        /// 期望类型。
        expected: String,
        /// 实际类型。
        found: String,
    },

    /// Token 的 `$value` 形状不在第一阶段支持范围内。
    #[error("Token `{path}` 的值无效：{message}")]
    InvalidValue {
        /// 出错 token 的完整路径。
        path: String,
        /// 具体原因。
        message: String,
    },

    /// 解析后的主题不满足 Vektra 第一阶段 Theme Profile。
    #[error("主题缺少 Vektra Profile 要求的 token `{path}`")]
    MissingProfileToken {
        /// 缺失 token 的完整路径。
        path: String,
    },
}
