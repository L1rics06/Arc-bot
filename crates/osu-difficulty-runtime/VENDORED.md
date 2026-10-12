# 上游来源与同步说明（vendored）

本目录是 [OPP](https://github.com/osuplusplus/OPP) 里 `src-tauri/crates/osu-difficulty-runtime`
的**原样副本**（MIT，见 `LICENSE`），用于 arc-bot 的相似谱面命令 `/similar`。

## 快照

| 项目 | 值 |
|---|---|
| 上游仓库 | `osuplusplus/OPP`，路径 `src-tauri/crates/osu-difficulty-runtime` |
| 上游分支 / commit | `main` @ `6ae83ac`（2026-10-05），copy 时 `diff -r` 与上游完全一致 |
| 许可 | MIT（`LICENSE`，Copyright (c) 2026 osuplusplus） |
| 本次引入 | 2026-10-12 |

**除本文件外，目录内容与上游保持一致**（包括 `README.md` 里指向 OPP `docs/` 的相对链接）。
这是为了保证"日后同步 = 重新 copy 该目录 + 保留本文件"。

## 依赖锁定（不要随意升级）

`Cargo.toml` 里有两个必须与数据集配套的 git 依赖：

- `rosu-pp` = `Apeuriox/rosu-pp` 分支 `pp-rework-202607`
  （数据集 Analyzer v4 的 aim/speed/**reading** 来自该快照；`Cargo.lock` 锁在 `6bd0e6a4`）
- `mania-pattern` = `osuplusplus/osu-difficulty-lab` rev `92b791c…`
  （mania 键型距离算法）

**禁止**对这两个依赖执行 `cargo update`。数据集里的 `analysis_versions` /
`dataset-manifest` 会与运行时常量校验，算法或快照漂移会导致数据集被判定为
`Incompatible` 而拒绝打开。

## 数据集版本绑定

- osu!standard：Analyzer v4（`five-dimension-slider-rosu-reading-v4`）
- osu!mania：`mania-roxy-interlude-similarity-v1` + `mania-mma-pattern-v2`

数据集由 `osu-difficulty-lab` 生成、通过其 Release 分发（不在 git 里）。
升级数据集 = 同时确认本 crate 的算法版本与之匹配；升级本 crate = 需要重新生成/下载数据集。

## 同步步骤

```bash
# 在 OPP 仓库确认目标 commit，然后：
rsync -a --delete \
  /path/to/OPP/src-tauri/crates/osu-difficulty-runtime/ \
  crates/osu-difficulty-runtime/ \
  --exclude VENDORED.md
cargo test -p osu-difficulty-runtime   # 上游自带合成数据集测试
```

若上游代码在 `cargo clippy --workspace --all-targets -- -D warnings`（CI 的
`RUSTFLAGS=-D warnings` 对 path 依赖同样生效）下报出新 lint，只做最小 `allow`
并在本文件记录对应改动与原因。
