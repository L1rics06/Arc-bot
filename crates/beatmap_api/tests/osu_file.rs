//! `beatmap_api::osu_file` 的测试：[Difficulty] / [General] 段解析与文件识别。

use beatmap_api::osu_file::{looks_like_osu_file, parse_difficulty, parse_mode};

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

#[test]
fn parses_mode_from_general_section() {
    let text = "[General]\nAudioFilename: audio.mp3\nMode: 3\n\n[Difficulty]\nMode: 0\n";
    assert_eq!(parse_mode(text).unwrap(), 3, "Mode 应只从 General 段读取");
    assert_eq!(parse_mode("[General]\nMode:0\n").unwrap(), 0);
}

#[test]
fn mode_defaults_to_standard_when_absent() {
    assert_eq!(parse_mode("[General]\nAudioFilename: a.mp3\n").unwrap(), 0);
    assert_eq!(parse_mode("").unwrap(), 0);
}

#[test]
fn mode_key_is_case_insensitive_and_ignores_comments() {
    let text = "[general]\n// Mode: 3\nmode: 2 // taiko\n";
    assert_eq!(parse_mode(text).unwrap(), 2);
}

#[test]
fn bad_mode_value_is_reported() {
    let err = parse_mode("[General]\nMode: mania\n").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Mode"), "错误应指出字段名：{msg}");
    assert!(msg.contains("mania"), "错误应带上原值：{msg}");
}

#[test]
fn recognizes_osu_file_magic() {
    assert!(looks_like_osu_file(b"osu file format v14\n[General]\n"));
    assert!(
        looks_like_osu_file(b"\xEF\xBB\xBFosu file format v14\n"),
        "带 UTF-8 BOM 的文件头也应识别"
    );
    assert!(!looks_like_osu_file(b"<!DOCTYPE html>\n<html>"));
    assert!(!looks_like_osu_file(b""));
}
