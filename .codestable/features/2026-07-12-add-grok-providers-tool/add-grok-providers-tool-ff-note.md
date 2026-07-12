---
doc_type: feature-ff-note
feature: add-grok-providers-tool
date: 2026-07-12
requirement:
tags: [grok, providers, config-switch, tool-page]
---

## 做了什么

融合 grok-build-switch 方案 A：在 Session Launcher 增加第三工具页 **Grok**，用供应商档案管理 `~/.grok/config.toml` 的上游切换（启用时自动备份，可还原）。

## 改了哪些

- `src-tauri/src/grok_provider/` — profile / store / config TOML 行级 apply / switcher
- `commands.rs` / `lib.rs` — Tauri 命令与状态挂载
- 前端 `AppTool=providers`、`ProvidersWorkspace`、`useGrokProviders`、`providers.css`
- 数据目录与 grok_switch 对齐：`~/.grok_switch/profiles.json` + `backups/`

## 怎么验证的

- `cargo test --lib`（含 grok_provider 配置 apply / activate 测试）
- `pnpm build` 通过

## 顺手发现（可选，不阻塞）

- 未移植：托盘、开机自启、HTTP 拉模型列表、完整 config 编辑器
- profiles 含 API Key 明文（与原 grok_switch 相同）；后续可做脱敏展示策略
