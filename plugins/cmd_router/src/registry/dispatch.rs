//! 分发：把一条消息交给命中的命令执行。

use arc_core::{ListMode, Reply, SharedState, extract_command};

use super::index::Registry;
use crate::text;

/// 分发一条消息。
///
/// - `None`：消息不是已知命令（调用方应静默，避免抢聊）；
/// - `Some(Ok)`：回复内容；
/// - `Some(Err)`：面向用户的使用错误（已带上下文与用法提示）。
pub(super) async fn dispatch<C: Send + Sync + 'static>(
    registry: &Registry<C>,
    state: SharedState<C>,
    raw_text: &str,
    user_id: i64,
) -> Option<Result<Box<dyn Reply>, String>> {
    let (name, rest) = extract_command(raw_text)?;
    let command = registry.find(&name)?;

    if command.admin_only && !state.meta().is_admin(user_id) {
        return Some(Err(text::dispatch::admin_only(&name, &command.usage)));
    }

    // `extract_command` 已经把命令词剥掉，所以参数解析不再跳过首 token
    let args = arc_core::Args::parse_with_command(rest, ListMode::Include, state);

    Some(
        command.handler.clone()(args.app().clone(), args)
            .await
            .map_err(|e| {
                let reason = e.to_string();
                if reason.trim().is_empty() {
                    text::dispatch::failed_without_reason(&command.name, &command.usage)
                } else {
                    text::dispatch::failed(&name, &command.usage, &reason)
                }
            }),
    )
}
