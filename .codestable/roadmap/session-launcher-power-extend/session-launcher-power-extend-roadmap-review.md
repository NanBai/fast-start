---
doc_type: roadmap-review
slug: session-launcher-power-extend
status: passed
reviewer: independent-task-agent
rounds: 3
created: 2026-07-13
last_reviewed: 2026-07-13
---

# session-launcher-power-extend 规划审查

## 审查方式

- 主 agent 起草 roadmap/items（draft）
- 独立 Task agent（plan / read-only）三轮审查：round1 `changes-requested` → 契约锁死 → round2 剩 OpenCode 源语义 → 修订 → round3 `passed`
- 主 agent 本地事实核验：`ops_ready` / `delete_session` OpenCode 分支 / `TerminalLauncher` trait / port terminate / grok status·backups / `clean_summary`

## Verdict

**passed** — 无 unresolved blocking / important finding；可交用户确认 epic 规划。

## 范围与目标

| 项 | 结论 |
|----|------|
| 轴 1/3/4 | 日常爽感 + Port/Grok 加深 + macOS 扩展面（终端 + CLI 编译期契约） |
| 明确不做 | Windows、托盘、动态插件、全文索引、回收站、Keychain、新 CLI 产品类型 |
| 与 next-wave | 加深而非重做；next-wave items 全 done |
| 最小闭环 | `launch-preflight` 合理 |

## 已关闭的重要锁点（实现硬约束）

1. **Preflight 矩阵**：未知 id → `Ok+session_not_found`；缓存窗 `source_unverified=warn` 不拦 launch；block 集合写死；launch 必须复用同一判定  
2. **源探测同源**：File / Directory / **SqliteRow(OpenCode=行)**；禁止用 `opencode.db` 文件存在性代替行；实现时按 `cli_type` 分发勿只 match `delete_target.kind`  
3. **Bulk delete**：走 `AppState::delete_session` 全路径；禁只调 `delete_session_target`；partial success；上限 50；整批后 **额外** sanitize recent；启动前须修订 delete requirement  
4. **Health / disk**：有界 du；OpenCode `approxBytes=null`  
5. **Port by project**：UI-only + 既有 terminate；无新 command  
6. **Grok health**：`issues[]` delta；禁止 secret/备份绝对路径  
7. **Summary**：仅 `clean_summary` ≤160  
8. **Terminal trait**：`launch(&CommandSpec, LaunchMode)` + `supports_tab`

## Residual risk（不挡通过）

- 批量删除不可撤销 → UI 必展示 failures  
- 按项目 terminate 依赖 workingDirectory 质量  
- 新终端 CI 难真 smoke → 可 drop item  
- 同步 inspect IO 需有界  
- `App.tsx` / state 体积：新代码优先拆文件  

## Phase 4 自查（主 agent）

| # | 结果 |
|---|------|
| 模块职责 | 通过 |
| 接口可执行 | 通过（三轮后） |
| slug 不冲突 | 通过 |
| DAG | 通过 |
| 最小闭环 | 通过 |
| 明确不做 | 通过 |
| 与 arch/req | bulk 与 delete req 冲突已闸门化 |
| 可证伪 / 原子性 / 收口 | 通过 |
| Goal Coverage | 通过 |
| yaml 校验 | 通过 |

## 用户确认前注意

- 确认后主文档 `status: draft` → `active`  
- 若轴 4 实际要 Windows 一等，本 roadmap 应 superseded 另开  
- `session-bulk-delete` 实现前需 `cs-req` 修订批量删除边界  
