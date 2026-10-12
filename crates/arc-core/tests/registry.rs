//! `arc_core::registry` 的测试：命令表的只读视图。

use std::sync::Arc;

use arc_core::command::{CommandSpec, RegistryBuilder};
use arc_core::{CommandInfo, CommandSource};

/// 一个最小的 [`CommandSource`] 实现，用于验证 trait 契约。
///
/// 真实实现（cmd_router）会过滤掉 `hidden` 命令；这里只模拟"可见 / 隐藏"两种结果。
struct FakeSource {
    visible: Vec<CommandInfo>,
    all: Vec<CommandInfo>,
}

fn info(name: &str, admin_only: bool) -> CommandInfo {
    CommandInfo {
        name: name.to_owned(),
        aliases: vec![],
        description: format!("{name} 的描述"),
        usage: format!("/{name}"),
        details: None,
        admin_only,
    }
}

impl CommandSource for FakeSource {
    fn visible(&self) -> Vec<CommandInfo> {
        self.visible.clone()
    }

    fn lookup(&self, name: &str) -> Option<CommandInfo> {
        let name = name.trim_start_matches(['/', '!']).to_lowercase();
        self.all.iter().find(|c| c.name == name).cloned()
    }
}

#[test]
fn source_can_be_mounted_on_state() {
    let state = Arc::new(arc_core::AppState::<()>::new(arc_core::AppMeta {
        name: "test".into(),
        admins: vec![],
        data_root: std::env::temp_dir(),
    }));

    assert!(state.commands().is_none(), "未挂载时应返回 None");

    let source: Arc<dyn CommandSource> = Arc::new(FakeSource {
        visible: vec![info("help", false)],
        all: vec![info("help", false), info("secret", true)],
    });
    state.set_commands(source);

    let mounted = state.commands().expect("挂载后应可读取");
    assert_eq!(mounted.visible().len(), 1, "可见列表只含未隐藏命令");
    assert!(mounted.lookup("/HELP").is_some(), "查找应忽略大小写与斜杠");
    assert!(mounted.lookup("secret").is_some(), "隐藏命令仍可被查找");
    assert!(mounted.lookup("nope").is_none());
}

#[test]
fn builder_commands_expose_metadata() {
    let mut builder = RegistryBuilder::<()>::new();
    builder.command(
        "map",
        CommandSpec::new("查询铺面", "/map <id>")
            .aliases(["m"])
            .details("细节"),
        |_s, _a| async move { Ok("ok") },
    );
    let commands = builder.into_commands();
    let cmd = &commands[0];
    assert_eq!(cmd.description, "查询铺面");
    assert_eq!(cmd.usage, "/map <id>");
    assert_eq!(cmd.details.as_deref(), Some("细节"));
    assert!(!cmd.admin_only && !cmd.hidden);
}
