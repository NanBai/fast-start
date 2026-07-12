---
doc_type: feature-ff-note
feature: add-grok-build-support
date: 2026-07-12
requirement:
tags: [grok-build, scanner, cli-integration, session-launcher]
---

## 做了什么

为 Session Launcher 增加 Grok Build CLI 支持：扫描 `~/.grok/sessions` 历史 session，列表展示，一键在外部终端 `grok --resume <id>` 恢复，并支持删除对应 session 目录。

## 改了哪些

- `src-tauri/src/models.rs` — `CliType::GrokBuild`（序列化 `grok-build`）
- `src-tauri/src/scanner/grok_build.rs` — 新 scanner：读 `summary.json`，cwd 优先 `info.cwd`，删除整目录
- `src-tauri/src/scanner.rs` — 注册 scanner；`command_spec` → `grok --resume <id>`、`cd: true`
- `src-tauri/src/security.rs` — 白名单加入 `grok`
- `src-tauri/src/launcher.rs` — wrapper 始终 prepend `~/.grok/bin`（login PATH 漏装也能找到 grok）；fallback PATH 同步补充
- `src/types.ts` / `AgentGroup.tsx` / `BrandMark.tsx` / `session-list.css` / `App.tsx` — 前端枚举、标签、分组样式
- `README.md` / `docs/user/session-launcher.md` / `AGENTS.md` — 文档同步

## 怎么验证的

- `cd src-tauri && cargo test --lib`（40 tests，含 grok_build / wrapper PATH）全过
- `pnpm build` 通过
- 用户确认 UI 扫描/启动效果 OK [2026-07-12]
- code review `add-grok-build-support-review.md` status: passed（subagent）

## 顺手发现（可选，不阻塞）

- 工作树另有 port-monitor 未提交改动，与本次无关；scoped-commit 应只收 Grok 相关文件
