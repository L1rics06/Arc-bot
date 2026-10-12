//! 领域模型：一条待办。

use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::local_text;

/// 标题与描述的长度上限（防止把整段聊天记录塞进文件名 / 回复里）。
pub const MAX_TITLE_LEN: usize = 60;
pub const MAX_DESC_LEN: usize = 500;

/// 一条待办。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    /// 稳定标识（`#001` 形式，创建时分配，永不变更）。
    pub id: String,
    /// 标题（一句话）。
    pub title: String,
    /// 详细描述，可为空。
    #[serde(default)]
    pub description: String,
    /// 状态。
    #[serde(default)]
    pub status: Status,
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
    /// 最近一次更新时间（UTC）；从未修改过则为 `None`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// issue 状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// 待处理。
    #[default]
    Open,
    /// 已完成。
    Done,
    /// 已搁置。
    Suspended,
}

impl Status {
    /// 从用户输入解析（接受常见中英文写法）。
    pub fn parse(input: &str) -> Result<Self, String> {
        match input.trim().to_lowercase().as_str() {
            "open" | "todo" | "待处理" | "待办" | "未完成" => Ok(Status::Open),
            "done" | "finished" | "完成" | "已完成" => Ok(Status::Done),
            "suspended" | "hold" | "搁置" | "挂起" | "暂停" => Ok(Status::Suspended),
            _ => Err(local_text::status::unknown(input)),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Status::Open => "待处理",
            Status::Done => "已完成",
            Status::Suspended => "已搁置",
        };
        f.write_str(text)
    }
}

/// 新建 issue 的输入。
#[derive(Debug, Clone)]
pub struct NewIssue {
    pub title: String,
    pub description: String,
}

impl NewIssue {
    /// 从命令文本解析：`标题，描述`（中英文逗号都可；没有逗号时整段作为标题）。
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();
        if input.is_empty() {
            return Err(local_text::add::USAGE.to_owned());
        }

        let (title, description) = match input.split_once(['，', ',']) {
            Some((title, desc)) => (title.trim(), desc.trim()),
            None => (input, ""),
        };

        if title.is_empty() {
            return Err(local_text::add::EMPTY_TITLE.to_owned());
        }
        if title.chars().count() > MAX_TITLE_LEN {
            return Err(local_text::add::title_too_long(MAX_TITLE_LEN));
        }
        if description.chars().count() > MAX_DESC_LEN {
            return Err(local_text::add::desc_too_long(MAX_DESC_LEN));
        }

        Ok(Self {
            title: title.to_owned(),
            description: description.to_owned(),
        })
    }
}
