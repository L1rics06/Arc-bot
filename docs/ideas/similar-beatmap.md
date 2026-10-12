# 相似谱面推荐（未实现的设计稿）

## 目标

用户发 `beatmapID` → 返回一组"手感 / 难度相近"的候选 `beatmapID`。

## 生命周期（原设计）

```
beatmapID → 下载 .osu → 解析出多维难度特征 → 在谱面库里检索 → 返回候选 ID 数组
```

## 现状（截至 2026-10）

前置件已经在 `plugins/beatmap_api` 里就绪：

- `osu_file::download_osu_file()`：下载并缓存 `.osu`（`data/beatmap_api/cache/osu_files/`）；
- `osu_file::parse_difficulty()`：解析 `[Difficulty]` 段，得到 CS/AR/OD/HP；
- `models::DifficultyStats::distance_to()`：四维 L1 距离，可作为初步排序；
- `client::OsuService::info()`：谱面元数据（星数、BPM、状态）+ TTL 缓存。

## 还需要决定的

1. **候选集从哪来**：只从 osu! API 随机取样？还是建立本地谱面库
   （需要批量抓取，注意 API 限流）？
2. **相似度用什么特征**：目前只有 CSV/AR/OD/HP 四维；
   若要更真实的"手感相似"，需要难度计算（如 rosu-pp 或自研特征提取）。
3. **输出形式**：只回 ID 列表，还是附上每条候选的谱面名与星数？

## 实现骨架（建议）

```
plugins/similar_beatmap/src/
  lib.rs         # register()：/similar <beatmapID>
  features.rs    # 特征提取（先用 osu_file 的 DifficultyStats）
  candidate.rs   # 候选集获取与相似度排序
```

依赖方向与其他业务模块一致：只依赖 `arc-core` 与 `beatmap_api`（不改 cmd_router 的装配顺序，
只需要在 cmd_router 的装配处加一行 `similar_beatmap::register(...)`）。
