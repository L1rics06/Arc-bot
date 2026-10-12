//! `/similar`：相似谱面主命令。
//!
//! 流程：解析参数 → 在数据集里定位目标谱面 → 命中就查询，未命中就现场
//! 下载 `.osu` 分析 → 渲染。模式（standard / mania）完全由数据决定，
//! 用户不需要显式选择；写错模式的专属选项会被明确拒绝。

use std::sync::Arc;

use arc_core::{Args, BoxedError, CommandSpec, RegistryBuilder};
use beatmap_api::OsuService;

use crate::dataset::TargetSpec;
use crate::error::{Error, Result};
use crate::local_text;
use crate::models::{Request, Ruleset};
use crate::result::QueryOutcome;

use super::common::Context;
use super::render;

/// 登记 `/similar` 命令。
pub fn register(builder: &mut RegistryBuilder<OsuService>, ctx: Arc<Context>) {
    let spec = CommandSpec::new("查找手感/难度相近的谱面", local_text::usage::SIMILAR)
        .aliases(["sim", "相似"])
        .details(local_text::usage::SIMILAR_DETAILS);
    builder.command("similar", spec, move |_state, args| {
        let ctx = ctx.clone();
        async move { handle(&ctx, &args).await }
    });
}

/// 命令入口：解析参数 → 查询 → 渲染。
async fn handle(ctx: &Context, args: &Args<OsuService>) -> Result<String, BoxedError> {
    let request = Request::parse(args)?;
    let outcome = run(ctx, &request).await?;
    Ok(render::render(&outcome))
}

/// 按数据集命中情况分派模式。
async fn run(ctx: &Context, request: &Request) -> Result<QueryOutcome> {
    match ctx.datasets().locate(request.map_id).await? {
        Some(Ruleset::Standard) => {
            reject_mania_options(request)?;
            ctx.datasets()
                .query_standard(
                    TargetSpec::InIndex(request.map_id),
                    request.standard.clone(),
                    request.limit,
                )
                .await
        }
        Some(Ruleset::Mania) => {
            reject_standard_options(request)?;
            ctx.datasets()
                .query_mania(
                    TargetSpec::InIndex(request.map_id),
                    request.mania.clone(),
                    request.limit,
                )
                .await
        }
        None => {
            // 两套数据集都没部署时，"现场分析"也无从谈起（分析器与归一化器都来自数据集）
            if ctx.datasets().none_deployed() {
                return Err(Error::NoDataset {
                    standard_dir: ctx.datasets().standard_dir(),
                    mania_dir: ctx.datasets().mania_dir(),
                });
            }
            analyze_and_query(ctx, request).await
        }
    }
}

/// 目标不在数据集里：下载 `.osu`、按 `[General] Mode` 分派，再现场分析。
async fn analyze_and_query(ctx: &Context, request: &Request) -> Result<QueryOutcome> {
    log::debug!("{}", local_text::log::analyzing(request.map_id));
    let bytes = download_osu_file(ctx, request.map_id).await?;
    if !beatmap_api::osu_file::looks_like_osu_file(&bytes) {
        return Err(Error::OsuFile(
            beatmap_api::local_text::osu_file::NOT_OSU_FILE.to_owned(),
        ));
    }

    // 只需要读 `[General] Mode`，非 UTF-8 的老谱面用有损解码即可
    let text = String::from_utf8_lossy(&bytes);
    let mode = beatmap_api::osu_file::parse_mode(&text)
        .map_err(|error| Error::OsuFile(error.to_string()))?;

    match mode {
        0 => {
            reject_mania_options(request)?;
            ctx.datasets()
                .query_standard(
                    TargetSpec::on_the_fly(request.map_id, bytes),
                    request.standard.clone(),
                    request.limit,
                )
                .await
        }
        3 => {
            reject_standard_options(request)?;
            ensure_playable_keys(&text)?;
            ctx.datasets()
                .query_mania(
                    TargetSpec::on_the_fly(request.map_id, bytes),
                    request.mania.clone(),
                    request.limit,
                )
                .await
        }
        other => Err(Error::UnsupportedMode { mode: other }),
    }
}

/// mania 只支持 4K/6K/7K：分析前先用 `[Difficulty] CircleSize` 做友好检查。
fn ensure_playable_keys(text: &str) -> Result<()> {
    let stats = beatmap_api::osu_file::parse_difficulty(text)
        .map_err(|error| Error::OsuFile(error.to_string()))?;
    let keys = stats.cs.round() as i32;
    if !matches!(keys, 4 | 6 | 7) {
        return Err(Error::UnsupportedKeys { keys });
    }
    Ok(())
}

/// 下载（或复用缓存的）目标谱面 `.osu` 内容。
async fn download_osu_file(ctx: &Context, map_id: u32) -> Result<Vec<u8>> {
    let Some(service) = ctx.state().client() else {
        return Err(Error::OsuFile(local_text::error::api_not_configured()));
    };
    let path = service.download_osu_file(map_id).await?;
    let bytes = tokio::fs::read(&path).await.map_err(|error| {
        Error::OsuFile(beatmap_api::local_text::osu_file::read_failed(
            &path.display().to_string(),
            &error.to_string(),
        ))
    })?;
    Ok(bytes)
}

/// mania 谱面不接受 standard 专属选项。
fn reject_standard_options(request: &Request) -> Result<()> {
    if request.standard.explicit {
        return Err(Error::Usage(local_text::hint::standard_only_flags()));
    }
    Ok(())
}

/// standard 谱面不接受 mania 专属选项。
fn reject_mania_options(request: &Request) -> Result<()> {
    if request.mania.explicit {
        return Err(Error::Usage(local_text::hint::mania_only_flags()));
    }
    Ok(())
}
