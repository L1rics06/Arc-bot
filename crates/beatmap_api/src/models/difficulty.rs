//! 多维难度数据（相似谱面算法的输入之一）。

use serde::{Deserialize, Serialize};

/// 四维难度设置。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DifficultyStats {
    /// 判定圈大小。
    pub cs: f32,
    /// 缩圈速度。
    pub ar: f32,
    /// 判定严格度。
    pub od: f32,
    /// 掉血速度。
    pub hp: f32,
}

impl DifficultyStats {
    /// 距离度量：用于后续"相似谱面"排序时的四维加权 L1 距离。
    pub fn distance_to(&self, other: &Self) -> f32 {
        (self.cs - other.cs).abs()
            + (self.ar - other.ar).abs()
            + (self.od - other.od).abs()
            + (self.hp - other.hp).abs()
    }
}
