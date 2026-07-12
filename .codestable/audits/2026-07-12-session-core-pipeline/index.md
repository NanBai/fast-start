---
doc_type: audit-index
audit: 2026-07-12-session-core-pipeline
scope: Session 核心链路（scanner/launcher/security/session_delete/state/commands + App/hooks/sessionUtils）
created: 2026-07-12
status: active
total_findings: 12
resolved_p1: 8
dimensions: [bug, security, performance, maintainability, arch-drift]
related_prior_audit: 2026-06-19-fullstack-audit
---

# session-core-pipeline 审计报告

## 范围

用户确认范围（非全仓盲扫）：

| 区域 | 路径 |
|---|---|
| 扫描 | `src-tauri/src/scanner.rs`、`scanner/{codex,claude_code,cursor,grok_build}.rs` |
| 启动 | `launcher.rs`、`security.rs` |
| 删除 | `session_delete.rs` |
| 编排 | `state.rs`、`commands.rs`、`models.rs` |
| 前端 | `src/App.tsx`、`hooks/useSessions.ts`、`hooks/usePreferences.ts`、`lib/sessionUtils.ts`、`types.ts` |

**排除**：port_monitor、样式、纯构建产物。

**维度**：bug / security / performance / maintainability / arch-drift（全五维）。

**对照**：`.codestable/attention.md`、`ARCHITECTURE.md`、`AGENTS.md`、compound learnings、2026-06-19 fullstack-audit（部分项已修复，见总评）。

## 总评

共 **12** 条：P0 **0**、P1 **7**、P2 **5**。主路径设计仍然扎实——前端只传稳定 `Session.id`、`delete_target` 不进 JSON、program 白名单、删除 root 校验、wrapper 统一注入。历史审计里「sqlite3 子进程」「CSP null」「Terminal 直注业务命令」等项已明显收敛。

当前最值得优先处理的是：

1. **Cursor cwd 提取**（空格截断 + 首个 blob 抢先）——直接导致 resume 静默失败或错误目录  
2. **Codex 单行解析失败拖垮整 CLI**——列表整段空  
3. **ARCHITECTURE 仍写三家 CLI**——与已落地的 Grok Build 脱节  

启动 PATH 的 login-shell 方案解决了 node/grok 找不到，但引入 stdout 污染与延迟，属于下一轮启动稳定性重点。

## 发现清单

| # | 性质 | 严重度 | 置信度 | 标题 | 文件 |
|---|---|---|---|---|---|
| 1 | bug | P1 | high | Cursor Workspace Path 遇空格截断 | [finding-01.md](finding-01.md) |
| 2 | bug | P1 | high | Codex 单行 JSON 损坏拖垮整 CLI 扫描 | [finding-02.md](finding-02.md) |
| 3 | bug | P1 | high | Cursor 缺时间戳时 last_active_at=now() 永久「最近」 | [finding-03.md](finding-03.md) |
| 4 | bug | P1 | medium | login shell PATH 解析易被 stdout 污染 | [finding-04.md](finding-04.md) |
| 5 | security | P1 | medium | Cursor cwd 取 blobs 中第一个可 canonicalize 的 Workspace Path | [finding-05.md](finding-05.md) |
| 6 | security | P2 | high | validate_command_spec 只校验 args.last() | [finding-06.md](finding-06.md) |
| 7 | performance | P1 | high | Codex/Claude 全量 read_to_string 大 jsonl | [finding-07.md](finding-07.md) |
| 8 | performance | P1 | high | Cursor 每个 chat 全表 SELECT blobs | [finding-08.md](finding-08.md) |
| 9 | performance | P2 | medium | 每次 launch 同步 zsh -lc 解析 PATH | [finding-09.md](finding-09.md) |
| 10 | maintainability | P2 | high | command 参数 session_id 实为列表稳定 Session.id | [finding-10.md](finding-10.md) |
| 11 | maintainability | P2 | high | 高风险大文件持续膨胀（state/App/launcher） | [finding-11.md](finding-11.md) |
| 12 | arch-drift | P1 | high | ARCHITECTURE 仍写三家 CLI，实现已是四家 | [finding-12.md](finding-12.md) |

## 按维度分布

| 性质 | P0 | P1 | P2 | 合计 |
|---|---|---|---|---|
| bug | 0 | 3 | 1 | 4 |
| security | 0 | 1 | 1 | 2 |
| performance | 0 | 2 | 1 | 3 |
| maintainability | 0 | 0 | 2 | 2 |
| arch-drift | 0 | 1 | 0 | 1 |
| **合计** | **0** | **7** | **5** | **12** |

## 与 2026-06-19 审计关系

| 旧 finding | 现状（本次核验） |
|---|---|
| Cursor 每 chat 起 sqlite3 子进程 | **已缓解**（rusqlite 进程内），但全表 blobs 读放大仍在 → 见 finding-08 |
| Codex 简介只扫前 64 行 | **已修复**（有 late-summary 测试） |
| Terminal 直注业务命令 / CSP null | **已修复**（wrapper + 非 null CSP） |
| 前端主组件过大 | **仍在**（App 现混 port）→ finding-11 |
| fixture 依赖本机数据 | **明显改善**（各 scanner 有 tempfile fixture） |

旧 audit 目录保持 `active`（范围不同，未整体 superseded）。

## 下一步建议

- **P1 优先 `cs-issue`**：finding-01、02、03、05（启动/列表正确性）  
- **P1 文档**：finding-12 用架构刷新（`cs-docs` / arch update），可与 Grok feature 收尾一起做  
- **P1/P2 启动稳定性**：finding-04、09（PATH 硬化 + 缓存）  
- **P1 `cs-refactor` 性能**：finding-07、08  
- **P2 可维护性**：finding-06、10、11 排期重构  

选中某条 finding 后可直接说「修 finding-0N」，本 run 可路由到 `cs-issue` / `cs-refactor`。

## P1 处置状态（2026-07-12）

| # | 状态 | 摘要 |
|---|---|---|
| 01 Cursor 空格路径 | resolved | 路径边界改为 JSON `\n`/`"`；允许多空格 |
| 02 Codex 坏行拖垮 | resolved | 坏行 skip + 流式读 |
| 03 Cursor now() 时间戳 | resolved | 回退 meta/chat mtime |
| 04 PATH stdout 污染 | resolved | 取末行 + 形态校验 + fallback |
| 05 Cursor 首 blob 抢先 | resolved | 多候选取最长路径；过滤相关 blob |
| 07 全量 read_to_string | resolved | Codex/Claude BufReader 流式 |
| 08 全表 blobs | resolved | WHERE 过滤 + ORDER BY length + LIMIT 64 |
| 12 ARCHITECTURE 三家 | resolved | v1.3 四 CLI + Grok + 安全白名单 |

## P2 处置状态（2026-07-12）

| # | 状态 | 摘要 |
|---|---|---|
| 06 args 只校 last | resolved | 按 CLI 形状校验 `resume`/`--resume` + id |
| 09 每次 zsh -lc | resolved | 进程内 `OnceLock` 缓存 PATH，wrapper 只注入 |
| 10 session_id 命名误导 | resolved | 参数改 `sessionListId` / `session_list_id` |
| 11 大文件膨胀 | resolved | launcher 拆 terminals；preferences / state/ports 抽出 |
