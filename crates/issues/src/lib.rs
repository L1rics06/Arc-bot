//! issues：轻量待办清单模块。
//!
//! ## 分层
//!
//! | 模块 | 内容 |
//! |---|---|
//! | [`models`] | 领域模型：`Issue` / `Status` / `NewIssue` |
//! | [`storage`] | 持久化：JSON 文件、编号分配、旧数据迁移 |
//! | [`handlers`] | 命令处理器（一条命令一个文件）与注册入口 |
//! | [`local_text`] | 用户可见文案集中处 |
//! | [`error`] | 面向用户的错误类型 |
//!
//! 依赖方向：`issues → arc-core`。命令的"描述"用 `arc_core` 的类型，
//! 不依赖 cmd_router（避免循环依赖）。
//!
//! ## 扩展方式
//!
//! 1. 新命令 → 在 `handlers/` 下新建一个文件，写 `register(builder, ctx)`；
//! 2. 在 `handlers/mod.rs` 的 `register_all` 里加一行；
//! 3. 文案放进 `local_text.rs`，不要在 handler 里写裸字符串。

pub mod error;
pub mod handlers;
pub mod local_text;
pub mod models;
pub mod storage;

pub use error::{Error, Result};
pub use handlers::{Context, MODULE_DIR, register};
pub use models::{Issue, NewIssue, Status};
pub use storage::Store;
