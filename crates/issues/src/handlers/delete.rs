//! `/del`：删除一条待办。

use std::sync::Arc;

use arc_core::command::{CommandSpec, RegistryBuilder};
use arc_core::{BoxedError, SharedState};

use super::common::{Context, require_id};
use crate::local_text;

/// 注册 `/del`。
pub fn register<C: Send + Sync + 'static>(builder: &mut RegistryBuilder<C>, ctx: Arc<Context<C>>) {
    builder.command(
        "del",
        CommandSpec::new("删除一条待办", local_text::usage::DEL).aliases(["delete", "rm"]),
        move |_state: SharedState<C>, args| {
            let ctx = ctx.clone();
            async move { handle(&ctx, &args).await }
        },
    );
}

/// 处理一条 `/del`。
async fn handle<C: Send + Sync + 'static>(
    ctx: &Context<C>,
    args: &arc_core::Args<C>,
) -> Result<String, BoxedError> {
    let id = require_id(args)?;
    let store = ctx.store().await.map_err(Box::new)?;
    let issue = store.delete(id).await.map_err(Box::new)?;
    Ok(local_text::del::deleted(&issue.id, &issue.title))
}
