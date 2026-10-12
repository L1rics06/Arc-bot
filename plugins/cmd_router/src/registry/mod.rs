//! 命令注册表：收集各业务模块登记的命令，冻结后按名字分发。

mod dispatch;
mod index;

pub use index::Registry;

use arc_core::command::{Command, RegistryBuilder};
use arc_core::{CommandInfo, CommandSource, Reply, SharedState};

use crate::text;

impl<C: Send + Sync + 'static> Registry<C> {
    /// 由收集器冻结：校验命令名冲突并建立索引。
    pub fn from_builder(builder: RegistryBuilder<C>) -> Self {
        let mut registry = Self::empty();
        for command in builder.into_commands() {
            registry.insert(command);
        }
        registry
    }

    /// 追加一条命令。
    ///
    /// 命令表冻结之后仍可登记，用于 `/help` 这类必须读全表的自举命令。
    pub fn insert(&mut self, command: Command<C>) {
        for key in std::iter::once(&command.name).chain(command.aliases.iter()) {
            if let Some(prev) = self.names.insert(key.clone(), self.commands.len()) {
                log::warn!(
                    "{}",
                    text::internal::duplicate(key, &self.commands[prev].name, &command.name)
                );
            }
        }
        self.commands.push(command);
    }

    /// 全部命令（含隐藏命令）。
    pub fn commands(&self) -> &[Command<C>] {
        &self.commands
    }

    /// 出现在 `/help` 列表中的命令。
    pub fn visible_commands(&self) -> impl Iterator<Item = &Command<C>> {
        self.commands.iter().filter(|c| !c.hidden)
    }

    /// 按名字或别名查找命令。
    pub fn find(&self, name: &str) -> Option<&Command<C>> {
        self.names.get(name).map(|&idx| &self.commands[idx])
    }

    /// 分发一条消息（见 [`dispatch`](self::dispatch::dispatch)）。
    ///
    /// - `None`：不是已知命令，调用方应静默；
    /// - `Some(Ok)`：回复内容；`Some(Err)`：面向用户的使用错误。
    pub async fn dispatch(
        &self,
        state: SharedState<C>,
        raw_text: &str,
        user_id: i64,
    ) -> Option<Result<Box<dyn Reply>, String>> {
        dispatch::dispatch(self, state, raw_text, user_id).await
    }
}

impl<C: Send + Sync + 'static> CommandSource for Registry<C> {
    fn visible(&self) -> Vec<CommandInfo> {
        self.visible_commands().map(to_info).collect()
    }

    fn lookup(&self, name: &str) -> Option<CommandInfo> {
        let name = name.trim_start_matches(['/', '!']).to_lowercase();
        self.find(&name).map(to_info)
    }
}

/// 命令 → help 用的元信息。
fn to_info<C>(c: &Command<C>) -> CommandInfo {
    CommandInfo {
        name: c.name.clone(),
        aliases: c.aliases.clone(),
        description: c.description.clone(),
        usage: c.usage.clone(),
        details: c.details.clone(),
        admin_only: c.admin_only,
    }
}
