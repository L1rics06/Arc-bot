//! issues 面向用户的文案集中处。
//!
//! 约定：所有会出现在 QQ 消息里的固定字符串都写在这里（命令回复、错误提示、
//! 用法说明），handler 与 store 只引用常量，不写裸字符串字面量。
//! 这样以后要改文案 / 加多语言时只动这一个文件。

/// 命令用法（出现在 `/help` 与错误提示中）。
pub mod usage {
    /// `/add` 的用法。
    pub const ADD: &str = "/add <标题>[，描述]";
    /// `/list` 的用法。
    pub const LIST: &str = "/list [页码]";
    /// `/show` 的用法。
    pub const SHOW: &str = "/show <编号>";
    /// `/edit` 的用法。
    pub const EDIT: &str = "/edit <编号> <新标题>[，新描述]";
    /// `/del` 的用法。
    pub const DEL: &str = "/del <编号>";
    /// `/status` 的用法。
    pub const STATUS: &str = "/status <编号> <open|done|suspended>";
}

/// 简短提示。
pub mod hint {
    /// 编号体系的说明，出现在多条命令的详细帮助里。
    pub const ID: &str = "issue 有稳定编号（如 #003），可用 /list 查看。";
    /// 缺编号时的提示。
    pub const NEED_ID: &str = "缺少编号（先用 /list 看编号，例如 /show #003）";
    /// 编号非法时的提示模板（接原始输入）。
    pub fn bad_id(raw: &str) -> String {
        format!("编号 `{raw}` 不是数字（例如 #003）")
    }
    /// 编号为 0。
    pub const ZERO_ID: &str = "编号从 1 开始";
    /// 页码非法。
    pub fn bad_page(raw: &str) -> String {
        format!("页码 `{raw}` 不是数字")
    }
    /// 编号达上限。
    pub fn id_exhausted(max: u32) -> String {
        format!("编号已达上限 {max}，请先清理已完成的 issue")
    }
    /// 存储名非法（内部错误，用户几乎不会看到）。
    pub fn bad_name(name: &str) -> String {
        format!("非法的存储名 `{name}`")
    }
}

/// `/add` 相关文案。
pub mod add {
    /// 使用说明。
    pub const USAGE: &str = "用法：/add <标题>[，描述]";
    /// 标题为空。
    pub const EMPTY_TITLE: &str = "标题不能为空，用法：/add <标题>[，描述]";
    /// 标题过长。
    pub fn title_too_long(max: usize) -> String {
        format!("标题过长（最多 {max} 字）")
    }
    /// 描述过长。
    pub fn desc_too_long(max: usize) -> String {
        format!("描述过长（最多 {max} 字）")
    }
    /// 详细帮助。
    pub fn details() -> String {
        format!(
            "示例：/add 修复登录，点了按钮没反应\n标题与描述用逗号分隔；只写标题也可以。{}",
            super::hint::ID
        )
    }
    /// 成功回执。
    pub fn created(id: &str, title: &str, total: usize) -> String {
        format!("已记录 {id}：{title}（当前共 {total} 条）")
    }
}

/// `/edit` 相关文案。
pub mod edit {
    /// 详细帮助。
    pub fn details() -> String {
        format!(
            "示例：/edit #003 改个标题，顺便补一句描述\n只写标题就是只改标题。{}",
            super::hint::ID
        )
    }
    /// 成功回执。
    pub fn updated(id: &str, title: &str) -> String {
        format!("已更新：{id} {title}")
    }
}

/// `/del` 相关文案。
pub mod del {
    /// 成功回执。
    pub fn deleted(id: &str, title: &str) -> String {
        format!("已删除：{id} {title}")
    }
}

/// `/status`、`/done`、`/reopen` 相关文案。
pub mod status {
    /// 缺少状态参数时的完整用法。
    pub fn usage() -> String {
        format!(
            "用法：{}\n可用状态：open（待处理）/ done（已完成）/ suspended（搁置）\n快捷：/done <编号>、/reopen <编号>",
            super::usage::STATUS
        )
    }
    /// 状态识别失败（由 [`crate::models::Status::parse`] 返回）。
    pub fn unknown(input: &str) -> String {
        format!("无法识别的状态 `{input}`，可用：open / done / suspended（待处理 / 已完成 / 搁置）")
    }
    /// 成功回执。
    pub fn changed(id: &str, title: &str, status: &str) -> String {
        format!("{id} {title} → {status}")
    }
}

/// `/list` 相关文案。
pub mod list {
    /// 没有任何 issue。
    pub const EMPTY: &str = "当前没有 issue，用 /add <标题>[，描述] 记一条。";
    /// 列表表头。
    pub fn header(
        total: usize,
        open: usize,
        done: usize,
        page: usize,
        total_pages: usize,
    ) -> String {
        format!(
            "Issues（共 {total} 条，待处理 {open} / 已完成 {done}）第 {page}/{total_pages} 页："
        )
    }
    /// 翻页提示。
    pub fn pager(total_pages: usize) -> String {
        format!("翻页：/list <页码>（共 {total_pages} 页）")
    }
}

/// `/show` 相关文案。
pub mod show {
    /// 字段名。
    pub const STATUS: &str = "状态：";
    pub const CREATED: &str = "创建：";
    pub const UPDATED: &str = "更新：";
    pub const DESC: &str = "描述：";
}

/// 错误信息。
pub mod error {
    /// 找不到指定编号。
    pub fn not_found(id: usize) -> String {
        format!("找不到编号 {id} 的 issue（先用 /list 看当前列表）")
    }
    /// 存储读写失败。
    pub fn storage(msg: &str) -> String {
        format!("存取失败：{msg}")
    }
}
