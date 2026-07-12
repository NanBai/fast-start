---
doc_type: feature-ff-note
feature: add-opencode-cli-support
date: 2026-07-12
requirement:
tags: [opencode, scanner, cli-integration, session-launcher]
---

## 做了什么

为 Session Launcher 增加 OpenCode CLI 支持：扫描本机 `opencode.db` 历史 session，列表展示，一键 `opencode --session <id>` 恢复，删除时只删 SQLite 中对应 session 行。

## 改了哪些

- `src-tauri/src/scanner/opencode.rs` — 新 scanner + SQLite 行删除
- `src-tauri/src/models.rs` / `scanner.rs` / `security.rs` / `state/mod.rs` / `launcher` — CliType、command_spec、白名单、`--session` 形状、删除分支、PATH
- 前端 `types` / `AgentGroup` / `BrandMark` / styles / App 副标题
- README / user/dev docs / AGENTS / ARCHITECTURE

## 怎么验证的

- `cd src-tauri && cargo test --lib`（54 tests，含 opencode fixture）
- `pnpm build` 通过

## 顺手发现（可选，不阻塞）

- OpenCode 二进制常在 `~/.bun/bin`；已加入 PATH fallback
- 删除走 SQL 而非 `opencode session delete` 子进程，避免额外 shell 面
