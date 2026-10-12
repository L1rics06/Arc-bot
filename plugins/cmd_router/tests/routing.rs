//! 端到端路由测试：用真实装配（issues + beatmap_api + /help）验证命令分发。
//!
//! 不连 QQ、不打网络：只测路由层与命令处理器之间的接线。

use std::sync::Arc;

use arc_core::{AppMeta, AppState, SharedState};
use beatmap_api::OsuService;
use cmd_router::{Registry, RegistryBuilder, Runner};

fn build_runner(data_root: &std::path::Path) -> Runner<OsuService> {
    let state: SharedState<OsuService> = Arc::new(AppState::new(AppMeta {
        name: "arc-bot-test".into(),
        admins: vec![42],
        data_root: data_root.to_path_buf(),
    }));

    let mut builder = RegistryBuilder::new();
    issues::register(&mut builder, state.clone());
    beatmap_api::register(&mut builder);

    let mut registry = Registry::from_builder(builder);
    state.set_commands(Arc::new(registry.clone()));
    cmd_router::builtins::register_all(&mut registry);

    Runner::new(Arc::new(registry), state)
}

async fn reply(runner: &Runner<OsuService>, text: &str) -> Option<String> {
    runner.handle(text, 7).await
}

#[tokio::test]
async fn issue_lifecycle_over_the_router() {
    let dir = tempfile::tempdir().unwrap();
    let runner = build_runner(dir.path());

    // 普通聊天不该被机器人接管（静默）
    assert_eq!(reply(&runner, "今天中午吃什么").await, None);
    // 未注册的命令同样静默
    assert_eq!(reply(&runner, "/不存在的命令").await, None);

    // 创建
    let add = reply(&runner, "/add 修复登录，点了按钮没反应")
        .await
        .expect("add 应有回复");
    assert!(add.contains("#001"), "{add}");

    // 列表
    let list = reply(&runner, "/list").await.expect("list 应有回复");
    assert!(list.contains("修复登录"), "{list}");
    assert!(list.contains("待处理 1"), "{list}");

    // 修改状态
    let done = reply(&runner, "/done #001").await.expect("done 应有回复");
    assert!(done.contains("已完成"), "{done}");

    // 查看详情
    let show = reply(&runner, "/show 1").await.expect("show 应有回复");
    assert!(show.contains("状态：已完成"), "{show}");

    // 编辑
    let edit = reply(&runner, "/edit #001 修复登录页，补充了细节")
        .await
        .expect("edit 应有回复");
    assert!(edit.contains("修复登录页"), "{edit}");

    // 删除
    let del = reply(&runner, "/del #001").await.expect("del 应有回复");
    assert!(del.contains("已删除"), "{del}");
    let list = reply(&runner, "/list").await.unwrap();
    assert!(list.contains("当前没有 issue"), "{list}");
}

#[tokio::test]
async fn unknown_issue_id_reports_clearly() {
    let dir = tempfile::tempdir().unwrap();
    let runner = build_runner(dir.path());
    let msg = reply(&runner, "/show 99").await.expect("应回复错误提示");
    assert!(msg.contains("找不到编号 99"), "{msg}");
}

#[tokio::test]
async fn help_lists_all_registered_commands() {
    let dir = tempfile::tempdir().unwrap();
    let runner = build_runner(dir.path());
    let help = reply(&runner, "/help").await.expect("help 应有回复");
    for cmd in ["/map", "/add", "/list", "/show", "/edit", "/del", "/status"] {
        assert!(help.contains(cmd), "/help 应包含 {cmd}：{help}");
    }

    // 单命令帮助
    let one = reply(&runner, "/help map").await.unwrap();
    assert!(one.contains("用法：/map <beatmapID>"), "{one}");
}

#[tokio::test]
async fn missing_beatmap_id_is_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    let runner = build_runner(dir.path());
    // 未配置 osu 凭据时也应给出可操作的提示，而不是 panic
    let msg = reply(&runner, "/map").await.expect("应回复错误提示");
    assert!(msg.contains("beatmapID"), "{msg}");
}

#[tokio::test]
async fn aliases_route_to_the_same_handler() {
    let dir = tempfile::tempdir().unwrap();
    let runner = build_runner(dir.path());
    reply(&runner, "/a 用别名创建").await.expect("别名应有回复");
    let list = reply(&runner, "/ls").await.expect("别名应有回复");
    assert!(list.contains("用别名创建"), "{list}");
}
