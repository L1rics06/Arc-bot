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

pub async fn get_metadata(beatmap_id: String) -> anyhow::Result<()> {
    let id = validate_bid(&beatmap_id).await?;

    let download_link = format!("https://osu.ppy.sh/osu/{}", id);

    Ok(())
}

#[kovi::plugin]
async fn main() {}
