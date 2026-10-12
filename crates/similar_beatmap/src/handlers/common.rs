//! 命令 handler 共用的上下文。

use arc_core::SharedState;
use beatmap_api::OsuService;

use crate::config;
use crate::dataset::Datasets;

/// 相似谱面模块的运行上下文：共享状态 + 惰性打开的数据集。
///
/// 数据集打开（反序列化索引 / 校验记录）放在首次命令调用时完成，
/// 避免拖慢机器人启动；同一进程内只打开一次。
pub struct Context {
    state: SharedState<OsuService>,
    datasets: Datasets,
}

impl Context {
    /// 绑定共享状态并准备数据集目录。
    pub fn new(state: SharedState<OsuService>) -> Self {
        let datasets = Datasets::new(state.module_dir(config::MODULE_DIR));
        Self { state, datasets }
    }

    /// 共享状态（用于取 `OsuService` 下载 `.osu`）。
    pub fn state(&self) -> &SharedState<OsuService> {
        &self.state
    }

    /// 数据集访问入口。
    pub fn datasets(&self) -> &Datasets {
        &self.datasets
    }
}
