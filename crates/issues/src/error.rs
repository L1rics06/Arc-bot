//! issues 错误类型。
//!
//! 约定：`Display` 输出**直接面向 QQ 用户**的文案，路由层收到后原样包装回复。
//! 文案模板集中在 [`crate::local_text::error`]，本文件只做类型与映射。

/// issues 模块操作失败。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 参数不合法（缺标题、编号不是数字等）。
    #[error("{0}")]
    Invalid(String),

    /// 找不到指定编号的 issue。
    #[error("{}", crate::local_text::error::not_found(*.0))]
    NotFound(usize),

    /// 存储读写失败。
    #[error("{}", crate::local_text::error::storage(.0))]
    Storage(String),
}

/// issues 专用 Result。
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<arc_core::Error> for Error {
    fn from(e: arc_core::Error) -> Self {
        Error::Storage(e.to_string())
    }
}
