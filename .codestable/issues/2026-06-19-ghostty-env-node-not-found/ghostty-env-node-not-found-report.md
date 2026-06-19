---
doc_type: issue-report
slug: ghostty-env-node-not-found
severity: high
status: fixed
tags: [launcher, ghostty, packaging, path]
created: 2026-06-19
---

# 问题报告：打包 .app 用 Ghostty 启动 codex session 报 `env: node: No such file or directory`

## 现象

从 **打包安装的 app**（`Session Launcher.app`，即 release 构建产物）选择「新窗口」或「新标签页」+ 终端选 **Ghostty**，点击启动 codex 的 session，Ghostty 窗口闪现并报错，窗口无法保持：

```
Last login: Fri Jun 19 19:38:42 on ttys005
env: node: No such file or directory

Ghostty failed to launch the requested command:
/usr/bin/login -flp xb /var/folders/.../T/fast-start-ghostty/run-5358.sh
Runtime: 47 ms
Press any key to close the window.
```

新标签页同理（套了一层 `/bin/bash --noprofile --norc -c exec -l ...`，仍是同一个 wrapper 脚本）。

## 复现条件

- 必现路径：**release 打包的 .app**（launchd 启动）+ 终端选 Ghostty + 启动任一 codex session
- `pnpm tauri dev` 下大概率不复现：dev 是从交互 shell 起的，PATH 完整
- Terminal.app / iTerm2 未报此错：它们经 AppleScript 在终端自己的 login shell 里执行，PATH 由终端补全

## 影响范围

- 严重度：high —— Ghostty 是默认可选终端之一，打包版核心功能「启动 codex session」完全不可用
- 受影响 CLI：codex（已确认）；claude / cursor-agent 同样是 node 脚本，理论同病（未实测但同根因）

## 初判方向（待 analyze 确认）

报错 `env: node` 来自 codex 的 shebang `#!/usr/bin/env node`。打包 .app 由 launchd 启动，进程 PATH 是极简版（`/usr/bin:/bin:/usr/sbin:/sbin`），不含 node 所在目录。Ghostty 的 wrapper 脚本（`launcher.rs::write_ghostty_wrapper`）PATH 兜底只加了 `~/.local/bin`，没覆盖用户实际装 node 的位置（`/opt/homebrew/bin` 等），导致 `env node` 找不到 node。

## 已采证据

- `codex` → `/Users/xb/.local/bin/codex`，shebang `#!/usr/bin/env node`，指向 `~/.local/lib/node_modules/@openai/codex/bin/codex.js`
- `which node` → `/opt/homebrew/bin/node`（v26.3.0）
- 模拟 launchd PATH（`PATH="/usr/bin:/bin:/usr/sbin:/sbin:/Users/xb/.local/bin" /usr/bin/env node -v`）→ `env: node: No such file or directory`，与报错完全一致
- 追加 `/opt/homebrew/bin` 后 → `v26.3.0` 成功

## 路径

走**快速通道**：根因单一明确（wrapper PATH 缺 node 所在目录）、修复集中在 `launcher.rs::write_ghostty_wrapper` 的 PATH 拼装一处、无跨模块影响。经与用户确认，采用「登录 shell 解析 PATH」方案。

## 关联

- attention.md 已记录「Ghostty `-e`/`--command` 被 `/usr/bin/login` 套壳」是已知坑；本 bug 是该坑在 PATH 维度的复发。
