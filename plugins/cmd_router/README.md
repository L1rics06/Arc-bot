# cmd_router

项目中**唯一**的消息监听与命令分发插件，同时负责启动期装配。

## 职责

1. 构造并共享 `AppState<OsuService>`（数据根目录、管理员名单、osu! 客户端）；
2. 收集各业务模块注册的命令（`issues::register`、`beatmap_api::register`），
   冻结成 `Registry`，再自举 `/help`；
3. 注册唯一的 `on_msg` 监听，把消息分发给命中的命令处理器。

## 结构

```
src/
  lib.rs            插件入口（#[kovi::plugin]）与装配顺序；管理员 ID 提取
  text.rs           用户可见文案：help 排版 / 分发错误 / 内部提示
  registry/
    mod.rs          Registry：冻结、索引、insert、dispatch 包装、CommandSource 实现
    index.rs        Registry 的数据结构（命令表 + 名字索引）
    dispatch.rs     分发的核心逻辑（权限 → 参数解析 → 调 handler → 错误包装）
  router.rs         Runner：唯一的 on_msg 监听（与 kovi 接线，可单测）
  builtins/
    mod.rs          register_all：登记全部内置命令
    help.rs         /help 自举命令与渲染
tests/
  routing.rs        端到端：真实装配 issues + beatmap_api + /help，跑完整 CRUD 流程
  registry.rs       分发、别名、权限、错误包装、隐藏命令、重名
  help.rs           /help 渲染
```

## 分发规则

- 只处理**以 `/` 或 `!` 开头**、且命令名已注册的消息；
  普通聊天与未知命令一律静默（不抢聊、不刷屏）。
- 纯图片 / 纯 @ 消息没有文本，直接忽略（旧实现会在这里 panic）。
- `admin_only` 命令在分发前按 `AppState` 里的管理员 ID 校验。
- handler 返回 `Err` 时回复 `命令/用法: 原因`，用法提示来自 `CommandSpec::usage`。

## 依赖方向

```
cmd_router → {issues, beatmap_api} → arc-core
```

不要在本 crate 里写业务逻辑；新功能加到对应业务模块，然后在 `lib.rs` 的 `main()`
装配处加一行 `register`。新增内置命令放 `builtins/`。
