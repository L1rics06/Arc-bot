//! 查询参数解析（纯函数，无 IO）与模式定义。
//!
//! 语义对齐 OPP `src-tauri/src/features/similarity/query.rs` 的校验与默认值：
//! 权重 0-2 且不能全为 0、区间过滤器上下限有序、结果条数受限。

use arc_core::Args;
use osu_difficulty_runtime::{DifficultyWeights, ManiaGameMod, QueryFilters};

use crate::error::{Error, Result};
use crate::local_text;

/// 默认返回条数。
pub const DEFAULT_RESULTS: usize = 5;
/// 单条消息最多返回的条数。
pub const MAX_RESULTS: usize = 20;
/// 默认参数权重（与 OPP 前端一致）。
const DEFAULT_PARAMETER_WEIGHT: f32 = 1.0;
/// 权重取值范围上限（runtime 要求 ≤ 2）。
const MAX_WEIGHT: f32 = 2.0;

/// 相似谱面支持的两个模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ruleset {
    Standard,
    Mania,
}

impl Ruleset {
    /// 展示名（用于错误文案）。
    pub const fn name(self) -> &'static str {
        match self {
            Ruleset::Standard => local_text::ruleset::STANDARD_NAME,
            Ruleset::Mania => local_text::ruleset::MANIA_NAME,
        }
    }

    /// 数据集目录对应的环境变量名。
    pub const fn env(self) -> &'static str {
        match self {
            Ruleset::Standard => local_text::ruleset::STANDARD_ENV,
            Ruleset::Mania => local_text::ruleset::MANIA_ENV,
        }
    }
}

/// 一次 `/similar` 的完整参数。
#[derive(Debug, Clone)]
pub struct Request {
    /// 目标谱面 ID。
    pub map_id: u32,
    /// 返回条数。
    pub limit: usize,
    /// standard 相关参数（非 standard 谱面时校验后忽略）。
    pub standard: StandardRequest,
    /// mania 相关参数（非 mania 谱面时校验后忽略）。
    pub mania: ManiaRequest,
}

/// standard 查询参数。
#[derive(Debug, Clone)]
pub struct StandardRequest {
    pub filters: QueryFilters,
    pub weights: DifficultyWeights,
    pub parameter_weight: f32,
    /// 用户是否显式给了 standard 专属选项（用于对 mania 谱面报错）。
    pub explicit: bool,
}

impl Default for StandardRequest {
    fn default() -> Self {
        Self {
            filters: QueryFilters::default(),
            weights: DifficultyWeights::default(),
            parameter_weight: DEFAULT_PARAMETER_WEIGHT,
            explicit: false,
        }
    }
}

/// mania 查询参数。
#[derive(Debug, Clone)]
pub struct ManiaRequest {
    pub candidate_mods: Vec<ManiaGameMod>,
    pub target_mod: ManiaGameMod,
    pub include_same_set: bool,
    /// 用户是否显式给了 mania 专属选项（用于对 standard 谱面报错）。
    pub explicit: bool,
}

impl Default for ManiaRequest {
    fn default() -> Self {
        Self {
            candidate_mods: vec![ManiaGameMod::Nm],
            target_mod: ManiaGameMod::Nm,
            include_same_set: false,
            explicit: false,
        }
    }
}

impl Request {
    /// 从命令参数解析出一次请求；参数非法时返回带用法提示的错误。
    pub fn parse<C>(args: &Args<C>) -> Result<Self> {
        Ok(Self {
            map_id: parse_map_id(args)?,
            limit: parse_limit(args)?,
            standard: parse_standard(args)?,
            mania: parse_mania(args)?,
        })
    }
}

/// standard 专属选项的 flag 名。
const STANDARD_FLAGS: [&str; 10] = [
    "star", "bpm", "len", "ar", "cs", "od", "density", "circle", "slider", "weights",
];

/// mania 专属选项的 flag 名。
const MANIA_FLAGS: [&str; 3] = ["mods", "target-mod", "same-set"];

/// 是否显式使用了 standard 专属选项（`--param` 单独判断）。
pub(crate) fn uses_standard_flags<C>(args: &Args<C>) -> bool {
    args.has_flag("param") || STANDARD_FLAGS.iter().any(|flag| args.has_flag(flag))
}

/// 是否显式使用了 mania 专属选项。
pub(crate) fn uses_mania_flags<C>(args: &Args<C>) -> bool {
    MANIA_FLAGS.iter().any(|flag| args.has_flag(flag))
}

/// 解析目标谱面 ID（编号或 osu.ppy.sh 链接）。
fn parse_map_id<C>(args: &Args<C>) -> Result<u32> {
    let raw = args
        .positional(0)
        .ok_or_else(|| Error::Usage(local_text::hint::need_id()))?;
    if raw.starts_with('-') {
        // Args 只在第 2 个 token 起识别 flag，用户把选项写在前面时会落到这里
        return Err(Error::Usage(local_text::hint::id_before_flags()));
    }
    beatmap_api::parse_beatmap_id(raw).map_err(Error::Usage)
}

/// 解析返回条数（`-n N` / `--count N`）。
fn parse_limit<C>(args: &Args<C>) -> Result<usize> {
    let raw = match args.flag("n").or_else(|| args.flag("count")) {
        Some(raw) => raw,
        None => {
            return if args.has_flag("n") || args.has_flag("count") {
                Err(Error::Usage(local_text::hint::flag_needs_value("n")))
            } else {
                Ok(DEFAULT_RESULTS)
            };
        }
    };
    let limit: usize = raw
        .trim()
        .parse()
        .map_err(|_| Error::Usage(local_text::hint::bad_limit(MAX_RESULTS)))?;
    if !(1..=MAX_RESULTS).contains(&limit) {
        return Err(Error::Usage(local_text::hint::bad_limit(MAX_RESULTS)));
    }
    Ok(limit)
}

/// 解析 standard 的五维权重与参数权重。
fn parse_standard<C>(args: &Args<C>) -> Result<StandardRequest> {
    let mut request = StandardRequest {
        explicit: uses_standard_flags(args),
        ..StandardRequest::default()
    };

    if let Some(raw) = args.flag("weights") {
        let parts: Vec<&str> = raw.split(',').collect();
        if parts.len() != 5 {
            return Err(Error::Usage(local_text::hint::bad_weights()));
        }
        let mut values = [0.0_f32; 5];
        for (slot, part) in values.iter_mut().zip(parts) {
            let value: f32 = part
                .trim()
                .parse()
                .map_err(|_| Error::Usage(local_text::hint::bad_weights()))?;
            if !value.is_finite() || !(0.0..=MAX_WEIGHT).contains(&value) {
                return Err(Error::Usage(local_text::hint::bad_weights()));
            }
            *slot = value;
        }
        request.weights = DifficultyWeights::from_array(values);
    } else if args.has_flag("weights") {
        return Err(Error::Usage(local_text::hint::flag_needs_value("weights")));
    }

    if let Some(raw) = args.flag("param") {
        let value: f32 = raw
            .trim()
            .parse()
            .map_err(|_| Error::Usage(local_text::hint::bad_parameter_weight(raw)))?;
        if !value.is_finite() || !(0.0..=MAX_WEIGHT).contains(&value) {
            return Err(Error::Usage(local_text::hint::bad_parameter_weight(raw)));
        }
        request.parameter_weight = value;
    } else if args.has_flag("param") {
        return Err(Error::Usage(local_text::hint::flag_needs_value("param")));
    }

    if request.weights.as_array().iter().all(|value| *value == 0.0)
        && request.parameter_weight == 0.0
    {
        return Err(Error::Usage(local_text::hint::bad_weights()));
    }

    let (min, max) = parse_range(args, "star", "星数", 0.0, 20.0)?;
    request.filters.min_star = min;
    request.filters.max_star = max;
    let (min, max) = parse_range(args, "bpm", "BPM", 0.0, 1000.0)?;
    request.filters.min_bpm = min;
    request.filters.max_bpm = max;
    let (min, max) = parse_range(args, "len", "时长（秒）", 0.0, 7200.0)?;
    request.filters.min_length_seconds = min;
    request.filters.max_length_seconds = max;
    let (min, max) = parse_range(args, "ar", "AR", 0.0, 11.0)?;
    request.filters.min_ar = min;
    request.filters.max_ar = max;
    let (min, max) = parse_range(args, "cs", "CS", 0.0, 10.0)?;
    request.filters.min_cs = min;
    request.filters.max_cs = max;
    let (min, max) = parse_range(args, "od", "OD", 0.0, 11.0)?;
    request.filters.min_od = min;
    request.filters.max_od = max;
    let (min, max) = parse_range(args, "density", "物件密度", 0.0, 100.0)?;
    request.filters.min_object_density = min;
    request.filters.max_object_density = max;
    let (min, max) = parse_range(args, "circle", "圆圈比例", 0.0, 1.0)?;
    request.filters.min_circle_ratio = min;
    request.filters.max_circle_ratio = max;
    let (min, max) = parse_range(args, "slider", "滑条比例", 0.0, 1.0)?;
    request.filters.min_slider_ratio = min;
    request.filters.max_slider_ratio = max;

    Ok(request)
}

/// 解析 mania 的 Mod 池、比较视角与同谱面集开关。
fn parse_mania<C>(args: &Args<C>) -> Result<ManiaRequest> {
    let mut request = ManiaRequest {
        explicit: uses_mania_flags(args),
        ..ManiaRequest::default()
    };

    if let Some(raw) = args.flag("mods") {
        let mut pool = Vec::new();
        for part in raw.split(',') {
            let game_mod = parse_mod(part.trim(), raw)?;
            if pool.contains(&game_mod) {
                return Err(Error::Usage(local_text::hint::duplicate_mod(game_mod)));
            }
            pool.push(game_mod);
        }
        if pool.is_empty() {
            return Err(Error::Usage(local_text::hint::bad_mods(raw)));
        }
        request.candidate_mods = pool;
    } else if args.has_flag("mods") {
        return Err(Error::Usage(local_text::hint::flag_needs_value("mods")));
    }

    // 比较视角缺省为 NM；池里没有 NM 时取池中第一项
    request.target_mod = match args.flag("target-mod") {
        Some(raw) => parse_mod(raw.trim(), raw)?,
        None if args.has_flag("target-mod") => {
            return Err(Error::Usage(local_text::hint::flag_needs_value(
                "target-mod",
            )));
        }
        None => request
            .candidate_mods
            .first()
            .copied()
            .unwrap_or(ManiaGameMod::Nm),
    };
    if !request.candidate_mods.contains(&request.target_mod) {
        return Err(Error::Usage(local_text::hint::target_mod_outside_pool(
            request.target_mod,
        )));
    }

    request.include_same_set = args.has_flag("same-set");
    Ok(request)
}

/// 解析单个 Mod 名（NM/DT/HT，NC/DC 视为 DT/HT）。
fn parse_mod(raw: &str, whole: &str) -> Result<ManiaGameMod> {
    match raw.to_ascii_uppercase().as_str() {
        "NM" | "NOMOD" => Ok(ManiaGameMod::Nm),
        "DT" | "NC" => Ok(ManiaGameMod::Dt),
        "HT" | "DC" => Ok(ManiaGameMod::Ht),
        _ => Err(Error::Usage(local_text::hint::bad_mods(whole))),
    }
}

/// 解析 `6-8` / `6+` / `-8` / `6` 形式的区间，返回 (下限, 上限)。
///
/// 单个数字视为下限（`--star 6` = 至少 6 星）。
fn parse_range<C>(
    args: &Args<C>,
    flag: &str,
    label: &str,
    lower: f32,
    upper: f32,
) -> Result<(Option<f32>, Option<f32>)> {
    let Some(raw) = args.flag(flag) else {
        return if args.has_flag(flag) {
            Err(Error::Usage(local_text::hint::flag_needs_value(flag)))
        } else {
            Ok((None, None))
        };
    };
    let raw = raw.trim();
    let bounds = match raw.split_once('-') {
        // `6-8` / `6-`；左半边为空（如 `-8`）在命令参数里不会出现
        // （Args 只把不以 `-` 开头的 token 当作取值），这里兜底报错
        Some((min, max)) if !min.trim().is_empty() => (
            parse_bound(min, label, raw, lower, upper)?,
            parse_bound(max, label, raw, lower, upper)?,
        ),
        Some(_) => {
            return Err(Error::Usage(local_text::hint::bad_range(
                label, raw, lower, upper,
            )));
        }
        None => (
            parse_bound(
                raw.strip_suffix('+').unwrap_or(raw),
                label,
                raw,
                lower,
                upper,
            )?,
            None,
        ),
    };
    if let (Some(min), Some(max)) = bounds
        && min > max
    {
        return Err(Error::Usage(local_text::hint::reversed_range(label, raw)));
    }
    Ok(bounds)
}

/// 解析区间的一侧；空字符串表示该侧不设限。
fn parse_bound(raw: &str, label: &str, whole: &str, lower: f32, upper: f32) -> Result<Option<f32>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    let value: f32 = raw
        .parse()
        .map_err(|_| Error::Usage(local_text::hint::bad_range(label, whole, lower, upper)))?;
    if !value.is_finite() || value < lower || value > upper {
        return Err(Error::Usage(local_text::hint::bad_range(
            label, whole, lower, upper,
        )));
    }
    Ok(Some(value))
}
