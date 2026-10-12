//! 数据集层与命令层的端到端测试（用合成数据集，无网络）。
//!
//! 覆盖：数据集打开与查询、模式分派、参数校验、数据集缺失提示。

mod common;

use std::sync::Arc;

use arc_core::{AppMeta, AppState, Args, ListMode, SharedState};
use beatmap_api::OsuService;
use similar_beatmap::dataset::{Datasets, TargetSpec};
use similar_beatmap::models::{ManiaRequest, Ruleset, StandardRequest};

/// 建一个临时数据根目录，返回 (guard, AppState)。
fn state() -> (tempfile::TempDir, SharedState<OsuService>) {
    let root = tempfile::tempdir().expect("temp dir");
    let state = Arc::new(AppState::new(AppMeta {
        name: "similar-beatmap-test".to_owned(),
        admins: Vec::new(),
        data_root: root.path().to_path_buf(),
    }));
    (root, state)
}

/// 数据集类型要能跨任务共享。
#[test]
fn datasets_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<osu_difficulty_runtime::Dataset>();
    assert_send_sync::<osu_difficulty_runtime::ManiaDataset>();
    assert_send_sync::<Datasets>();
}

#[tokio::test]
async fn standard_query_returns_neighbours_and_skips_same_set() {
    let (_root, state) = state();
    common::install_standard(&state.module_dir(similar_beatmap::config::MODULE_DIR));
    let datasets = Datasets::new(state.module_dir(similar_beatmap::config::MODULE_DIR));

    assert_eq!(
        datasets.locate(10).await.expect("locate"),
        Some(Ruleset::Standard)
    );
    assert_eq!(datasets.locate(999).await.expect("locate"), None);

    let outcome = datasets
        .query_standard(TargetSpec::InIndex(10), StandardRequest::default(), 3)
        .await
        .expect("query");

    assert_eq!(outcome.target.title, "Title 10");
    assert!(outcome.same_set_excluded);
    // 21 与 20 同一个谱面集，去重后应只剩 20；30 作为次优候选出现
    let ids: Vec<u64> = outcome.rows.iter().map(|row| row.beatmap_id).collect();
    assert_eq!(ids, vec![20, 30]);
    assert!(outcome.rows[0].distance <= outcome.rows[1].distance);
}

#[tokio::test]
async fn standard_filters_and_weights_are_applied() {
    let (_root, state) = state();
    common::install_standard(&state.module_dir(similar_beatmap::config::MODULE_DIR));
    let datasets = Datasets::new(state.module_dir(similar_beatmap::config::MODULE_DIR));

    // 星数过滤掉 20/21，只留 30
    let request = StandardRequest {
        filters: osu_difficulty_runtime::QueryFilters {
            min_star: Some(7.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let outcome = datasets
        .query_standard(TargetSpec::InIndex(10), request, 5)
        .await
        .expect("query");
    let ids: Vec<u64> = outcome.rows.iter().map(|row| row.beatmap_id).collect();
    assert_eq!(ids, vec![30]);

    // 关闭参数权重后仍应返回候选（走难度维度排序）
    let request = StandardRequest {
        parameter_weight: 0.0,
        ..Default::default()
    };
    let outcome = datasets
        .query_standard(TargetSpec::InIndex(10), request, 5)
        .await
        .expect("query");
    assert!(!outcome.rows.is_empty());
}

#[tokio::test]
async fn standard_query_reports_unknown_beatmap() {
    let (_root, state) = state();
    common::install_standard(&state.module_dir(similar_beatmap::config::MODULE_DIR));
    let datasets = Datasets::new(state.module_dir(similar_beatmap::config::MODULE_DIR));

    let error = datasets
        .query_standard(TargetSpec::InIndex(999), StandardRequest::default(), 5)
        .await
        .expect_err("unknown beatmap");
    assert!(error.to_string().contains("不在数据集中"), "{error}");
}

#[tokio::test]
async fn standard_dataset_missing_is_actionable() {
    let (_root, state) = state();
    let datasets = Datasets::new(state.module_dir(similar_beatmap::config::MODULE_DIR));

    // 目录里没有数据集：定位时跳过，查询时给出可操作提示
    assert_eq!(datasets.locate(10).await.expect("locate"), None);
    let error = datasets
        .query_standard(TargetSpec::InIndex(10), StandardRequest::default(), 5)
        .await
        .expect_err("missing dataset");
    let message = error.to_string();
    assert!(message.contains("未找到 osu!standard 数据集"), "{message}");
    assert!(message.contains("SIMILAR_STANDARD_DATASET"), "{message}");
}

#[tokio::test]
async fn mania_query_isolates_key_counts_and_honours_mod_pool() {
    let (_root, state) = state();
    common::install_mania(&state.module_dir(similar_beatmap::config::MODULE_DIR));
    let datasets = Datasets::new(state.module_dir(similar_beatmap::config::MODULE_DIR));

    assert_eq!(
        datasets.locate(10).await.expect("locate"),
        Some(Ruleset::Mania)
    );
    assert_eq!(
        datasets.locate(30).await.expect("locate"),
        None,
        "standard 没有部署，mania 里也没有 30"
    );

    // 默认 NM：候选只应包含 4K 谱面（20；11 与目标同集被排除）
    let outcome = datasets
        .query_mania(TargetSpec::InIndex(10), ManiaRequest::default(), 5)
        .await
        .expect("query");
    let ids: Vec<u64> = outcome.rows.iter().map(|row| row.beatmap_id).collect();
    assert_eq!(ids, vec![20]);
    assert_eq!(
        outcome.rows[0]
            .mania
            .as_ref()
            .expect("mania info")
            .key_count,
        4
    );
    assert!(outcome.same_set_excluded);

    // 放开同谱面集后应能看到同集的 11
    let request = ManiaRequest {
        include_same_set: true,
        ..ManiaRequest::default()
    };
    let outcome = datasets
        .query_mania(TargetSpec::InIndex(10), request, 5)
        .await
        .expect("query");
    let ids: Vec<u64> = outcome.rows.iter().map(|row| row.beatmap_id).collect();
    assert!(ids.contains(&11), "{ids:?}");
    assert!(!outcome.same_set_excluded);

    // DT 候选池：候选记录来自 Mod 特征文件
    let request = ManiaRequest {
        candidate_mods: vec![osu_difficulty_runtime::ManiaGameMod::Dt],
        target_mod: osu_difficulty_runtime::ManiaGameMod::Dt,
        ..ManiaRequest::default()
    };
    let outcome = datasets
        .query_mania(TargetSpec::InIndex(10), request, 5)
        .await
        .expect("query");
    assert_eq!(
        outcome.rows[0].mania.as_ref().expect("mania info").game_mod,
        osu_difficulty_runtime::ManiaGameMod::Dt
    );
}

/// 命令级端到端：注册 → 解析参数 → 查询 → 渲染。
mod command {
    use super::*;
    use arc_core::RegistryBuilder;

    /// 用给定数据根目录跑一条 `/similar` 命令文本，返回回复或错误文案。
    async fn run(root: &std::path::Path, text: &str) -> Result<String, String> {
        let state: SharedState<OsuService> = Arc::new(AppState::new(AppMeta {
            name: "similar-beatmap-test".to_owned(),
            admins: Vec::new(),
            data_root: root.to_path_buf(),
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
    async fn similar_command_renders_results() {
        let root = tempfile::tempdir().expect("temp dir");
        let state_dir = root.path().join(similar_beatmap::config::MODULE_DIR);
        common::install_standard(&state_dir);
        common::install_mania(&state_dir);

        let reply = run(root.path(), "10 -n 2").await.expect("standard reply");
        assert!(reply.contains("相似谱面 · 目标："), "{reply}");
        assert!(reply.contains("Title 20"), "{reply}");
        assert!(reply.contains("距离"), "{reply}");
        assert!(reply.contains("https://osu.ppy.sh/b/20"), "{reply}");

        let error = run(root.path(), "10 --mods NM --star 6-8")
            .await
            .expect_err("mania-only flags on a standard map");
        assert!(error.contains("只对 osu!mania 谱面有效"), "{error}");

        let reply = run(root.path(), "30 --weights 1,2,2,0,0.25")
            .await
            .expect("standard reply");
        assert!(reply.contains("距离"), "{reply}");
    }

    #[tokio::test]
    async fn similar_command_without_dataset_is_actionable() {
        let root = tempfile::tempdir().expect("temp dir");
        let error = run(root.path(), "10").await.expect_err("no dataset");
        assert!(error.contains("没有部署相似谱面数据集"), "{error}");
        assert!(error.contains("standard"), "{error}");
        assert!(error.contains("SIMILAR_STANDARD_DATASET"), "{error}");
    }

    #[tokio::test]
    async fn similar_command_rejects_bad_arguments() {
        let root = tempfile::tempdir().expect("temp dir");
        let error = run(root.path(), "--star 6-8 10")
            .await
            .expect_err("flags first");
        assert!(error.contains("写在最前面"), "{error}");

        let error = run(root.path(), "10 -n 99").await.expect_err("bad limit");
        assert!(error.contains("返回条数"), "{error}");
    }

    #[tokio::test]
    async fn similar_command_reports_unknown_target() {
        let root = tempfile::tempdir().expect("temp dir");
        let state_dir = root.path().join(similar_beatmap::config::MODULE_DIR);
        common::install_standard(&state_dir);

        // 数据集存在但目标不在其中，且 osu! API 未初始化：应给出可操作提示而不是 panic
        let error = run(root.path(), "999999")
            .await
            .expect_err("unknown target");
        assert!(
            error.contains("osu! API 未初始化") || error.contains("不在数据集中"),
            "{error}"
        );
    }
}
