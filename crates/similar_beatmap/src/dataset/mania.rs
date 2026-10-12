//! osu!mania 查询编排（同步阻塞，调用方负责放进 `spawn_blocking`）。
//!
//! 键数隔离、Mod 池与距离权重由 `osu_difficulty_runtime` 负责；这里只做
//! "按视角 Mod 解析目标 → 组装查询 → 转换结果"。数据集带键型记录时
//! runtime 会自动改用键型距离，无需在这里区分。

use osu_difficulty_runtime::{
    ManiaDataset, ManiaGameMod, ManiaQueryOptions, ManiaQueryResult, ManiaQueryTarget,
};

use crate::dataset::{TargetSpec, mania_category, online_url};
use crate::error::{Error, Result};
use crate::models::{ManiaRequest, Ruleset};
use crate::result::{CandidateRow, ManiaRowInfo, QueryOutcome, TargetInfo, TargetSource};

/// 在 mania 数据集上完成一次查询。
pub fn query(
    dataset: &ManiaDataset,
    target: &TargetSpec,
    request: &ManiaRequest,
    limit: usize,
) -> Result<QueryOutcome> {
    let (query_target, source) = resolve_target(dataset, target, request.target_mod)?;
    let options = ManiaQueryOptions {
        result_limit: limit,
        include_same_set: request.include_same_set,
        candidate_mods: request.candidate_mods.clone(),
    };
    let results = dataset
        .query(&query_target, &options)
        .map_err(|error| Error::from_runtime(Ruleset::Mania, error))?;

    Ok(QueryOutcome {
        target: target_info(&query_target),
        rows: results.into_iter().map(row).collect(),
        source,
        data_cutoff_at: dataset.info().data_cutoff_at,
        same_set_excluded: !request.include_same_set,
        candidate_mods: request.candidate_mods.clone(),
    })
}

/// 解析查询目标：数据集内记录优先，否则现场分析 `.osu`。
fn resolve_target(
    dataset: &ManiaDataset,
    target: &TargetSpec,
    game_mod: ManiaGameMod,
) -> Result<(ManiaQueryTarget, TargetSource)> {
    match target {
        TargetSpec::InIndex(map_id) => {
            if !dataset.contains(u64::from(*map_id)) {
                return Err(Error::NotInDataset { map_id: *map_id });
            }
            let query_target = dataset
                .target_for_id_with_mod(u64::from(*map_id), game_mod)
                .map_err(|error| Error::from_runtime(Ruleset::Mania, error))?;
            Ok((query_target, TargetSource::InIndex))
        }
        TargetSpec::OnTheFly { map_id, bytes } => {
            let query_target = dataset
                .analyze_target_with_mod(bytes, Some(u64::from(*map_id)), game_mod)
                .map_err(|error| Error::from_runtime(Ruleset::Mania, error))?;
            Ok((query_target, TargetSource::OnTheFly))
        }
    }
}

/// 目标谱面信息。
fn target_info(target: &ManiaQueryTarget) -> TargetInfo {
    TargetInfo {
        artist: target.metadata.artist.clone(),
        title: target.metadata.title.clone(),
        version: target.metadata.version.clone(),
        url: online_url(&target.metadata.online_url),
        star_rating: None,
        mania: Some(ManiaRowInfo {
            key_count: target.metadata.key_count,
            family: target.metadata.mode_family,
            pattern: target.metadata.dominant_pattern,
            game_mod: target.game_mod,
            difficulty_percentile: target.record.difficulty_percentile,
            category: mania_category(target.pattern.as_ref()),
        }),
    }
}

/// 一条候选。
fn row(result: ManiaQueryResult) -> CandidateRow {
    let info = ManiaRowInfo {
        key_count: result.metadata.key_count,
        family: result.metadata.mode_family,
        pattern: result.metadata.dominant_pattern,
        game_mod: result.game_mod,
        difficulty_percentile: result.record.difficulty_percentile,
        category: mania_category(result.pattern.as_ref()),
    };
    CandidateRow {
        beatmap_id: result.metadata.beatmap_id,
        beatmapset_id: result.metadata.beatmapset_id,
        artist: result.metadata.artist,
        title: result.metadata.title,
        version: result.metadata.version,
        url: online_url(&result.metadata.online_url),
        star_rating: None,
        distance: result.final_distance,
        mania: Some(info),
    }
}
