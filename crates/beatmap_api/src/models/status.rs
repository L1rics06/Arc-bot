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
/// 前后空白会被忽略；`0` 视为非法（osu! 的 ID 从 1 开始）。
pub fn parse_beatmap_id(input: &str) -> Result<u32, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(local_text::hint::need_id());
    }
    let id: u32 = trimmed
        .parse()
        .map_err(|_| local_text::hint::bad_id(trimmed))?;
    if id == 0 {
        return Err(local_text::hint::ZERO_ID.to_owned());
    }
    Ok(id)
}
