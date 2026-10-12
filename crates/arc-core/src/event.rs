//! 事件与回复原语。
//!
//! 业务 handler 只依赖这里的 `dyn Reply`，不直接依赖任何协议端（OneBot）类型，
//! 这样回复方式以后可以扩展成图片 / 富文本 / Markdown 而不动业务代码。

use std::fmt::{Display, Formatter};

/// 命令处理器发送给用户的回复内容。
pub trait Reply: Send + Sync + 'static {
    /// 渲染为最终发给协议端的文本。
    fn render(&self) -> String;
}

impl Display for dyn Reply {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render())
    }
}

/// 纯文本回复。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text(pub String);

impl Text {
    /// 构造纯文本回复。
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// 装箱成 `Box<dyn Reply>`，供 handler 返回。
    pub fn boxed(s: impl Into<String>) -> Box<dyn Reply> {
        Box::new(Self(s.into()))
    }
}

impl Reply for Text {
    fn render(&self) -> String {
        self.0.clone()
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// 回复构造快捷方式。
pub mod reply {
    use super::{Reply, Text};

    /// 构造纯文本回复。
    pub fn text(s: impl Into<String>) -> Box<dyn Reply> {
        Text::boxed(s)
    }
}

/// 可转换为回复的类型集合。
///
/// 命令 handler 的返回类型统一写成 `Result<Box<dyn Reply>, E>`；
/// 若要直接返回 `String` / `&str`，用 [`IntoReply::into_reply`] 装箱即可。
pub trait IntoReply {
    fn into_reply(self) -> Box<dyn Reply>;
}

impl IntoReply for Text {
    fn into_reply(self) -> Box<dyn Reply> {
        Box::new(self)
    }
}

impl IntoReply for Box<dyn Reply> {
    fn into_reply(self) -> Box<dyn Reply> {
        self
    }
}

impl IntoReply for String {
    fn into_reply(self) -> Box<dyn Reply> {
        Box::new(Text(self))
    }
}

impl IntoReply for &str {
    fn into_reply(self) -> Box<dyn Reply> {
        Box::new(Text(self.to_owned()))
    }
}
