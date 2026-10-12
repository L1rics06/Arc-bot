//! 旧版本（v0.1）数据的迁移。
//!
//! 旧格式是"每条 issue 一个 JSON 文件、文件名为标题"，字段为
//! `id`（uuid）/ `name` / `description` / `time`。迁移时重新分配编号，
//! 原目录**保留不删**，并在新目录写 `.migrated_from_legacy` 标记，只执行一次。

use arc_core::JsonStore;

use super::id::{fmt_id, parse_stored_id};
use crate::error::{Error, Result};
use crate::models::{Issue, Status};

/// 旧版本存储目录（相对 CWD）。
pub const LEGACY_DIR: &str = "legacy-data";

/// 迁移完成标记文件名。
pub(super) const MIGRATION_MARKER: &str = ".migrated_from_legacy";

/// 把旧目录下的 JSON 文件迁移进 store。
///
/// 返回迁移条数；旧目录不存在返回 `Ok(None)`。
pub async fn migrate_from_legacy(
    json: &JsonStore,
    legacy: &std::path::Path,
) -> Result<Option<usize>> {
    if !tokio::fs::try_exists(legacy).await.unwrap_or(false) {
        return Ok(None);
    }

    let mut existing: Vec<u32> = json
        .list_names()
        .await?
        .into_iter()
        .filter_map(|n| parse_stored_id(&n).ok())
        .collect();
    let mut next = existing.iter().copied().max().unwrap_or(0) + 1;

    let mut dir = tokio::fs::read_dir(legacy)
        .await
        .map_err(|e| Error::Storage(format!("读取旧目录 {} 失败：{e}", legacy.display())))?;

    let mut count = 0;
    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| Error::Storage(format!("遍历旧目录失败：{e}")))?
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let bytes = match tokio::fs::read(&path).await {
            Ok(bytes) => bytes,
            Err(e) => {
                log::warn!("跳过无法读取的旧文件 {}：{e}", path.display());
                continue;
            }
        };
        let mut issue: Issue = match serde_json::from_slice::<LegacyIssue>(&bytes) {
            Ok(legacy) => legacy.into_issue(),
            Err(e) => {
                log::warn!("跳过无法解析的旧文件 {}：{e}", path.display());
                continue;
            }
        };

        // 旧数据没有编号，或者编号与已存在的冲突时重新分配
        let old_id: Option<u32> = parse_stored_id(issue.id.trim_start_matches('#')).ok();
        let id = match old_id {
            Some(id) if id > 0 && !existing.contains(&id) => id,
            _ => {
                let id = next;
                next += 1;
                id
            }
        };
        existing.push(id);
        issue.id = fmt_id(id);

        if let Err(e) = json.save(&issue.id, &issue).await {
            log::warn!("迁移 {} 失败：{e}", path.display());
            continue;
        }
        count += 1;
    }
    Ok(Some(count))
}

/// 旧版本（v0.1）的数据格式：`name` 字段 + 文件名为标题。
#[derive(Debug, serde::Deserialize)]
struct LegacyIssue {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "chrono::Utc::now")]
    time: chrono::DateTime<chrono::Utc>,
}

impl LegacyIssue {
    fn into_issue(self) -> Issue {
        Issue {
            id: String::new(), // 由迁移逻辑分配
            title: if self.name.trim().is_empty() {
                "未命名".to_owned()
            } else {
                self.name
            },
            description: self.description,
            status: Status::Open,
            created_at: self.time,
            updated_at: None,
        }
    }
}
