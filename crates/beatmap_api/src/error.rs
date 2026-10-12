//! beatmap_api 错误类型。
//!
//! 约定：`Display` 输出**直接面向 QQ 用户**的文案，路由层收到后原样包装回复。
//! 文案模板集中在 [`crate::local_text`]，本文件只做类型与映射。

/// beatmap_api 操作失败。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 凭据缺失或格式不对。
    #[error("{0}")]
    Config(String),

    /// osu! API 请求失败（网络 / 认证 / 限流等）。
    #[error("{0}")]
    Api(String),

    /// 谱面存在但缺少谱面集信息（被删 / graveyard 谱面会出现）。
    #[error("{}", crate::local_text::hint::missing_mapset(*.0))]
    MissingMapset(u32),

    /// 输入不合法。
    #[error("{0}")]
    Invalid(String),

    /// 缓存读写失败。
    #[error("{}", crate::local_text::cache::failed(.0))]
    Cache(String),

    /// `.osu` 文件下载或解析失败。
    #[error("{}", crate::local_text::osu_file::download_failed(.0))]
    OsuFile(String),
}

/// beatmap_api 专用 Result。
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<arc_core::Error> for Error {
    fn from(e: arc_core::Error) -> Self {
        Error::Cache(e.to_string())
    }
}
