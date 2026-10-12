//! 查询结果的领域模型。
//!
//! 从 `osu_difficulty_runtime` 的类型解耦出来：只保留渲染需要的字段，
//! 便于 [`crate::handlers::render`] 做纯函数渲染，也便于测试构造。

use osu_difficulty_runtime::{ManiaGameMod, ManiaModeFamily, ManiaPattern};

/// 查询目标的来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetSource {
    /// 数据集里已有记录。
    InIndex,
    /// 数据集里没有，现场下载 `.osu` 分析得到。
    OnTheFly,
}

/// 目标谱面信息。
#[derive(Debug, Clone, PartialEq)]
pub struct TargetInfo {
    pub artist: String,
    pub title: String,
    pub version: String,
    /// 在线链接（数据集或分析器提供，可能为空）。
    pub url: Option<String>,
    /// NoMod 星数（standard）。
    pub star_rating: Option<f32>,
    /// mania 的键数 / 家族 / 主键型 / 难度分位。
    pub mania: Option<ManiaRowInfo>,
}

/// 一条候选谱面。
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateRow {
    pub beatmap_id: u64,
    pub beatmapset_id: u64,
    pub artist: String,
    pub title: String,
    pub version: String,
    pub url: Option<String>,
    /// NoMod 星数（standard；旧数据集可能缺失）。
    pub star_rating: Option<f32>,
    /// 距离（越小越相似，按归一化后的特征计算）。
    pub distance: f32,
    /// mania 的键数 / 家族 / 主键型 / 分位与分类标记。
    pub mania: Option<ManiaRowInfo>,
}

/// mania 候选的附加信息。
#[derive(Debug, Clone, PartialEq)]
pub struct ManiaRowInfo {
    pub key_count: u8,
    pub family: ManiaModeFamily,
    pub pattern: ManiaPattern,
    /// 该候选数据使用的 Mod（NM/DT/HT）。
    pub game_mod: ManiaGameMod,
    /// 难度分位（同一键数语料内的相对位置，0-1）。
    pub difficulty_percentile: f32,
    /// 键型记录的分类标记（如 `SV`），无键型记录时为 `None`。
    pub category: Option<String>,
}

/// 一次查询的完整结果。
#[derive(Debug, Clone, PartialEq)]
pub struct QueryOutcome {
    pub target: TargetInfo,
    pub rows: Vec<CandidateRow>,
    pub source: TargetSource,
    /// 数据集覆盖到的时间（Unix 秒）。
    pub data_cutoff_at: Option<i64>,
    /// 是否排除了同一谱面集（standard 固定排除）。
    pub same_set_excluded: bool,
    /// mania 的候选 Mod 池（用于提示）。
    pub candidate_mods: Vec<ManiaGameMod>,
}
