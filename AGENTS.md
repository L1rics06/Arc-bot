# AGENTS.md

给在本仓库工作的 AI agent（以及人类协作者）的工程约定。
**新增功能前先读这一页**：这里的每一条都对应一次已经踩过的坑。

## 项目定位

Rust 编写的 QQ 机器人（osu! 相关 + 个人待办），
基于 [Kovi](https://github.com/thricecola/Kovi) 0.13 + OneBot 11 协议端（NapCat / LLOneBot 等）。

## 仓库布局

```
Cargo.toml                  # workspace 定义 + 统一依赖版本（workspace.dependencies）
AGENTS.md                   # 本文件：工程约定
kovi.conf.toml              # OneBot 连接配置 + 管理员名单
kovi.plugin.toml            # 每个 kovi 插件的启动开关与访问控制
src/main.rs                 # 入口：dotenv → 驱动 → build_bot! → run（不含业务逻辑）
crates/                     # 库 crate：可复用、不含 kovi 插件入口
  arc-core/                 #   核心库（不依赖 kovi / OneBot）
  beatmap_api/              #   osu! 数据模块
  issues/                   #   待办清单模块
  similar_beatmap/          #   相似谱面模块（/similar）
  osu-difficulty-runtime/   #   vendored 上游快照：相似谱面算法与数据集格式（不要手改）
plugins/                    # kovi 插件 crate：会被 build_bot! 挂载
  cmd_router/               #   唯一的消息监听与命令分发
data/                       # 运行期数据（gitignore）
  similar_beatmap/          #   相似谱面数据集（自行下载，gitignore）
legacy-data/                # v0.1 的 issue 数据，首次 /add 时自动迁移（gitignore）
docs/ideas/                 # 未实现的设计稿
```

**crates/ 与 plugins/ 的区别**：能否被独立复用。业务逻辑必须在 `crates/` 里，
`plugins/` 只放"与框架接线"的那一层。新增业务模块默认加到 `crates/`。

## 依赖方向（严格单向，不允许成环）

```
main → cmd_router → {issues, beatmap_api, similar_beatmap} → arc-core
                        similar_beatmap → osu-difficulty-runtime
```

- 业务模块（issues / beatmap_api / similar_beatmap）**只依赖 arc-core**
  （similar_beatmap 另外依赖 beatmap_api 与 vendored 的 osu-difficulty-runtime），
  不依赖 cmd_router。
  命令的"描述"类型（`CommandSpec` / `RegistryBuilder`）住在 `arc_core::command`，
  就是为了让这一点成立。
- arc-core 是纯库：不引入 kovi、OneBot 或任何协议端类型。
  它只认识 `AppState<C>`（`C` 是共享的外部客户端类型，如 `OsuService`）。
- 需要"读命令表"的框架级命令（如 `/help`）通过 `arc_core::CommandSource` trait 访问。

## vendored 上游 crate

`crates/osu-difficulty-runtime` 是 OPP 的上游快照，**保持与上游一致**（含其
`README.md`），只允许追加 `VENDORED.md`：

- 同步方式、上游 commit、许可与依赖锁定见该目录的 `VENDORED.md`。
- **不要**为"顺手修一下"改它的源码；需要改动时同步回上游或在 VENDORED.md 里记录。
- 数据集的算法版本与它强绑定（Analyzer v4 / mania v1）：升级它就必须换数据集。
- `rosu-pp` fork 与 `mania-pattern` 两个 git 依赖锁在 `Cargo.lock`，不要 `cargo update`。

## 模块骨架标准（每个 crate 都要遵守）

**不要写单一大文件。** 按职责拆文件，一个文件只做一件事。标准骨架：

| 目录/文件 | 放什么 | 是否必需 |
|---|---|---|
| `src/lib.rs` | crate 文档 + 模块表 + 分层说明 + `mod` 声明与 re-export | 必需 |
| `src/models.rs` 或 `src/models/` | 领域模型与输入解析（纯数据，无 IO）。模型多了就拆成目录（`models/beatmap_info.rs` 等） | 有模型就必需 |
| `src/local_text.rs` 或 `src/text.rs` | **用户可见文案集中处**：命令回复、错误提示、用法说明 | 必需 |
| `src/error.rs` | 面向用户的错误类型（`thiserror`），`Display` 即用户文案 | 必需 |
| `src/storage/` | 持久化：存储实现、编号分配、迁移。`mod.rs` + 按职责分文件 | 有持久化就必需 |
| `src/handlers/` 或 `src/service/` | 对外命令：**一条命令一个文件**，各自暴露 `register(...)` | 有命令就必需 |
| `src/handlers/render.rs` | 面向用户的文本渲染（纯函数） | 有渲染就必需 |
| `tests/` | **全部测试**（见下） | 必需 |

参考实现（照抄这个结构就行）：

```
crates/issues/
  src/
    lib.rs            # 文档 + 模块表 + re-export
    error.rs          # Error 枚举（Display 引用 local_text）
    local_text.rs     # usage::* / hint::* / add::* / list::* / error::*
    models.rs         # Issue / Status / NewIssue
    storage/          # mod.rs + store.rs（读写）+ id.rs（编号）+ legacy.rs（迁移）
    handlers/         # mod.rs（register_all）+ common.rs（共享上下文）+ 一条命令一个文件
      add.rs list.rs show.rs edit.rs delete.rs status.rs render.rs
  tests/              # models.rs / render.rs / store.rs —— 一个主题一个文件
```

### 硬性规则

1. **一个文件一件事**。超过 ~200 行的业务文件就该考虑拆（测试不算，它们已经独立）。
2. **文案不写在逻辑里**：所有会发给用户的字符串放 `local_text.rs` / `text.rs`，
   逻辑代码只引用常量或调用文案函数。禁止在 handler 里写字面量回复。
3. **渲染与逻辑分离**：格式化文本的函数放 `render.rs`，只接收数据、返回 `String`，不做 IO。
4. **handler 三件套**：`register(builder, ctx)`（登记命令）+ `handle(...)`（编排）+
   业务规则放 `storage` / `models`。handler 里不写 SQL / 文件细节。
5. **`mod.rs` 只做声明与 re-export**，不写实现。
6. **命令一条一个文件**：新增命令 = 新建文件 + 在 `handlers/mod.rs` 的 `register_all`
   加一行。
7. **测试放 `tests/` 目录**，一个主题一个文件（`tests/models.rs`、`tests/store.rs`…），
   文件名与被测模块对应。不在 `src/` 里写 `#[cfg(test)] mod tests`。
   需要被测试的内部函数，用 `pub` + 文档说明暴露（而不是把测试塞回 src）。

### 测试约定

- 单元测试与集成测试统一放 `tests/`，用 `#[test]` / `#[tokio::test]`。
- 测试**不连 QQ、不打网络**：用 `tempfile` 造临时目录，用纯函数测解析与渲染。
- 涉及真实数据的回归（如旧数据迁移）可以读仓库里的真实文件，但路径要用
  `env!("CARGO_MANIFEST_DIR")` 定位，缺文件时跳过而不是失败。

## 硬性约定

1. **一个进程只允许一个 `on_msg` 监听**：全部消息入口都在 `cmd_router::router`。
   新增命令不要自己注册监听器，而是注册进命令表。
2. **命令只在消息开头匹配**：`/map 123` 合法，`你看 /map 123` 不触发（避免抢聊）。
   未注册的命令与普通聊天一律静默。
3. **不允许 `unwrap()` / `expect()`**（测试与"常量不可能失败"的位置除外）。
   用户输入、文件系统、网络、API 响应永远按可能失败处理。
4. **持久化数据一律走 `arc_core::JsonStore`**：原子写（临时文件 + rename）、
   实体名校验（防路径穿越）、错误必须上报，不允许 `let _ = save(...)` 静默吞错。
5. **用户可见文案是中文**，错误信息必须可操作：
   `找不到编号 99 的 issue（先用 /list 看当前列表）` 而不是 `NotFound`。
6. **数据目录用 `state.module_dir("<模块名>")`**，不要写相对路径常量。
   运行期目录都在 `data/` 下。
7. **共享外部客户端（如 osu! API）只初始化一次**：
   放进 `AppState::init_client`，由 cmd_router 启动时填充，命令里用 `state.client()` 取。
   绝不在命令或插件里重新 `Osu::new(...)`。
8. **错误分层**：业务模块用 `thiserror` 定义枚举，`Display` 即用户文案；
   handler 返回 `Result<Box<dyn Reply>, BoxedError>`；
   路由层统一包装成 `命令/用法: 原因` 并附上用法提示。

## 新增一条命令的标准步骤

1. 在业务模块的 `handlers/` 下新建文件（一条命令一个文件），写 `register(builder, ctx)`
   与 `handle(...)`；参数用 `args.positional(n)` / `args.flag("x")` / `args.rest_or_all()` 取。
2. 用 `CommandSpec` 写清 `description` / `usage` / `details`，需要管理员就 `.admin_only()`；
   `usage` 必须以 `/命令名` 开头（`/help` 列表与错误提示都靠它，`make()` 也会从中推导命令名）。
3. 文案加进 `local_text.rs`。
4. 在 `handlers/mod.rs` 的 `register_all` 里加一行。
5. 补测试：纯函数测试放 `tests/`（渲染、解析、存储），涉及路由的加到
   `plugins/cmd_router/tests/routing.rs`。

## 新增一个业务模块的步骤

1. `crates/<name>/` 建 crate，依赖 `arc-core`（+ workspace 依赖）。
2. 按上面的模块骨架建立文件（`error.rs` / `local_text.rs` / `models.rs` /
   `handlers/` / `tests/`）。
3. 暴露 `pub fn register<C>(builder: &mut RegistryBuilder<C>, state: SharedState<C>)`。
4. 在 `plugins/cmd_router/src/lib.rs` 的 `main()` 里加一行
   `name::register(&mut builder, state.clone())`（`/help` 必须最后自举）。
5. 需要新的共享客户端类型就改 cmd_router 里的 `AppState::<T>` 并更新 `init_client`。

## 本地校验（提交前必须全绿）

```bash
cargo fmt --all                                  # 格式化
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```

CI（`.github/workflows/ci.yml`）执行同样的四项。

## 运行

```bash
cp .env.example .env    # 填 CLIENT_ID / CLIENT_SECRET（只影响 /map）
cargo run               # 首次运行 kovi 会引导生成 kovi.conf.toml
```

`.env` 缺失或凭据错误**不会**中断启动：机器人照常上线，`/map` 返回可操作的提示；
`/similar` 仍可用（数据集查询不依赖 API），只是无法现场分析未收录的谱面。

相似谱面数据集放到 `data/similar_beatmap/{standard,mania}`（或由
`SIMILAR_STANDARD_DATASET` / `SIMILAR_MANIA_DATASET` 指定），来源与版本要求见
`docs/similar-beatmap.md`。

## 已知待办

- 命令级限流（防刷屏）尚未实现。
- 相似谱面的"按玩家最近成绩推荐"未实现：需要 osu! 用户级 OAuth
  （授权码流程 + token 持久化），目前只有 client credentials。
- 相似谱面数据集需要手动下载（`osu-difficulty-lab` 的 Release），
  见 `docs/similar-beatmap.md`；数据集与 runtime 版本必须成套升级。
