//! `/edit`：修改标题 / 描述。

use std::sync::Arc;

use arc_core::command::{CommandSpec, RegistryBuilder};
use arc_core::{BoxedError, SharedState};

use super::common::{Context, require_id};
use crate::local_text;
use crate::models::NewIssue;

/// 注册 `/edit`。
pub fn register<C: Send + Sync + 'static>(builder: &mut RegistryBuilder<C>, ctx: Arc<Context<C>>) {
    builder.command(
        "edit",
        CommandSpec::new("修改标题 / 描述", local_text::usage::EDIT)
            .details(local_text::edit::details()),
        move |_state: SharedState<C>, args| {
            let ctx = ctx.clone();
            async move { handle(&ctx, &args).await }
        },
    );
}

/// 处理一条 `/edit`。
async fn handle<C: Send + Sync + 'static>(
    ctx: &Context<C>,
    args: &arc_core::Args<C>,
) -> Result<String, BoxedError> {
    let id = require_id(args)?;
    // 第一个位置参数是编号，其余（或 `--` 之后）作为新内容
    let rest = args.positionals()[1..].join(" ");
    let new = NewIssue::parse(&rest)?;

    let store = ctx.store().await.map_err(Box::new)?;
    let issue = store
        .update(id, |issue| {
            issue.title = new.title;
            issue.description = new.description;
            Ok(())
        })
        .await
        .map_err(Box::new)?;
    Ok(local_text::edit::updated(&issue.id, &issue.title))
}
