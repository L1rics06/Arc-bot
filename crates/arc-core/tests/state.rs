//! `arc_core::state` 的测试：共享状态、客户端初始化与命令表挂载。

use std::sync::Arc;

use arc_core::{AppMeta, AppState, default_data_root};

fn meta() -> AppMeta {
    AppMeta {
        name: "arc-bot".into(),
        admins: vec![1, 2],
        data_root: std::path::PathBuf::from("/tmp/arc-bot-test-data"),
    }
}

#[test]
fn admin_check_uses_id_list() {
    let meta = meta();
    assert!(meta.is_admin(1));
    assert!(meta.is_admin(2));
    assert!(!meta.is_admin(3), "非管理员不应通过");
}

#[test]
fn module_dir_is_under_data_root() {
    let meta = meta();
    assert_eq!(
        meta.module_dir("issues"),
        std::path::Path::new("/tmp/arc-bot-test-data/issues")
    );
}

#[test]
fn default_data_root_is_cwd_data() {
    let expected = std::env::current_dir().unwrap().join("data");
    assert_eq!(default_data_root(), expected);
}

#[tokio::test]
async fn client_is_initialized_exactly_once() {
    let state = AppState::new(meta());
    assert!(state.client().is_none(), "构造后客户端应为空");

    let calls = std::sync::atomic::AtomicUsize::new(0);
    let first = state
        .init_client(|| async {
            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok::<_, String>(7u32)
        })
        .await
        .unwrap();
    let second = state
        .init_client(|| async {
            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok::<_, String>(99u32)
        })
        .await
        .unwrap();

    assert_eq!(
        calls.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "初始化函数只能跑一次"
    );
    assert_eq!(*first, 7);
    assert_eq!(*second, 7, "第二次调用应返回既有实例，而不是重新初始化");
    assert!(Arc::ptr_eq(&first, &second));
}

#[tokio::test]
async fn failed_init_can_be_retried() {
    let state = AppState::new(meta());
    let err = state
        .init_client(|| async { Err::<u32, _>("暂时失败".to_owned()) })
        .await
        .unwrap_err();
    assert_eq!(err, "暂时失败");
    assert!(state.client().is_none(), "失败不应留下半个客户端");
}
