//! `.osu` 文本解析（纯函数，无 IO）。

use crate::error::{Error, Result};
use crate::local_text;
use crate::models::DifficultyStats;

/// `.osu` 文件头的固定前缀。
const FILE_MAGIC: &str = "osu file format v";

/// 按字节判断内容是否为 osu! 谱面文件（容忍 UTF-8 BOM）。
///
/// 从 osu.ppy.sh 拿到的可能是错误页 / 登录页，交给分析器之前先筛一遍。
pub fn looks_like_osu_file(bytes: &[u8]) -> bool {
    bytes
        .strip_prefix(b"\xEF\xBB\xBF".as_slice())
        .unwrap_or(bytes)
        .starts_with(FILE_MAGIC.as_bytes())
}

/// 从 `.osu` 文件文本中解析 `[General]` 段的游戏模式。
///
/// 0 = osu!standard、1 = taiko、2 = catch、3 = mania；
/// 键缺失时按 osu! 规范取默认值 standard(0)。
pub fn parse_mode(text: &str) -> Result<u8> {
    for (key, value) in section_pairs(text, "General") {
        if key.eq_ignore_ascii_case("Mode") {
            return value.parse().map_err(|_| {
                Error::OsuFile(local_text::osu_file::bad_field("General", key, value))
            });
        }
    }
    Ok(0)
}

/// 从 `.osu` 文件文本中解析 `[Difficulty]` 段的四维难度。
///
/// 解析规则（osu! 文件格式）：
/// - 段由 `[SectionName]` 开头，直到下一个段为止；
/// - 段内是 `Key:Value`，`//` 之后是注释；
/// - 键名大小写不敏感；缺失的键使用 osu! 官方默认值
///   （AR/OD/CS/HP 默认 5，`HPDrainRate` 才是 HP）。
pub fn parse_difficulty(text: &str) -> Result<DifficultyStats> {
    let mut stats = DifficultyStats {
        cs: 5.0,
        ar: 5.0,
        od: 5.0,
        hp: 5.0,
    };

    for (key, value) in section_pairs(text, "Difficulty") {
        let parsed: f32 = value.parse().map_err(|_| {
            Error::OsuFile(local_text::osu_file::bad_field("Difficulty", key, value))
        })?;
        match key.to_ascii_lowercase().as_str() {
            "circlesize" => stats.cs = parsed,
            "approachrate" => stats.ar = parsed,
            "overalldifficulty" => stats.od = parsed,
            "hpdrainrate" => stats.hp = parsed,
            _ => {}
        }
    }

    Ok(stats)
}

/// 遍历 `[section]` 段内的 `Key:Value` 行。
///
/// 段名大小写不敏感；返回的键名保留原样，值已去掉注释与首尾空白。
fn section_pairs<'a>(text: &'a str, section: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
    let mut in_section = false;
    text.lines().filter_map(move |raw| {
        let line = raw.split("//").next().unwrap_or("").trim();
        if line.is_empty() {
            return None;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line[1..line.len() - 1].eq_ignore_ascii_case(section);
            return None;
        }
        if !in_section {
            return None;
        }
        let (key, value) = line.split_once(':')?;
        Some((key.trim(), value.trim()))
    })
}
