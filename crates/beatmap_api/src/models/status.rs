//! 谱面状态与 beatmapID 解析。

use std::fmt::{Display, Formatter};

use rosu_v2::model::beatmap::RankStatus;
use serde::{Deserialize, Serialize};

use crate::local_text;

/// 谱面状态（只保留会展示 / 参与筛选的部分）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapStatus {
    Ranked,
    Approved,
    Qualified,
    Loved,
    Pending,
    Wip,
    Graveyard,
    /// 由协议端 / API 返回的未知状态。
    Unknown,
}

impl MapStatus {
    /// rosu-v2 的响应状态 → 领域状态。
    pub fn from_rosu(status: RankStatus) -> Self {
        match status {
            RankStatus::Ranked => MapStatus::Ranked,
            RankStatus::Approved => MapStatus::Approved,
            RankStatus::Qualified => MapStatus::Qualified,
            RankStatus::Loved => MapStatus::Loved,
            RankStatus::Pending => MapStatus::Pending,
            RankStatus::WIP => MapStatus::Wip,
            RankStatus::Graveyard => MapStatus::Graveyard,
        }
    }
}

impl Display for MapStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            MapStatus::Ranked => "Ranked",
            MapStatus::Approved => "Approved",
            MapStatus::Qualified => "Qualified",
            MapStatus::Loved => "Loved",
            MapStatus::Pending => "Pending",
            MapStatus::Wip => "Work in progress",
            MapStatus::Graveyard => "Graveyard",
            MapStatus::Unknown => "Unknown",
        };
        f.write_str(text)
    }
}

/// 把字符串形式的 beatmapID 解析成合法 ID。
///
/// 支持两种写法：纯编号（`12345`）与 osu.ppy.sh 谱面链接
/// （`https://osu.ppy.sh/b/12345`、`https://osu.ppy.sh/beatmapsets/6789#osu/12345`）。
/// 前后空白会被忽略；`0` 视为非法（osu! 的 ID 从 1 开始）。
pub fn parse_beatmap_id(input: &str) -> Result<u32, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(local_text::hint::need_id());
    }
    if let Ok(id) = trimmed.parse::<u32>() {
        if id == 0 {
            return Err(local_text::hint::ZERO_ID.to_owned());
        }
        return Ok(id);
    }
    beatmap_id_from_link(trimmed).ok_or_else(|| local_text::hint::bad_id(trimmed))
}

/// 从 osu.ppy.sh 链接中提取 beatmapID（语义与 OPP 的 `parse_beatmap_id` 一致）。
///
/// 依次尝试 `#<mode>/<id>` 片段与 `…/b/<id>`、`…/beatmaps/<id>` 路径；
/// 非 osu.ppy.sh 主机、或链接里没有 beatmapID（如只有谱面集 ID）时返回 `None`。
fn beatmap_id_from_link(input: &str) -> Option<u32> {
    // 允许带/不带协议头，以及 // 开头的写法
    let without_scheme = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .or_else(|| input.strip_prefix("//"))
        .unwrap_or(input);
    let (host, rest) = without_scheme.split_once('/')?;
    let host = host.split(':').next().unwrap_or(host).to_ascii_lowercase();
    if host != "osu.ppy.sh" && host != "www.osu.ppy.sh" {
        return None;
    }

    // `#osu/12345`、`#mania/12345` 这类片段优先级最高
    let (path, fragment) = match rest.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment)),
        None => (rest, None),
    };
    if let Some(fragment) = fragment {
        let last = fragment.split('/').next_back().unwrap_or("");
        if let Ok(id) = last.parse::<u32>()
            && id > 0
        {
            return Some(id);
        }
    }

    let path = path.split('?').next().unwrap_or(path);
    let segments: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    for prefix in ["beatmaps", "b"] {
        if let Some(index) = segments.iter().position(|part| *part == prefix)
            && let Some(id) = segments
                .get(index + 1)
                .and_then(|part| part.parse::<u32>().ok())
            && id > 0
        {
            return Some(id);
        }
    }
    None
}
