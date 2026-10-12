//! issue 存储：JSON 文件 + 稳定编号分配 + 旧数据迁移。
//!
//! 设计要点：
//! - 每个 issue 一个文件，文件名是分配到的编号（纯数字，天然安全）；
//! - 编号创建后永不变更，删除不复用；
//! - 同一进程内的"分配编号 → 写文件"整体串行化，避免并发 /add 抢占编号；
//! - 旧版本数据（见 [`LEGACY_DIR`]）首次运行自动迁移，原目录保留不删。
//!
//! 对外只暴露 [`Store`]；编号格式化与迁移逻辑是模块内部实现。

mod id;
mod legacy;
mod store;

pub use id::parse_user_id;
pub use legacy::LEGACY_DIR;
pub use store::Store;
