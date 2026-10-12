# 相似谱面（`/similar`）

玩法 / 手感相近的谱面检索。算法、数据格式与版本校验来自 vendored 的
`crates/osu-difficulty-runtime`（移植自 [OPP](https://github.com/osuplusplus/OPP)，
见该目录的 [VENDORED.md](../crates/osu-difficulty-runtime/VENDORED.md)）。

## 数据流

```
消息 /similar <ID|链接>
   → similar_beatmap::handlers::similar（参数解析、模式分派）
   → dataset::Datasets（惰性打开数据集 + spawn_blocking 隔离阻塞）
   → osu_difficulty_runtime（只读查询：SQLite 元数据 + 特征文件 + 索引）
   → handlers::render（中文渲染）
```

三种情况：

| 情况 | 行为 |
|---|---|
| 谱面在数据集里 | 直接查数据集（零网络） |
| 谱面不在数据集里 | 下载 `.osu`（缓存在 `data/beatmap_api/cache/osu_files/`）→ 按 `[General] Mode` 走 standard / mania 分析器 → 查询，回复里注明"现场分析结果" |
| 数据集未部署 | 返回"放在哪、怎么配"的可操作提示；机器人启动不受影响 |

模式自动分派：先看两套数据集谁收录了这张图，都没有才下载 `.osu` 读 `Mode`。
taiko / catch 明确报"不支持"；mania 只支持 4K/6K/7K。

## 算法概要（上游定义，此处只做索引）

- **osu!standard（Analyzer v4，`five-dimension-slider-rosu-reading-v4`）**：
  五维向量 `[aim, speed, reading, slider, overlap]`。aim/speed/reading 来自锁定的
  `Apeuriox/rosu-pp` rework 快照；slider 是"滑条占比 + 变速频率"的合成值；
  overlap 是 3000ms 窗口内的物件重叠 / 遮挡应变。排序用归一化后的加权距离
  （默认 aim 1 / speed 2 / reading 2 / slider 0 / overlap 0.25，参数权重 1）。
- **osu!mania（`mania-roxy-interlude-similarity-v1` + `mania-mma-pattern-v2`）**：
  8 维强度 + 16 维键型（覆盖率、细分键型、结构统计…）= 24 维记录，按 **键数隔离**
  比较；候选 Mod 池 NM/DT/HT 来自预计算记录，带键型记录时用键型距离
  （键型 50% / 强度 25% / 结构 15% / LN 比例 7% / 时长 3%）。
- 结果**默认排除与目标同一谱面集**（mania 可用 `--same-set` 放开）；standard 固定排除。
- 报告的距离是归一化特征上的欧氏距离（0 表示完全一致），不是官方星数差。

命令用法与全部选项：`/help similar`（或看 `crates/similar_beatmap/src/local_text.rs`
里的 `usage::SIMILAR_DETAILS`）。

## 数据集

数据集由 [osu-difficulty-lab](https://github.com/osuplusplus/osu-difficulty-lab) 生成，
**不在 git 里**，通过其 Release 附件分发。两套数据集互相独立、不能混用。

```text
data/similar_beatmap/standard/           # Analyzer v4
├── metadata.sqlite                      # beatmaps / analyses / analysis_versions(必需)
├── features-v{N}.bin                    # 头 ODLNORM + 定长记录
├── indexes/difficulty-main.hnsw(.sha256)
└── normalizers/v{N}.bin

data/similar_beatmap/mania/              # Mania v1 + 键型 v2
├── mania-metadata.sqlite
├── mania-features-v1.bin                # 头 ODLMAN1
├── mania-mod-features-v1.bin            # DT/HT 预计算记录（可选）
├── mania-mma-features.bin               # 键型记录（可选，缺则退回 v1 距离）
├── normalizers/mania-v1.bin
├── indexes/mania-v1.buckets(+.sha256)
└── dataset-manifest.json
```

目录位置：默认 `data/similar_beatmap/{standard,mania}`（`data/` 已在 `.gitignore` 里），
可用环境变量 `SIMILAR_STANDARD_DATASET` / `SIMILAR_MANIA_DATASET` 覆盖：

```bash
SIMILAR_STANDARD_DATASET=/srv/datasets/osu-standard
SIMILAR_MANIA_DATASET=/srv/datasets/osu-mania
```

判断"是否部署"的标记文件：standard 看 `metadata.sqlite`，mania 看 `mania-metadata.sqlite`。

## 版本绑定（升级时必须成套）

| 组件 | 当前要求 |
|---|---|
| standard 数据集 | Analyzer v4（`five-dimension-slider-rosu-reading-v4`） |
| mania 数据集 | `mania-roxy-interlude-similarity-v1`（normalization v1）+ 可选 `mania-mma-pattern-v2` |
| `osu-difficulty-runtime` | vendored 快照，`rosu-pp` 锁在 `Apeuriox/rosu-pp@pp-rework-202607#6bd0e6a4` |

运行时会把数据集 `analysis_versions`（standard）与 `mania_state.analyzer_snapshot`
（mania）和自身常量比对，**不匹配直接拒绝打开**并提示"数据集与算法版本不配套"。
所以：

- 只换数据集 → 必须与当前 vendored 运行时版本一致；
- 升级 vendored 运行时 → 需要同步换数据集，并跑一遍
  `cargo test -p osu-difficulty-runtime`（上游自带合成数据集测试）；
- **不要** `cargo update` 那两个 git 依赖（`rosu-pp` fork 与 `mania-pattern`），
  `Cargo.lock` 里锁的 commit 就是数据集的分析快照。

## 性能与资源

- 数据集只读、进程内只打开一次（首次 `/similar` 时才加载，启动不受影响）。
  首次打开 standard 要反序列化 HNSW 索引（约 40MB 级），mania 要全量校验记录。
- 查询走"全量精确扫描"（约 10 万级记录，毫秒级）；`--param 0` 时按上游语义走 HNSW 近似。
- 现场分析一张谱面需要 1-3 秒 CPU（standard 的 rosu-pp 计算）。
- 内存占用与数据集规模成正比（standard 打开后约数十 MB 量级）；部署前先确认机器余量。

## 测试

```bash
cargo test -p similar_beatmap              # 合成数据集，无网络、无真实数据集
cargo test -p similar_beatmap real_dataset # 真实数据集冒烟（缺数据集自动跳过）
```

- `tests/common/mod.rs`：按上游文件格式构造合成数据集（standard + mania）。
- `tests/query.rs`：数据集打开 / 查询 / 模式分派 / 命令级端到端（注册 → 分发 → 渲染）。
- `tests/real_dataset.rs`：真实数据集冒烟，可用环境变量指定目录。

## 已知限制

- 不支持 taiko / catch，mania 不支持 5K/8K+ 与 Key Mod。
- "按玩家最近成绩推荐"未实现（需要 osu! 用户级 OAuth）。
- 命令级限流未实现（见 AGENTS.md 的已知待办）。
