//! 面向用户的文本渲染（纯函数，只读数据、返回 `String`）。
//!
//! 所有字面量都在 [`crate::local_text::render`]，这里只负责拼装与数值格式。

use chrono::{DateTime, Utc};
use osu_difficulty_runtime::ManiaGameMod;

use crate::local_text;
use crate::result::{CandidateRow, ManiaRowInfo, QueryOutcome, TargetInfo, TargetSource};

/// 渲染一次查询结果。
pub fn render(outcome: &QueryOutcome) -> String {
    let mut lines = vec![format!(
        "{}{}",
        local_text::render::TARGET_PREFIX,
        target_label(&outcome.target)
    )];

    if outcome.rows.is_empty() {
        lines.push(local_text::render::EMPTY.to_owned());
    } else {
        for (index, row) in outcome.rows.iter().enumerate() {
            lines.push(local_text::render::row(index + 1, &row_body(row)));
        }
    }

    lines.push(footer(outcome));
    lines.join("\n")
}

/// 目标行：`artist - title [version] · ★7.24 · https://…`。
fn target_label(target: &TargetInfo) -> String {
    let mut parts = vec![label(&target.artist, &target.title, &target.version)];
    if let Some(stars) = target.star_rating {
        parts.push(star(stars));
    }
    if let Some(mania) = &target.mania {
        parts.push(mania_summary(mania));
    }
    if let Some(url) = &target.url {
        parts.push(url.clone());
    }
    parts.join(local_text::render::SEPARATOR)
}

/// 一条候选：`artist - title [version] · 度量 · 链接`。
fn row_body(row: &CandidateRow) -> String {
    let mut parts = vec![label(&row.artist, &row.title, &row.version)];
    parts.push(metrics(row));
    if let Some(url) = &row.url {
        parts.push(url.clone());
    }
    parts.join(local_text::render::SEPARATOR)
}

/// 候选的度量部分：standard 用星数，mania 用键数 / 家族 / 分位，最后都带距离。
fn metrics(row: &CandidateRow) -> String {
    let mut parts = Vec::new();
    if let Some(stars) = row.star_rating {
        parts.push(star(stars));
    }
    if let Some(mania) = &row.mania {
        parts.push(mania_summary(mania));
        if mania.game_mod != ManiaGameMod::Nm {
            parts.push(mania.game_mod.as_str().to_owned());
        }
    }
    parts.push(distance(row.distance));
    parts.join(local_text::render::SEPARATOR)
}

/// mania 摘要：`4K RC/Stream [SV] · 分位 87%`。
fn mania_summary(info: &ManiaRowInfo) -> String {
    let mut body = format!(
        "{}{} {}",
        info.key_count,
        local_text::render::KEYS_SUFFIX,
        local_text::render::mania_style(info.family, info.pattern)
    );
    if let Some(category) = &info.category {
        body = format!("{body} {}", local_text::render::mania_category(category));
    }
    format!(
        "{body}{}{} {:.0}%",
        local_text::render::SEPARATOR,
        local_text::render::PERCENTILE,
        info.difficulty_percentile * 100.0
    )
}

/// 星数。
fn star(stars: f32) -> String {
    format!("{}{stars:.2}", local_text::render::STAR)
}

/// 距离（越小越相似）。
fn distance(value: f32) -> String {
    format!("{} {value:.3}", local_text::render::DISTANCE)
}

/// footer：条数、同谱面集说明、Mod 池、数据时间、现场分析说明。
fn footer(outcome: &QueryOutcome) -> String {
    let mut parts = vec![local_text::render::count(outcome.rows.len())];
    parts.push(if outcome.same_set_excluded {
        local_text::render::SAME_SET_EXCLUDED.to_owned()
    } else {
        local_text::render::SAME_SET_INCLUDED.to_owned()
    });
    if !outcome.candidate_mods.is_empty() {
        parts.push(local_text::render::mods(&outcome.candidate_mods));
    }
    if let Some(date) = outcome.data_cutoff_at.and_then(format_date) {
        parts.push(local_text::render::data_cutoff(&date));
    }
    if outcome.source == TargetSource::OnTheFly {
        parts.push(local_text::render::ON_THE_FLY.to_owned());
    }
    parts.join(local_text::render::SEPARATOR)
}

/// `artist - title [version]`。
fn label(artist: &str, title: &str, version: &str) -> String {
    format!("{artist} - {title} [{version}]")
}

/// Unix 秒 → `YYYY-MM-DD`；时间戳非法时返回 `None`。
fn format_date(seconds: i64) -> Option<String> {
    let date = DateTime::<Utc>::from_timestamp(seconds, 0)?;
    Some(date.format("%Y-%m-%d").to_string())
}
