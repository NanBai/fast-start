---
doc_type: audit-finding
audit: 2026-06-19-fullstack-audit
finding_id: "performance-02"
nature: performance
severity: P1
confidence: high
suggested_action: cs-refactor
status: open
---

# Finding 02：Cursor 扫描对每个 chat 启动一次 sqlite3 子进程

## 速答

Cursor scanner 在双层目录遍历中，每遇到一个 chat 就启动一次 `sqlite3` 子进程读取 `store.db`，session 数量上来后扫描成本会线性放大且进程启动开销很高。

## 关键证据

- `src-tauri/src/scanner/cursor.rs:41` — 遍历 `~/.cursor/chats/<workspace-hash>`。
- `src-tauri/src/scanner/cursor.rs:46` — 对每个 hash 下的 chat 继续遍历。
- `src-tauri/src/scanner/cursor.rs:64` — 每个 chat 调用 `extract_workspace_path(&chat_dir.join("store.db"))`。
- `src-tauri/src/scanner/cursor.rs:111` — `Command::new("sqlite3")` 每次启动一个外部进程。
- `src-tauri/src/scanner/cursor.rs:113` — `SELECT data FROM blobs;` 读取整张 blobs 表，再由 Rust 字符串扫描。

## 影响

Cursor 历史会话越多，扫描越慢；同时 `sqlite3` 是否在用户 PATH 中也会影响可用性。该成本会叠加到 finding-01 的启动阻塞上。

## 修复方向

优先用 Rust SQLite 库直接读库，或至少把查询改成只筛包含 `Workspace Path:` 的行；同时考虑限制并发和缓存 workspace hash 到 cwd 的映射。

## 建议动作

`cs-refactor`，因为这是扫描实现优化，预期不改变 UI 与命令语义。
