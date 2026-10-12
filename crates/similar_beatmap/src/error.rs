//! 面向用户的错误类型：`Display` 即发给用户的文案。
//!
//! 路由层会把它包成 `similar/用法: 原因`，所以这里的文案要能独立读通。

use std::path::PathBuf;

use osu_difficulty_runtime::{RuntimeError, RuntimeErrorKind};

use crate::local_text;
use crate::models::Ruleset;

/// similar_beatmap 的错误。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 参数 / 用法错误（文案已含用法提示）。
    #[error("{0}")]
    Usage(String),
    /// 数据集目录没有部署，也没有可用环境变量。
    #[error("{}", local_text::error::dataset_missing(*ruleset, &dir.display().to_string()))]
    DatasetMissing { ruleset: Ruleset, dir: PathBuf },
    /// 数据集存在但打不开（文件缺失、损坏等）。
    #[error("{}", local_text::error::dataset_unavailable(*ruleset, &dir.display().to_string(), detail))]
    DatasetOpen {
        ruleset: Ruleset,
        dir: PathBuf,
        detail: String,
    },
    /// 数据集与当前算法版本不配套。
    #[error("{}", local_text::error::dataset_incompatible(*ruleset, detail))]
    DatasetIncompatible { ruleset: Ruleset, detail: String },
    /// 查询过程失败（记录损坏等）。
    #[error("{}", local_text::error::dataset_query_failed(detail))]
    DatasetQuery { detail: String },
    /// 目标谱面不在数据集中。
    #[error("{}", local_text::error::not_in_dataset(*map_id))]
    NotInDataset { map_id: u32 },
    /// 两套数据集都没有部署。
    #[error("{}", local_text::error::no_dataset(&standard_dir.display().to_string(), &mania_dir.display().to_string()))]
    NoDataset {
        standard_dir: PathBuf,
        mania_dir: PathBuf,
    },
    /// `.osu` 下载 / 读取 / 解析失败（文案来自 beatmap_api 或本模块）。
    #[error("{0}")]
    OsuFile(String),
    /// 目标谱面的模式不支持。
    #[error("{}", local_text::error::unsupported_mode(*mode))]
    UnsupportedMode { mode: u8 },
    /// mania 键数不支持。
    #[error("{}", local_text::error::unsupported_keys(*keys))]
    UnsupportedKeys { keys: i32 },
    /// 后台任务失败等内部错误。
    #[error("{}", local_text::error::internal(detail))]
    Internal { detail: String },
}

impl Error {
    /// 数据集打开失败。
    pub(crate) fn dataset_open(ruleset: Ruleset, dir: &std::path::Path, detail: String) -> Self {
        Error::DatasetOpen {
            ruleset,
            dir: dir.to_path_buf(),
            detail,
        }
    }

    /// 把 runtime 的错误按模式归属到本模块的错误。
    pub(crate) fn from_runtime(ruleset: Ruleset, error: RuntimeError) -> Self {
        let detail = error.to_string();
        match error.kind() {
            RuntimeErrorKind::Incompatible => Error::DatasetIncompatible { ruleset, detail },
            _ => Error::DatasetQuery { detail },
        }
    }
}

impl From<beatmap_api::Error> for Error {
    fn from(error: beatmap_api::Error) -> Self {
        // beatmap_api 的文案（下载失败 / 读取失败 / 解析失败）本身就是给用户看的
        Error::OsuFile(error.to_string())
    }
}

/// 本模块的 `Result`。
pub type Result<T, E = Error> = std::result::Result<T, E>;
