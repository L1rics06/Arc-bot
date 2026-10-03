use serde::{Deserialize, Serialize};

///
/// 我们只对api返回的部分info感兴趣
/// 所以我感觉在这里留下做一层抽象还是有必要的

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatmapInfo {
    artist: String,
    creator: String,
}
