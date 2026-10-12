//! 参数与命令名解析。
//!
//! 支持两种写法：
//! - 位置参数：`/map 123`
//! - 选项：`-n 5` / `--count 5` / 开关 `--all`
//!
//! 解析规则：双引号 `"..."` 内整段作为一个参数；`--` 之后的内容整体归入 [`Args::rest`]。
//! [`Args::positionals`] 是否包含命令词由 [`ListMode`] 决定（路由层已经剥掉命令词，
//! 所以用 [`ListMode::Include`]）。

mod parse;

pub use parse::{Args, BoxedError, CommandFuture, ListMode, extract_command, tokenize};
