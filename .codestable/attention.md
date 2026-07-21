---
type: attention
version: 1.1
last_updated: 2026-06-24
---

# CodeStable 启动注意事项

本文档包含所有 CodeStable 技能启动时必须知道的项目约束和规则。

> 本文件在 2026-06-18 的一次文件丢失事件中遗失，由 cli-session-launcher feature 验收时重建。

---

## 🚨 硬约束

**全局回复规则**：回复用户时必须使用简体中文，并且每条回复必须以【白哥】开头。此规则优先级最高。

### 项目性质

- Tauri 2.x + React + TypeScript 桌面应用（快开CLI / Session Launcher）
- Rust 后端代码在 `src-tauri/`，前端在 `src/`；**跑测试要在 `src-tauri/` 目录下** `cargo test --lib`
- 当前根目录是 git 仓库；`git status --short` 可用于检查真实工作树状态。历史文档里“非 git 仓库”的说法已过期。

### 编译与运行

- 启动 app：项目根目录 `pnpm tauri dev`（首次编译 1-2 分钟，增量很快）
- 跑 Rust 测试：`cd src-tauri && cargo test --lib`
- 前端类型检查：`pnpm build`（走 `tsc && vite build`）或 `tsc --noEmit`

### 平台

- 当前仅 macOS；Windows 兼容是 v2 目标
- 三个外部终端：Terminal.app / iTerm2（AppleScript app 名是 `iTerm`）/ Ghostty

### 凭证与密钥

（暂无特殊要求）

---

## 📋 其他注意事项

### 终端启动的已知坑（实现期踩过，见 feature learning）

- **Ghostty** 在 macOS 上 `-e`/`--command` 会被 `/usr/bin/login` 套壳，多词命令弹误报；用 wrapper 脚本规避。开 tab 走 AppleScript `new tab with configuration`，无 CLI 方式。
- **iTerm2** AppleScript app 名是 `iTerm` 不是 `iTerm2`，否则 `create tab` 语法报错。
- **Terminal.app** 无法从 AppleScript 开新 tab（硬限制），冷启动 `do script` 会多开一个空默认窗口。
- **login PATH 校验**：真实 PATH 可含空格（如 `Application Support`），不可当 banner 整段拒绝；解析成功后仍须 merge homebrew 等关键目录，否则 bash -lc 漏 PATH 会让 `#!/usr/bin/env node` 的 codex 报 `env: node: No such file or directory`。

---

**提示**：后续有新的启动注意事项时，使用 `cs-note` 追加到本文档。
