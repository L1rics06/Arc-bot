//! 命令注册与编排。

mod common;
mod render;
mod similar;

use std::sync::Arc;

use arc_core::{RegistryBuilder, SharedState};
use beatmap_api::OsuService;

pub use common::Context;
pub use render::render;

/// 把相似谱面模块的命令登记进命令表。
///
/// 由 `cmd_router` 装配时调用；命令本身不注册监听器（见 AGENTS.md）。
pub fn register(builder: &mut RegistryBuilder<OsuService>, state: SharedState<OsuService>) {
    let ctx = Arc::new(Context::new(state));
    similar::register(builder, ctx);
}
