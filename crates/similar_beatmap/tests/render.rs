//! 文本渲染测试（纯函数）。

use osu_difficulty_runtime::{ManiaGameMod, ManiaModeFamily, ManiaPattern};
use similar_beatmap::render;
use similar_beatmap::result::{CandidateRow, ManiaRowInfo, QueryOutcome, TargetInfo, TargetSource};

/// standard 的一次查询结果。
fn standard_outcome(rows: Vec<CandidateRow>, source: TargetSource) -> QueryOutcome {
    QueryOutcome {
        target: TargetInfo {
            artist: "Fixture Artist".to_owned(),
            title: "Title 10".to_owned(),
            version: "Diff 10".to_owned(),
            url: Some("https://osu.ppy.sh/b/10".to_owned()),
            star_rating: Some(6.0),
            mania: None,
        },
        rows,
        source,
        data_cutoff_at: Some(1_700_000_000),
        same_set_excluded: true,
        candidate_mods: Vec::new(),
    }
}

/// 一条 standard 候选。
fn standard_row(id: u64, stars: f32, distance: f32) -> CandidateRow {
    CandidateRow {
        beatmap_id: id,
        beatmapset_id: id / 10,
        artist: "Fixture Artist".to_owned(),
        title: format!("Title {id}"),
        version: format!("Diff {id}"),
        url: Some(format!("https://osu.ppy.sh/b/{id}")),
        star_rating: Some(stars),
        distance,
        mania: None,
    }
}

#[test]
fn renders_standard_results() {
    let text = render(&standard_outcome(
        vec![standard_row(20, 6.1, 0.012), standard_row(30, 8.0, 0.5)],
        TargetSource::InIndex,
    ));
    let lines: Vec<&str> = text.lines().collect();

    assert!(lines[0].contains("相似谱面 · 目标："), "{}", lines[0]);
    assert!(lines[0].contains("Fixture Artist - Title 10 [Diff 10]"));
    assert!(lines[0].contains("★6.00"));
    assert!(lines[0].contains("https://osu.ppy.sh/b/10"));

    assert!(lines[1].starts_with("1. "), "{}", lines[1]);
    assert!(lines[1].contains("★6.10"));
    assert!(lines[1].contains("距离 0.012"));
    assert!(lines[2].starts_with("2. "));

    let footer = lines.last().expect("footer");
    assert!(footer.contains("共 2 条"), "{footer}");
    assert!(footer.contains("已排除同一谱面集"), "{footer}");
    assert!(footer.contains("数据截至 2023-11-14"), "{footer}");
    assert!(!footer.contains("现场分析"), "数据集内目标不应提示现场分析");
}

#[test]
fn renders_empty_result_with_hint() {
    let text = render(&standard_outcome(Vec::new(), TargetSource::InIndex));
    assert!(text.contains("没有找到符合条件的候选"), "{text}");
    assert!(text.contains("共 0 条"), "{text}");
}

#[test]
fn renders_on_the_fly_note() {
    let text = render(&standard_outcome(
        vec![standard_row(20, 6.1, 0.012)],
        TargetSource::OnTheFly,
    ));
    assert!(text.contains("现场分析"), "{text}");
}

#[test]
fn renders_mania_results() {
    let mania_info = |game_mod: ManiaGameMod, percentile: f32| ManiaRowInfo {
        key_count: 4,
        family: ManiaModeFamily::Rc,
        pattern: ManiaPattern::Stream,
        game_mod,
        difficulty_percentile: percentile,
        category: Some("SV".to_owned()),
    };
    let outcome = QueryOutcome {
        target: TargetInfo {
            artist: "Fixture Artist".to_owned(),
            title: "Mania Title 10".to_owned(),
            version: "Mania Diff 10".to_owned(),
            url: None,
            star_rating: None,
            mania: Some(mania_info(ManiaGameMod::Nm, 0.5)),
        },
        rows: vec![CandidateRow {
            beatmap_id: 20,
            beatmapset_id: 2,
            artist: "Fixture Artist".to_owned(),
            title: "Mania Title 20".to_owned(),
            version: "Mania Diff 20".to_owned(),
            url: Some("https://osu.ppy.sh/b/20".to_owned()),
            star_rating: None,
            distance: 0.031,
            mania: Some(mania_info(ManiaGameMod::Dt, 0.87)),
        }],
        source: TargetSource::InIndex,
        data_cutoff_at: None,
        same_set_excluded: false,
        candidate_mods: vec![ManiaGameMod::Nm, ManiaGameMod::Dt],
    };

    let text = render(&outcome);
    assert!(text.contains("4K RC/Stream [SV]"), "{text}");
    assert!(text.contains("分位 87%"), "{text}");
    assert!(text.contains("距离 0.031"), "{text}");
    assert!(text.contains("DT"), "候选使用的 Mod 应显示：{text}");
    assert!(text.contains("含同一谱面集"), "{text}");
    assert!(text.contains("候选 NM/DT"), "{text}");
    assert!(!text.contains("数据截至"), "没有 cutoff 时不显示：{text}");
}

#[test]
fn renders_missing_star_rating_gracefully() {
    let mut row = standard_row(20, 6.1, 0.012);
    row.star_rating = None;
    row.url = None;
    let mut outcome = standard_outcome(vec![row], TargetSource::InIndex);
    outcome.target.star_rating = None;

    let text = render(&outcome);
    assert!(!text.contains('★'), "{text}");
    assert!(text.contains("距离 0.012"), "{text}");
}
