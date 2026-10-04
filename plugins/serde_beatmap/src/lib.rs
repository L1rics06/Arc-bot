use anyhow::Context;
use rosu_v2::Osu;

///检查 beatmapID 的合法性
async fn validate_bid(id: &str) -> anyhow::Result<u32> {
    let n: u32 = id
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

pub async fn map_info(raw_text: &str, rosu: &Osu) -> anyhow::Result<String> {
    let beatmap_id = raw_text.replace("/map", "");
    let id = validate_bid(&beatmap_id.to_string()).await?;
    get_map_info(id, &rosu).await
}

/// 使用rosu封装的方法获取map的元数据
async fn get_map_info(beatmap_id: u32, rosu: &Osu) -> anyhow::Result<String> {
    let map_extend = rosu
        .beatmap()
        .map_id(beatmap_id)
        .await
        .context("使用rosu查找铺面时发生错误")?;

    //这里直接用unwrap是因为 我认为肯定存在set
    let map_set = map_extend.mapset.unwrap();

    Ok(format!(
        "
        == 谱面集信息 ===\n\
         标题     : {} - {}\n\
         作者     : {}\n\
         === 当前难度 ===\n\
         难度名   : {}\n\
         星数     : {:.2}\n\
         BPM      : {}\n\
         CS/AR/OD/HP: {}/{}/{}/{}",
        map_set.artist,
        map_set.title,
        map_set.creator_name,
        map_extend.version,
        map_extend.stars,
        map_extend.bpm,
        map_extend.cs,
        map_extend.ar,
        map_extend.od,
        map_extend.hp,
    ))
}

#[kovi::plugin]
async fn main() {}
