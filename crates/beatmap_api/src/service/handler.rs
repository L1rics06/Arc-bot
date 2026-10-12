//! `/map` 命令的注册与处理。
//!
//! 参数：`<beatmapID>`。

use arc_core::command::{CommandSpec, RegistryBuilder};
use arc_core::{BoxedError, SharedState};

use super::render;
use crate::client::OsuService;
use crate::local_text;
use crate::models::parse_beatmap_id;

/// 把 beatmap_api 的命令注册进路由表。
pub fn register(builder: &mut RegistryBuilder<OsuService>) {
    builder.command(
        "map",
        CommandSpec::new("查询 osu! 谱面信息", local_text::usage::MAP).aliases(["m", "谱面"]),
        move |state: SharedState<OsuService>, args| async move { handle(state, args).await },
    );
}

/// 处理一条 `/map`。
async fn handle(
    state: SharedState<OsuService>,
    args: arc_core::Args<OsuService>,
) -> Result<String, BoxedError> {
    let input = args.positional(0).ok_or_else(local_text::hint::need_id)?;
    let map_id = parse_beatmap_id(input)?;

    let service = require_service(&state)?;
    let info = service.info(map_id).await.map_err(Box::new)?;
    Ok(render::render(&info))
}

/// 从共享状态取出 osu! 服务；未初始化时给出可操作的提示。
fn require_service(state: &SharedState<OsuService>) -> Result<std::sync::Arc<OsuService>, String> {
    state
        .client()
        .cloned()
        .ok_or_else(local_text::hint::api_not_configured)
}
