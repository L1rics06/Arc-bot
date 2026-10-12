//! 谱面信息领域模型，以及从 rosu-v2 响应到本模型的映射。

use serde::{Deserialize, Serialize};

use super::{DifficultyStats, MapStatus};

/// 谱面集（一层 mapset）信息。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeatmapInfo {
    /// 难度 ID（beatmap id）。
    pub map_id: u32,
    /// 谱面集 ID。
    pub set_id: u32,
    /// 艺术家，如 `Camellia`。
    pub artist: String,
    /// 曲名，如 `Ghost`。
    pub title: String,
    /// 谱面集作者（mapper）。
    pub creator: String,
    /// 难度名，如 `Insane`。
    pub version: String,
    /// 星数。
    pub stars: f32,
    /// BPM。
    pub bpm: f32,
    /// 谱面状态（ranked / loved / graveyard 等）。
    pub status: MapStatus,
    /// 四维难度设置。
    pub difficulty: DifficultyStats,
}
