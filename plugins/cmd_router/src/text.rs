//! cmd_router 的文案集中处。
//!
//! 约定：会出现在 QQ 消息里的固定字符串写在这里，其余模块只引用常量 / 函数。

/// `/help` 列表的排版与提示。
pub mod help {
    /// 列表标题。
    pub const TITLE: &str = "Arc-bot 命令一览（/help <命令名> 看详细用法）：";
    /// 列表末尾的提示。
    pub const FOOTER: &str = "提示：命令必须写在消息开头，例如 `/map 123`。";
    /// 管理员专属命令的前缀。
    pub const ADMIN_PREFIX: &str = "[管理] ";
    /// 单命令帮助里的字段名。
    pub const NAME: &str = "命令：";
    pub const ALIASES: &str = "别名：";
    pub const DESCRIPTION: &str = "说明：";
    pub const USAGE: &str = "用法：";
    pub const ADMIN_ONLY: &str = "权限：仅管理员";
    /// 找不到指定命令。
    pub fn not_found(name: &str) -> String {
        format!("没有找到命令 `{name}`，用 /help 查看全部命令")
    }
}

/// 路由分发的错误文案。
pub mod dispatch {
    /// 权限不足。
    pub fn admin_only(name: &str, usage: &str) -> String {
        format!("`/{name}` 仅管理员可用（用法：{usage}）")
    }
    /// handler 报错（非空原因）。
    pub fn failed(name: &str, usage: &str, reason: &str) -> String {
        format!("{name}/{usage}: {reason}")
    }
    /// handler 报错但没有原因。
    pub fn failed_without_reason(name: &str, usage: &str) -> String {
        format!("{name} 执行失败（用法：{usage}）")
    }
}

/// 装配期的内部错误（用户几乎看不到）。
pub mod internal {
    /// 命令表尚未挂载到共享状态。
    pub const NO_COMMAND_TABLE: &str = "命令表尚未初始化（这属于程序内部错误）";
    /// 重复注册命令。
    pub fn duplicate(key: &str, first: &str, second: &str) -> String {
        format!("命令 `{key}` 被重复注册（先 {first} 后 {second}），后者生效")
    }
}
