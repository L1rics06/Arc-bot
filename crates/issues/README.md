# issues

轻量待办清单模块（v0.1 的 `/add` `/list` 标准化版本）。

## 命令

| 命令 | 说明 |
|---|---|
| `/add <标题>[，描述]` | 新建，分配稳定编号（`#001`） |
| `/list [页码]` | 分页列表（每页 10），按编号升序 |
| `/show <编号>` | 详情（状态 / 创建时间 / 描述） |
| `/edit <编号> <新标题>[，新描述]` | 修改 |
| `/del <编号>` | 删除 |
| `/status <编号> <open\|done\|suspended>` | 改状态 |
| `/done` / `/reopen <编号>` | 状态快捷方式 |

编号接受 `3` / `#3` / `#003` 三种写法。

## 结构

```
src/
  lib.rs            分层说明 + re-export
  error.rs          Error 枚举（Display 文案来自 local_text）
  local_text.rs     用户可见文案：usage / hint / add / list / show / edit / del / status / error
  models.rs         Issue / Status / NewIssue（输入解析、长度与状态校验）
  storage/
    mod.rs          声明与 re-export
    store.rs        Store：读写、并发控制、打开目录
    id.rs           编号格式化（#003）与解析（用户输入 / 文件名）
    legacy.rs       旧数据迁移（legacy-data/ → data/issues/，只跑一次）
  handlers/
    mod.rs          register：把所有命令登记进路由表
    common.rs       Context：惰性打开的 Store + 编号参数解析
    add.rs list.rs show.rs edit.rs delete.rs status.rs   # 一条命令一个文件
    render.rs       列表 / 详情的文本渲染（纯函数）
tests/
  models.rs         输入解析与序列化兼容
  render.rs         列表分页、时间展示、截断
  store.rs          编号分配、并发、更新 / 删除、迁移（含真机数据回归）
```

## 存储设计

- 每个 issue 一个文件，文件名就是编号（`#003.json`）；
- 编号创建后永不变更，删除不复用；`/add` 的"分配编号 + 写盘"在 `write_lock` 内串行；
- 目录：`data/issues/`（kovi 规范）；
- **旧数据迁移**：首次打开时把 `legacy-data/*.json`（v0.1 格式：`name`/`time` 字段）
  导入并重新编号，原目录保留不删；完成后写 `.migrated_from_legacy` 标记，只执行一次。

## 扩展方式

新增命令：在 `handlers/` 新建文件 + 在 `handlers/mod.rs` 的 `register` 加一行，
文案加到 `local_text.rs`。详见仓库 [AGENTS.md](../../AGENTS.md)。

## 测试

```bash
cargo test -p issues
```
