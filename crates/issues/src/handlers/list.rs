//! `/list`：分页列出待办。

use std::sync::Arc;

use arc_core::command::{CommandSpec, RegistryBuilder};
use arc_core::{BoxedError, SharedState};

use super::common::Context;
use super::render;
use crate::error::Error;
use crate::local_text;

/// 注册 `/list`。
pub fn register<C: Send + Sync + 'static>(builder: &mut RegistryBuilder<C>, ctx: Arc<Context<C>>) {
    builder.command(
        "list",
        CommandSpec::new("列出待办（每页 10 条）", local_text::usage::LIST).aliases(["ls"]),
        move |_state: SharedState<C>, args| {
            let ctx = ctx.clone();
            async move { handle(&ctx, &args).await }
        },
    );
}

/// 处理一条 `/list`。
async fn handle<C: Send + Sync + 'static>(
    ctx: &Context<C>,
    args: &arc_core::Args<C>,
) -> Result<String, BoxedError> {
    let page = parse_page(args)?;
    let store = ctx.store().await.map_err(Box::new)?;
    let issues = store.list().await.map_err(Box::new)?;
    Ok(render::list(&issues, page))
}

/// 解析页码，缺省为第 1 页。
fn parse_page<C>(args: &arc_core::Args<C>) -> Result<usize, Error> {
    match args.positional(0) {
        Some(raw) => raw
            .parse::<usize>()
            .map_err(|_| Error::Invalid(local_text::hint::bad_page(raw))),
        None => Ok(1),
    }
}
