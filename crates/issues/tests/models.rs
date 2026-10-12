//! `issues::models` 的测试：输入解析、长度校验与状态解析。

use issues::models::{MAX_TITLE_LEN, NewIssue, Status};

#[test]
fn parse_title_only() {
    let new = NewIssue::parse("修复登录").unwrap();
    assert_eq!(new.title, "修复登录");
    assert_eq!(new.description, "");
}

#[test]
fn parse_title_and_description() {
    let new = NewIssue::parse("修复登录，点击按钮没反应").unwrap();
    assert_eq!(new.title, "修复登录");
    assert_eq!(new.description, "点击按钮没反应");
}

#[test]
fn parse_accepts_ascii_comma() {
    let new = NewIssue::parse("t, d").unwrap();
    assert_eq!(new.description, "d");
}

#[test]
fn parse_rejects_empty_title() {
    assert!(NewIssue::parse("").is_err());
    assert!(NewIssue::parse("，只有描述").is_err());
}

#[test]
fn parse_rejects_too_long() {
    let long = "字".repeat(MAX_TITLE_LEN + 1);
    assert!(NewIssue::parse(&long).is_err());
}

#[test]
fn status_parse_aliases() {
    assert_eq!(Status::parse("done").unwrap(), Status::Done);
    assert_eq!(Status::parse("已完成").unwrap(), Status::Done);
    assert_eq!(Status::parse("待处理").unwrap(), Status::Open);
    assert_eq!(Status::parse("搁置").unwrap(), Status::Suspended);
    assert!(Status::parse("瞎写").is_err());
}

#[test]
fn status_display_is_chinese() {
    assert_eq!(Status::Open.to_string(), "待处理");
    assert_eq!(Status::Done.to_string(), "已完成");
    assert_eq!(Status::Suspended.to_string(), "已搁置");
}

#[test]
fn issue_serialization_is_backward_compatible() {
    // 旧数据只有 id/title/description/created_at，缺 status/updated_at 时应能读出来
    let json = r##"{
        "id": "#001",
        "title": "旧数据",
        "description": "",
        "created_at": "2026-01-01T00:00:00Z"
    }"##;
    let issue: issues::Issue = serde_json::from_str(json).unwrap();
    assert_eq!(issue.status, Status::Open);
    assert_eq!(issue.updated_at, None, "从未修改过的旧数据不应显示更新时间");
}
