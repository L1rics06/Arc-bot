//! 命令类型：一条命令长什么样，以及它怎么被"装箱"进注册表。

use std::sync::Arc;

use crate::args::{BoxedError, CommandFuture};
use crate::event::IntoReply;
use crate::state::SharedState;

/// 命令处理器函数类型（已装箱、可跨任务共享）。
pub type Handler<C> =
    Arc<dyn Fn(SharedState<C>, crate::args::Args<C>) -> CommandFuture + Send + Sync>;

/// 注册进路由表的一条命令。
pub struct Command<C> {
    /// 主命令名（不含 `/`，如 `map`）。
    pub name: String,
    /// 别名（同样不含 `/`）。
    pub aliases: Vec<String>,
    /// 一句话描述，出现在 `/help` 列表中。
    pub description: String,
    /// 用法示例，如 `/map <beatmapID>`。
    pub usage: String,
    /// 更多说明（可选），出现在 `/help <命令>` 中。
    pub details: Option<String>,
    /// 是否仅管理员可用。
    pub admin_only: bool,
    /// 隐藏命令（不出现在 /help 列表，但仍可用）。
    pub hidden: bool,
    /// 处理函数。
    pub handler: Handler<C>,
}

impl<C> std::fmt::Debug for Command<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("aliases", &self.aliases)
            .field("admin_only", &self.admin_only)
            .finish_non_exhaustive()
    }
}

impl<C: Send + Sync + 'static> Clone for Command<C> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            aliases: self.aliases.clone(),
            description: self.description.clone(),
            usage: self.usage.clone(),
            details: self.details.clone(),
            admin_only: self.admin_only,
            hidden: self.hidden,
            handler: self.handler.clone(),
        }
    }
}

/// 命令描述（与 handler 分开，便于业务侧先描述再注册）。
#[derive(Debug, Clone, Default)]
pub struct CommandSpec {
    /// 主命令名（不含 `/`）。由 [`RegistryBuilder::command`] 填充。
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub usage: String,
    pub details: Option<String>,
    pub admin_only: bool,
    pub hidden: bool,
}

impl CommandSpec {
    /// 快捷构造：描述 + 用法。
    pub fn new(description: impl Into<String>, usage: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            usage: usage.into(),
            ..Default::default()
        }
    }

    /// 添加别名。
    pub fn aliases(mut self, aliases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.aliases = aliases.into_iter().map(Into::into).collect();
        self
    }

    /// 添加详细说明。
    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// 限制为管理员可用。
    pub fn admin_only(mut self) -> Self {
        self.admin_only = true;
        self
    }

    /// 从 /help 列表中隐藏。
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
}

/// 把 handler 装箱成 [`Handler`]（需要手工构造 [`Command`] 时使用）。
pub fn boxed<C, F, Fut, R>(handler: F) -> Handler<C>
where
    C: Send + Sync + 'static,
    F: Fn(SharedState<C>, crate::args::Args<C>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<R, BoxedError>> + Send + 'static,
    R: IntoReply + 'static,
{
    Arc::new(move |state, args| {
        let fut = handler(state, args);
        Box::pin(async move { fut.await.map(|r| r.into_reply()) })
    })
}

/// 描述 + handler → [`Command`]。
///
/// 描述里没写命令名时，从 `usage`（形如 `/help <命令名>`）里推导；
/// 推导失败时名称为空，注册时会打出冲突告警。
pub fn make<C: Send + Sync + 'static>(mut spec: CommandSpec, handler: Handler<C>) -> Command<C> {
    if spec.name.is_empty() {
        spec.name = name_from_usage(&spec.usage).unwrap_or_default();
    }
    Command {
        name: spec.name,
        aliases: spec.aliases,
        description: spec.description,
        usage: spec.usage,
        details: spec.details,
        admin_only: spec.admin_only,
        hidden: spec.hidden,
        handler,
    }
}

/// 从用法文本（`/help <命令名>`）里取命令名；取不到合法命令名时返回 `None`。
pub fn name_from_usage(usage: &str) -> Option<String> {
    let token = usage.split_whitespace().next()?;
    let name = token.trim_start_matches(['/', '!']).to_lowercase();
    if name.is_empty() || token.starts_with(|c: char| c.is_alphanumeric()) {
        // 形如 `help <参数>`（漏了 `/`）或纯文本说明，都不是可靠的命令名
        return None;
    }
    Some(name)
}
