//! 各命令 handler 共用的上下文与参数解析。

use std::sync::Arc;

use arc_core::{Args, SharedState};
use tokio::sync::OnceCell;

use crate::error::Error;
use crate::local_text;
use crate::storage::{Store, parse_user_id};

/// issues 模块在 `data/` 下的目录名。
pub const MODULE_DIR: &str = "issues";

/// issues 模块运行上下文：共享状态 + 惰性打开的存储。
///
/// 存储打开是异步的（要扫目录 / 迁移旧数据），放到首次命令调用时完成，
/// 避免拖慢机器人启动；同一进程内只打开一次。
pub struct Context<C> {
    state: SharedState<C>,
    store: OnceCell<Arc<Store>>,
}

impl<C: Send + Sync + 'static> Context<C> {
    /// 绑定共享状态。
    pub fn new(state: SharedState<C>) -> Self {
        Self {
            state,
            store: OnceCell::new(),
        }
    }

    /// 取存储；首次调用时创建目录并迁移旧数据。
    ///
    /// 返回 `Arc` 而非引用：handler 里需要把它移进 `async move`。
    pub async fn store(&self) -> Result<Arc<Store>, Error> {
        let dir = self.state.module_dir(MODULE_DIR);
        self.store
            .get_or_try_init(|| async { Store::open(&dir).await.map(Arc::new) })
            .await
            .cloned()
    }

    /// 共享状态。
    pub fn state(&self) -> &SharedState<C> {
        &self.state
    }
}

/// 解析第一个位置参数为编号（`3` / `#3` / `#003`）。
pub(crate) fn require_id<C>(args: &Args<C>) -> Result<u32, Error> {
    let raw = args
        .positional(0)
        .ok_or_else(|| Error::Invalid(local_text::hint::NEED_ID.to_owned()))?;
    parse_user_id(raw)
}
