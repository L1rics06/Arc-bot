//! 下载 `.osu` 文件到本地缓存。

use arc_core::JsonStore;

use super::parse::parse_difficulty;
use crate::error::{Error, Result};
use crate::local_text;
use crate::models::DifficultyStats;

/// 下载地址模板：`https://osu.ppy.sh/osu/<beatmapID>`。
pub const OSU_FILE_URL: &str = "https://osu.ppy.sh/osu";

/// 下载 `.osu` 文件并写入缓存目录（`<dir>/osu_files/<beatmap_id>.osu`），返回本地路径。
///
/// 已缓存时直接复用，不再请求网络。
pub async fn download_osu_file(
    client: &reqwest::Client,
    store: &JsonStore,
    beatmap_id: u32,
) -> Result<std::path::PathBuf> {
    let path = store
        .dir()
        .join("osu_files")
        .join(format!("{beatmap_id}.osu"));

    if tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Ok(path);
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| Error::OsuFile(local_text::osu_file::download_failed(&e.to_string())))?;
    }

    let url = format!("{OSU_FILE_URL}/{beatmap_id}");
    let response =
        client.get(&url).send().await.map_err(|e| {
            Error::OsuFile(local_text::osu_file::request_failed(&url, &e.to_string()))
        })?;

    let status = response.status();
    if !status.is_success() {
        return Err(Error::OsuFile(local_text::osu_file::bad_status(
            &url,
            &status.to_string(),
        )));
    }

    let bytes = response.bytes().await.map_err(|e| {
        Error::OsuFile(local_text::osu_file::read_body_failed(&url, &e.to_string()))
    })?;

    tokio::fs::write(&path, &bytes).await.map_err(|e| {
        Error::OsuFile(local_text::osu_file::write_failed(
            &path.display().to_string(),
            &e.to_string(),
        ))
    })?;
    log::info!("已下载 .osu 文件：{}", path.display());
    Ok(path)
}

/// 下载并解析：返回该谱面的四维难度。
pub async fn fetch_difficulty(
    client: &reqwest::Client,
    store: &JsonStore,
    beatmap_id: u32,
) -> Result<DifficultyStats> {
    let path = download_osu_file(client, store, beatmap_id).await?;
    let text = tokio::fs::read_to_string(&path).await.map_err(|e| {
        Error::OsuFile(local_text::osu_file::read_failed(
            &path.display().to_string(),
            &e.to_string(),
        ))
    })?;
    parse_difficulty(&text)
}
