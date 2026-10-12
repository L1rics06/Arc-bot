# arc-core

arc-bot 的核心库。**不依赖 kovi / OneBot / 任何协议端类型**，只提供业务无关的基础设施。

## 结构

```
src/
  lib.rs       crate 文档 + 模块表 + re-export
  args/
    mod.rs     声明与 re-export
    parse.rs   Args（引号成组、--flag、-- 之后归 rest）、tokenize、extract_command
  command/
    mod.rs     声明与 re-export（CommandResult 类型别名）
    spec.rs    Command / CommandSpec / Handler / boxed / make / name_from_usage
    builder.rs RegistryBuilder：业务模块登记命令用的收集器
  error.rs     统一错误 Error / Result（跨模块兜底）
  event.rs     Reply trait 与 Text：与协议端无关的回复原语
  registry.rs  CommandSource / CommandInfo：命令表的只读视图
  state.rs     AppState<C> / AppMeta / SharedState / default_data_root
  storage.rs   JsonStore：文件型 JSON 存储（原子写、名校验、错误不吞）
tests/
  args.rs      命令名提取与参数解析（含 `--page` 不是分隔符的回归）
  command.rs   命令描述、收集器、handler 装箱
  registry.rs  CommandSource 挂载到 AppState 的契约
  state.rs     管理员判定、目录解析、客户端只初始化一次
  storage.rs   读写往返、原子写、过滤、非法名与坏 JSON 上报
```

| 模块 | 内容 |
|---|---|
| `state` | `AppState<C>`：启动时构造一次、以 `Arc` 共享给全部命令的状态。`C` 是共享外部客户端的类型（如 `OsuService`），通过 `init_client` 惰性初始化一次 |
| `command` | `CommandSpec` / `Command` / `RegistryBuilder`：命令的**数据描述**（不含分发逻辑，分发在 cmd_router） |
| `args` | `Args`：命令行参数解析；`extract_command` 从消息开头提取命令名 |
| `event` | `Reply` trait 与 `Text`：与协议端无关的回复原语 |
| `registry` | `CommandSource` / `CommandInfo`：命令表的只读视图，供 `/help` 这类自举命令使用 |
| `storage` | `JsonStore`：文件型 JSON 存储 |
| `error` | 统一错误 `Error` / `Result` |

## 为什么不依赖 kovi

arc-core 是"共享状态 + 命令描述"的层。业务模块（issues / beatmap_api）只依赖它，
因此它们不必知道 cmd_router 的存在，也不可能与路由层成环。
命令的**执行**（分发、权限、help 聚合）全部留在 cmd_router。

## 关键约定

- 数据目录一律用 `AppState::module_dir("<模块名>")`，运行期数据都落在 `<cwd>/data/` 下。
- 用户可见错误通过各模块自己的 `Display` 提供（中文、可操作）；
  `Error` 只做跨模块兜底。
- `Reply` 只描述"发什么"，不描述"怎么发"——协议端适配在 cmd_router。
- 存储实体名会被 `JsonStore` 校验（拒绝 `/`、`..` 等），杜绝路径穿越。
