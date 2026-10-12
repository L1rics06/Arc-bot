//! similar_beatmap 面向用户的文案集中处。
//!
//! 约定：所有会出现在 QQ 消息里的固定字符串都写在这里，其余模块只引用常量或调用函数。

use osu_difficulty_runtime::{ManiaGameMod, ManiaModeFamily, ManiaPattern};

use crate::models::Ruleset;

/// 命令用法。
pub mod usage {
    /// `/similar` 的用法。
    pub const SIMILAR: &str = "/similar <谱面ID或链接> [-n 5]";
    /// `/similar` 的详细说明（`/help similar` 展示）。
    pub const SIMILAR_DETAILS: &str = "\
按「玩法/手感相近」查找谱面，谱面 ID 可以写成编号或 osu.ppy.sh 链接（选项要写在 ID 之后）。

osu!standard（默认权重 aim 1 / speed 2 / reading 2 / slider 0 / overlap 0.25，参数权重 1）：
  -n 5                            返回条数（1-20）
  --weights 1,2,2,0,0.25          五维权重：瞄准,速度,读图,滑条,物件重叠（0-2，不能全为 0）
  --param 1.0                     AR/CS/OD 参数权重（0-2，0 表示只看难度维度）
  --star 6-8                      星数区间（`6+` 或单个数字表示下限，只有上限时写成 `0-8`）
  --bpm 150-220                   BPM 区间
  --len 60-300                    时长区间（秒）
  --ar 9-10 --cs 3.5-4.5 --od 7-10   AR / CS / OD 区间
  --density --circle --slider     物件密度、圆圈比例、滑条比例区间

osu!mania（自动按谱面键数隔离，仅支持 4K/6K/7K）：
  --mods NM,DT,HT                 候选 Mod 池（默认 NM；也可写 NC/DC 分别代表 DT/HT）
  --target-mod DT                 以哪个 Mod 的视角比较目标（默认 NM，须在候选池里）
  --same-set                      允许返回同一谱面集里的其他难度

说明：谱面不在数据集里时会现场下载 .osu 分析（首次分析需要数秒）；
结果按距离升序，默认排除与目标同一谱面集。";
}

/// 各类提示与错误文案。
pub mod error {
    use super::*;

    /// 目标谱面不在数据集里。
    pub fn not_in_dataset(map_id: u32) -> String {
        format!("目标谱面 {map_id} 不在数据集中（可用 /map 确认谱面编号是否有效）")
    }

    /// 两套数据集都没有部署。
    pub fn no_dataset(standard_dir: &str, mania_dir: &str) -> String {
        format!(
            "没有部署相似谱面数据集：把数据集解压到 {standard_dir}（standard）或 {mania_dir}（mania），\
             或用环境变量 {} / {} 指定路径；详见 docs/similar-beatmap.md",
            ruleset::STANDARD_ENV,
            ruleset::MANIA_ENV
        )
    }

    /// 数据集目录没有部署。
    pub fn dataset_missing(ruleset: Ruleset, dir: &str) -> String {
        format!(
            "未找到 {} 数据集（{dir}）：把数据集完整解压到该目录，或用环境变量 {} 指定路径；详见 docs/similar-beatmap.md",
            ruleset.name(),
            ruleset.env()
        )
    }

    /// 数据集存在但无法打开 / 查询。
    pub fn dataset_unavailable(ruleset: Ruleset, dir: &str, detail: &str) -> String {
        format!(
            "{} 数据集无法使用（{dir}）：{detail}；数据集必须与算法版本配套，详见 docs/similar-beatmap.md",
            ruleset.name()
        )
    }

    /// 数据集与当前算法版本不配套。
    pub fn dataset_incompatible(ruleset: Ruleset, detail: &str) -> String {
        format!(
            "{} 数据集与当前算法版本不配套：{detail}；请下载与该算法版本配套的完整数据集，详见 docs/similar-beatmap.md",
            ruleset.name()
        )
    }

    /// 查询过程失败。
    pub fn dataset_query_failed(detail: &str) -> String {
        format!("相似谱面查询失败：{detail}（数据集可能不完整或已损坏）")
    }

    /// 目标谱面的模式不支持。
    pub fn unsupported_mode(mode: u8) -> String {
        format!("只支持 osu!standard 与 osu!mania 谱面（该谱面模式代码为 {mode}）")
    }

    /// mania 谱面键数不支持。
    pub fn unsupported_keys(keys: i32) -> String {
        format!("osu!mania 相似谱面只支持 4K/6K/7K（该谱面为 {keys}K）")
    }

    /// 目标谱面不在数据集里、需要现场分析，但 osu! API 未初始化。
    pub fn api_not_configured() -> String {
        "目标谱面不在数据集中，需要现场下载 .osu 分析，但 osu! API 未初始化（检查 .env 的 CLIENT_ID / CLIENT_SECRET）"
            .to_owned()
    }

    /// 内部错误（后台任务失败等）。
    pub fn internal(detail: &str) -> String {
        format!("内部错误：{detail}")
    }
}

/// 参数解析的提示。
pub mod hint {
    use super::*;

    /// 缺少谱面 ID。
    pub fn need_id() -> String {
        format!("缺少谱面 ID，用法：{}", usage::SIMILAR)
    }

    /// 选项写在了谱面 ID 前面。
    pub fn id_before_flags() -> String {
        format!(
            "请把谱面 ID 写在最前面，选项跟在后面，例如：{}",
            usage::SIMILAR
        )
    }

    /// 对 mania 谱面用了 standard 专属选项。
    pub fn standard_only_flags() -> String {
        format!(
            "这些选项只对 osu!standard 谱面有效（--weights/--param/--star 等）：{}",
            usage::SIMILAR
        )
    }

    /// 对 standard 谱面用了 mania 专属选项。
    pub fn mania_only_flags() -> String {
        format!(
            "这些选项只对 osu!mania 谱面有效（--mods/--target-mod/--same-set）：{}",
            usage::SIMILAR
        )
    }

    /// 选项缺少取值。
    pub fn flag_needs_value(flag: &str) -> String {
        format!("选项 --{flag} 需要一个取值，请看用法：{}", usage::SIMILAR)
    }

    /// 返回条数超范围。
    pub fn bad_limit(max: usize) -> String {
        format!("返回条数需在 1-{max} 之间")
    }

    /// 区间写法非法。
    pub fn bad_range(label: &str, raw: &str, lower: f32, upper: f32) -> String {
        format!(
            "{label} 区间 `{raw}` 无法解析（写法：`6-8`、`6+` 或单个数字表示下限；只有上限时写成 `0-8`，取值范围 {lower}-{upper}）"
        )
    }

    /// 区间上下限颠倒。
    pub fn reversed_range(label: &str, raw: &str) -> String {
        format!("{label} 区间 `{raw}` 的下限比上限大")
    }

    /// 权重写法非法。
    pub fn bad_weights() -> String {
        "权重需写成 5 个 0-2 的数字，用逗号分隔，例如 --weights 1,2,2,0,0.25".to_owned()
    }

    /// 参数权重非法。
    pub fn bad_parameter_weight(raw: &str) -> String {
        format!("参数权重 `{raw}` 需是 0-2 的数字")
    }

    /// Mod 池为空或写法非法。
    pub fn bad_mods(raw: &str) -> String {
        format!("Mod 池 `{raw}` 无法解析（只支持 NM/DT/HT，或用 NC/DC 代表 DT/HT）")
    }

    /// Mod 池里有重复项。
    pub fn duplicate_mod(game_mod: ManiaGameMod) -> String {
        format!("Mod 池里 {} 重复了", game_mod.as_str())
    }

    /// `--target-mod` 不在候选池里。
    pub fn target_mod_outside_pool(game_mod: ManiaGameMod) -> String {
        format!(
            "--target-mod {} 不在 --mods 候选池里（比较视角必须来自候选池）",
            game_mod.as_str()
        )
    }
}

/// 渲染用到的字段名、单位与分隔符。
pub mod render {
    use super::*;

    /// 目标行前缀。
    pub const TARGET_PREFIX: &str = "相似谱面 · 目标：";
    /// 字段分隔符。
    pub const SEPARATOR: &str = " · ";
    /// 星数前缀（standard）。
    pub const STAR: &str = "★";
    /// 距离前缀。
    pub const DISTANCE: &str = "距离";
    /// 难度分位前缀（mania）。
    pub const PERCENTILE: &str = "分位";
    /// 键数后缀（mania）。
    pub const KEYS_SUFFIX: &str = "K";
    /// 没有候选时的提示。
    pub const EMPTY: &str =
        "没有找到符合条件的候选（可以放宽 --star / --bpm 等筛选，或换一个权重组合）";
    /// 标准谱面默认排除同一谱面集。
    pub const SAME_SET_EXCLUDED: &str = "已排除同一谱面集";
    /// mania 显式包含同一谱面集。
    pub const SAME_SET_INCLUDED: &str = "含同一谱面集";
    /// 目标为现场分析结果的说明。
    pub const ON_THE_FLY: &str = "目标谱面不在数据集中，为现场分析结果";
    /// 数据集时间范围说明。
    pub fn data_cutoff(date: &str) -> String {
        format!("数据截至 {date}")
    }
    /// 结果条数说明。
    pub fn count(total: usize) -> String {
        format!("共 {total} 条")
    }
    /// 序号 + 内容。
    pub fn row(index: usize, body: &str) -> String {
        format!("{index}. {body}")
    }
    /// 渲染 mania 的模式族与主键型。
    pub fn mania_style(family: ManiaModeFamily, pattern: ManiaPattern) -> String {
        format!("{}/{}", family.as_str(), pattern.as_str())
    }
    /// 渲染 mania 的 SV 等分类标记。
    pub fn mania_category(category: &str) -> String {
        format!("[{category}]")
    }
    /// 现场分析的 Mod 池说明。
    pub fn mods(pool: &[ManiaGameMod]) -> String {
        let list = pool
            .iter()
            .map(|game_mod| game_mod.as_str())
            .collect::<Vec<_>>()
            .join("/");
        format!("候选 {list}")
    }
}

/// 模式名与对应的环境变量。
pub mod ruleset {
    /// osu!standard 的展示名。
    pub const STANDARD_NAME: &str = "osu!standard";
    /// osu!mania 的展示名。
    pub const MANIA_NAME: &str = "osu!mania";
    /// osu!standard 数据集目录的环境变量。
    pub const STANDARD_ENV: &str = "SIMILAR_STANDARD_DATASET";
    /// osu!mania 数据集目录的环境变量。
    pub const MANIA_ENV: &str = "SIMILAR_MANIA_DATASET";
}

/// 日志文案。
pub mod log {
    /// 数据集首次打开成功。
    pub fn dataset_opened(name: &str, records: usize, dir: &str) -> String {
        format!("{name} 相似谱面数据集已加载：{records} 条记录（{dir}）")
    }
    /// 现场分析耗时提示。
    pub fn analyzing(map_id: u32) -> String {
        format!("谱面 {map_id} 不在数据集中，开始现场分析")
    }
}
