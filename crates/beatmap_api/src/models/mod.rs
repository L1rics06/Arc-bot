//! 领域模型：只保留业务真正用到的字段，与 rosu-v2 的响应结构解耦。
//!
//! 这样做的目的：
//! - 缓存落盘格式稳定，不随 rosu-v2 升级而漂移；
//! - 相似谱面算法需要的"多维难度"有明确的载体（[`DifficultyStats`]）。

mod beatmap_info;
mod difficulty;
mod status;

pub use beatmap_info::BeatmapInfo;
pub use difficulty::DifficultyStats;
pub use status::{MapStatus, parse_beatmap_id};
