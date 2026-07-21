---
doc_type: issue-fix
slug: login-path-space-node-not-found
severity: high
status: fixed
tags: [launcher, ghostty, path, packaging]
created: 2026-07-15
fix_date: 2026-07-15
---

# 修复记录：点击 session 报 `env: node: No such file or directory`

## 根因

打包 `.app` + Ghostty 启动 codex 时，wrapper 注入的 PATH 里没有 `node`（`/opt/homebrew/bin`）。

触发链：

1. 用户默认 shell 是 zsh；login PATH 含 `/Users/.../Application Support/...`（合法空格）。
2. `is_plausible_path` 旧实现把**任意空白**都当 banner 拒绝 → zsh PATH 被丢弃。
3. 回退到 `bash -lc` 的 PATH：在 launchd 极简环境里常**不含 homebrew**。
4. 该 PATH 仍含 `~/.local/bin`，能找到 `codex`，但 codex shebang 是 `#!/usr/bin/env node` → `env: node: No such file or directory`。

与 2026-06-19 `ghostty-env-node-not-found` 同症状、不同根因：那次是 wrapper 未解析 login PATH；这次是校验过严 + 未把关键目录 merge 进「看起来成功」的 PATH。

## 改动

单文件：`src-tauri/src/launcher/mod.rs`

1. `is_plausible_path`：允许 PATH 条目内空格；仅拒绝「含空格且无 `:`」的整句 banner。
2. `resolve_login_path_once`：login 解析成功后仍 `merge_critical_path_dirs`（homebrew / `~/.local/bin` 等）。
3. `fallback_path_string` 复用同一 merge，去掉无效的 `.nvm/versions/node` 目录级条目。
4. 单测：Application Support 路径可接受；merge 补 homebrew 且不重复。

## 验证

- `cd src-tauri && cargo test --lib launcher::`：7/7 通过。
- `cargo test --lib`：全量 lib 测试通过。
- 模拟 launchd PATH + 修复后 PATH 规则：`env node` → v26.5.0，`codex` 可解析。
- Ghostty `open -na ... -e` 诊断 wrapper：NODE=/opt/homebrew/bin/node。

## 部署

需重新构建/安装 Session Launcher.app（或 `pnpm tauri dev`）后生效；旧安装包仍带旧逻辑。

## 关联

- `.codestable/issues/2026-06-19-ghostty-env-node-not-found/`
- attention.md 终端 PATH 校验坑
