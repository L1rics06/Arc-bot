//! 面向用户的文本渲染。
//!
//! 只做格式化，不做 IO——便于单测。文案模板见 [`crate::local_text`]。

use std::fmt::Write as _;

use chrono::{DateTime, FixedOffset, Utc};

use crate::local_text;
use crate::models::Issue;

/// `/list` 每页条数。
pub const PAGE_SIZE: usize = 10;

/// 东八区，用于展示时间（机器人主要面向中文用户）。
fn beijing() -> FixedOffset {
    FixedOffset::east_opt(8 * 3600).expect("东八区偏移量常量合法")
}

fn fmt_time(t: &DateTime<Utc>) -> String {
    t.with_timezone(&beijing())
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

/// 渲染分页列表。
///
/// `page` 从 1 开始；越界时回落到边界页。
pub fn list(issues: &[Issue], page: usize) -> String {
    if issues.is_empty() {
        return local_text::list::EMPTY.to_owned();
    }

    let total_pages = issues.len().div_ceil(PAGE_SIZE);
    let page = page.clamp(1, total_pages);
    let start = (page - 1) * PAGE_SIZE;
    let end = (start + PAGE_SIZE).min(issues.len());

    let open = issues
        .iter()
        .filter(|i| i.status == crate::models::Status::Open)
        .count();
    let done = issues
        .iter()
        .filter(|i| i.status == crate::models::Status::Done)
        .count();

    let mut out = local_text::list::header(issues.len(), open, done, page, total_pages);
    out.push('\n');
    for issue in &issues[start..end] {
        let _ = writeln!(
            out,
            "{} [{}] {} {}",
            issue.id,
            issue.status,
            issue.title,
            first_line(&issue.description)
        );
    }
    if total_pages > 1 {
        out.push_str(&local_text::list::pager(total_pages));
        out.push('\n');
    }
    out
}

/// 渲染单条详情。
pub fn show(issue: &Issue) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{} {}", issue.id, issue.title);
    let _ = writeln!(out, "{}{}", local_text::show::STATUS, issue.status);
    let _ = writeln!(
        out,
        "{}{}",
        local_text::show::CREATED,
        fmt_time(&issue.created_at)
    );
    if let Some(updated_at) = issue.updated_at.filter(|t| *t > issue.created_at) {
        let _ = writeln!(
            out,
            "{}{}",
            local_text::show::UPDATED,
            fmt_time(&updated_at)
        );
    }
    if !issue.description.is_empty() {
        let _ = writeln!(out, "{}{}", local_text::show::DESC, issue.description);
    }
    out
}

/// 取描述的第一行、限长，用于列表展示。
fn first_line(text: &str) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        return String::new();
    }
    let mut shown: String = line.chars().take(24).collect();
    if line.chars().count() > 24 {
        shown.push('…');
    }
    format!("— {shown}")
}
