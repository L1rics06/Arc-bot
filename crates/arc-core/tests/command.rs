//! `arc_core::command` 的测试：命令描述与收集器。

use arc_core::command::{CommandSpec, RegistryBuilder, boxed, make, name_from_usage};
use arc_core::{AppMeta, AppState, SharedState};
use std::sync::Arc;

fn state() -> SharedState<()> {
    Arc::new(AppState::new(AppMeta {
        name: "test".into(),
        admins: vec![],
        data_root: std::env::temp_dir(),
    }))
}

#[test]
fn name_inferred_from_usage() {
    assert_eq!(name_from_usage("/help [命令名]").as_deref(), Some("help"));
    assert_eq!(name_from_usage("!map 123").as_deref(), Some("map"));
    assert_eq!(name_from_usage("纯文本说明"), None);
    assert_eq!(name_from_usage(""), None);
}

#[test]
fn make_infers_name_from_usage() {
    let handler =
        boxed(|_state: SharedState<()>, _args| async move { Ok::<_, arc_core::BoxedError>("ok") });
    let command = make(CommandSpec::new("显示帮助", "/help [命令名]"), handler);
    assert_eq!(command.name, "help", "空名称应从 usage 推导");
}

#[test]
fn builder_sets_name_and_collects() {
    let mut builder = RegistryBuilder::<()>::new();
    builder.command(
        "hi",
        CommandSpec::new("打招呼", "/hi").aliases(["hello"]),
        |_state, _args| async move { Ok("hi!") },
    );
    assert_eq!(builder.len(), 1);

    let commands = builder.into_commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].name, "hi");
    assert_eq!(commands[0].aliases, vec!["hello".to_owned()]);
}

#[tokio::test]
async fn handler_receives_parsed_args() {
    let mut builder = RegistryBuilder::<()>::new();
    builder.command(
        "echo",
        CommandSpec::new("回显第一个参数", "/echo <文本>"),
        |_state, args| async move {
            Ok::<_, arc_core::BoxedError>(args.positional(0).unwrap_or("<空>").to_owned())
        },
    );
    let command = &builder.into_commands()[0];

    let args =
        arc_core::Args::parse_with_command("hello world", arc_core::ListMode::Include, state());
    let reply = (command.handler)(args.app().clone(), args).await.unwrap();
    assert_eq!(reply.render(), "hello");
}
