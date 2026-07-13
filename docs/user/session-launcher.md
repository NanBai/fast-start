---
doc_type: user-guide
slug: session-launcher
component: session-launcher
status: current
summary: 使用 Session Launcher 查找、收藏、启动和删除本地 AI CLI session，并管理本机开发端口
tags: [session-list, search, favorites, terminal, deletion, port-monitor]
last_reviewed: 2026-07-07
---

# 使用 Session Launcher 恢复工作现场

## 功能简介

Session Launcher 会扫描本机 Codex、Claude Code、Cursor、Grok Build 和 OpenCode 的历史 session，按 agent 或按项目目录整理成列表。你可以搜索目标 session、收藏高频项目、选择外部终端，并从当前列表直接恢复工作现场。应用还提供 Port 工具页，用于查看本机开发服务端口并关闭当前用户的残留进程。

## 前置条件

- 当前版本面向 macOS。
- 本机需要安装并使用过 Codex、Claude Code、Cursor Agent、Grok Build 或 OpenCode 中至少一种 CLI。
- 启动 session 前，需要安装至少一个可用终端：Terminal.app、iTerm2 或 Ghostty。
- 恢复 session 时，目标 CLI 命令需要能在终端里正常执行。
- Port 工具页依赖 macOS 自带的 `/usr/sbin/lsof`、`/bin/ps` 和 `/bin/kill`。

## 如何使用

### 启动应用

开发环境中运行：

```bash
pnpm tauri dev
```

应用打开后会自动扫描本机 session。扫描失败的 CLI 会在页面上显示错误，其他 CLI 的结果仍会展示。

顶部的 `Session` / `Port` / `Grok` 切换用于在历史 session、端口监控和 Grok 登录方式管理之间切换。`Cmd+K` 会聚焦当前工具页的搜索框（Grok 页除外）。

### Grok 工具页

- **官方账号**：清除 `config.toml` 中由 API 供应商写入的上游覆盖，回退使用 `grok login` 的 OAuth（`~/.grok/auth.json`）。切换后需新开 Grok 会话才生效。
- **API 供应商**：保存 Base URL / API Key / 模型档案到 `~/.grok_switch/profiles.json`，启用时写入 `~/.grok/config.toml` 并自动备份。
- **隐私保护**：一键合并本地遥测相关开关（不替代 Grok 账号侧 Coding data sharing 或 `/privacy`）。
- **置顶与排序**：卡片可置顶、可拖动调整顺序，偏好保存在本机 app 设置中。

### 切换列表视图

控制栏提供 **按 Agent** / **按项目** 切换：

- **按 Agent**（默认）：先按 CLI 分组，再在组内按项目目录聚合。
- **按项目**：同一 `projectDir` 下的多 CLI session 归到同一项目组；行内显示 Agent 标签。

视图偏好保存在本机，重启后保留。

### 查找 session

1. 在顶部搜索框输入项目名、项目路径、session 简介或 agent 名。
2. 列表会在当前“最近几天”范围内继续过滤。
3. 没有匹配结果时，列表区域会显示“没有匹配的 session”。
4. 清空搜索后，列表恢复原本分组和折叠状态。

快捷键：

- `Cmd+K`：聚焦搜索框。
- `Esc`：搜索框有内容时清空搜索；无内容时失焦。
- `↑` / `↓`：在搜索结果中切换活跃 session。
- `Enter`：启动当前活跃 session。

### 调整时间范围

使用“最近 1 天 / 3 天 / 7 天 / 14 天 / 30 天 / 全部”菜单控制列表范围。搜索只会在当前时间范围内继续收窄结果。

### 收藏项目

1. 展开某个 agent 分组。
2. 在项目 header 右侧点击星标按钮。
3. 收藏后的项目会在同一 agent 下排到非收藏项目之前。
4. 收藏偏好保存在本机，重启应用后仍会保留。

收藏粒度是项目目录，不是单条 session。

### 启动 session

1. 选择打开方式：新标签页或新窗口。
2. 选择终端：Terminal.app、iTerm2 或 Ghostty。
3. 点击 session 行右侧“启动”按钮，或在搜索结果里按 `Enter`。

注意：Terminal.app 不支持从应用中打开新标签页。选择“新标签页”并使用 Terminal.app 时，应用会打开新窗口。

### 删除 session

1. 在 session 行上右键。
2. 点击“删除此 session”。
3. 在确认弹窗中点击“删除”。

删除只作用于当前行对应的 CLI 本地 session 源文件、Cursor/Grok session 目录或 OpenCode 数据库中的 session 行，不会删除项目工作目录。删除失败时，列表不会假装移除该 session。

### 查看端口

1. 点击顶部 `Port`。
2. 使用搜索框按端口号、进程名、父进程、PID、用户、监听地址或项目路径过滤。
3. 使用“项目服务 / 全部端口”切换范围。
4. 使用协议菜单查看全部、TCP 或 UDP。

“项目服务”默认只包含当前用户、本地监听、非系统/应用目录的 TCP LISTEN 端口。端口行会显示进程、PID、地址、状态和工作目录；工作目录可复制或打开。

### 关闭端口服务

1. 在 Port 工具页点击端口行右侧“关闭”。
2. 多个进程占用同一端口时，可以展开端口后关闭单个进程，或在分组行关闭全部。
3. 在确认弹窗中点击“关闭服务”。

关闭服务会对关闭前重新扫描后仍一致的当前用户 PID 发送 `TERM` 信号。前端不会直接传 PID；如果端口记录过期、记录已变化、进程不属于当前用户或系统拒绝关闭，应用会显示错误并保留列表。

### 端口自动刷新

Port 工具页默认每 3 秒自动刷新。可以用“自动刷新”开关关闭；关闭后仍可点击刷新按钮手动扫描。该偏好保存在本机。

### 切换主题

使用主题菜单选择“黑”、“白”或“跟随系统”。主题偏好保存在本机。

## 常见问题

Q: 搜索会重新扫描本机文件吗？

A: 不会。搜索只过滤当前已经扫描到的 session 列表。

Q: 为什么 Terminal.app 选择新标签页还是打开新窗口？

A: Terminal.app 的 AppleScript 接口不支持可靠创建新标签页，当前版本按新窗口处理。

Q: 删除 session 后能恢复吗？

A: 应用没有回收站或撤销能力。删除前请确认这条 session 不再需要。

Q: 收藏项目会同步到其他设备吗？

A: 不会。收藏只保存在当前机器的本地偏好文件中。

Q: 关闭端口会杀掉系统服务吗？

A: 应用会在关闭前重新扫描并校验端口记录，只允许关闭当前用户拥有且记录未变化的端口进程。系统服务或其他用户进程会被拒绝或由 macOS 权限拦截。

## 相关功能

- 开发与验收指南：`../dev/release-readiness.md`
- 项目入口：`../../README.md`
