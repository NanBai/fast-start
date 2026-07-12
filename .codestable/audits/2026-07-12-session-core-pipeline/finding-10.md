---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "maintainability-01"
nature: maintainability
severity: P2
confidence: high
suggested_action: cs-refactor
status: resolved
---

# Finding 10：command 参数 session_id 实为列表稳定 Session.id

## 速答

Tauri command / `find_session` 参数名 `session_id` 匹配的是 `Session.id`（列表稳定 UUID），不是 CLI 原始 `Session.session_id`，命名易导致误用。

## 关键证据

- `src-tauri/src/state.rs:130-140` — `find(|session| session.id == session_id)`  
- `commands.rs` `launch_session(session_id)` / `delete_session(session_id)`  
- 前端正确传 `session.id`（`useSessions.ts`），但后端命名误导

## 影响

后续改 command 或调试时极易传 CLI raw id → 「未找到」；文档/日志混淆。

## 修复方向

重命名为 `id` / `list_session_id`，前后端与 AGENTS 契约对齐。

## 建议动作

`cs-refactor`。

## 处置

2026-07-12 P2 批量已修。
