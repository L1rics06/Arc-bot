use std::sync::Arc;

use kovi::PluginBuilder as plugin;
use kovi_onebot::*;

use issues::{add_issues, issues_help, list_issues};
use rosu_v2::Osu;
use serde_beatmap::map_info;

/// 这个函数用于拼装每一个插件的用法，要求每一个插件需要提供返回用法文本
async fn help_text() -> String {
    let mut help_text = String::from("Arc-bot 用法:\n");
    help_text += &issues_help().await;
    help_text
}

/// todo!: 实现消息类型的枚举
///从协议端收到消息之后给消息路由到对应的插件
async fn route(event: Arc<MsgEvent>, rosu: Arc<Osu>) {
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
    } else if message.contains("/map") {
        match map_info(message, &rosu).await {
            Ok(reply) => event.reply(reply),
            Err(e) => event.reply(format!("查询铺面时发生错误:{}", e)),
        }
    } else {
        event.reply("未知命令，输入/help查看用法");
    }
}

// 因为目前还没有实现bot状态的插件读取，比如rosu的身份读取，
// 所以这里临时重复初始化了一次rosu-v2
#[kovi::plugin]
async fn main() {
    let client_id: u64 = std::env::var("CLIENT_ID").unwrap().parse().unwrap();
    let client_secret = std::env::var("CLIENT_SECRET").unwrap();
    let osu = Arc::new(Osu::new(client_id, client_secret).await.unwrap());

    plugin::on(move |event| {
        let osu = Arc::clone(&osu);

        async move {
            route(event, osu).await;
        }
    });
}
