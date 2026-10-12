//! `/add`：记一条待办。
//!
//! 参数：`<标题>[，描述]`（逗号分隔，只写标题也可以）。

use std::sync::Arc;

use arc_core::command::{CommandSpec, RegistryBuilder};
use arc_core::{BoxedError, SharedState};

use super::common::Context;
use crate::local_text;
use crate::models::NewIssue;

/// 注册 `/add`。
pub fn register<C: Send + Sync + 'static>(builder: &mut RegistryBuilder<C>, ctx: Arc<Context<C>>) {
    builder.command(
        "add",
        CommandSpec::new("记一条待办", local_text::usage::ADD)
            .aliases(["a"])
            .details(local_text::add::details()),
        move |_state: SharedState<C>, args| {
            let ctx = ctx.clone();
            async move { handle(&ctx, &args).await }
        },
    );
}

/// 处理一条 `/add`。
async fn handle<C: Send + Sync + 'static>(
    ctx: &Context<C>,
    args: &arc_core::Args<C>,
) -> Result<String, BoxedError> {
    let new = NewIssue::parse(&args.rest_or_all())?;
    let store = ctx.store().await.map_err(Box::new)?;

    let issue = store.create(new).await.map_err(Box::new)?;
    let total = store
        .list()
        .await
        .map(|list| list.len())
        .unwrap_or_default();
    Ok(local_text::add::created(&issue.id, &issue.title, total))
}
