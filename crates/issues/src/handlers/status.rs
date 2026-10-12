//! 状态类命令：`/status`、`/done`、`/reopen`。
//!
//! 三者共用 [`set_status`]，只有状态参数不同。

use std::sync::Arc;

use arc_core::command::{CommandSpec, RegistryBuilder};
use arc_core::{BoxedError, SharedState};

use super::common::{Context, require_id};
use crate::error::Error;
use crate::local_text;
use crate::models::Status;

/// 注册 `/status`。
pub fn register<C: Send + Sync + 'static>(builder: &mut RegistryBuilder<C>, ctx: Arc<Context<C>>) {
    builder.command(
        "status",
        CommandSpec::new("修改待办状态", local_text::usage::STATUS).details(format!(
            "快捷命令：/done <编号>、/reopen <编号>\n{}",
            local_text::hint::ID
        )),
        move |_state: SharedState<C>, args| {
            let ctx = ctx.clone();
            async move {
                let id = require_id(&args)?;
                let raw = args
                    .positional(1)
                    .ok_or_else(|| Error::Invalid(local_text::status::usage()))?;
                let status = Status::parse(raw).map_err(Error::Invalid)?;
                set_status(&ctx, id, status).await
            }
        },
    );
}

/// 注册 `/done`（快捷标记完成）。
pub fn register_done<C: Send + Sync + 'static>(
    builder: &mut RegistryBuilder<C>,
    ctx: Arc<Context<C>>,
) {
    builder.command(
        "done",
        CommandSpec::new("把待办标记为已完成", "/done <编号>"),
        move |_state: SharedState<C>, args| {
            let ctx = ctx.clone();
            async move {
                let id = require_id(&args)?;
                set_status(&ctx, id, Status::Done).await
            }
        },
    );
}

/// 注册 `/reopen`（快捷标记待处理）。
pub fn register_reopen<C: Send + Sync + 'static>(
    builder: &mut RegistryBuilder<C>,
    ctx: Arc<Context<C>>,
) {
    builder.command(
        "reopen",
        CommandSpec::new("把待办重新标记为待处理", "/reopen <编号>"),
        move |_state: SharedState<C>, args| {
            let ctx = ctx.clone();
            async move {
                let id = require_id(&args)?;
                set_status(&ctx, id, Status::Open).await
            }
        },
    );
}

/// 三个命令共用的状态修改。
async fn set_status<C: Send + Sync + 'static>(
    ctx: &Context<C>,
    id: u32,
    status: Status,
) -> Result<String, BoxedError> {
    let store = ctx.store().await.map_err(Box::new)?;
    let issue = store
        .update(id, |issue| {
            issue.status = status;
            Ok(())
        })
        .await
        .map_err(Box::new)?;
    Ok(local_text::status::changed(
        &issue.id,
        &issue.title,
        &issue.status.to_string(),
    ))
}
