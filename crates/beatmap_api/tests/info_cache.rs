//! `beatmap_api::cache` 的测试：TTL 过期与 schema 版本失效。

use arc_core::JsonStore;
use beatmap_api::cache::InfoCache;
use beatmap_api::models::{BeatmapInfo, DifficultyStats, MapStatus};
use chrono::{Duration, Utc};

fn sample(map_id: u32) -> BeatmapInfo {
    BeatmapInfo {
        map_id,
        set_id: 1,
        artist: "artist".into(),
        title: "title".into(),
        creator: "creator".into(),
        version: "Insane".into(),
        stars: 5.5,
        bpm: 180.0,
        status: MapStatus::Ranked,
        difficulty: DifficultyStats {
            cs: 4.0,
            ar: 9.0,
            od: 9.0,
            hp: 6.0,
        },
    }
}

#[tokio::test]
async fn put_then_get_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let cache = InfoCache::new(dir.path());
    cache.put(&sample(1)).await.unwrap();
    assert_eq!(cache.get(1).await.unwrap(), Some(sample(1)));
    assert_eq!(cache.get(2).await.unwrap(), None, "未缓存的谱面应返回 None");
}

#[tokio::test]
async fn expired_entry_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let cache = InfoCache::with_ttl(dir.path(), Duration::seconds(-1));
    cache.put(&sample(1)).await.unwrap();
    assert_eq!(cache.get(1).await.unwrap(), None, "过期条目应被忽略");
}

#[tokio::test]
async fn wrong_schema_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonStore::new(dir.path());
    // 手写一条"未来版本"的缓存
    let entry = serde_json::json!({
        "schema": 999u32,
        "cached_at": Utc::now(),
        "info": sample(1),
    });
    store.save("1", &entry).await.unwrap();

    let cache = InfoCache::new(dir.path());
    assert_eq!(cache.get(1).await.unwrap(), None, "版本不匹配应回源");
}

#[tokio::test]
async fn fresh_entry_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    {
        let cache = InfoCache::new(dir.path());
        cache.put(&sample(7)).await.unwrap();
    }
    let cache = InfoCache::new(dir.path());
    assert_eq!(cache.get(7).await.unwrap(), Some(sample(7)));
}
