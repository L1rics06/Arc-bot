//! osu! API 客户端与查询服务。
//!
//! 客户端是**进程级单例**：由 cmd_router 在启动时初始化一次，写进
//! [`AppState`](arc_core::AppState) 的 `client` 槽位，所有命令共享
//! （旧实现里 main 与路由插件各初始化一次，会重复认证并建两份连接池）。

use std::path::Path;
use std::time::Duration as StdDuration;

use arc_core::JsonStore;
use rosu_v2::Osu;
use rosu_v2::model::beatmap::BeatmapExtended;

use crate::cache::InfoCache;
use crate::error::{Error, Result};
use crate::models::{BeatmapInfo, DifficultyStats, MapStatus};

/// HTTP 请求超时（下载 `.osu` 文件用）。
const HTTP_TIMEOUT: StdDuration = StdDuration::from_secs(15);
/// 业务服务：持有 rosu 客户端、本地缓存与 HTTP 客户端。
///
/// 通过 `Arc<OsuService>` 在命令之间共享；构造失败不影响机器人启动，
/// 只是依赖 osu! API 的命令不可用。
pub struct OsuService {
    osu: Osu,
    cache: InfoCache,
    http: reqwest::Client,
}

impl OsuService {
    /// 从环境变量读取凭据并构造服务。
    ///
    /// 凭据缺失 / 格式不对 / 认证失败都返回 `Err`，由调用方决定降级方式。
    pub async fn from_env(cache_dir: &Path) -> Result<Self> {
        let client_id: u64 = std::env::var("CLIENT_ID")
            .map_err(|_| Error::Config("未设置环境变量 CLIENT_ID".to_owned()))?
            .trim()
            .parse()
            .map_err(|e| Error::Config(format!("CLIENT_ID 不是合法的数字：{e}")))?;

        let client_secret = std::env::var("CLIENT_SECRET")
            .map_err(|_| Error::Config("未设置环境变量 CLIENT_SECRET".to_owned()))?;

        let osu = Osu::new(client_id, client_secret)
            .await
            .map_err(|e| Error::Api(format!("认证失败，请检查 CLIENT_ID / CLIENT_SECRET：{e}")))?;

        let http = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .map_err(|e| Error::OsuFile(format!("构造 HTTP 客户端失败：{e}")))?;

        Ok(Self {
            osu,
            cache: InfoCache::new(cache_dir),
            http,
        })
    }

    /// 底层 rosu 客户端（需要调用本模块未封装的 API 时使用）。
    pub fn osu(&self) -> &Osu {
        &self.osu
    }

    /// 本地缓存。
    pub fn cache(&self) -> &InfoCache {
        &self.cache
    }

    /// 读取谱面信息：先查本地缓存，未命中再打 API 并回写缓存。
    pub async fn info(&self, map_id: u32) -> Result<BeatmapInfo> {
        if let Some(info) = self.cache.get(map_id).await? {
            log::debug!("谱面 {map_id} 命中本地缓存");
            return Ok(info);
        }

        let info = self.fetch_info(map_id).await?;
        if let Err(e) = self.cache.put(&info).await {
            // 缓存失败不影响本次查询结果，仅记录
            log::warn!("写入谱面缓存失败：{e}");
        }
        Ok(info)
    }

    /// 强制从 osu! API 拉取（不走缓存）。
    pub async fn fetch_info(&self, map_id: u32) -> Result<BeatmapInfo> {
        let map = self
            .osu
            .beatmap()
            .map_id(map_id)
            .await
            .map_err(|e| map_api_err(map_id, e))?;

        BeatmapInfo::try_from_rosu(&map)
    }

    /// 下载（或复用）该谱面的 `.osu` 文件，返回本地路径。
    pub async fn download_osu_file(&self, map_id: u32) -> Result<std::path::PathBuf> {
        let store = JsonStore::new(self.cache.dir());
        crate::osu_file::download_osu_file(&self.http, &store, map_id).await
    }

    /// 下载并解析该谱面的四维难度。
    pub async fn difficulty(&self, map_id: u32) -> Result<DifficultyStats> {
        let path = self.download_osu_file(map_id).await?;
        let text = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| Error::OsuFile(format!("读取 {} 失败：{e}", path.display())))?;
        crate::osu_file::parse_difficulty(&text)
    }
}

/// 从环境变量初始化共享的 osu! 服务。
///
/// 这是 [`AppState::init_client`](arc_core::AppState::init_client) 的初始化函数，
/// 由 cmd_router 在启动时调用**一次**；之后的命令都从共享状态里取。
pub async fn init_from_env(cache_dir: &Path) -> Result<OsuService> {
    OsuService::from_env(cache_dir).await
}

/// 供业务侧把服务包装成命令返回的 `Arc` 句柄。
pub type SharedOsu = std::sync::Arc<OsuService>;

impl BeatmapInfo {
    /// 从 rosu-v2 的响应映射到本项目的领域模型。
    ///
    /// 原实现直接 `mapset.unwrap()`：被删除 / graveyard 的谱面 `mapset` 是 `None`，
    /// 会把机器人打崩。这里改为返回明确错误。
    pub fn try_from_rosu(map: &BeatmapExtended) -> Result<Self> {
        let mapset = map
            .mapset
            .as_deref()
            .ok_or(Error::MissingMapset(map.map_id))?;

        Ok(Self {
            map_id: map.map_id,
            set_id: mapset.mapset_id,
            artist: mapset.artist.clone(),
            title: mapset.title.clone(),
            creator: mapset.creator_name.to_string(),
            version: map.version.clone(),
            stars: map.stars,
            bpm: map.bpm,
            status: MapStatus::from_rosu(map.status),
            difficulty: DifficultyStats {
                cs: map.cs,
                ar: map.ar,
                od: map.od,
                hp: map.hp,
            },
        })
    }
}

fn map_api_err(map_id: u32, e: rosu_v2::error::OsuError) -> Error {
    match e {
        rosu_v2::error::OsuError::NotFound => {
            Error::Invalid(format!("找不到 beatmapID `{map_id}`，请确认编号是否正确"))
        }
        other => Error::Api(other.to_string()),
    }
}
