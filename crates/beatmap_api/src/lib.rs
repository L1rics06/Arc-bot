//! beatmap_api：osu! 铺面数据模块。
//!
//! ## 分层
//!
//! | 模块 | 内容 |
//! |---|---|
//! | [`models`] | 领域模型：`BeatmapInfo` / `DifficultyStats` / `MapStatus`，与 rosu-v2 解耦 |
//! | [`client`] | API 客户端与查询服务（`OsuService`），进程内共享一个实例 |
//! | [`cache`] | 谱面元数据本地缓存（TTL + schema 版本号） |
//! | [`osu_file`] | `.osu` 文件下载与 `[Difficulty]` 解析（相似谱面管线的前置件） |
//! | [`service`] | `/map` 命令的注册与渲染 |
//! | [`local_text`] | 用户可见文案集中处 |
//! | [`error`] | 面向用户的错误类型 |
//!
//! 依赖方向：`beatmap_api → arc-core`。命令的"描述"用 `arc_core` 的类型，
//! 不依赖 cmd_router（避免循环依赖）。

pub mod cache;
pub mod client;
pub mod error;
pub mod local_text;
pub mod models;
pub mod osu_file;
pub mod service;

pub use client::OsuService;
pub use error::{Error, Result};
pub use models::{BeatmapInfo, DifficultyStats, MapStatus, parse_beatmap_id};
pub use service::{register, render};

/// 本模块命令使用的共享状态类型别名。
pub type BeatmapState = arc_core::SharedState<OsuService>;
