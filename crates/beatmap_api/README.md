# beatmap_api

osu! 铺面数据模块（原 `serde_beatmap`）。

## 命令

| 命令 | 说明 |
|---|---|
| `/map <beatmapID>` | 谱面集信息 + 当前难度（星数 / BPM / CS/AR/OD/HP / 状态） |

## 结构

```
src/
  lib.rs            分层说明 + re-export
  error.rs          Error 枚举（Display 文案来自 local_text）
  local_text.rs     用户可见文案：usage / hint / render 字段名 / 各模块错误模板
  models/
    mod.rs          re-export
    beatmap_info.rs BeatmapInfo（领域模型，与 rosu-v2 解耦）
    difficulty.rs   DifficultyStats + distance_to（相似谱面用的距离度量）
    status.rs       MapStatus（含 rosu → 领域 的映射）、parse_beatmap_id
  client.rs         OsuService：rosu 客户端 + 缓存 + HTTP；init_from_env（共享初始化入口）
  cache.rs          InfoCache：谱面元数据缓存（TTL 24h + schema 版本号）
  osu_file/
    mod.rs          re-export
    download.rs     .osu 下载与本地复用
    parse.rs        [Difficulty] 段解析（纯函数）
  service/
    mod.rs          re-export
    handler.rs      /map 命令注册与处理
    render.rs       谱面信息渲染（纯函数）
tests/
  models.rs         ID 解析、状态映射、难度距离、序列化往返
  info_cache.rs     TTL 过期、schema 失效、重开复用
  osu_file.rs       [Difficulty] 解析规则（注释 / 缺键 / 非法值 / 段名大小写）
  render.rs         /map 输出格式
```

## 关键设计

- **进程内单例**：`OsuService` 由 cmd_router 启动时通过 `AppState::init_client`
  初始化一次，命令用 `state.client()` 取（旧实现 main 与路由插件各初始化一次，
  会重复认证并建两份连接池）。
- **凭据失败不影响启动**：`init_from_env` 返回 `Err`，由 cmd_router 记 warn，
  `/map` 给出可操作提示。
- **不再 panic**：原实现 `mapset.unwrap()` 在被删 / graveyard 谱面上会崩，
  现在返回 `Error::MissingMapset`；未知 beatmapID 返回 `Error::Invalid`。
- **缓存**：元数据按 24h TTL 缓存在 `data/beatmap_api/cache/`；
  结构变化时递增 `SCHEMA_VERSION`，旧条目自动忽略。
- **领域模型与 rosu-v2 解耦**：缓存格式稳定，rosu 升级不影响落盘数据；
  `DifficultyStats` 是相似谱面算法的现成载体。

## 测试

```bash
cargo test -p beatmap_api   # 不打网络
```
