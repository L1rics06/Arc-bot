//! 消息路由：项目里**唯一**的 `on_msg` 监听入口。
//!
//! 收到消息后：
//! 1. 无文本消息（纯图片 / 纯 @ / 表情）直接忽略，不 panic；
//! 2. 不是已注册命令 → 静默（不抢聊，也避免与其它插件刷屏）；
//! 3. 命中命令表 → 执行 handler 并回复；出错回复带上下文的使用提示。

use std::sync::Arc;

use arc_core::SharedState;
use kovi::PluginBuilder as plugin;
use kovi_onebot::*;

use crate::registry::Registry;

/// 路由核心：单条消息的分发逻辑（与 kovi 解耦，便于测试）。
pub struct Runner<C> {
    registry: Arc<Registry<C>>,
    state: SharedState<C>,
}

impl<C: Send + Sync + 'static> Runner<C> {
    /// 构造路由。
    pub fn new(registry: Arc<Registry<C>>, state: SharedState<C>) -> Self {
        Self { registry, state }
    }

    /// 处理一条消息文本。返回 `None` 表示不响应（静默）。
    pub async fn handle(&self, text: &str, user_id: i64) -> Option<String> {
        self.registry
            .dispatch(self.state.clone(), text, user_id)
            .await
            .map(|result| match result {
                Ok(reply) => reply.to_string(),
                Err(msg) => msg,
            })
    }

    /// 注册监听：项目接入点，整个进程只允许调用一次。
    pub fn mount(registry: Arc<Registry<C>>, state: SharedState<C>) {
        let runner = Arc::new(Runner::new(registry, state));
        plugin::on(move |event: Arc<MsgEvent>| {
            let runner = runner.clone();
            async move {
                // 纯图片 / 纯 @ / 表情消息没有文本，直接忽略（旧实现在这里 unwrap panic）
                let Some(text) = event.borrow_text() else {
                    return;
                };
                let user_id = event.user_id;
                if let Some(reply) = runner.handle(text, user_id).await {
                    event.reply(reply);
                }
            }
        });
    }
}
