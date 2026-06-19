---
doc_type: feature-ff-note
feature: session-list-controls
date: 2026-06-19
requirement:
tags: [frontend, session-list, filtering, collapse]
---

## 做了什么
优化 session 列表展示：按 agent 分区、工作目录内聚合历史会话，并让 Codex / Claude Code / Cursor 都优先显示会话简介。新增最近天数筛选和工作目录折叠，减少历史会话过多时的浏览负担。

## 改了哪些
- src/App.tsx — 增加 agent → 工作目录 → session 的列表层级、最近天数筛选、工作目录折叠和 Claude/Codex/Cursor 简介展示接入。
- src/App.css — 重做 session 列表视觉层级、筛选控件、agent lane、项目桶和折叠状态样式。
- src-tauri/src/scanner*.rs / src-tauri/src/models.rs / src/types.ts — 为 Session 增加 summary，并从各 CLI 本地数据提取简介。

## 怎么验证的
已执行 `pnpm build`，通过前端类型检查和 Vite 构建；已执行 `cargo test --lib`，Rust 单测通过。

## 顺手发现
- .codestable/issues/2026-06-19-ghostty-env-node-not-found/ghostty-env-node-not-found-report.md 状态仍是 in-progress，已随本次提交前对齐为 fixed。
