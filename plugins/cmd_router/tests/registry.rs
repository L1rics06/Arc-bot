//! `cmd_router::registry` 的测试：分发、别名、权限与错误包装。

use std::sync::Arc;

use arc_core::{AppMeta, AppState, BoxedError, SharedState};
use cmd_router::{CommandSpec, Registry, RegistryBuilder};

fn state(admins: Vec<i64>) -> SharedState<()> {
    Arc::new(AppState::new(AppMeta {
        name: "test".into(),
        admins,
        data_root: std::env::temp_dir(),
    }))
}

fn registry() -> Registry<()> {
    let mut builder = RegistryBuilder::new();
    builder.command(
        "hi",
        CommandSpec::new("打招呼", "/hi").aliases(["hello"]),
        |_state, _args| async move { Ok("hi!") },
    );
    builder.command(
        "secret",
        CommandSpec::new("管理命令", "/secret").admin_only(),
        |_state, _args| async move { Ok("ok") },
    );
    Registry::from_builder(builder)
}

#[tokio::test]
async fn dispatch_known_command() {
    let result = registry().dispatch(state(vec![1]), "/hi", 42).await;
    let reply = result.expect("应识别为命令").expect("应成功");
    assert_eq!(reply.to_string(), "hi!");
}

#[tokio::test]
async fn dispatch_alias() {
    let result = registry().dispatch(state(vec![1]), "/hello", 42).await;
    assert!(result.expect("应识别为命令").is_ok());
}

#[tokio::test]
async fn unknown_command_is_silent() {
    let registry = registry();
    assert!(
        registry
            .dispatch(state(vec![1]), "/nope", 42)
            .await
            .is_none()
    );
    assert!(
        registry
            .dispatch(state(vec![1]), "随便聊聊", 42)
            .await
            .is_none()
    );
    assert!(registry.dispatch(state(vec![1]), "", 42).await.is_none());
}

#[tokio::test]
async fn admin_only_is_enforced() {
    let registry = registry();
    let denied = match registry
        .dispatch(state(vec![1]), "/secret", 99)
        .await
        .unwrap()
    {
        Err(msg) => msg,
        Ok(reply) => panic!("非管理员应被拒绝，却得到回复：{reply}"),
    };
    assert!(denied.contains("仅管理员"), "{denied}");

    let allowed = registry
        .dispatch(state(vec![1]), "/secret", 1)
        .await
        .unwrap();
    assert!(allowed.is_ok());
}

#[tokio::test]
async fn handler_error_carries_usage() {
    let mut builder = RegistryBuilder::new();
    builder.command(
        "boom",
        CommandSpec::new("总是失败", "/boom <x>"),
        |_state, _args| async move { Err::<String, BoxedError>("内部爆炸".into()) },
    );
    let registry = Registry::from_builder(builder);

    let err = match registry
        .dispatch(state(vec![]), "/boom 1", 1)
        .await
        .unwrap()
    {
        Err(msg) => msg,
        Ok(reply) => panic!("应返回错误，却得到回复：{reply}"),
    };
    assert!(err.contains("内部爆炸"), "错误信息应保留原因: {err}");
    assert!(err.contains("/boom <x>"), "错误信息应带用法提示: {err}");
}

#[test]
fn visible_commands_excludes_hidden() {
    let mut builder = RegistryBuilder::<()>::new();
    builder.command(
        "shown",
        CommandSpec::new("可见", "/shown"),
        |_s, _a| async move { Ok("1") },
    );
    builder.command(
        "hidden",
        CommandSpec::new("隐藏", "/hidden").hidden(),
        |_s, _a| async move { Ok("2") },
    );
    let registry = Registry::from_builder(builder);

    let names: Vec<_> = registry
        .visible_commands()
        .map(|c| c.name.clone())
        .collect();
    assert_eq!(names, vec!["shown".to_owned()]);
    assert!(registry.find("hidden").is_some(), "隐藏命令仍可分发");
}

#[test]
fn duplicate_name_keeps_last() {
    let mut builder = RegistryBuilder::<()>::new();
    builder.command(
        "dup",
        CommandSpec::new("第一版", "/dup"),
        |_s, _a| async move { Ok("1") },
    );
    builder.command(
        "dup",
        CommandSpec::new("第二版", "/dup"),
        |_s, _a| async move { Ok("2") },
    );
    let registry = Registry::from_builder(builder);
    assert_eq!(registry.find("dup").unwrap().description, "第二版");
}

#[test]
fn lookup_accepts_slash_and_case() {
    use arc_core::CommandSource;
    let registry = registry();
    let source = &registry as &dyn CommandSource;
    assert!(
        source.lookup("/HELLO").is_some(),
        "别名查找应忽略大小写与斜杠"
    );
    assert!(source.lookup("nope").is_none());
}
