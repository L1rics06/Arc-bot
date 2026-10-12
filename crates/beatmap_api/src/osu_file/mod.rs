//! `.osu` 谱面文件：下载与解析。
//!
//! 这是"相似谱面"管线的第一步：拿到本地 `.osu` 文本后才能做后续的难度分析。
//! 目前只解析 `[Difficulty]` 段（AR/OD/CS/HP），其余段落到需要时再扩展。

mod download;
mod parse;

pub use download::{OSU_FILE_URL, download_osu_file, fetch_difficulty};
pub use parse::parse_difficulty;
