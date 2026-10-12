//! similar_beatmap：相似谱面（osu!standard / osu!mania）。
//!
//! `/similar <谱面ID或链接>` 在预计算数据集里找"玩法 / 手感相近"的谱面；
//! 目标不在数据集里时现场下载 `.osu` 分析（复用 [`beatmap_api::osu_file`]）。
//!
//! ## 分层
//!
//! | 模块 | 内容 |
//! |---|---|
//! | [`models`] | 参数解析（权重 / 区间过滤器 / Mod 池）与模式定义 |
//! | [`config`] | 数据集目录解析（环境变量 + `data/similar_beatmap/` 默认路径） |
//! | [`dataset`] | 数据集适配层：惰性打开、`spawn_blocking` 阻塞隔离、模式分派 |
//! | [`result`] | 查询结果的领域模型（与 runtime 类型解耦，便于渲染与测试） |
//! | [`handlers`] | `/similar` 的注册、编排与渲染 |
//! | [`local_text`] | 用户可见文案集中处 |
//! | [`error`] | 面向用户的错误类型 |
//!
//! ## 依赖方向
//!
//! `cmd_router → similar_beatmap → {beatmap_api, osu-difficulty-runtime} → arc-core`
//!
//! 相似度算法、数据文件格式与版本校验来自 vendored 的 [`osu_difficulty_runtime`]
//! （来源与同步方式见 `crates/osu-difficulty-runtime/VENDORED.md`）。本 crate
//! 不重写算法，只做"命令 → 数据集查询 → 中文渲染"的编排。
//!
//! 数据集不在 git 里：由 `osuplusplus/osu-difficulty-lab` 的 Release 提供，
//! 目录位置与版本绑定说明见 `docs/similar-beatmap.md`。

pub mod config;
pub mod dataset;
pub mod models;
pub mod result;

mod error;
mod handlers;
mod local_text;

pub use error::{Error, Result};
pub use handlers::{register, render};
