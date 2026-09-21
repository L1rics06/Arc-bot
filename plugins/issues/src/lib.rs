use std::env;

use anyhow::Ok;
use chrono::DateTime;
use kovi::{
    PluginBuilder as plugin,
    chrono::{self, Utc},
    utils::save_json_data,
};
use kovi_onebot::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
    id: String,
    name: String,
    description: String,
    time: DateTime<Utc>,
}

// Todo: 实际上我想实现的是在解析函数调用之前就能被分类，所以可能要补充一个消息类型分类插件
async fn parse_text(text: &str) -> anyhow::Result<Issue> {
    let mut text = text.to_string();
    text = text.replace("/add", "");
    Ok(Issue {
        id: Uuid::new_v4().to_string(),
        name: text
            .split("，")
            .next()
            .ok_or_else(|| anyhow::anyhow!("text为或者无法切割"))?
            .to_string(),
        description: text,
        time: Utc::now(), //依赖系统时区
    })
}

#[kovi::plugin]
async fn main() {
    plugin::on_msg(|event| async move {
        let raw_text = event.borrow_text();
        match raw_text {
            Some(text) => {
                let issue = parse_text(text).await?;
                let path = env::current_dir()?.join(format!("{}.json", issue.name));
                let _ = save_json_data(&issue, path);
                event.reply(format!("增加事件成功"));
                Ok(())
            }
            None => Err(anyhow::anyhow!("Fail to borrow text")),
        }
    });
}
