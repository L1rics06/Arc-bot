//! 命令处理器：一条命令一个文件。
//!
//! 每个 handler 文件暴露一个 `register(builder, ctx)`，由 [`register`] 统一调用。
//! 公共部分（共享上下文、渲染、参数解析）在 [`common`]，命令文案在
//! [`crate::local_text`]。

pub mod add;
pub mod common;
pub mod delete;
pub mod edit;
pub mod list;
pub mod render;
pub mod show;
pub mod status;

use arc_core::SharedState;
use arc_core::command::RegistryBuilder;
use std::sync::Arc;

pub use common::{Context, MODULE_DIR};

/// 把 issues 的全部命令注册进路由表。
///
/// 新增命令时：在 `handlers/` 下新建文件，然后在这里加一行。
pub fn register<C: Send + Sync + 'static>(builder: &mut RegistryBuilder<C>, state: SharedState<C>) {
    let ctx = Arc::new(Context::new(state));

    add::register(builder, ctx.clone());
    list::register(builder, ctx.clone());
    show::register(builder, ctx.clone());
    edit::register(builder, ctx.clone());
    delete::register(builder, ctx.clone());
    status::register(builder, ctx.clone());
    status::register_done(builder, ctx.clone());
    status::register_reopen(builder, ctx);
}
