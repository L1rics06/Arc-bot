//! 谱面元数据本地缓存。
//!
//! 目的：同一谱面被反复查询时不重复打 osu! API（API 有速率限制）。
//! 策略：每条缓存带写入时间，超过 TTL 视为过期，读取时回源并覆盖。
//! 键直接使用 `map_id`（纯数字），天然是安全文件名。

use arc_core::{Error as CoreError, JsonStore};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::models::BeatmapInfo;

/// 缓存条目格式版本：结构变化时 +1，旧版本条目会被忽略并重新拉取。
const SCHEMA_VERSION: u32 = 1;

/// 缓存有效期。
pub const DEFAULT_TTL: Duration = Duration::hours(24);

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    schema: u32,
    cached_at: DateTime<Utc>,
    info: BeatmapInfo,
}

/// 谱面元数据缓存。
#[derive(Debug, Clone)]
pub struct InfoCache {
    store: JsonStore,
    ttl: Duration,
}

impl InfoCache {
    /// 使用默认 TTL 构造。
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Self {
        Self::with_ttl(dir, DEFAULT_TTL)
    }

    /// 指定 TTL 构造。
    pub fn with_ttl(dir: impl Into<std::path::PathBuf>, ttl: Duration) -> Self {
        Self {
            store: JsonStore::new(dir),
            ttl,
        }
    }

    /// 读取未过期的缓存；不存在或已过期返回 `None`。
    pub async fn get(&self, map_id: u32) -> Result<Option<BeatmapInfo>> {
        let entry: Option<CacheEntry> = self
            .store
            .get(&map_id.to_string())
            .await
            .map_err(cache_err)?;

        let Some(entry) = entry else {
            return Ok(None);
        };
        if entry.schema != SCHEMA_VERSION {
            log::debug!("缓存 {map_id} 版本不匹配，忽略");
            return Ok(None);
        }
        if Utc::now() - entry.cached_at >= self.ttl {
            log::debug!("缓存 {map_id} 已过期，忽略");
            return Ok(None);
        }
        Ok(Some(entry.info))
    }

    /// 写入缓存。
    pub async fn put(&self, info: &BeatmapInfo) -> Result<()> {
        let entry = CacheEntry {
            schema: SCHEMA_VERSION,
            cached_at: Utc::now(),
            info: info.clone(),
        };
        self.store
            .save(&info.map_id.to_string(), &entry)
            .await
            .map_err(cache_err)
    }

    /// 存储目录（`.osu` 文件等附属数据也放在这里）。
    pub fn dir(&self) -> &std::path::Path {
        self.store.dir()
    }

    /// 供 `.osu` 文件下载复用的存储句柄。
    pub fn store(&self) -> &JsonStore {
        &self.store
    }
}

fn cache_err(e: CoreError) -> Error {
    Error::Cache(e.to_string())
}
