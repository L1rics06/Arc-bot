//! 命令元信息的只读视图。
//!
//! `/help` 这类需要"自我介绍"的命令必须能读到命令表，但命令表本身住在
//! cmd_router 里。为避免循环依赖，这里只定义最小 trait：
//! cmd_router 实现它并挂到 [`AppState`](crate::state::AppState) 上，
//! 业务模块（如 help 命令）只依赖 arc-core 就能查询。

/// 展示给用户的单条命令说明。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInfo {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub usage: String,
    pub details: Option<String>,
    pub admin_only: bool,
}

/// 命令表查询接口。
pub trait CommandSource: Send + Sync + 'static {
    /// 全部可展示命令。
    fn visible(&self) -> Vec<CommandInfo>;

    /// 按名字或别名查一条命令。
    fn lookup(&self, name: &str) -> Option<CommandInfo>;
}
