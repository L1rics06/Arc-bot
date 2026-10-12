//! 跨模块共享状态。
//!
//! [`AppState`] 在启动时构造一次，以 `Arc` 形式共享给所有命令处理器，
//! 解决"每个插件各自初始化一份外部客户端"的问题。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::OnceCell;

use crate::registry::CommandSource;

/// 机器人运行期元信息，从 kovi 全局 `Bot` 提取，与具体协议端解耦。
#[derive(Debug, Clone)]
pub struct AppMeta {
    /// 机器人名称（kovi 配置的 self_id 对应的昵称 / 或占位名）。
    pub name: String,
    /// 管理员用户 ID 列表（主管理员 + 副管理员）。
    pub admins: Vec<i64>,
    /// kovi 规范的数据根目录：`<cwd>/data`。
    pub data_root: PathBuf,
}

impl AppMeta {
    /// 判断某个用户是否为管理员。
    pub fn is_admin(&self, user_id: i64) -> bool {
        self.admins.contains(&user_id)
    }

    /// 数据根目录下某个业务模块的专属目录（如 `data/issues`）。
    pub fn module_dir(&self, module: &str) -> PathBuf {
        self.data_root.join(module)
    }
}

/// 全局共享的应用状态。
///
/// 泛型参数 `C` 是"共享外部客户端"的类型（如 `rosu_v2::Osu`）。
/// 这样 arc-core 不必依赖任何具体的外部 API 库，业务模块用
/// `Arc<AppState<Osu>>` 即可拿到同一个客户端实例。
pub struct AppState<C> {
    meta: AppMeta,
    client: OnceCell<Arc<C>>,
    commands: OnceCell<Arc<dyn CommandSource>>,
}

impl<C> AppState<C> {
    /// 构造状态。此时共享客户端尚未初始化（凭据可能缺失，稍后由启动流程填充）。
    pub fn new(meta: AppMeta) -> Self {
        Self {
            meta,
            client: OnceCell::new(),
            commands: OnceCell::new(),
        }
    }

    /// 运行期元信息。
    pub fn meta(&self) -> &AppMeta {
        &self.meta
    }

    /// 数据根目录下某业务模块的目录。
    pub fn module_dir(&self, module: &str) -> PathBuf {
        self.meta.module_dir(module)
    }

    /// 尝试初始化共享客户端；已初始化时直接返回既有实例。
    ///
    /// `init` 只在首次调用时执行，因此整个进程最多认证 / 建连一次。
    pub async fn init_client<F, Fut, E>(&self, init: F) -> Result<Arc<C>, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<C, E>>,
    {
        self.client
            .get_or_try_init(|| async { init().await.map(Arc::new) })
            .await
            .cloned()
    }

    /// 直接写入共享客户端（不执行初始化逻辑），主要给测试和启动装配用。
    pub fn set_client(&self, client: C) {
        let _ = self.client.set(Arc::new(client));
    }

    /// 读取共享客户端；未初始化时返回 `None`。
    pub fn client(&self) -> Option<&Arc<C>> {
        self.client.get()
    }

    /// 挂载命令表（由 cmd_router 装配时调用一次），供 `/help` 等命令自查。
    pub fn set_commands(&self, source: Arc<dyn CommandSource>) {
        let _ = self.commands.set(source);
    }

    /// 读取命令表；未挂载时返回 `None`。
    pub fn commands(&self) -> Option<&Arc<dyn CommandSource>> {
        self.commands.get()
    }
}

/// 常用的共享状态类型：`Arc<AppState<T>>`。
pub type SharedState<C> = Arc<AppState<C>>;

/// 存储层数据目录的默认命名常量。
pub const DATA_ROOT_DIR: &str = "data";

/// 计算 kovi 规范的数据根目录（`<cwd>/data`）。
///
/// 与 `kovi::utils::get_data_root_path` 保持一致；
/// 单独实现一份是为了让 arc-core 不依赖 kovi。
pub fn default_data_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    cwd.join(DATA_ROOT_DIR)
}
