//! 真实数据集冒烟测试：数据集存在才跑，缺文件时自动跳过。
//!
//! 本地准备：把 `osu-difficulty-lab` Release 里的数据集解压到
//! `data/similar_beatmap/{standard,mania}`，或用环境变量
//! `SIMILAR_STANDARD_DATASET` / `SIMILAR_MANIA_DATASET` 指定目录。
//! 详见 `docs/similar-beatmap.md`。

use std::path::PathBuf;

use osu_difficulty_runtime::{
    Dataset, DifficultyWeights, ManiaDataset, ManiaQueryOptions, ManiaQueryTarget, QueryFilters,
    QueryOptions, WeightingMode,
};
use rusqlite::{Connection, OpenFlags};

/// 数据集目录：环境变量优先，其次仓库内默认路径。
fn standard_dir() -> PathBuf {
    dataset_dir("SIMILAR_STANDARD_DATASET", "standard")
}

/// mania 数据集目录。
fn mania_dir() -> PathBuf {
    dataset_dir("SIMILAR_MANIA_DATASET", "mania")
}

/// 解析数据集目录。
fn dataset_dir(env: &str, name: &str) -> PathBuf {
    match std::env::var(env) {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value.trim()),
        _ => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/similar_beatmap")
            .join(name),
    }
}

/// 从 SQLite 里取一个谱面 ID（用于冒烟查询）。
fn first_beatmap_id(path: &PathBuf, table: &str, column: &str) -> Option<u64> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()?;
    connection
        .query_row(
            &format!("SELECT {column} FROM {table} LIMIT 1"),
            [],
            |row| row.get::<_, i64>(0),
        )
        .ok()
        .map(|id| id as u64)
}

#[test]
fn standard_dataset_smoke() {
    let dir = standard_dir();
    let metadata = dir.join("metadata.sqlite");
    if !metadata.is_file() {
        eprintln!("跳过：未找到 standard 数据集（{}）", dir.display());
        return;
    }

    let dataset = Dataset::open(&dir).expect("open standard dataset");
    let info = dataset.info();
    assert!(
        info.record_count > 1_000,
        "记录数异常：{}",
        info.record_count
    );

    let map_id = first_beatmap_id(&metadata, "beatmaps", "beatmap_id").expect("beatmap id");
    let target = dataset.target_for_id(map_id).expect("target");
    let results = dataset
        .query(
            &target,
            &QueryOptions {
                weighting: WeightingMode::Manual {
                    difficulty_weights: DifficultyWeights::default(),
                    parameter_weight: 1.0,
                },
                filters: QueryFilters::default(),
                result_limit: 5,
            },
        )
        .expect("query");

    assert!(!results.is_empty(), "应至少返回一条候选");
    for result in &results {
        assert_ne!(result.record.beatmap_id, map_id, "不应把自己作为候选");
        assert!(result.final_distance.is_finite());
    }
}

#[test]
fn mania_dataset_smoke() {
    let dir = mania_dir();
    let metadata = dir.join("mania-metadata.sqlite");
    if !metadata.is_file() {
        eprintln!("跳过：未找到 mania 数据集（{}）", dir.display());
        return;
    }

    let dataset = ManiaDataset::open(&dir).expect("open mania dataset");
    assert!(dataset.info().record_count > 1_000, "记录数异常");

    let map_id = first_beatmap_id(&metadata, "mania_beatmaps", "beatmap_id").expect("beatmap id");
    let target: ManiaQueryTarget = dataset.target_for_id(map_id).expect("target");
    let query = ManiaQueryOptions {
        result_limit: 5,
        include_same_set: false,
        candidate_mods: vec![osu_difficulty_runtime::ManiaGameMod::Nm],
    };
    let results = dataset.query(&target, &query).expect("query");

    assert!(!results.is_empty(), "应至少返回一条候选");
    for result in &results {
        assert_eq!(
            result.record.key_count, target.record.key_count,
            "mania 候选必须与目标同键数"
        );
    }
}
