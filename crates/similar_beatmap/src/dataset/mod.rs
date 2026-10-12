//! 数据集适配层：惰性打开、阻塞隔离与模式分派。
//!
//! `osu_difficulty_runtime` 的 API 全部是同步阻塞的（打开时要反序列化整个 HNSW 索引、
//! 校验 mania 全量记录；查询时要扫描记录），因此这里统一用
//! [`tokio::task::spawn_blocking`] 包起来，绝不占用 async 运行时线程。
//!
//! 数据集只在首次用到时打开（[`OnceCell`]），机器人启动不受影响；并发首次调用由
//! `OnceCell` 去重。目录里没有对应数据集的标记文件时直接返回"未部署"错误。

mod mania;
mod standard;

use std::path::PathBuf;
use std::sync::Arc;

use osu_difficulty_runtime::{Dataset, ManiaDataset};
use tokio::sync::OnceCell;

use crate::config;
use crate::error::{Error, Result};
use crate::local_text;
use crate::models::{ManiaRequest, Ruleset, StandardRequest};
use crate::result::QueryOutcome;

/// 查询目标：数据集里已有记录，或需要现场分析的 `.osu` 文件。
#[derive(Debug, Clone)]
pub enum TargetSpec {
    /// 数据集内的谱面 ID。
    InIndex(u32),
    /// 现场分析：带上请求的 beatmapID 与 `.osu` 内容。
    ///
    /// 带上请求 ID 是为了避免老旧谱面文件里过时/为 0 的内嵌 ID 影响身份
    /// （与 runtime 的 `analyze_target_with_mod` 约定一致）。
    OnTheFly { map_id: u32, bytes: Vec<u8> },
}

impl TargetSpec {
    /// 现场分析目标。
    pub fn on_the_fly(map_id: u32, bytes: Vec<u8>) -> Self {
        TargetSpec::OnTheFly { map_id, bytes }
    }
}

/// 两套数据集的惰性持有者。
pub struct Datasets {
    module_dir: PathBuf,
    standard: OnceCell<Arc<Dataset>>,
    mania: OnceCell<Arc<ManiaDataset>>,
}

impl Datasets {
    /// 绑定数据模块目录（`data/similar_beatmap`）。
    pub fn new(module_dir: PathBuf) -> Self {
        Self {
            module_dir,
            standard: OnceCell::new(),
            mania: OnceCell::new(),
        }
    }

    /// 判断目标谱面落在哪个数据集里。
    ///
    /// 返回 `None` 表示两个数据集里都没有，需要现场下载 `.osu` 分析。
    /// 未部署的数据集会被跳过（不报错）。
    pub async fn locate(&self, map_id: u32) -> Result<Option<Ruleset>> {
        for ruleset in [Ruleset::Standard, Ruleset::Mania] {
            let dir = config::dir_for(&self.module_dir, ruleset);
            if !config::is_deployed(&dir, ruleset) {
                continue;
            }
            let hit = match ruleset {
                Ruleset::Standard => self.standard().await?.contains(u64::from(map_id)),
                Ruleset::Mania => self.mania().await?.contains(u64::from(map_id)),
            };
            if hit {
                return Ok(Some(ruleset));
            }
        }
        Ok(None)
    }

    /// 两套数据集是否都没有部署（用于给出"先放数据集"的可操作提示）。
    pub fn none_deployed(&self) -> bool {
        ![Ruleset::Standard, Ruleset::Mania].iter().any(|ruleset| {
            config::is_deployed(&config::dir_for(&self.module_dir, *ruleset), *ruleset)
        })
    }

    /// standard 数据集目录。
    pub fn standard_dir(&self) -> PathBuf {
        config::standard_dir(&self.module_dir)
    }

    /// mania 数据集目录。
    pub fn mania_dir(&self) -> PathBuf {
        config::mania_dir(&self.module_dir)
    }

    /// standard 数据集（首次调用时打开）。
    pub async fn standard(&self) -> Result<Arc<Dataset>> {
        let dir = config::standard_dir(&self.module_dir);
        if !config::is_deployed(&dir, Ruleset::Standard) {
            return Err(Error::DatasetMissing {
                ruleset: Ruleset::Standard,
                dir,
            });
        }
        self.standard
            .get_or_try_init(|| async { open_standard(dir).await })
            .await
            .cloned()
    }

    /// mania 数据集（首次调用时打开）。
    pub async fn mania(&self) -> Result<Arc<ManiaDataset>> {
        let dir = config::mania_dir(&self.module_dir);
        if !config::is_deployed(&dir, Ruleset::Mania) {
            return Err(Error::DatasetMissing {
                ruleset: Ruleset::Mania,
                dir,
            });
        }
        self.mania
            .get_or_try_init(|| async { open_mania(dir).await })
            .await
            .cloned()
    }

    /// 在 standard 数据集上查询。
    pub async fn query_standard(
        &self,
        target: TargetSpec,
        request: StandardRequest,
        limit: usize,
    ) -> Result<QueryOutcome> {
        let dataset = self.standard().await?;
        run_blocking(move || standard::query(&dataset, &target, &request, limit)).await
    }

    /// 在 mania 数据集上查询。
    pub async fn query_mania(
        &self,
        target: TargetSpec,
        request: ManiaRequest,
        limit: usize,
    ) -> Result<QueryOutcome> {
        let dataset = self.mania().await?;
        run_blocking(move || mania::query(&dataset, &target, &request, limit)).await
    }
}

/// 打开 standard 数据集（阻塞）。
async fn open_standard(dir: PathBuf) -> Result<Arc<Dataset>> {
    let path = dir.clone();
    let dataset = run_blocking(move || {
        Dataset::open(&path)
            .map_err(|error| Error::dataset_open(Ruleset::Standard, &path, error.to_string()))
    })
    .await?;
    log::info!(
        "{}",
        local_text::log::dataset_opened(
            Ruleset::Standard.name(),
            dataset.info().record_count,
            &dir.display().to_string()
        )
    );
    Ok(Arc::new(dataset))
}

/// 打开 mania 数据集（阻塞）。
async fn open_mania(dir: PathBuf) -> Result<Arc<ManiaDataset>> {
    let path = dir.clone();
    let dataset = run_blocking(move || {
        ManiaDataset::open(&path)
            .map_err(|error| Error::dataset_open(Ruleset::Mania, &path, error.to_string()))
    })
    .await?;
    log::info!(
        "{}",
        local_text::log::dataset_opened(
            Ruleset::Mania.name(),
            dataset.info().record_count,
            &dir.display().to_string()
        )
    );
    Ok(Arc::new(dataset))
}

/// 把阻塞任务放进线程池并转成 `Result`。
async fn run_blocking<T, F>(job: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(job)
        .await
        .map_err(|error| Error::Internal {
            detail: error.to_string(),
        })?
}

/// 数据集里的在线链接（空字符串视为缺失）。
pub(super) fn online_url(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_owned())
    }
}

/// mania 键型记录的分类标记；空标记返回 `None`。
pub(super) fn mania_category(
    record: Option<&osu_difficulty_runtime::ManiaMmaRecord>,
) -> Option<String> {
    let category = osu_difficulty_runtime::mania_display_category(record?).trim();
    if category.is_empty() {
        None
    } else {
        Some(category.to_owned())
    }
}
