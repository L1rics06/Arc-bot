//! `arc_core::args` 的测试：命令名提取与参数解析。

use std::sync::Arc;

use arc_core::args::{Args, ListMode, extract_command};
use arc_core::{AppMeta, AppState, SharedState};

fn state() -> SharedState<()> {
    Arc::new(AppState::new(AppMeta {
        name: "test".into(),
        admins: vec![1],
        data_root: std::env::temp_dir(),
    }))
}

#[test]
fn extract_requires_leading_slash() {
    assert_eq!(
        extract_command("/map 123"),
        Some(("map".into(), "123".to_owned()))
    );
    assert_eq!(
        extract_command("请看 /add 这条消息"),
        None,
        "命令必须出现在消息开头"
    );
    assert_eq!(extract_command("/"), None);
    assert_eq!(extract_command("/add"), Some(("add".into(), String::new())));
    assert_eq!(extract_command("你好"), None);
}

#[test]
fn extract_is_case_insensitive_and_trims() {
    assert_eq!(
        extract_command("  /MAP   123  "),
        Some(("map".into(), "123".to_owned()))
    );
}

#[test]
fn extract_stops_at_non_command_char() {
    // `/map:123` —— 冒号不是命令字符，命令解析到 `map`，剩余 `:123` 由参数层处理
    assert_eq!(
        extract_command("/map:123"),
        Some(("map".into(), ":123".to_owned()))
    );
}

#[test]
fn parse_positionals_drop_command() {
    let args = Args::parse("map 123 extra", state());
    assert_eq!(
        args.positionals(),
        &["123".to_owned(), "extra".to_owned()],
        "位置参数不应包含命令词"
    );
    assert_eq!(args.positional(0), Some("123"));
    assert_eq!(args.positional(1), Some("extra"));
    assert_eq!(args.positional(2), None);
}

#[test]
fn parse_include_mode_keeps_every_token() {
    let args = Args::parse_with_command("123 extra", ListMode::Include, state());
    assert_eq!(
        args.positionals(),
        &["123".to_owned(), "extra".to_owned()],
        "路由层已剥掉命令词，参数层不应再丢第一个 token"
    );
}

#[test]
fn parse_flags() {
    let args = Args::parse("list --page 2 --all -n 5", state());
    assert_eq!(args.flag("page"), Some("2"));
    assert_eq!(args.flag("n"), Some("5"));
    assert!(args.has_flag("all"));
    assert!(!args.has_flag("nope"));
}

#[test]
fn parse_quoted_group() {
    let args = Args::parse(r#"add "修复 登录 页面""#, state());
    assert_eq!(args.positionals(), &["修复 登录 页面".to_owned()]);
}

#[test]
fn parse_rest_after_double_dash() {
    let args = Args::parse("edit #7 -- 新的描述 -带减号 也可以", state());
    assert_eq!(args.positionals(), &["#7".to_owned()]);
    assert_eq!(
        args.rest_or_all(),
        "新的描述 -带减号 也可以",
        "`--` 之后的内容应整体归入 rest，且不再被解析成 flag"
    );
    assert!(!args.has_flag("带减号"), "rest 段里的 `-x` 不应被当成 flag");
}

#[test]
fn rest_or_all_falls_back_to_positionals() {
    let args = Args::parse("edit #7 新描述", state());
    assert_eq!(args.rest_or_all(), "#7 新描述");
}

#[test]
fn long_flag_is_not_treated_as_separator() {
    // 回归：`--page` 前正好有一个空格，早期实现会把它误判成 `--` 分隔符
    let args = Args::parse("list --page 2", state());
    assert_eq!(args.positionals(), &[] as &[String]);
    assert_eq!(args.flag("page"), Some("2"));
}

#[test]
fn parse_flag_parse_typed() {
    let args = Args::parse("list -p 3", state());
    assert_eq!(args.flag_parse::<u32>("p").unwrap().unwrap(), 3);
    let bad = Args::parse("list -p xx", state());
    assert!(bad.flag_parse::<u32>("p").unwrap().is_err());
}

#[test]
fn parse_escapes() {
    let args = Args::parse(r#"add 标题\ 带空格 尾巴"#, state());
    assert_eq!(
        args.positionals(),
        &["标题 带空格".to_owned(), "尾巴".to_owned()]
    );
}
