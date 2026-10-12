//! osu!standard 查询编排（同步阻塞，调用方负责放进 `spawn_blocking`）。
//!
//! 权重、过滤器与排序语义全部交给 `osu_difficulty_runtime`，这里只负责
//! "解析目标 → 组装 QueryOptions → 转换结果"。始终使用固定权重（Manual），
//! 与 OPP 应用层一致（动态权重已在上游停用）。

use osu_difficulty_runtime::{Dataset, QueryOptions, QueryResult, QueryTarget, WeightingMode};

use crate::dataset::{TargetSpec, online_url};
use crate::error::{Error, Result};
use crate::models::{Ruleset, StandardRequest};
use crate::result::{CandidateRow, QueryOutcome, TargetInfo, TargetSource};

/// 在 standard 数据集上完成一次查询。
pub fn query(
    dataset: &Dataset,
    target: &TargetSpec,
    request: &StandardRequest,
    limit: usize,
) -> Result<QueryOutcome> {
    let (query_target, source) = resolve_target(dataset, target)?;
    let options = QueryOptions {
        weighting: WeightingMode::Manual {
            difficulty_weights: request.weights,
            parameter_weight: request.parameter_weight,
        },
        filters: request.filters.clone(),
        result_limit: limit,
    };
    let results = dataset
        .query(&query_target, &options)
        .map_err(|error| Error::from_runtime(Ruleset::Standard, error))?;

    Ok(QueryOutcome {
        target: target_info(&query_target),
        rows: results.into_iter().map(row).collect(),
        source,
        data_cutoff_at: dataset.info().data_cutoff_at,
        // runtime 固定排除与目标同一谱面集
        same_set_excluded: true,
        candidate_mods: Vec::new(),
    })
}

/// 解析查询目标：数据集内记录优先，否则现场分析 `.osu`。
fn resolve_target(dataset: &Dataset, target: &TargetSpec) -> Result<(QueryTarget, TargetSource)> {
    match target {
        TargetSpec::InIndex(map_id) => {
            if !dataset.contains(u64::from(*map_id)) {
                return Err(Error::NotInDataset { map_id: *map_id });
            }
            let query_target = dataset
                .target_for_id(u64::from(*map_id))
                .map_err(|error| Error::from_runtime(Ruleset::Standard, error))?;
            Ok((query_target, TargetSource::InIndex))
        }
        TargetSpec::OnTheFly { bytes, .. } => {
            let query_target = dataset
                .analyze_target(bytes)
                .map_err(|error| Error::from_runtime(Ruleset::Standard, error))?;
            Ok((query_target, TargetSource::OnTheFly))
        }
    }
}

/// 目标谱面信息。
fn target_info(target: &QueryTarget) -> TargetInfo {
    TargetInfo {
        artist: target.metadata.artist.clone(),
        title: target.metadata.title.clone(),
        version: target.metadata.version.clone(),
        url: online_url(target.metadata.beatmap_id, &target.metadata.online_url),
        star_rating: target.metadata.star_rating,
        mania: None,
    }
}

/// 一条候选。
fn row(result: QueryResult) -> CandidateRow {
    CandidateRow {
        beatmap_id: result.metadata.beatmap_id,
        beatmapset_id: result.metadata.beatmapset_id,
        artist: result.metadata.artist,
        title: result.metadata.title,
        version: result.metadata.version,
        url: online_url(result.metadata.beatmap_id, &result.metadata.online_url),
        star_rating: result.metadata.star_rating,
        distance: result.final_distance,
        mania: None,
    }
}
