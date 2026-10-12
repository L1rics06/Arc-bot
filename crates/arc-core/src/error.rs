//! arc-core 统一错误类型。
//!
//! 分层约定：
//! - 各业务模块（issues / beatmap_api）用 `thiserror` 定义自己的错误枚举，
//!   [`Display`](std::fmt::Display) 输出面向 QQ 用户的文案；
//! - 跨模块传播 / 框架层兜底使用本模块的 [`Error`]，通过 `#[from] anyhow::Error` 汇聚；
//! - 命令 handler 返回 `Result<Box<dyn Reply>, E>`，其中 `E: std::error::Error + Send + Sync + 'static`。

/// arc-bot 统一错误类型。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 配置缺失或格式错误（如环境变量、目录路径）。
    #[error("配置错误：{0}")]
    Config(String),

    /// 数据读写失败。
    #[error("数据读写失败：{0}")]
    Storage(String),

    /// 其它未分类错误，保留完整错误链。
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// arc-bot 统一 Result 别名。
pub type Result<T, E = Error> = std::result::Result<T, E>;
