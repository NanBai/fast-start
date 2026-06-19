---
doc_type: issue-fix
slug: ghostty-env-node-not-found
severity: high
status: fixed
tags: [launcher, ghostty, packaging, path]
created: 2026-06-19
fix_date: 2026-06-19
---

# 修复记录：Ghostty 启动 codex session 报 `env: node: No such file or directory`

## 根因

打包成 `Session Launcher.app` 后由 **launchd** 启动，GUI 进程的 PATH 是极简版
`/usr/bin:/bin:/usr/sbin:/sbin`，不含 node 所在目录。

Ghostty 的 wrapper 脚本（`src-tauri/src/launcher.rs::write_ghostty_wrapper`）原本用
`std::env::var("PATH")`（即 launchd 的极简 PATH）+ 兜底 `~/.local/bin`。
codex 是 `#!/usr/bin/env node` 脚本，`~/.local/bin/codex` 找到了，但其 shebang 在
PATH 里找不到 `node`（node 在 `/opt/homebrew/bin`），于是 `env: node: No such file or directory`。

> 注：Terminal.app / iTerm2 不受影响——它们经 AppleScript 在终端自己的 login shell 里执行，PATH 由终端补全。仅 Ghostty wrapper 受限。

## 修复方案

与用户确认采用「**登录 shell 解析 PATH**」：wrapper 脚本不再继承进程 PATH，
改为调用用户的登录 shell（`zsh -lc` / `bash -lc`）解析出真实 PATH，跟随用户实际环境
（node 装哪都能找到，覆盖 nvm/volta/asdf/homebrew/usr-local 等各种安装方式）。

解析失败时兜底到 `fallback_path_string()`：进程 PATH + 常见 node/CLI 安装目录，绝不为空。

## 改动

**单文件单点**：`src-tauri/src/launcher.rs`

1. `write_ghostty_wrapper`：PATH 拼装逻辑改为脚本内联登录 shell 解析。
2. 新增 `fallback_path_string()` 辅助函数（进程 PATH + `/opt/homebrew/{bin,sbin}` +
   `/usr/local/{bin,sbin}` + `~/{.local/bin,.cargo/bin,.nvm/versions/node,.volta/bin,.asdf/shims}`）。

生成的 wrapper（关键顺序：**函数定义必须在调用之前**，否则 POSIX shell 下 `command not found`）：

```sh
#!/bin/sh
resolve_login_path() {
  for sh in zsh bash; do
    command -v "$sh" >/dev/null 2>&1 || continue
    p=$($sh -lc 'printf %s "$PATH"' 2>/dev/null) && [ -n "$p" ] && printf %s "$p" && return
  done
  printf %s '<fallback>'
}
PATH=$(resolve_login_path)
export PATH
cd '<cwd>' && exec codex resume '<id>'
```

## 踩坑修正

初版把 `PATH=$(resolve_login_path)` 写在函数定义**之前**，POSIX shell 解析时报
`command not found`、PATH 变空，bug 复发。端到端验证时发现，已修正为函数定义在前。
**教训**：生成的 shell 脚本必须本地实跑验证，不能只靠 Rust 测试断言子串。

## 验证

- `cargo test --lib`：5/5 通过（含 `ghostty_wrapper_cd_then_execs_command`，确认仍含 `cd` + `exec`）。
- **端到端实跑**：模拟 launchd 极简 PATH（`PATH="/usr/bin:/bin:/usr/sbin:/sbin"`）执行真实 wrapper →
  resolve 出的 PATH 含 `/opt/homebrew/bin`，`env node` → `v26.3.0`（修复前为 `No such file or directory`）。
- 覆盖三家 CLI：codex / claude / cursor-agent 同为 `env node` 脚本，同一修复生效。

## 关联

- attention.md「Ghostty `-e` 被 `/usr/bin/login` 套壳」已知坑的 PATH 维度复发。
- 该修复对未来新增 CLI agent 同样生效（只要它们是 login shell PATH 能找到的可执行）。
