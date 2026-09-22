use std::{env, path::Path};

use chrono::DateTime;
use kovi::{
    chrono::{self, Utc},
    tokio::fs,
    utils::{load_json_data, save_json_data},
};
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

///
pub async fn issues_help() -> String {
    "Issues用法：1./add 增加issue \n2./list 展示当前所有issues".to_string()
}

/// 解析/add 命令传入的text，返回结构体
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

/// 读取所有的issues，并准备格式化文本
async fn format_issues(issues: Vec<Issue>) -> String {
    let mut text = String::new();
    let mut count = 0;

    for issue in issues {
        count += 1;
        text += &format!(
            "{}. {} 创建时间：{}\n",
            count, issue.description, issue.time
        );
    }

    text
}

/// /list 命令的框架，收到来自message的text，返回需要发送的格式化文本
pub async fn list_issues() -> anyhow::Result<String> {
    let entrys = read_issues(STORAGE_DIR.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("读取发生错误{}", e))?;

    Ok(format_issues(entrys).await)
}

/// /add 命令的框架，收到来自message的text，返回可能的错误
pub async fn add_issues(raw_text: &str) -> anyhow::Result<()> {
    let issue = parse_text(raw_text).await?;
    let path = Path::new(STORAGE_DIR).join(format!("{}.json", issue.name));
    let _ = save_json_data(&issue, path);

    Ok(())
}

#[kovi::plugin]
async fn main() {}
