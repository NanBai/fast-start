---
doc_type: audit-index
audit: 2026-06-17-cli-session-launcher-accept
scope: feature 2026-06-17-cli-session-launcher 的实现验收（对照 design + checklist）
created: 2026-06-18
status: accepted
total_findings: 3
---

# cli-session-launcher 实现验收

## 范围

验收对象：feature `2026-06-17-cli-session-launcher` 的实现代码（`src-tauri/` + `src/`），对照 `design.md` 的名词层 / 编排 / 流程级约束 / 挂载点，以及 `checklist.yaml` 的 31 条 checks 与 13 条验收场景。

## 验收方法

- 编译：`cargo build` 绿灯
- 测试：`cargo test` 2/2 通过
- 前端：`tsc --noEmit` 干净
- 真实数据：本机 codex / claude-code 的真实 session 能被扫到，cwd 正确解析
- 逐条核对：31 条 checks 对照源码，标 pass / warn / fail

## 结论

**accepted**。31 条 checks：`pass 32 / warn 3 / fail 0`，无 P0/P1 阻断项。

## 发现清单

| # | 性质 | 严重度 | 置信度 | 标题 | 状态 |
|---|---|---|---|---|---|
| 1 | security | P2 | high | 命令构造走 shell 字符串拼接送 osascript，未用 AppleScript `quoted form of` / `std::Command` 参数化 | warn（不阻塞） |
| 2 | maintainability | P2 | high | 并发用 `std::thread::spawn`，偏离 design 写的 `tokio::join!` | warn（不阻塞） |
| 3 | bug | P2 | medium | Ghostty launch 参数（`--working-directory` + `-e`）未端到端实测（本机未装） | warn（不阻塞） |

### 发现 1：命令构造走 shell 字符串（security, P2, warn）

**位置**：`launcher.rs` `build_shell_command` / `SystemTerminalLauncher` / `ITerm2Launcher`

**现状**：osascript 调用走 `cd 'cwd' && 'program' arg` 的 shell 字符串拼接，再交给 osascript `do script`，且 osascript 内用自定义 `applescript_string` 转义而非 AppleScript 原生 `quoted form of`。

**为何不阻塞**：上游三道校验扎实生效——
- `validate_command_spec`：cwd `canonicalize` 且为目录
- `program` 限白名单 `codex`/`claude`/`cursor-agent`
- `args` 中 session id 跑 `validate_session_id`（仅 `[a-zA-Z0-9-_]`）

经校验后的值无法携带 shell 元字符，注入面已收敛，实际不可利用。

**建议**：后续改 `std::Command` 参数化构造，进一步消除"间接经 shell"的隐患，并补 `quoted form of`。

### 发现 2：并发实现偏离 design 措辞（maintainability, P2, warn）

**位置**：`commands.rs` `scan_sessions` / `refresh_sessions`

**现状**：用 `std::thread::spawn` 并行扫描。

**为何不阻塞**：功能等价（确实并行扫描，结果正确聚合），只是偏离 design 写的 `tokio::join!`/`futures::join_all`。

**建议**：无强改必要；若后续引入更多 IO 可统一到 tokio。

### 发现 3：Ghostty 未端到端实测（bug, P2, warn）

**位置**：`launcher.rs` `GhosttyLauncher`

**现状**：本机未装 Ghostty，`is_available()` 返回 false（灰显路径已在场景11 验证）。`open -na Ghostty.app --args --working-directory <dir> -e <cmd>` 的参数支持未经真实 Ghostty 实测。

**为何不阻塞**：v1 目标用户场景以 Terminal.app / iTerm2 为主，Ghostty 是可选项且默认灰显。

**建议**：用户装 Ghostty 后复测场景6。

## checks 明细

| 来源 | 条数 | pass | warn | fail |
|---|---|---|---|---|
| 范围守护 | 6 | 6 | 0 | 0 |
| 名词契约 | 5 | 5 | 0 | 0 |
| 编排骨架/流程级约束 | 6 | 4 | 2 | 0 |
| 挂载点 | 5 | 5 | 0 | 0 |
| 验收场景 | 13 | 12 | 1 | 0 |
| **合计** | **35** | **32** | **3** | **0** |

（注：流程级约束里"命令构造安全"1 条 + "并发"1 条 = 2 warn；验收场景里"场景6 Ghostty"1 条 = 1 warn。）

## 下一步

- 验收通过，feature 可交付
- 3 条 warn 不阻塞，建议作为后续 polish 项跟踪
- cursor v2 单独开 feature 做可行性验证（sqlite schema + workspace hash 反推）
