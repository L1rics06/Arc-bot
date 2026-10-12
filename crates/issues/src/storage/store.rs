//! issue 存储的读写实现。

use std::sync::atomic::{AtomicU32, Ordering};

use arc_core::JsonStore;
use tokio::sync::Mutex;

use super::id::{MAX_ID, fmt_id, parse_stored_id};
use super::legacy::{LEGACY_DIR, MIGRATION_MARKER, migrate_from_legacy};
use crate::error::{Error, Result};
use crate::local_text;
use crate::models::{Issue, NewIssue, Status};

/// issue 存储。
pub struct Store {
    json: JsonStore,
    /// 旧版本数据目录（迁移来源，可为绝对路径）。
    legacy_dir: std::path::PathBuf,
    /// 下一个可用编号。`/add` 会把它渲染成 `#003`。
    next_id: AtomicU32,
    /// 串行化"分配编号 + 写盘"，避免并发 /add 抢到同一个编号。
    write_lock: Mutex<()>,
}

impl Store {
    /// 打开（必要时创建）目录，扫描已有编号，并在首次运行时迁移旧数据。
    pub async fn open(dir: impl Into<std::path::PathBuf>) -> Result<Self> {
        Self::open_with_legacy(dir, LEGACY_DIR).await
    }

    /// 同上，但显式指定旧数据目录（测试与自定义部署布局用）。
    pub async fn open_with_legacy(
        dir: impl Into<std::path::PathBuf>,
        legacy_dir: impl Into<std::path::PathBuf>,
    ) -> Result<Self> {
        let json = JsonStore::new(dir);
        json.ensure_dir().await?;

        let store = Self {
            json,
            legacy_dir: legacy_dir.into(),
            next_id: AtomicU32::new(1),
            write_lock: Mutex::new(()),
        };
        store.migrate_legacy().await;
        store.reload_next_id().await?;
        Ok(store)
    }

    /// 按编号读取。
    pub async fn get(&self, id: u32) -> Result<Option<Issue>> {
        Ok(self.json.get(&fmt_id(id)).await?)
    }

    /// 全部 issue，按编号升序。
    pub async fn list(&self) -> Result<Vec<Issue>> {
        let mut issues: Vec<Issue> = self.json.list().await?;
        issues.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(issues)
    }

    /// 新增 issue，返回创建结果（带分配到的编号）。
    pub async fn create(&self, new: NewIssue) -> Result<Issue> {
        let _guard = self.write_lock.lock().await;

        let id = self.take_id()?;
        let issue = Issue {
            id: fmt_id(id),
            title: new.title,
            description: new.description,
            status: Status::Open,
            created_at: chrono::Utc::now(),
            updated_at: None,
        };
        self.json.save(&issue.id, &issue).await?;
        Ok(issue)
    }

    /// 修改一条 issue（`/edit`、`/status` 共用），返回修改后的结果。
    ///
    /// `f` 返回 `Err` 时不落盘。
    pub async fn update<F>(&self, id: u32, f: F) -> Result<Issue>
    where
        F: FnOnce(&mut Issue) -> Result<()>,
    {
        let _guard = self.write_lock.lock().await;

        let key = fmt_id(id);
        let mut issue = self
            .json
            .get::<Issue>(&key)
            .await?
            .ok_or(Error::NotFound(id as usize))?;

        f(&mut issue)?;
        issue.updated_at = Some(chrono::Utc::now());
        self.json.save(&key, &issue).await?;
        Ok(issue)
    }

    /// 删除，返回被删掉的 issue。
    pub async fn delete(&self, id: u32) -> Result<Issue> {
        let _guard = self.write_lock.lock().await;

        let key = fmt_id(id);
        let issue = self
            .json
            .get::<Issue>(&key)
            .await?
            .ok_or(Error::NotFound(id as usize))?;
        self.json.remove(&key).await?;
        Ok(issue)
    }

    /// 存储目录。
    pub fn dir(&self) -> &std::path::Path {
        self.json.dir()
    }

    /// 分配下一个编号。
    fn take_id(&self) -> Result<u32> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        if id > MAX_ID {
            return Err(Error::Invalid(local_text::hint::id_exhausted(MAX_ID)));
        }
        Ok(id)
    }

    /// 扫描目录确定下一个可用编号（取"最大编号 + 1"，不复用已删除编号）。
    async fn reload_next_id(&self) -> Result<()> {
        let mut max = 0u32;
        for name in self.json.list_names().await? {
            if let Ok(id) = parse_stored_id(&name) {
                max = max.max(id);
            }
        }
        self.next_id.store(max + 1, Ordering::SeqCst);
        Ok(())
    }

    /// 首次运行时把旧版本数据迁移进来（失败只记 warn，不影响启动）。
    async fn migrate_legacy(&self) {
        let marker = self.json.dir().join(MIGRATION_MARKER);
        if tokio::fs::try_exists(&marker).await.unwrap_or(false) {
            return;
        }

        match migrate_from_legacy(&self.json, &self.legacy_dir).await {
            Ok(Some(count)) if count > 0 => log::info!("已从旧目录迁移 {count} 条 issue"),
            Ok(_) => {}
            Err(e) => log::warn!("旧 issue 数据迁移失败（不影响启动）：{e}"),
        }

        if let Err(e) = tokio::fs::write(&marker, b"1\n").await {
            log::debug!("写入迁移标记失败：{e}");
        }
    }
}
