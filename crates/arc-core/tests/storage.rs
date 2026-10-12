//! `arc_core::storage` 的测试：JSON 文件存储的读写、过滤与错误上报。

use arc_core::{Error, JsonStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Item {
    name: String,
    value: u32,
}

fn item(name: &str, value: u32) -> Item {
    Item {
        name: name.to_owned(),
        value,
    }
}

#[tokio::test]
async fn save_get_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonStore::new(dir.path());
    store.save("a", &item("a", 1)).await.unwrap();
    assert_eq!(store.get::<Item>("a").await.unwrap(), Some(item("a", 1)));
    assert_eq!(store.get::<Item>("missing").await.unwrap(), None);
}

#[tokio::test]
async fn save_is_atomic_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonStore::new(dir.path());
    store.save("a", &item("a", 1)).await.unwrap();
    store.save("a", &item("a", 2)).await.unwrap();
    assert_eq!(store.get::<Item>("a").await.unwrap(), Some(item("a", 2)));

    let mut entries = tokio::fs::read_dir(dir.path()).await.unwrap();
    let mut names = Vec::new();
    while let Some(e) = entries.next_entry().await.unwrap() {
        names.push(e.file_name().to_string_lossy().into_owned());
    }
    assert_eq!(names, vec!["a.json".to_owned()], "不应残留临时文件");
}

#[tokio::test]
async fn list_filters_non_json_and_sorts() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonStore::new(dir.path());
    store.save("b", &item("b", 2)).await.unwrap();
    store.save("a", &item("a", 1)).await.unwrap();
    tokio::fs::write(dir.path().join("note.txt"), b"ignore me")
        .await
        .unwrap();

    let names = store.list_names().await.unwrap();
    assert_eq!(names, vec!["a".to_string(), "b".to_string()]);

    let items = store.list::<Item>().await.unwrap();
    assert_eq!(items, vec![item("a", 1), item("b", 2)]);
}

#[tokio::test]
async fn list_missing_dir_is_empty() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonStore::new(dir.path().join("nope"));
    assert!(store.list::<Item>().await.unwrap().is_empty());
}

#[tokio::test]
async fn remove_reports_whether_existed() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonStore::new(dir.path());
    store.save("a", &item("a", 1)).await.unwrap();
    assert!(store.remove("a").await.unwrap());
    assert!(!store.remove("a").await.unwrap());
}

#[tokio::test]
async fn invalid_json_is_reported_not_swallowed() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonStore::new(dir.path());
    tokio::fs::write(dir.path().join("broken.json"), b"{not json")
        .await
        .unwrap();
    let err = store.get::<Item>("broken").await.unwrap_err();
    match err {
        Error::Storage(msg) => assert!(msg.contains("broken"), "错误应带文件名：{msg}"),
        other => panic!("应是 Storage 错误，得到 {other:?}"),
    }
}

#[tokio::test]
async fn rejects_unsafe_names() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonStore::new(dir.path());

    for bad in ["../evil", "a/b", "a\\b", "..", "", "."] {
        let err = store
            .save(bad, &item("x", 1))
            .await
            .expect_err("非法名字应被拒绝");
        match err {
            Error::Storage(msg) => assert!(msg.contains("非法的存储名"), "{bad}: {msg}"),
            other => panic!("{bad} 应是 Storage 错误，得到 {other:?}"),
        }
    }
    assert!(store.get::<Item>("../evil").await.is_err());
    assert!(store.remove("../evil").await.is_err());
}
