---
doc_type: decision
category: convention
slug: ghostty-launch-via-wrapper-script
title: Ghostty 命令执行统一走 wrapper 脚本，不直接用 -e/--command 传多词命令
status: active
created: 2026-06-18
source: feature 2026-06-17-cli-session-launcher
tags: [ghostty, macos, launcher, convention]
---

# Ghostty 命令执行统一走 wrapper 脚本

## 背景

Session Launcher 需要在 Ghostty 里 cd 到目录并执行 `codex resume <id>` 这类多词命令。Ghostty 在 macOS 上启动终端命令有多种入口（`-e`、`--command`、AppleScript `command` 配置项），但都绕不开同一个平台机制。

## 决定

**所有要在 Ghostty 里执行的命令，统一生成一个临时 wrapper 脚本，让 `-e` / `--command` / AppleScript `command` 只执行该脚本的单一路径，绝不直接传多词命令字符串。**

wrapper 脚本形如（写到 `$TMPDIR/fast-start-ghostty/run-<pid>.sh`，权限 `0700`）：

```sh
#!/bin/sh
export PATH=<原 PATH>:~/.local/bin   # 补 ~/.local/bin，codex/claude 装在这
cd '<cwd>' && exec <program> <args>
```

调用：
- 有窗口开 tab：AppleScript `new tab with configuration {command:<脚本路径>, initial working directory:<cwd>}`
- 无窗口：`open -na Ghostty.app --args -e <脚本路径>`

## 理由

Ghostty 在 macOS 上**一定把 `-e`/`--command` 的命令包进 `/usr/bin/login -flp <user>`**（官方文档 `abnormal-command-exit-runtime` 附近一句带过："shell processes are launched via the login command"）。`login` 解析多词命令（`codex resume abc`）时，会把 `resume`/`abc` 当成自己的参数（用户名等）解析失败，弹红色误报：

```
Ghostty failed to launch the requested command:
/usr/bin/login -flp xb codex resume abc
```

命令其实经 PTY 执行了，但误报扰人且让用户以为失败。传**单个脚本路径**时 login 看到的是单个可执行文件，不解析多词参数，无误报。

顺带两个必须一起做的点：
- **补 PATH**：Ghostty tab 不走 login shell，默认 PATH 不含 `~/.local/bin`（codex/claude/cursor-agent 装在那），会 `command not found`。wrapper 里必须 `export PATH` 补上。
- **`-e` 自动设 `quit-after-last-window-closed=true`**：agent 退出后 Ghostty 干净退出不留孤儿进程（不补 wrapper 直接 `-e` 多词命令时，命令跑着但关窗会"复活"，因为 Ghostty 进程被 login 持有不退出）。这是 wrapper 方案的额外收益。

## 考虑过的替代方案

- **直接 `open -na Ghostty.app --args -e codex resume <id>`**：放弃。经 login 套壳弹误报，且多窗口/孤儿问题。
- **`--command="codex resume <id>"`**（文档说走 `/bin/sh -c`）：放弃。macOS 上**仍然套 login**（文档没写但实测如此），同样误报。
- **`--input="codex resume <id>\n"` 发按键到 login shell**：放弃。命令能跑，但 login 误报仍在（启动即套 login），没解决问题。
- **`direct:` 前缀（`--command="direct:..."`）绕过 shell**：未深入。理论上可能绕开 login，但 `direct:` 不支持含空格/`~` 的参数解析，对 `codex resume <uuid>` 这种多参数不可靠，不如 wrapper 稳。

## 后果与约束

- **任何后续 feature 调 Ghostty 执行命令，都必须走 wrapper 机制**，不能图省事直接 `-e`/`--command` 多词命令。否则立即重踩 login 误报。
- wrapper 脚本是临时文件（`$TMPDIR` 下，按 pid 命名），不需手动清理；系统重启自动清。
- 这是 macOS 专属约束（Ghostty 的 login 套壳是 macOS 平台行为）。未来支持 Linux/Windows 时需重新评估（Linux 上 `-e`/`--command` 可能不套 login）。

## 相关文档

- learning（含完整试错过程）：`.codestable/compound/2026-06-18-learning-macos-terminal-launch-pitfalls.md` 坑 1
- 架构（终端拉起策略）：`.codestable/architecture/ARCHITECTURE.md` "终端拉起策略"节
- 代码：`src-tauri/src/launcher.rs::write_ghostty_wrapper` / `GhosttyLauncher::launch`
