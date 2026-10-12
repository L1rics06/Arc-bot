//! arc-bot 核心库。
//!
//! **不依赖 kovi / OneBot / 任何协议端类型**，只提供业务无关的基础设施：
//!
//! | 模块 | 内容 |
//! |---|---|
//! | [`state`] | `AppState<C>`：启动时构造一次、以 `Arc` 共享给全部命令的应用状态 |
//! | [`command`] | 命令的**数据描述**（`CommandSpec` / `Command` / `RegistryBuilder`） |
//! | [`args`] | 命令参数解析与命令名提取 |
//! | [`event`] | 与协议端无关的回复原语（`Reply` / `Text`） |
//! | [`registry`] | 命令表的只读视图（供 `/help` 这类自举命令使用） |
//! | [`storage`] | 文件型 JSON 存储（原子写、不吞错） |
//! | [`error`] | 统一错误类型 |
//!
//! 模块骨架与命名约定见仓库根目录 `AGENTS.md`。

pub mod args;
pub mod command;
pub mod error;
pub mod event;
pub mod registry;
pub mod state;
pub mod storage;

pub use args::{Args, BoxedError, CommandFuture, ListMode, extract_command};
pub use command::{
    Command, CommandResult, CommandSpec, Handler, RegistryBuilder, boxed as boxed_handler,
};
pub use error::{Error, Result};
pub use event::{IntoReply, Reply, Text, reply};
pub use registry::{CommandInfo, CommandSource};
pub use state::{AppMeta, AppState, SharedState, default_data_root};
pub use storage::JsonStore;
