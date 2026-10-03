use std::sync::Arc;

use kovi::PluginBuilder as plugin;
use kovi_onebot::*;

use issues::{add_issues, issues_help, list_issues};

/// 这个函数用于拼装每一个插件的用法，要求每一个插件需要提供返回用法文本
async fn help_text() -> String {
    let mut help_text = String::from("Arc-bot 用法:\n");
    help_text += &issues_help().await;
    help_text
}

/// todo!: 实现消息类型的枚举
///从协议端收到消息之后给消息路由到对应的插件
async fn route(event: Arc<MsgEvent>) {
    let message = event.borrow_text().unwrap();

    if message.contains("/add") {
        match add_issues(message).await {
            Ok(()) => event.reply("Issue添加成功"),
            Err(e) => event.reply(format!("生成Issue时发生错误:{}", e)),
        }
    } else if message.contains("/list") {
        match list_issues().await {
            Ok(reply) => event.reply(reply),
            Err(e) => event.reply(format!("读取Issue时发生错误:{}", e)),
        }
    } else if message.contains("/help") {
        event.reply(help_text().await);
    } else {
        event.reply("未知命令，输入/help查看用法");
    }
}

#[kovi::plugin]
async fn main() {
    plugin::on(route);
}
