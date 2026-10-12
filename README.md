# arc-bot

Rust 编写的 QQ 机器人：osu! 铺面查询 + 相似谱面 + 个人待办清单。
基于 [Kovi](https://github.com/thricecola/Kovi) 0.13 与 OneBot 11，
可连接 NapCat / LLOneBot 等协议端。

> 工程约定、模块骨架标准、新增命令 / 模块的标准步骤见 [AGENTS.md](AGENTS.md)。

## 快速开始

```bash
cp .env.example .env    # 填 osu! API 凭据（只影响 /map，不填也能启动）
cargo run               # 首次运行 kovi 会引导生成 kovi.conf.toml（OneBot 连接 + 管理员）
```

## 命令

| 命令 | 说明 |
|---|---|
| `/help [命令名]` | 列出全部命令或查看某条命令的详细用法（别名 `/h` `/帮助`） |
| `/map <beatmapID>` | 查询 osu! 铺面信息（别名 `/m` `/谱面`） |
| `/similar <谱面ID或链接>` | 查找手感 / 难度相近的谱面，支持 standard 五维权重与 mania 键型池（别名 `/sim` `/相似`） |
| `/add <标题>[，描述]` | 记一条待办（别名 `/a`） |
| `/list [页码]` | 列出待办，每页 10 条（别名 `/ls`） |
| `/show <编号>` | 查看某条待办的详情 |
| `/edit <编号> <新标题>[，新描述]` | 修改标题 / 描述 |
| `/del <编号>` | 删除一条待办（别名 `/delete` `/rm`） |
| `/status <编号> <open\|done\|suspended>` | 修改状态 |
| `/done <编号>` / `/reopen <编号>` | 标记完成 / 重新打开 |

约定：命令必须写在**消息开头**（`/map 123` 会触发，`你看 /map 123` 不会），
普通聊天不会被机器人接管。管理侧还有 kovi 自带的 `.kovi` 命令（插件启停、访问控制）。

## 架构

```
arc-bot (src/main.rs —— 只做装配)
└── plugins/cmd_router            唯一的 on_msg 监听 + 命令分发 + /help
    ├── crates/issues             待办清单：JSON 存储 + 完整 CRUD
    ├── crates/beatmap_api        osu! 数据：API 客户端 / 缓存 / .osu 下载解析
    ├── crates/similar_beatmap    相似谱面：数据集查询 + 现场分析 + 中文渲染
    │   └── crates/osu-difficulty-runtime   vendored 的算法与数据格式（MIT，见其 VENDORED.md）
    └── crates/arc-core           核心库：AppState、错误、存储抽象、命令描述
```

`crates/` 是可直接复用的库（业务逻辑都在这），`plugins/` 只放与 kovi 框架接线的
那一层。每个 crate 的内部结构（`models/` `local_text.rs` `handlers/` `tests/`…）
遵循[模块骨架标准](AGENTS.md#模块骨架标准每个-crate-都要遵守)。

事件流：

```
OneBot 消息 → cmd_router 唯一监听 → 命令解析（仅开头匹配）
           → 命令表分发（按名字 / 别名） → handler(state, args) → 回复
```

设计要点：

- **共享客户端只初始化一次**：osu! API 客户端由 cmd_router 启动时放进
  `AppState::init_client`，所有命令通过 `state.client()` 复用。
- **凭据缺失不阻断启动**：`.env` 有问题时机器人照常上线，`/map` 返回可操作的提示。
- **数据落在 kovi 规范的 `data/` 目录**：`data/issues/`（待办）、
  `data/beatmap_api/cache/`（谱面元数据缓存 + `.osu` 文件）、
  `data/similar_beatmap/{standard,mania}/`（相似谱面数据集，需自行下载）。
  v0.1 的旧待办数据（`legacy-data/`）首次 `/add` 时自动迁移，原目录保留。
- **业务模块互不依赖**：issues 与 beatmap_api 都只依赖 arc-core；
  命令的"描述"类型住在 arc-core，从结构上避免循环依赖。
- **文案与逻辑分离**：所有用户可见字符串集中在各 crate 的 `local_text.rs` /
  `text.rs`，改文案只动一个文件。

## 开发

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace        # 不连 QQ、不打网络
```

## 相似谱面（`/similar`）

玩法 / 手感相近的谱面检索，算法与数据格式移植自
[OPP](https://github.com/osuplusplus/OPP)：
osu!standard 用五维向量（aim / speed / reading / slider / overlap），
osu!mania 用 8 维强度 + 16 维键型的 24 维记录（4K/6K/7K，NM/DT/HT）。

数据集不在 git 里，需要从 [osu-difficulty-lab](https://github.com/osuplusplus/osu-difficulty-lab)
的 Release 下载并解压到：

```text
data/similar_beatmap/standard/   # metadata.sqlite、features-v*.bin、indexes/、normalizers/
data/similar_beatmap/mania/      # mania-metadata.sqlite、mania-*.bin、indexes/、normalizers/
```

也可以用环境变量 `SIMILAR_STANDARD_DATASET` / `SIMILAR_MANIA_DATASET` 指向别处
（见 [.env.example](.env.example)）。没有数据集时机器人照常启动，`/similar` 会返回
"该放哪里、怎么配"的提示；谱面不在数据集里时会现场下载 `.osu` 分析（需要 osu! API 凭据）。

算法版本、数据集版本绑定与存储格式见 [docs/similar-beatmap.md](docs/similar-beatmap.md)，
命令行选项见 `/help similar`。

## 路线图

- 命令级限流（防刷屏）。
- 相似谱面的"按玩家最近成绩推荐"（需要 osu! 用户级 OAuth 授权流程）。
- 移植 [OPP](https://github.com/osuplusplus/OPP) 的其他功能。

*Restart, and accumulate slowly.*
