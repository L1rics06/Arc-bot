//! `.osu` 文本解析（纯函数，无 IO）。

use crate::error::{Error, Result};
use crate::local_text;
use crate::models::DifficultyStats;

/// 从 `.osu` 文件文本中解析 `[Difficulty]` 段的四维难度。
///
/// 解析规则（osu! 文件格式）：
/// - 段由 `[SectionName]` 开头，直到下一个段为止；
/// - 段内是 `Key:Value`，`//` 之后是注释；
/// - 键名大小写不敏感；缺失的键使用 osu! 官方默认值
///   （AR/OD/CS/HP 默认 5，`HPDrainRate` 才是 HP）。
pub fn parse_difficulty(text: &str) -> Result<DifficultyStats> {
    let mut in_section = false;
    let mut cs = None;
    let mut ar = None;
    let mut od = None;
    let mut hp = None;

    for raw in text.lines() {
        let line = raw.split("//").next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            in_section = line[1..line.len() - 1].eq_ignore_ascii_case("Difficulty");
            continue;
        }
        if !in_section {
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        let parsed: f32 = value
            .parse()
            .map_err(|_| Error::OsuFile(local_text::osu_file::bad_field(key, value)))?;

        match key.trim().to_ascii_lowercase().as_str() {
            "circlesize" => cs = Some(parsed),
            "approachrate" => ar = Some(parsed),
            "overalldifficulty" => od = Some(parsed),
            "hpdrainrate" => hp = Some(parsed),
            _ => {}
        }
    }

    Ok(DifficultyStats {
        cs: cs.unwrap_or(5.0),
        ar: ar.unwrap_or(5.0),
        od: od.unwrap_or(5.0),
        hp: hp.unwrap_or(5.0),
    })
}
