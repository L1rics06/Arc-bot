mod models;
use anyhow::Context;
use rosu_v2::Osu;

///检查 beatmapID 的合法性
async fn validate_bid(id: &str) -> anyhow::Result<u64> {
    let n: u64 = id
        .trim()
        .parse()
        .map_err(|e| anyhow::anyhow!("无法解析beatmapID：{}", e))?;
    if n == 0 {
        return Err(anyhow::anyhow!("无法解析beatmapID: ID需为非零正整数"));
    }
    Ok(n)
}

/// 下载.osu文件
pub async fn get_dotosu_file(beatmap_id: String) -> anyhow::Result<()> {
    let id = validate_bid(&beatmap_id).await?;

    let download_link = format!("https://osu.ppy.sh/osu/{}", id);

    Ok(())
}

/// 使用rosu封装的方法获取map的元数据
pub async fn get_map_info(beatmap_id: u32, rosu: Osu) -> anyhow::Result<String> {
    let map_extend = rosu
        .beatmap()
        .map_id(beatmap_id)
        .await
        .context("使用rosu查找铺面时发生错误")?;

    let map_set = map_extend.mapset;

    todo!()
}

#[kovi::plugin]
async fn main() {}
