//! `issues::storage` 的测试：编号分配、并发、更新 / 删除与旧数据迁移。

use issues::models::{NewIssue, Status};
use issues::{Error, Store};

fn new_issue(title: &str) -> NewIssue {
    NewIssue {
        title: title.to_owned(),
        description: String::new(),
    }
}

fn id_of(issue: &issues::Issue) -> u32 {
    issue.id.trim_start_matches('#').parse().unwrap()
}

/// 测试里统一用"不存在旧目录"的方式打开，避免扫到仓库里的真实数据。
async fn open(dir: &std::path::Path) -> Store {
    Store::open_with_legacy(dir, dir.join("__no_legacy__"))
        .await
        .unwrap()
}

#[tokio::test]
async fn create_assigns_sequential_ids() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path()).await;
    assert_eq!(store.create(new_issue("a")).await.unwrap().id, "#001");
    assert_eq!(store.create(new_issue("b")).await.unwrap().id, "#002");
}

#[tokio::test]
async fn ids_continue_after_reopen() {
    let dir = tempfile::tempdir().unwrap();
    {
        let store = open(dir.path()).await;
        store.create(new_issue("a")).await.unwrap();
        store.create(new_issue("b")).await.unwrap();
    }
    let store = open(dir.path()).await;
    assert_eq!(
        store.create(new_issue("c")).await.unwrap().id,
        "#003",
        "重开后编号不应从 1 重新开始"
    );
}

#[tokio::test]
async fn deleted_ids_are_not_reused() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path()).await;
    let a = store.create(new_issue("a")).await.unwrap();
    store.delete(id_of(&a)).await.unwrap();
    assert_eq!(store.create(new_issue("b")).await.unwrap().id, "#002");
}

#[tokio::test]
async fn list_is_ordered_by_id() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path()).await;
    store.create(new_issue("a")).await.unwrap();
    store.create(new_issue("b")).await.unwrap();
    store.create(new_issue("c")).await.unwrap();
    let ids: Vec<_> = store
        .list()
        .await
        .unwrap()
        .into_iter()
        .map(|i| i.id)
        .collect();
    assert_eq!(ids, vec!["#001", "#002", "#003"]);
}

#[tokio::test]
async fn update_and_delete_missing_is_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path()).await;
    assert!(matches!(
        store.delete(9).await.unwrap_err(),
        Error::NotFound(9)
    ));
    assert!(matches!(
        store.update(9, |_| Ok(())).await.unwrap_err(),
        Error::NotFound(9)
    ));
}

#[tokio::test]
async fn update_leaves_file_untouched_on_error() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path()).await;
    let a = store.create(new_issue("a")).await.unwrap();
    let err = store
        .update(id_of(&a), |_| Err(Error::Invalid("不改".into())))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    let stored = store.get(id_of(&a)).await.unwrap().unwrap();
    assert_eq!(stored.title, "a");
}

#[tokio::test]
async fn update_sets_status_and_timestamp() {
    let dir = tempfile::tempdir().unwrap();
    let store = open(dir.path()).await;
    let a = store.create(new_issue("a")).await.unwrap();
    let updated = store
        .update(id_of(&a), |issue| {
            issue.status = Status::Done;
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(updated.status, Status::Done);
    assert!(updated.updated_at.is_some(), "修改后应记录更新时间");
}

#[tokio::test]
async fn concurrent_creates_get_distinct_ids() {
    let dir = tempfile::tempdir().unwrap();
    let store = std::sync::Arc::new(open(dir.path()).await);

    let mut handles = Vec::new();
    for i in 0..8 {
        let store = store.clone();
        handles.push(tokio::spawn(async move {
            store.create(new_issue(&format!("i{i}"))).await.unwrap().id
        }));
    }
    let mut ids = Vec::new();
    for h in handles {
        ids.push(h.await.unwrap());
    }
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 8, "并发创建不应产生重复编号");
}

#[tokio::test]
async fn migrates_legacy_files_once() {
    let base = tempfile::tempdir().unwrap();
    let legacy = base.path().join("legacy");
    let new_dir = base.path().join("data/issues");
    tokio::fs::create_dir_all(&legacy).await.unwrap();

    // 旧格式：id 是 uuid、字段名是 name/time
    for (i, name) in ["重构 hub 界面", "修一下登录"].iter().enumerate() {
        let legacy_issue = serde_json::json!({
            "id": format!("uuid-{i}"),
            "name": name,
            "description": format!("{name} 的描述"),
            "time": "2026-01-01T00:00:00Z",
        });
        tokio::fs::write(
            legacy.join(format!("{name}.json")),
            serde_json::to_vec(&legacy_issue).unwrap(),
        )
        .await
        .unwrap();
    }

    let store = Store::open_with_legacy(&new_dir, &legacy).await.unwrap();
    let migrated = store.list().await.unwrap();
    assert_eq!(migrated.len(), 2, "旧数据应被迁移进来");
    assert_eq!(migrated[0].id, "#001");
    assert_eq!(migrated[1].id, "#002");
    assert!(migrated.iter().any(|i| i.title == "重构 hub 界面"));

    // 再打开一次不应重复迁移
    let reopened = Store::open_with_legacy(&new_dir, &legacy).await.unwrap();
    assert_eq!(reopened.list().await.unwrap().len(), 2);

    // 旧目录保留不删（避免误删用户数据）
    assert!(
        tokio::fs::try_exists(legacy.join("修一下登录.json"))
            .await
            .unwrap()
    );
}

/// 真机数据回归：仓库里 `legacy-data/` 的实际文件必须能被迁移解析。
#[tokio::test]
async fn real_legacy_data_is_parseable() {
    let legacy = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../legacy-data");
    if !tokio::fs::try_exists(&legacy).await.unwrap_or(false) {
        eprintln!("跳过：{} 不存在", legacy.display());
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let store = Store::open_with_legacy(dir.path(), &legacy).await.unwrap();
    let migrated = store.list().await.unwrap();

    assert!(
        !migrated.is_empty(),
        "真实旧数据应至少迁移出一条（检查 {} 的格式）",
        legacy.display()
    );
    assert!(
        migrated.iter().all(|i| !i.title.trim().is_empty()),
        "迁移后的标题不应为空"
    );
}
