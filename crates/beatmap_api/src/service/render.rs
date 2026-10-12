//! `/map` 的文本渲染（纯函数，便于单测）。

use std::fmt::Write as _;

use crate::local_text;
use crate::models::BeatmapInfo;

/// 渲染谱面信息文本。
pub fn render(info: &BeatmapInfo) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", local_text::render::SET_HEADER);
    let _ = writeln!(
        out,
        " {}       : {} - {}",
        local_text::render::TITLE,
        info.artist,
        info.title
    );
    let _ = writeln!(
        out,
        " {}       : {}",
        local_text::render::CREATOR,
        info.creator
    );
    let _ = writeln!(
        out,
        " {}       : {}",
        local_text::render::STATUS,
        info.status
    );
    let _ = writeln!(out, "{}", local_text::render::DIFF_HEADER);
    let _ = writeln!(
        out,
        " {}     : {}",
        local_text::render::VERSION,
        info.version
    );
    let _ = writeln!(
        out,
        " {}       : {:.2}",
        local_text::render::STARS,
        info.stars
    );
    let _ = writeln!(out, " {}        : {}", local_text::render::BPM, info.bpm);
    let _ = write!(
        out,
        " {}: {}/{}/{}/{}",
        local_text::render::DIFFICULTY,
        info.difficulty.cs,
        info.difficulty.ar,
        info.difficulty.od,
        info.difficulty.hp
    );
    out
}
