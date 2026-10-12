//! 参数解析实现。
//!
//! 对外接口在父模块 [`super`]，这里只放实现与类型。

use std::collections::HashMap;
use std::sync::Arc;

use crate::event::Reply;
use crate::state::AppState;

/// 命令处理器：接收共享状态与解析后的参数，返回回复。
///
/// 返回 `Err` 时由路由层统一转换为给用户的错误提示。
pub type CommandFuture = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Box<dyn Reply>, BoxedError>> + Send + 'static>,
>;

/// 处理器返回的任意错误（要求 `Send + Sync + 'static`，便于跨 task 传递）。
pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// 一次性解析完成的命令参数。
///
/// 由路由层从消息文本解析后交给 handler；handler 是 `Fn`（可能被并发调用），
/// 因此 `Args` 通过 `&Args` 借用传入，不要求 `Clone`。
pub struct Args<C> {
    /// 命令名之后的全部 flag。
    flags: HashMap<String, Vec<String>>,
    /// 位置参数（是否含命令词取决于 [`ListMode`]）。
    positionals: Vec<String>,
    /// `--` 之后的原始文本。
    rest: String,
    /// 触发本次调用的完整原始消息文本。
    raw: String,
    /// 共享状态。
    app: Arc<AppState<C>>,
}

impl<C> Args<C> {
    /// 解析参数（把第一个 token 视为命令词并丢弃）。
    pub fn parse(raw: impl Into<String>, app: Arc<AppState<C>>) -> Self {
        Self::parse_with_command(raw, ListMode::Skip, app)
    }

    /// 解析参数，指定是否保留命令词在位置参数中。
    ///
    /// - 路由层已经在 `extract_command` 里剥掉命令词，用 [`ListMode::Include`]；
    /// - 直接解析整条消息（含命令词）时用 [`ListMode::Skip`]。
    pub fn parse_with_command(
        raw: impl Into<String>,
        mode: ListMode,
        app: Arc<AppState<C>>,
    ) -> Self {
        let raw = raw.into();

        let mut flags: HashMap<String, Vec<String>> = HashMap::new();
        let mut positionals = Vec::new();
        let mut rest: Vec<String> = Vec::new();
        let mut after_separator = false;

        let tokens = tokenize(&raw);
        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];

            if after_separator {
                rest.push(token.clone());
                i += 1;
                continue;
            }

            // `--` 之后的整段文本归入 rest（保持原样，不再解析 flag）
            if token == "--" {
                after_separator = true;
                i += 1;
                continue;
            }

            let mut consumed = 1;
            if i > 0 && token.starts_with('-') && token.len() > 1 {
                // `--name` / `-n`；后面跟不是 `-` 开头的 token 时作为它的值
                let name = token.trim_start_matches('-').to_owned();
                let value = match tokens.get(i + 1) {
                    Some(next) if !next.starts_with('-') => {
                        consumed = 2;
                        Some(next.clone())
                    }
                    _ => None,
                };
                let entry = flags.entry(name).or_default();
                if let Some(value) = value {
                    entry.push(value);
                }
            } else {
                positionals.push(token.clone());
            }
            i += consumed;
        }

        if matches!(mode, ListMode::Skip) && !positionals.is_empty() {
            positionals.remove(0);
        }

        let rest = if after_separator {
            rest.join(" ")
        } else {
            String::new()
        };

        Self {
            flags,
            positionals,
            rest,
            raw,
            app,
        }
    }

    /// 共享状态。
    pub fn app(&self) -> &Arc<AppState<C>> {
        &self.app
    }

    /// 触发调用的完整原始消息。
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// 第 `idx` 个位置参数。
    pub fn positional(&self, idx: usize) -> Option<&str> {
        self.positionals.get(idx).map(String::as_str)
    }

    /// 全部位置参数。
    pub fn positionals(&self) -> &[String] {
        &self.positionals
    }

    /// 位置参数拼成的整段文本以及 `--` 之后的内容。
    ///
    /// 若没有 `--` 分隔符，退化为全部位置参数拼接，方便"命令 + 一段自然语言"的用法。
    pub fn rest_or_all(&self) -> String {
        if self.rest.is_empty() && !self.positionals.is_empty() {
            self.positionals.join(" ")
        } else {
            self.rest.clone()
        }
    }

    /// 取选项的第一个值；`-n` 与 `--count` 视为同一选项。
    pub fn flag(&self, name: &str) -> Option<&str> {
        self.flags
            .get(name)
            .and_then(|values| values.first())
            .map(String::as_str)
    }

    /// 取选项的多个值。
    pub fn flag_all(&self, name: &str) -> &[String] {
        self.flags.get(name).map(Vec::as_slice).unwrap_or(&[])
    }

    /// 选项是否存在（开关）。
    pub fn has_flag(&self, name: &str) -> bool {
        self.flags.contains_key(name)
    }

    /// 取选项并解析为指定类型。
    pub fn flag_parse<T: std::str::FromStr>(&self, name: &str) -> Option<Result<T, String>> {
        self.flag(name).map(|v| {
            v.parse::<T>()
                .map_err(|_| format!("选项 --{name} 的值 `{v}` 无法解析"))
        })
    }
}

/// 位置参数是否包含命令词。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMode {
    /// 丢弃第一个 token（命令词）。
    Skip,
    /// 保留全部 token。
    Include,
}

/// 按 shell 风格切分参数：双引号内整段成组，反斜杠转义下一个字符。
pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => in_quote = !in_quote,
            c if c.is_whitespace() && !in_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// 从完整消息文本中提取命令名（**仅在消息开头匹配**）。
///
/// 规则：
/// - 消息以 `/` 或 `!` 开头，后跟字母 / 数字 / 中文；
/// - 命令名允许内部连字符（`/some-cmd`）；
/// - 返回 `(命令名, 剩余参数文本)`；不是命令时返回 `None`。
pub fn extract_command(text: &str) -> Option<(String, String)> {
    let text = text.trim();
    let mut chars = text.chars();
    match chars.next()? {
        '/' | '!' => {}
        _ => return None,
    }

    let body = &text[1..];
    let end = body
        .find(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_'))
        .unwrap_or(body.len());

    if end == 0 {
        return None;
    }
    let name = body[..end].to_lowercase();
    Some((name, body[end..].trim().to_owned()))
}
