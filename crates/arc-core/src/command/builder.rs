//! 命令收集器：业务模块通过它登记自己的命令。

use super::spec::{Command, CommandSpec, Handler};
use crate::args::BoxedError;
use crate::event::IntoReply;
use crate::state::SharedState;

/// 命令收集器。
///
/// 只负责"收集"，不负责分发；路由实现（cmd_router）把收集结果冻结成自己的注册表。
pub struct RegistryBuilder<C> {
    commands: Vec<Command<C>>,
}

impl<C: Send + Sync + 'static> Default for RegistryBuilder<C> {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}

impl<C: Send + Sync + 'static> RegistryBuilder<C> {
    /// 新建空收集器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前已登记的命令数量。
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// 取出全部命令（冻结时使用）。
    pub fn into_commands(self) -> Vec<Command<C>> {
        self.commands
    }

    /// 注册一条命令。
    ///
    /// handler 内可用 [`IntoReply`] 把 `String` / `&str` 直接变成回复。
    pub fn command<F, Fut, R>(
        &mut self,
        name: impl Into<String>,
        mut spec: CommandSpec,
        handler: F,
    ) -> &mut Self
    where
        F: Fn(SharedState<C>, crate::args::Args<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<R, BoxedError>> + Send + 'static,
        R: IntoReply + 'static,
    {
        spec.name = name.into();
        let handler: Handler<C> = super::spec::boxed(handler);
        self.commands.push(super::spec::make(spec, handler));
        self
    }
}
