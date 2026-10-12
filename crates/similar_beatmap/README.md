# similar_beatmap

相似谱面模块：`/similar <谱面ID或链接>` 在预计算数据集里找"玩法 / 手感相近"的谱面。

## 分层

| 文件 / 目录 | 内容 |
|---|---|
| `src/lib.rs` | crate 文档 + 模块表 + `register` |
| `src/models.rs` | 参数解析（ID / 链接、`-n`、区间过滤器、五维权重、Mod 池）与 `Ruleset` |
| `src/config.rs` | 数据集目录解析（环境变量 + `data/similar_beatmap/` 默认路径） |
| `src/dataset/` | 数据集适配层：`mod.rs`（惰性打开 + `spawn_blocking` + 模式分派）、`standard.rs`、`mania.rs` |
| `src/result.rs` | 查询结果的领域模型（与 runtime 类型解耦） |
| `src/handlers/` | `similar.rs`（一条命令一个文件）+ `render.rs`（纯函数渲染）+ `common.rs`（`Context`） |
| `src/local_text.rs` | 用户可见文案集中处（含 `/help similar` 的详细说明） |
| `src/error.rs` | 面向用户的错误类型 |
| `tests/` | `common/`（合成数据集 fixture）、`models.rs`、`render.rs`、`query.rs`、`real_dataset.rs` |

## 依赖方向

```text
cmd_router → similar_beatmap → {beatmap_api, osu-difficulty-runtime} → arc-core
```

相似度算法与数据格式来自 vendored 的 [`osu-difficulty-runtime`](../osu-difficulty-runtime/VENDORED.md)：
本 crate 只做"命令 → 数据集查询 → 中文渲染"的编排，不重写算法。

## 要点

- `osu_difficulty_runtime` 的 API 全是**同步阻塞**的（打开时要反序列化索引、查询要扫全表），
  因此所有调用都经 `spawn_blocking`，不在 async 线程上跑。
- 数据集**惰性打开**（`tokio::sync::OnceCell`），目录里没有标记文件时直接给"未部署"提示。
- 数据集位置与版本绑定、部署与升级注意事项见 [docs/similar-beatmap.md](../../docs/similar-beatmap.md)。

## 本地校验

```bash
cargo test -p similar_beatmap
cargo clippy -p similar_beatmap --all-targets -- -D warnings
```
