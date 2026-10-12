//! `issues::handlers::render` 的测试：列表分页、详情与时间展示。

use chrono::{DateTime, Utc};
use issues::models::{Issue, Status};

fn issue(id: &str, title: &str, status: Status) -> Issue {
    let t = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    Issue {
        id: id.to_owned(),
        title: title.to_owned(),
        description: "描述".to_owned(),
        status,
        created_at: t,
        updated_at: None,
    }
}

#[test]
fn empty_list_hints_add() {
    assert!(issues::handlers::render::list(&[], 1).contains("/add"));
}

#[test]
fn list_shows_status_and_counts() {
    let issues = vec![
        issue("#001", "a", Status::Open),
        issue("#002", "b", Status::Done),
    ];
    let text = issues::handlers::render::list(&issues, 1);
    assert!(text.contains("待处理 1"));
    assert!(text.contains("已完成 1"));
    assert!(text.contains("#001 [待处理] a"));
}

#[test]
fn list_paginates() {
    let items: Vec<_> = (1..=12)
        .map(|i| issue(&format!("#{i:03}"), &format!("title{i}"), Status::Open))
        .collect();
    let page1 = issues::handlers::render::list(&items, 1);
    let page2 = issues::handlers::render::list(&items, 2);
    assert!(page1.contains("#001"));
    assert!(!page1.contains("#011"));
    assert!(page2.contains("#011"));
    assert!(page2.contains("第 2/2 页"));
}

#[test]
fn out_of_range_page_falls_back() {
    let items = vec![issue("#001", "a", Status::Open)];
    let text = issues::handlers::render::list(&items, 99);
    assert!(text.contains("第 1/1 页"));
}

#[test]
fn show_contains_fields() {
    let text = issues::handlers::render::show(&issue("#007", "标题", Status::Suspended));
    assert!(text.contains("#007 标题"));
    assert!(text.contains("已搁置"));
    assert!(text.contains("描述：描述"));
    assert!(
        text.contains("2026-01-01 08:00"),
        "时间应按东八区展示: {text}"
    );
}

#[test]
fn long_description_is_truncated_in_list() {
    let mut item = issue("#001", "标题", Status::Open);
    item.description = "字".repeat(100);
    let text = issues::handlers::render::list(&[item], 1);
    assert!(text.contains('…'), "超长描述应被截断: {text}");
}
