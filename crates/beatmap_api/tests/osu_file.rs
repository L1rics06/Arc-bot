//! `beatmap_api::osu_file` 的测试：[Difficulty] 段的解析规则。

use beatmap_api::osu_file::parse_difficulty;

const SAMPLE: &str = "\
osu file format v14

[General]
AudioFilename: audio.mp3

[Difficulty]
HPDrainRate:6.5
CircleSize:4
OverallDifficulty:9.3
ApproachRate:9.5
SliderMultiplier:1.8

[TimingPoints]
0,375,4,2,1,60,1,0
";

#[test]
fn parses_difficulty_section() {
    let stats = parse_difficulty(SAMPLE).unwrap();
    assert_eq!(stats.hp, 6.5);
    assert_eq!(stats.cs, 4.0);
    assert_eq!(stats.od, 9.3);
    assert_eq!(stats.ar, 9.5);
}

#[test]
fn ignores_comments_and_other_sections() {
    let text = "[General]\nCircleSize:9\n\n[Difficulty]\n// CircleSize:1\nCircleSize:5\n";
    let stats = parse_difficulty(text).unwrap();
    assert_eq!(stats.cs, 5.0, "只应读取 Difficulty 段，且忽略注释");
}

#[test]
fn missing_keys_fall_back_to_defaults() {
    let stats = parse_difficulty("[Difficulty]\nCircleSize:7\n").unwrap();
    assert_eq!(stats.cs, 7.0);
    assert_eq!(stats.ar, 5.0);
    assert_eq!(stats.od, 5.0);
    assert_eq!(stats.hp, 5.0, "osu! 默认值是 5");
}

#[test]
fn section_name_is_case_insensitive() {
    let stats = parse_difficulty("[difficulty]\nCircleSize:3.5\n").unwrap();
    assert_eq!(stats.cs, 3.5);
}

#[test]
fn bad_value_is_reported() {
    let err = parse_difficulty("[Difficulty]\nCircleSize:abc\n").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("CircleSize"), "错误应指出字段名：{msg}");
}
