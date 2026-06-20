---
doc_type: audit-finding
audit: 2026-06-19-fullstack-audit
finding_id: "maintainability-03"
nature: maintainability
severity: P1
confidence: high
suggested_action: cs-refactor
status: open
---

# Finding 03：扫描测试依赖本机真实 CLI 数据，容易假绿

## 速答

现有测试会读取开发者本机的 Codex / Claude / Cursor 历史数据；数据不存在时部分测试直接跳过，CI 或新机器上无法稳定验证核心解析逻辑。

## 关键证据

- `src-tauri/src/scanner.rs:121` — 测试直接调用 `CodexScanner.scan_sessions()`，依赖 `~/.codex/sessions`。
- `src-tauri/src/scanner.rs:122` — 测试直接调用 `ClaudeCodeScanner.scan_sessions()`，依赖 `~/.claude/projects`。
- `src-tauri/src/scanner.rs:133` — 只要求 codex 或 claude 其中一个扫描成功，覆盖面不稳定。
- `src-tauri/src/scanner/cursor.rs:156` — Cursor 测试硬编码本机 `~/.cursor/chats/.../store.db` 路径。
- `src-tauri/src/scanner/cursor.rs:160` — 数据不存在时 `return`，测试通过但没有验证任何行为。

## 影响

解析格式变更、目录遍历回归、Cursor workspace 提取失败等问题可能在测试中漏掉。尤其当前功能刚加入多 CLI 简介解析，测试假绿会降低后续改动信心。

## 修复方向

把 scanner root 抽成可注入路径，测试用临时目录和最小 jsonl / sqlite fixture；Cursor 的 `extract_workspace_path` 用临时 sqlite 数据库验证。

## 建议动作

`cs-refactor`，因为这是测试结构和依赖注入改造，不应改变运行行为。
