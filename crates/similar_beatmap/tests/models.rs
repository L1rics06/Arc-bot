//! `similar_beatmap::models` 的参数解析测试（纯函数，无数据集）。

use std::sync::Arc;

use arc_core::{AppMeta, AppState, Args, ListMode};
use osu_difficulty_runtime::{DifficultyWeights, ManiaGameMod};
use similar_beatmap::models::Request;

/// 用一段命令文本解析参数（等价于路由层剥掉命令词后的形态）。
fn parse(text: &str) -> Result<Request, String> {
    let state = Arc::new(AppState::<beatmap_api::OsuService>::new(AppMeta {
        name: "test".to_owned(),
        admins: Vec::new(),
        data_root: std::env::temp_dir(),
    }));
    let args = Args::parse_with_command(text, ListMode::Include, state);
    Request::parse(&args).map_err(|error| error.to_string())
}

#[test]
fn parses_plain_id_and_defaults() {
    let request = parse("12345").expect("parse");
    assert_eq!(request.map_id, 12345);
    assert_eq!(request.limit, 5);
    assert_eq!(request.standard.parameter_weight, 1.0);
    assert_eq!(
        request.standard.weights.as_array(),
        DifficultyWeights::default().as_array()
    );
    assert!(!request.standard.explicit);
    assert_eq!(request.mania.candidate_mods, vec![ManiaGameMod::Nm]);
    assert_eq!(request.mania.target_mod, ManiaGameMod::Nm);
    assert!(!request.mania.include_same_set);
}

#[test]
fn parses_links_and_flags_after_id() {
    let request = parse("https://osu.ppy.sh/beatmapsets/1#osu/999 -n 12").expect("parse");
    assert_eq!(request.map_id, 999);
    assert_eq!(request.limit, 12);
}

#[test]
fn rejects_missing_or_misordered_id() {
    let missing = parse("").expect_err("missing id");
    assert!(missing.contains("缺少谱面 ID"), "{missing}");

    let misordered = parse("--star 6-8 123").expect_err("flags first");
    assert!(misordered.contains("写在最前面"), "{misordered}");
}

#[test]
fn rejects_bad_limits() {
    assert!(parse("1 -n 0").is_err());
    assert!(parse("1 -n 21").is_err());
    assert!(parse("1 -n abc").is_err());
    assert!(parse("1 -n").is_err(), "缺取值应报错");
    assert_eq!(parse("1 --count 20").expect("count").limit, 20);
}

#[test]
fn parses_ranges_in_all_shapes() {
    let request = parse("1 --star 6-8 --bpm 150+ --len 0-240").expect("parse");
    assert_eq!(request.standard.filters.min_star, Some(6.0));
    assert_eq!(request.standard.filters.max_star, Some(8.0));
    assert_eq!(request.standard.filters.min_bpm, Some(150.0));
    assert_eq!(request.standard.filters.max_bpm, None);
    assert_eq!(request.standard.filters.min_length_seconds, Some(0.0));
    assert_eq!(request.standard.filters.max_length_seconds, Some(240.0));

    let single = parse("1 --od 8").expect("single value is a lower bound");
    assert_eq!(single.standard.filters.min_od, Some(8.0));
    assert_eq!(single.standard.filters.max_od, None);

    let open_ended = parse("1 --star 6-").expect("min only keeps upper bound open");
    assert_eq!(open_ended.standard.filters.min_star, Some(6.0));
    assert_eq!(open_ended.standard.filters.max_star, None);
}

#[test]
fn rejects_bad_ranges() {
    let reversed = parse("1 --star 8-6").expect_err("reversed");
    assert!(reversed.contains("下限比上限大"), "{reversed}");

    let out_of_range = parse("1 --ar 12-13").expect_err("out of range");
    assert!(out_of_range.contains("取值范围"), "{out_of_range}");

    assert!(parse("1 --cs a-b").is_err());
    assert!(parse("1 --star").is_err(), "缺取值应报错");
}

#[test]
fn parses_weights_and_parameter_weight() {
    let request = parse("1 --weights 2,0,1,0.5,0 --param 0").expect("parse");
    assert_eq!(
        request.standard.weights.as_array(),
        [2.0, 0.0, 1.0, 0.5, 0.0]
    );
    assert_eq!(request.standard.parameter_weight, 0.0);
    assert!(request.standard.explicit);
}

#[test]
fn rejects_bad_weights() {
    assert!(parse("1 --weights 1,2,3").is_err(), "数量不足");
    assert!(parse("1 --weights 3,0,0,0,0").is_err(), "超出 0-2");
    assert!(
        parse("1 --weights 0,0,0,0,0 --param 0").is_err(),
        "不能全为 0"
    );
    assert!(parse("1 --param 9").is_err());
    assert!(parse("1 --weights").is_err(), "缺取值应报错");
}

#[test]
fn parses_mania_mod_pool() {
    let request = parse("1 --mods NM,DT,HT --target-mod DT --same-set").expect("parse");
    assert_eq!(
        request.mania.candidate_mods,
        vec![ManiaGameMod::Nm, ManiaGameMod::Dt, ManiaGameMod::Ht]
    );
    assert_eq!(request.mania.target_mod, ManiaGameMod::Dt);
    assert!(request.mania.include_same_set);
    assert!(request.mania.explicit);
}

#[test]
fn mania_mod_aliases_and_defaults() {
    let aliases = parse("1 --mods nc,dc").expect("parse");
    assert_eq!(
        aliases.mania.candidate_mods,
        vec![ManiaGameMod::Dt, ManiaGameMod::Ht]
    );
    assert_eq!(
        aliases.mania.target_mod,
        ManiaGameMod::Dt,
        "池里没有 NM 时以池中第一项为比较视角"
    );
}

#[test]
fn rejects_bad_mania_mods() {
    assert!(parse("1 --mods NM,XX").is_err());
    let duplicate = parse("1 --mods DT,NC").expect_err("duplicate mod");
    assert!(duplicate.contains("重复"), "{duplicate}");

    let outside = parse("1 --mods NM --target-mod DT").expect_err("target outside pool");
    assert!(outside.contains("不在 --mods 候选池"), "{outside}");
}
