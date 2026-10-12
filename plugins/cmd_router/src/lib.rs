//! arc-bot 命令路由插件。
//!
//! ## 职责
//!
//! 1. 构造并共享 [`arc_core::AppState`]（数据根目录、管理员名单、osu! 客户端）；
//! 2. 收集各业务模块登记的命令，冻结成 [`Registry`]，再自举 `/help`；
//! 3. 注册唯一的 `on_msg` 监听（[`Runner`]），按命令表分发消息。
//!
//! ## 分层
//!
//! | 模块 | 内容 |
//! |---|---|
//! | [`registry`] | 命令表的索引、冻结与分发 |
//! | [`router`] | 唯一的消息监听入口（与 kovi 的接线） |
//! | [`builtins`] | 内置命令（`/help`） |
//! | [`text`] | 用户可见文案集中处 |
//!
//! ## 依赖方向
//!
//! `main → cmd_router → {issues, beatmap_api} → arc-core`
//!
//! 不要在本 crate 里写业务逻辑：新功能加到对应业务模块，
//! 然后在 [`main`] 的装配处加一行 `register`。

use std::sync::Arc;

use arc_core::{AppMeta, AppState, default_data_root};
use kovi::PluginBuilder as plugin;

pub mod builtins;
pub mod registry;
pub mod router;
pub mod text;

pub use arc_core::command::{Command, CommandSpec, RegistryBuilder};
pub use registry::Registry;
pub use router::Runner;

/// 本插件在 kovi 中的名字（与 `kovi.plugin.toml`、`build_bot!` 中的插件名一致）。
///
/// 直接取 crate 名，避免两处名字漂移；若改 `Cargo.toml` 的包名，
/// kovi 的 `build_bot!` 与 `kovi.plugin.toml` 里的引用也要同步修改。
pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

/// cmd_router 插件入口。
///
/// `#[kovi::plugin]` 要求入口是无参的 `async fn`，因此配置只能来自环境与 kovi 自身配置。
#[kovi::plugin]
async fn main() {
    // 1. 共享状态：数据根目录按 kovi 规范取 `<cwd>/data`
    let meta = AppMeta {
        name: PLUGIN_NAME.to_owned(),
        admins: admin_ids(),
        data_root: default_data_root(),
    };
    let state = Arc::new(AppState::<beatmap_api::OsuService>::new(meta));

    // 2. 各业务模块把命令注册进同一张收集器
    let mut builder = RegistryBuilder::new();
    issues::register(&mut builder, state.clone());
    beatmap_api::register(&mut builder);

    // 3. 共享 osu! 服务：整个进程只认证 / 初始化一次
    let cache_dir = state.module_dir("beatmap_api").join("cache");
    match state
        .init_client(|| async { beatmap_api::client::init_from_env(&cache_dir).await })
        .await
    {
        Ok(_) => log::info!("osu! API 客户端初始化完成，/map 可用"),
        Err(e) => log::warn!("osu! API 客户端初始化失败，/map 命令将不可用：{e:?}"),
    }

    // 4. 冻结命令表并挂到共享状态上，再追加需要读表的 /help
    let mut registry = Registry::from_builder(builder);
    state.set_commands(Arc::new(registry.clone()));
    builtins::register_all(&mut registry);

    // 5. 挂载唯一的路由监听
    Runner::mount(Arc::new(registry), state);
}

/// 取管理员 ID 列表（整数形式的 QQ 号）。
///
/// kovi 配置里的 ID 可能被写成字符串，这种 ID 无法参与消息级权限判定，
/// 这里跳过并告警，而不是 panic。
fn admin_ids() -> Vec<i64> {
    let bot = plugin::get_runtime_bot();
    let admins = bot.get_all_admin().unwrap_or_else(|e| {
        log::warn!("读取管理员列表失败，命令级权限判定将拒绝所有非管理员：{e}");
        Vec::new()
    });

    admins
        .into_iter()
        .filter_map(|id| match id.try_as_i64() {
            Some(id) => Some(id),
            None => {
                log::warn!("管理员 ID `{id:?}` 不是整数，已忽略");
                None
            }
        })
        .collect()
}
