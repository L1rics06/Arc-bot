use std::{env, path::Path, sync::Arc};

use chrono::DateTime;
use kovi::{
    PluginBuilder as plugin,
    chrono::{self, Utc},
    tokio::fs,
    utils::{load_json_data, save_json_data},
};
use kovi_onebot::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//存储 issues json文件的相对位置
const STORAGE_DIR: &str = "issues";

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
    id: String,
    name: String,
    description: String,
    time: DateTime<Utc>,
}

/// Todo: 实际上我想实现的是在解析函数调用之前就能被分类，所以可能要补充一个消息类型分类插件
async fn parse_text(text: &str) -> anyhow::Result<Issue> {
    let mut text = text.to_string();
    text = text.replace("/add", "");
    Ok(Issue {
        id: Uuid::new_v4().to_string(),
        name: text
            .split("，")
            .next()
            .ok_or_else(|| anyhow::anyhow!("text为或者无法切割"))?
            .to_string()
            .replace("，", ""),
        description: text,
        time: Utc::now(), //依赖系统时区
    })
}

/// 读取所有存储在文件中的issues
async fn read_issues(path: String) -> anyhow::Result<Vec<Issue>, Box<dyn std::error::Error>> {
    let mut outputs = Vec::new();

    //异步版与普通情况不ReadDir并未实现迭代器功能，只能手动用await
    let mut dir = fs::read_dir(path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let file = entry.path();
        let default_issue = Issue {
            id: "Default".to_string(),
            name: "".to_string(),
            description: "".to_string(),
            time: Utc::now(),
        };
        outputs.push(load_json_data(default_issue, file)?);
    }

    Ok(outputs)
}

/// 发送所有的issues
async fn list_issues(issues: Vec<Issue>, event: &Arc<MsgEvent>) {
    let mut text = String::new();
    let mut count = 0;

    for issue in issues {
        count += 1;
        text += &format!("{}. {}\n", count, issue.description);
    }

    event.reply(text);
}

/// 运行时
async fn run(event: Arc<MsgEvent>) -> anyhow::Result<()> {
    let raw_text = event.borrow_text();
    match raw_text {
        Some(text) => {
            if text.contains("/add") {
                let issue = parse_text(text).await?;
                let path = Path::new(STORAGE_DIR).join(format!("{}.json", issue.name));
                let _ = save_json_data(&issue, path);

                event.reply(format!("增加事件成功"));
                Ok(())
            } else if text.contains("/list") {
                let entrys = read_issues(STORAGE_DIR.to_string())
                    .await
                    .map_err(|e| anyhow::anyhow!("读取发生错误{}", e))?;

                list_issues(entrys, &event).await;
                Ok(())
            } else {
                Ok(())
            }
        }
        None => Err(anyhow::anyhow!("Fail to borrow text")),
    }
}

#[kovi::plugin]
async fn main() {
    plugin::on(run);
}
