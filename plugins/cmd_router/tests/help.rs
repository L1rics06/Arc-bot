//! `/help` 的渲染测试（纯函数），以及它在注册表里的自举行为。

use arc_core::CommandInfo;
use cmd_router::builtins::help::{render_all, render_one};

fn info(name: &str) -> CommandInfo {
    CommandInfo {
        name: name.to_owned(),
        aliases: vec![],
        description: "测试命令".into(),
        usage: format!("/{name} <参数>"),
        details: None,
        admin_only: false,
    }
}

#[test]
fn render_all_lists_every_command() {
    let text = render_all(&[info("map"), info("list")]);
    assert!(text.contains("/map <参数>"), "{text}");
    assert!(text.contains("/list <参数>"), "{text}");
    assert!(text.contains("命令一览"), "{text}");
}

#[test]
fn render_all_marks_admin_only() {
    let mut secret = info("secret");
    secret.admin_only = true;
    let text = render_all(&[secret]);
    assert!(text.contains("[管理]"), "{text}");
}

#[test]
fn render_one_contains_fields() {
    let mut cmd = info("secret");
    cmd.admin_only = true;
    cmd.details = Some("管理专用".into());
    cmd.aliases = vec!["s".into()];

    let text = render_one(&cmd);
    assert!(text.contains("命令：secret"), "{text}");
    assert!(text.contains("别名：s"), "{text}");
    assert!(text.contains("用法：/secret <参数>"), "{text}");
    assert!(text.contains("仅管理员"), "{text}");
    assert!(text.contains("管理专用"), "{text}");
}
