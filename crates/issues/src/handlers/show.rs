//! `/show`：查看某条待办的详情。

use std::sync::Arc;

use arc_core::command::{CommandSpec, RegistryBuilder};
use arc_core::{BoxedError, SharedState};

use super::common::{Context, require_id};
use super::render;
use crate::error::Error;
use crate::local_text;

/// 注册 `/show`。
pub fn register<C: Send + Sync + 'static>(builder: &mut RegistryBuilder<C>, ctx: Arc<Context<C>>) {
    builder.command(
        "show",
        CommandSpec::new("查看某条待办的详情", local_text::usage::SHOW),
        move |_state: SharedState<C>, args| {
            let ctx = ctx.clone();
            async move { handle(&ctx, &args).await }
        },
    );
}

/// 处理一条 `/show`。
async fn handle<C: Send + Sync + 'static>(
    ctx: &Context<C>,
    args: &arc_core::Args<C>,
) -> Result<String, BoxedError> {
    let id = require_id(args)?;
    let store = ctx.store().await.map_err(Box::new)?;
    let issue = store
        .get(id)
        .await
        .map_err(Box::new)?
        .ok_or(Error::NotFound(id as usize))?;
    Ok(render::show(&issue))
}
