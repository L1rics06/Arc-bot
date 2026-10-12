//! 命令表的索引与冻结状态。

use std::collections::HashMap;

use arc_core::command::Command;

/// 冻结后的命令注册表。
pub struct Registry<C> {
    pub(super) commands: Vec<Command<C>>,
    /// 命令名 / 别名 → commands 下标。
    pub(super) names: HashMap<String, usize>,
}

impl<C: Send + Sync + 'static> Clone for Registry<C> {
    fn clone(&self) -> Self {
        Self {
            commands: self.commands.clone(),
            names: self.names.clone(),
        }
    }
}

impl<C: Send + Sync + 'static> Registry<C> {
    /// 空注册表（用于测试装配）。
    pub fn empty() -> Self {
        Self {
            commands: Vec::new(),
            names: HashMap::new(),
        }
    }
}
