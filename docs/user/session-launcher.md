---
doc_type: user-guide
slug: session-launcher
component: session-launcher
status: current
summary: 使用 Session Launcher 查找、收藏、启动和删除本地 AI CLI session
tags: [session-list, search, favorites, terminal, deletion]
last_reviewed: 2026-06-24
---

# 使用 Session Launcher 恢复工作现场

## 功能简介

Session Launcher 会扫描本机 Codex、Claude Code 和 Cursor 的历史 session，按 agent 和项目目录整理成列表。你可以搜索目标 session、收藏高频项目、选择外部终端，并从当前列表直接恢复工作现场。

## 前置条件

- 当前版本面向 macOS。
- 本机需要安装并使用过 Codex、Claude Code 或 Cursor Agent 中至少一种 CLI。
- 启动 session 前，需要安装至少一个可用终端：Terminal.app、iTerm2 或 Ghostty。
- 恢复 session 时，目标 CLI 命令需要能在终端里正常执行。

## 如何使用

### 启动应用

开发环境中运行：

```bash
pnpm tauri dev
```

应用打开后会自动扫描本机 session。扫描失败的 CLI 会在页面上显示错误，其他 CLI 的结果仍会展示。

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

删除只作用于当前行对应的 CLI 本地 session 源文件或 Cursor chat 目录，不会删除项目工作目录。删除失败时，列表不会假装移除该 session。

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

## 相关功能

- 开发与验收指南：`../dev/release-readiness.md`
- 项目入口：`../../README.md`
