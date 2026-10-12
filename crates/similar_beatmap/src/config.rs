//! 数据集目录解析。
//!
//! 优先读环境变量；没有配置时按 `data/similar_beatmap/{standard,mania}` 约定寻找。
//! 目录里是否存在 `metadata.sqlite` / `mania-metadata.sqlite` 用来判断"是否部署过"，
//! 避免每次命令都尝试打开不存在的目录。

use std::path::{Path, PathBuf};

use crate::local_text;
use crate::models::Ruleset;

/// 模块在 `data/` 下的目录名。
pub const MODULE_DIR: &str = "similar_beatmap";

/// standard 数据集目录。
pub fn standard_dir(module_dir: &Path) -> PathBuf {
    resolve_dir(
        local_text::ruleset::STANDARD_ENV,
        module_dir.join("standard"),
    )
}

/// mania 数据集目录。
pub fn mania_dir(module_dir: &Path) -> PathBuf {
    resolve_dir(local_text::ruleset::MANIA_ENV, module_dir.join("mania"))
}

/// 某个模式的数据集目录。
pub fn dir_for(module_dir: &Path, ruleset: Ruleset) -> PathBuf {
    match ruleset {
        Ruleset::Standard => standard_dir(module_dir),
        Ruleset::Mania => mania_dir(module_dir),
    }
}

/// 目录里是否已经放了对应模式的数据集。
pub fn is_deployed(dir: &Path, ruleset: Ruleset) -> bool {
    let marker = match ruleset {
        Ruleset::Standard => "metadata.sqlite",
        Ruleset::Mania => "mania-metadata.sqlite",
    };
    dir.join(marker).is_file()
}

/// 环境变量优先，其次默认目录（空字符串视为未设置）。
fn resolve_dir(env: &str, fallback: PathBuf) -> PathBuf {
    match std::env::var(env) {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value.trim()),
        _ => fallback,
    }
}
