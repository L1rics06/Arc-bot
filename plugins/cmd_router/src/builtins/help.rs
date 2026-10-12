//! `/help` 命令：从注册表自动聚合帮助信息，不再手工拼接字符串。
//!
//! 新增命令只要在注册表里填好 `description` / `usage`，`/help` 会自动收录。
//!
//! `/help` 是一条"自举"命令：它必须读最终的命令表，而命令表的构建又需要先完成
//! 其它命令的注册，因此它通过 [`Registry::insert`] 在表冻结之后追加。

use arc_core::command::{CommandSpec, boxed};
use arc_core::event::reply;
use arc_core::{BoxedError, CommandInfo};

use super::super::registry::Registry;
use super::super::text;

/// 把 `/help` 追加进已冻结的命令表。
pub fn register<C: Send + Sync + 'static>(registry: &mut Registry<C>) {
    let handler = boxed(|state, args| async move {
        let result = (|| -> Result<String, BoxedError> {
            let source = state.commands().ok_or(text::internal::NO_COMMAND_TABLE)?;

            Ok(match args.positional(0) {
                Some(name) => match source.lookup(name) {
                    Some(info) => render_one(&info),
                    None => text::help::not_found(name),
                },
                None => render_all(&source.visible()),
            })
        })();
        result.map(reply::text)
    });

    registry.insert(arc_core::command::make(
        CommandSpec::new("显示全部命令或某条命令的详细用法", "/help [命令名]")
            .aliases(["h", "?", "帮助"]),
        handler,
    ));
}

/// 渲染 `/help` 总览。
pub fn render_all(commands: &[CommandInfo]) -> String {
    let mut out = String::from(text::help::TITLE);
    out.push('\n');
    for cmd in commands {
        let admin = if cmd.admin_only {
            text::help::ADMIN_PREFIX
        } else {
            ""
        };
        out.push_str(&format!("{admin}{:<24} {}\n", cmd.usage, cmd.description));
    }
    out.push_str(text::help::FOOTER);
    out
}

/// 渲染 `/help <命令名>`。
pub fn render_one(info: &CommandInfo) -> String {
    let mut out = format!("{}{}\n", text::help::NAME, info.name);
    if !info.aliases.is_empty() {
        out.push_str(&format!(
            "{}{}\n",
            text::help::ALIASES,
            info.aliases.join(", ")
        ));
    }
    out.push_str(&format!(
        "{}{}\n",
        text::help::DESCRIPTION,
        info.description
    ));
    out.push_str(&format!("{}{}\n", text::help::USAGE, info.usage));
    if info.admin_only {
        out.push_str(text::help::ADMIN_ONLY);
        out.push('\n');
    }
    if let Some(details) = &info.details {
        out.push('\n');
        out.push_str(details);
        out.push('\n');
    }
    out
}
