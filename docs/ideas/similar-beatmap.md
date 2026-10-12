# 相似谱面推荐（设计稿 → 已实现）

> **状态：已实现（2026-10）。** 实际实现与本文最初设想不同，最终文档见
> [docs/similar-beatmap.md](../similar-beatmap.md)，代码见 `crates/similar_beatmap/`
> 与 vendored 的 `crates/osu-difficulty-runtime/`。本文件保留为决策记录。

## 目标（不变）

用户发 `beatmapID` → 返回一组"手感 / 难度相近"的候选谱面。

## 当初的困惑与最终答案

1. **候选集从哪来**：没有走"osu! API 随机取样"，而是采用 **预计算数据集**
   （`osu-difficulty-lab` 生成、Release 分发）：全量 ranked 谱面离线算好特征与索引，
   查询是本地只读检索，零网络、零 API 限流风险。
2. **相似度用什么特征**：四维 CS/AR/OD/HP 不足以表达"手感"。
   最终移植 OPP 的算法：osu!standard 是五维
   `[aim, speed, reading, slider, overlap]`（难度属性取自锁定的 rosu-pp 快照，
   slider/overlap 为自研特征）；osu!mania 是 24 维强度 + 键型记录、按键数隔离比较。
3. **输出形式**：返回谱面名 + 星数/分位 + 距离 + 在线链接（不是裸 ID 列表），并注明
   是否排除了同谱面集、数据集时间范围。

## 与最初的骨架设想相比的实际结构

```
crates/similar_beatmap/src/
  lib.rs error.rs local_text.rs config.rs
  models.rs result.rs
  dataset/{mod,standard,mania}.rs     # 数据集适配 + 阻塞隔离 + 模式分派
  handlers/{mod,common,similar,render}.rs
  tests/{common/mod,models,render,query,real_dataset}.rs
crates/osu-difficulty-runtime/        # vendored 算法与数据格式（MIT）
```

依赖方向与当初一致（只加一行 `similar_beatmap::register(...)`），但多了一层
vendored crate：算法本身不重写，只做"命令 → 数据集查询 → 中文渲染"的编排。

## 仍未做

- 按玩家最近成绩 / BP 推荐（需要 osu! 用户级 OAuth）。
- taiko / catch，mania 5K/8K+ 与 Key Mod。
