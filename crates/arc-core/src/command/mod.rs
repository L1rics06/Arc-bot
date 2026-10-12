//! 命令的**数据描述**：业务模块用它声明命令，路由实现用它分发。
//!
//! 为什么住在 arc-core 而不是路由插件里：业务模块（issues / beatmap_api）需要描述
//! 自己的命令，但不应该依赖 cmd_router；把"描述"下沉到这里，依赖方向就永远是单向的。
//! 命令的**执行**（分发、权限、help 聚合）在 cmd_router。

mod builder;
mod spec;

pub use builder::RegistryBuilder;
pub use spec::{Command, CommandSpec, Handler, boxed, make, name_from_usage};

use crate::args::BoxedError;
use crate::event::Reply;

/// 便于业务侧返回带上下文的错误。
pub type CommandResult = Result<Box<dyn Reply>, BoxedError>;
