---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "performance-02"
nature: performance
severity: P1
confidence: high
suggested_action: cs-refactor
status: resolved
---

# Finding 08：Cursor 每个 chat 全表 SELECT blobs

## 速答

已从 sqlite3 子进程改为 rusqlite，但仍 `SELECT data FROM blobs` 无过滤/LIMIT，大对话 blob 全量进进程。本机约 256 个 store.db。

## 关键证据

- `src-tauri/src/scanner/cursor.rs:139` — `SELECT data FROM blobs;`  
- 历史 audit finding-02 已修子进程问题；读放大仍在  
- 找到 cwd 后可 break，但已加载的巨大 row 成本已付

## 影响

Cursor 重度用户刷新明显变慢。

## 修复方向

SQL 侧限制大小/分页、优先小 blob 或已知 meta 表；找到 cwd+query 即停并避免加载全文。

## 建议动作

`cs-refactor`。

## 处置

2026-07-12 已在代码中修复（P1 批量）。详见对应 scanner/launcher/ARCHITECTURE 改动。
