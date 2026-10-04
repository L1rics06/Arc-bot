use rosu_v2::model::beatmap::RankStatus;
/// 我们只对api返回的部分info感兴趣
/// 所以我感觉在这里留下做一层抽象还是有必要的
/// 我还在考虑这里是选用自己的结构体，还是直接调用它的结构体
#[derive(Debug, Clone)]
pub struct BeatmapInfo {
    map_id: u32,
    set_id: u32,
    artist: String,
    creator: String,
    status: RankStatus,
}

#[derive(Debug, Default)]
pub struct DiffStatics {
    ar: f32,
    od: f32,
    cs: f32,
    hp: f32,
}
