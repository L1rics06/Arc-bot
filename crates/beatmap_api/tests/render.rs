//! `beatmap_api::service::render` 的测试：`/map` 输出格式。

use beatmap_api::models::{BeatmapInfo, DifficultyStats, MapStatus};
use beatmap_api::render;

fn sample() -> BeatmapInfo {
    BeatmapInfo {
        map_id: 1,
        set_id: 2,
        artist: "Camellia".into(),
        title: "Ghost".into(),
        creator: "Mapper".into(),
        version: "Insane".into(),
        stars: 6.25,
        bpm: 220.0,
        status: MapStatus::Ranked,
        difficulty: DifficultyStats {
            cs: 4.0,
            ar: 9.3,
            od: 9.0,
            hp: 6.0,
        },
    }
}

#[test]
fn render_contains_key_fields() {
    let text = render(&sample());
    assert!(text.contains("Camellia - Ghost"));
    assert!(text.contains("Mapper"));
    assert!(text.contains("Insane"));
    assert!(text.contains("6.25"));
    assert!(text.contains("220"));
    assert!(text.contains("Ranked"));
    assert!(text.contains("CS/AR/OD/HP"), "应标明难度四维标签:\n{text}");
    assert!(text.contains("4/9.3/9/6"), "应包含四维数值:\n{text}");
}

#[test]
fn render_has_both_sections() {
    let text = render(&sample());
    assert!(text.contains("谱面集信息"));
    assert!(text.contains("当前难度"));
}

#[test]
fn render_survives_extreme_values() {
    let mut info = sample();
    info.stars = 0.0;
    info.bpm = 0.0;
    info.difficulty = DifficultyStats {
        cs: 0.0,
        ar: 0.0,
        od: 0.0,
        hp: 0.0,
    };
    let text = render(&info);
    assert!(text.contains("0.00"));
    assert!(!text.is_empty());
}
