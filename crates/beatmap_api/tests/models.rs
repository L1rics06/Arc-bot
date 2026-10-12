//! `beatmap_api::models` 的测试：ID 解析、难度距离、状态映射与序列化兼容。

use beatmap_api::models::{DifficultyStats, MapStatus, parse_beatmap_id};
use rosu_v2::model::beatmap::RankStatus;

#[test]
fn parse_id_accepts_positive() {
    assert_eq!(parse_beatmap_id(" 123 ").unwrap(), 123);
}

#[test]
fn parse_id_rejects_invalid() {
    assert!(parse_beatmap_id("").is_err());
    assert!(parse_beatmap_id("abc").is_err());
    assert!(parse_beatmap_id("0").is_err());
    assert!(parse_beatmap_id("-5").is_err());
}

#[test]
fn parse_id_accepts_osu_links() {
    assert_eq!(
        parse_beatmap_id("https://osu.ppy.sh/b/12345").unwrap(),
        12345
    );
    assert_eq!(
        parse_beatmap_id("https://osu.ppy.sh/beatmaps/12345").unwrap(),
        12345
    );
    assert_eq!(
        parse_beatmap_id("https://osu.ppy.sh/beatmapsets/6789#osu/12345").unwrap(),
        12345
    );
    assert_eq!(
        parse_beatmap_id("https://osu.ppy.sh/beatmapsets/6789#mania/54321").unwrap(),
        54321
    );
    assert_eq!(
        parse_beatmap_id("http://www.osu.ppy.sh/b/7").unwrap(),
        7,
        "www 主机与 http 协议也应接受"
    );
    assert_eq!(
        parse_beatmap_id("osu.ppy.sh/b/12345").unwrap(),
        12345,
        "省略协议头也应接受"
    );
    assert_eq!(
        parse_beatmap_id("https://osu.ppy.sh/b/12345?m=0").unwrap(),
        12345,
        "查询参数应被忽略"
    );
}

#[test]
fn parse_id_rejects_links_without_beatmap_id() {
    assert!(
        parse_beatmap_id("https://osu.ppy.sh/beatmapsets/6789").is_err(),
        "只有谱面集 ID 时应报错"
    );
    assert!(parse_beatmap_id("https://example.com/b/12345").is_err());
    assert!(parse_beatmap_id("https://osu.ppy.sh/").is_err());
}

#[test]
fn stats_distance_is_l1_over_four_dims() {
    let a = DifficultyStats {
        cs: 4.0,
        ar: 9.0,
        od: 9.0,
        hp: 6.0,
    };
    let b = DifficultyStats {
        cs: 4.5,
        ar: 9.5,
        od: 9.0,
        hp: 6.0,
    };
    assert!((a.distance_to(&b) - 1.0).abs() < f32::EPSILON);
    assert_eq!(a.distance_to(&a), 0.0);
}

#[test]
fn status_roundtrip_through_json() {
    let json = serde_json::to_string(&MapStatus::Graveyard).unwrap();
    assert_eq!(json, "\"graveyard\"");
    let back: MapStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back, MapStatus::Graveyard);
}

#[test]
fn rosu_status_maps_to_domain_status() {
    assert_eq!(MapStatus::from_rosu(RankStatus::Ranked), MapStatus::Ranked);
    assert_eq!(
        MapStatus::from_rosu(RankStatus::Approved),
        MapStatus::Approved
    );
    assert_eq!(
        MapStatus::from_rosu(RankStatus::Qualified),
        MapStatus::Qualified
    );
    assert_eq!(MapStatus::from_rosu(RankStatus::Loved), MapStatus::Loved);
    assert_eq!(
        MapStatus::from_rosu(RankStatus::Pending),
        MapStatus::Pending
    );
    assert_eq!(MapStatus::from_rosu(RankStatus::WIP), MapStatus::Wip);
    assert_eq!(
        MapStatus::from_rosu(RankStatus::Graveyard),
        MapStatus::Graveyard
    );
}

#[test]
fn status_display_is_human_readable() {
    assert_eq!(MapStatus::Ranked.to_string(), "Ranked");
    assert_eq!(MapStatus::Wip.to_string(), "Work in progress");
}
