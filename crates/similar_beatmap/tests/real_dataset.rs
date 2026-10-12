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

/// 从 SQLite 里取一个可用于冒烟的谱面 ID。
///
/// 数据集里混有离线生成的合成 ID（超出 u32 范围，SQLite 里可能以负数保存），
/// 命令层的 ID 是 u32，因此这里只取在线范围内、且带在线链接的一张。
fn first_beatmap_id(path: &PathBuf, table: &str, column: &str) -> Option<u64> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()?;
    connection
        .query_row(
            &format!(
                "SELECT {column} FROM {table} \
                 WHERE {column} > 0 AND {column} <= 4294967295 AND online_url != '' LIMIT 1"
            ),
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

/// 命令级冒烟：真实数据目录下跑 `/similar`，验证从分发到渲染的完整回复。
mod command {
    use std::sync::Arc;

    use arc_core::{AppMeta, AppState, Args, ListMode, RegistryBuilder, SharedState};
    use beatmap_api::OsuService;

    use super::{first_beatmap_id, mania_dir, standard_dir};

    /// 仓库里的数据根目录（`data/`，模块目录是其下的 `similar_beatmap/`）。
    fn data_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
    }

    /// 直接调用命令处理器，返回回复或错误文案。
    async fn run(text: &str) -> Result<String, String> {
        let state: SharedState<OsuService> = Arc::new(AppState::new(AppMeta {
            name: "similar-beatmap-smoke".to_owned(),
            admins: Vec::new(),
            data_root: data_root(),
        }));
        let mut builder = RegistryBuilder::new();
        similar_beatmap::register(&mut builder, state.clone());
        let command = builder
            .into_commands()
            .into_iter()
            .find(|command| command.name == "similar")
            .expect("similar command registered");
        let args = Args::parse_with_command(text, ListMode::Include, state.clone());
        match (command.handler)(state, args).await {
            Ok(reply) => Ok(reply.to_string()),
            Err(error) => Err(error.to_string()),
        }
    }

    #[tokio::test]
    async fn standard_command_smoke() {
        let dir = standard_dir();
        let metadata = dir.join("metadata.sqlite");
        let Some(map_id) = first_beatmap_id(&metadata, "beatmaps", "beatmap_id") else {
            eprintln!("跳过：未找到 standard 数据集（{}）", dir.display());
            return;
        };

        let reply = run(&format!("{map_id} -n 3"))
            .await
            .expect("standard reply");
        println!("=== /similar {map_id} -n 3 ===\n{reply}\n");
        assert!(reply.contains("相似谱面 · 目标："), "{reply}");
        assert!(reply.contains("距离"), "{reply}");
        assert!(reply.contains("https://osu.ppy.sh/b/"), "{reply}");
        assert!(reply.contains("数据截至"), "{reply}");
        assert!(
            !super::has_synthetic_link(&reply),
            "合成 ID 不应生成在线链接：{reply}"
        );
    }

    #[tokio::test]
    async fn mania_command_smoke() {
        let dir = mania_dir();
        let metadata = dir.join("mania-metadata.sqlite");
        let Some(map_id) = first_beatmap_id(&metadata, "mania_beatmaps", "beatmap_id") else {
            eprintln!("跳过：未找到 mania 数据集（{}）", dir.display());
            return;
        };

        let reply = run(&format!("{map_id} --mods NM,DT -n 3"))
            .await
            .expect("mania reply");
        println!("=== /similar {map_id} --mods NM,DT -n 3 ===\n{reply}\n");
        assert!(reply.contains("相似谱面 · 目标："), "{reply}");
        assert!(reply.contains('K'), "mania 应显示键数：{reply}");
        assert!(reply.contains("距离"), "{reply}");
    }
}

/// 回复里是否出现了超出 u32 范围的假链接（合成 ID）。
fn has_synthetic_link(reply: &str) -> bool {
    reply.split("osu.ppy.sh/b/").skip(1).any(|rest| {
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits
            .parse::<u64>()
            .is_ok_and(|id| id > u64::from(u32::MAX))
    })
}
