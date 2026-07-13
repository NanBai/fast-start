---
doc_type: feature-design
feature: 2026-07-13-launch-preflight
requirement: quick-session-access
roadmap: session-launcher-power-extend
roadmap_item: launch-preflight
status: approved
summary: 只读启动预检；launch 复用 block 判定；UI 展示 checks
tags: [launch, preflight, safety]
---

# launch-preflight 设计文档

## 0. 术语约定

| 术语 | 定义 | 防冲突 |
|------|------|--------|
| session_list_id | 列表稳定 `Session.id` | 非 CLI 原生 session_id |
| ops_ready | full scan 完成，sessions 含可用 delete_target | 缓存窗为 false |
| PreflightResult | 预检结果；未知 id 也 Ok 进 checks | 非 command 层 panic |
| source 探测 | 与 inspect 同源：File/Directory/SqliteRow | OpenCode=行，非 db 文件 |

## 1. 决策与约束

**需求摘要**：启动前只读检查 cwd / program PATH / 源；有 block 则 launch 失败且不写 wrapper。  
**成功标准**：坏 cwd 的 session 预检 ok=false 且 launch 返回中文错误；缓存窗源仅 warn 且不拦 launch。  
**明确不做**：不写 wrapper；不改 resume 命令；不做全局快捷键；不在缓存窗假装校验源。

**关键决策**：

1. 新模块 `launch_preflight`（新文件），纯函数 `preflight_session` + command `preflight_launch`
2. `launch_session` 必须先调用同一预检；任一 block → `Err` 不启动
3. 未知 id → `Ok(PreflightResult{checks:[session_not_found block]})`，不 command Err
4. program 解析经可注入 `ResolveProgram`；**生产实现复用** launcher 内 login PATH 缓存（`CACHED_LOGIN_PATH` / 等价抽出），禁止只用 `std::env::var("PATH")` 与 wrapper 分叉
5. **共享源探测**（硬交付，与 health 共用，禁止两套 IO）：

```text
// 建议路径 src-tauri/src/session_source.rs（名称可微调，语义不可变）
fn check_session_source(session: &Session, ops_ready: bool) -> SourceCheck
// SourceCheck: Unverified | Missing | Present { approx_bytes: Option<u64>, size_capped: bool }
// OpenCode：强制按 CliType::OpenCode 做「行是否存在」探测；
//   禁止 match delete_target.kind==File 或 path.exists(opencode.db) 代替行
// File/Directory：path 存在性 + kind 匹配
// ops_ready=false 或无足够元数据 → Unverified
```

6. preflight 与 inspect **必须** import 上述同一函数；本 feature 可先落地该模块，health 只复用

**复杂度**：走桌面工具默认档位，无偏离。

## 2. 名词与编排

### 2.1 名词层

**现状**：`preview_launch_command` / `launch_session` 校验 CommandSpec，不查源、不查 PATH 上 program 是否可解析。

**变化**（roadmap §4.1 硬约束）：

```text
PreflightResult { sessionListId, ok, checks[], preview: LaunchCommandPreview|null }
PreflightCheck { code, severity: block|warn, message }
codes: session_not_found | cwd_missing | cwd_not_dir | program_not_found
     | source_missing | source_unverified | invalid_session_id | invalid_spec
```

判定矩阵与源表见 roadmap §4.1（本 design 不得改语义）。

### 2.2 编排层

```text
UI 点预检/启动
  → preflight_launch(session_list_id)
  → find session + ops_ready
  → preflight_session(...)
  → 返回 checks + 可选 preview（与 preview_launch_command 同组装路径）

launch_session
  → 同一 preflight_session
  → 有 block → Err(message)
  → 无 block → 现有 launch 路径
```

**流程级约束**：只读；preview 失败可 null；block message 中文用户可读。

### 2.3 挂载点清单

1. `session_source::check_session_source`（共享，health 复用）  
2. Tauri command `preflight_launch` + lib 注册  
3. `AppState.launch_session` 入口门闩  
4. 前端启动前/失败展示 checks  
5. cargo 单测：矩阵 + OpenCode 行 / 缓存窗 warn + PATH resolver

### 2.4 推进策略

先纯函数 + 测试锁矩阵 → command → launch 挂接 → UI。

### 2.5 结构健康度

**做微重构？** 否强制。预检逻辑进**新文件**，不往 `state/mod.rs` 继续堆长函数；仅在 launch 入口加一行调用。

## 3. 验收契约

| 场景 | 期望 |
|------|------|
| cwd 不存在 | block + launch 失败 |
| ops_ready=false | source_unverified warn，可 launch |
| OpenCode 行已删、db 在 | source_missing block（按行） |
| 未知 id | ok=false，session_not_found |
| 正常 session | ok=true，可 launch |

**验证**：`cd src-tauri && cargo test --lib`；`pnpm build`；手工坏 cwd smoke。

## 4. 与架构文档

扩展 launch 安全口径：除 CommandSpec 外增加 preflight block 门闩。收口条目回写 ARCHITECTURE。
