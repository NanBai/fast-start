---
doc_type: dev-guide
slug: release-readiness
component: session-launcher
status: current
summary: Session Launcher 本地开发、验证和交付前检查指南
tags: [development, verification, tauri, release, port-monitor]
last_reviewed: 2026-07-07
---

# Session Launcher 开发与交付前检查

## 概述

本文面向维护者和贡献者，说明如何在本地运行 Session Launcher、执行最小充分验证，并在交付前确认核心桌面路径可用。

## 前置依赖

- macOS
- Node.js、pnpm
- Rust、Cargo
- Tauri CLI 依赖链
- 至少一个可用终端：Terminal.app、iTerm2 或 Ghostty

Rust 后端代码在 `src-tauri/`，前端代码在 `src/`。

## 快速上手

安装依赖：

```bash
pnpm install
```

启动开发应用：

```bash
pnpm tauri dev
```

只启动前端开发服务器：

```bash
pnpm dev
```

构建前端：

```bash
pnpm build
```

运行 Rust 单元测试：

```bash
cd src-tauri && cargo test --lib
```

## 核心概念

| 概念 | 当前实现 |
|---|---|
| session 来源 | Codex / Claude Code / Cursor / Grok Build 的本机历史数据 |
| 扫描入口 | `scan_sessions` 复用缓存，`refresh_sessions` 强制重扫 |
| 启动入口 | `launch_session(sessionListId = session.id)` 反查缓存并生成 `CommandSpec`（参数是列表稳定 id，不是 CLI 原始 sessionId） |
| 删除入口 | `delete_session(sessionListId = session.id)` 删除当前行对应源载体 |
| 端口入口 | `scan_ports` 复用缓存，`refresh_ports` 强制重扫，`terminate_port_processes(port_ids)` 关闭前重新扫描校验当前用户端口进程 |
| 偏好存储 | `preferences.json`，包含终端、打开方式、主题、收藏项目和端口自动刷新 |
| 快速访问 | 前端派生：最近天数过滤、搜索过滤、收藏排序 |

## 交付前检查

### 1. 基础命令

```bash
pnpm build
cd src-tauri && cargo test --lib
```

期望结果：

- 前端 `tsc && vite build` 通过。
- Rust lib 测试全部通过。

### 2. 原生窗口 smoke

运行：

```bash
pnpm tauri dev
```

检查：

- 应用窗口可以打开，默认窗口宽度下控制栏不水平溢出。
- 列表显示 Codex、Claude Code、Cursor、Grok Build 分组；没有数据时状态提示清晰。
- “刷新”按钮可以重新扫描。
- 最近天数筛选能改变可见数量。
- 搜索框可输入，`Cmd+K` 能聚焦，`Esc` 能清空。
- 搜索结果里 `↑` / `↓` 能切换活跃项，`Enter` 能启动当前项。
- 收藏项目后刷新或重启仍保留。
- 主题切换后刷新或重启仍保留。
- 切换到 Port 工具页后可以看到端口列表或清晰空态。
- Port 搜索、项目服务/全部端口、协议筛选和自动刷新开关可操作。

### 3. 终端启动 smoke

至少确认一个真实 session 可以启动：

1. 选择 Terminal.app、iTerm2 或 Ghostty。
2. 分别检查“新窗口”和“新标签页”模式。
3. 点击“启动”，确认外部终端进入项目目录并执行对应 CLI resume 命令。

注意：

- Terminal.app 不支持可靠打开新标签页，会回退为新窗口。
- iTerm2 的 AppleScript app 名是 `iTerm`。
- Ghostty 通过 wrapper 脚本启动命令，避免多词命令被 login shell 误解。

### 4. 删除 smoke

删除是真实破坏性动作，只在可丢弃 session 上验证：

1. 选择一条可删除的 session。
2. 右键打开菜单，点击“删除此 session”。
3. 取消一次，确认没有调用删除。
4. 再次右键并确认删除。
5. 刷新列表，确认该 session 不再出现。

不要用真实重要 session 做删除 smoke。

### 5. 响应式布局 smoke

手动拖拽窗口或用浏览器 mock 检查以下宽度：

- 1200
- 1040
- 900
- 800
- 720
- 640
- 560
- 480
- 360

期望结果：

- 页面没有水平滚动条。
- 搜索、最近天数、打开方式、终端、主题控件都在窗口内。
- 360px 最小宽度下仍可完成搜索、筛选、启动和删除确认。

### 6. 端口监控 smoke

端口关闭是真实破坏性动作，只在可丢弃开发服务上验证：

1. 启动一个临时本地服务，例如在临时目录运行 `python3 -m http.server 8765`。
2. 打开 Port 工具页并刷新，确认能搜索到 `8765`。
3. 在项目服务或全部端口范围内确认该端口显示 PID、地址和工作目录。
4. 点击“关闭”，先取消一次，确认进程仍在。
5. 再次点击“关闭”并确认，确认服务进程退出且端口列表重新扫描。

不要对数据库、系统服务或重要开发服务执行关闭 smoke。

## 已知限制与注意事项

- 当前版本只面向 macOS。
- 不支持全局系统快捷键；快捷键只在应用窗口获得焦点时生效。
- 不支持云同步、账号体系、导入导出或跨设备收藏。
- 不支持单条 session 收藏；收藏对象是项目目录。
- 删除 session 不提供撤销。
- 关闭端口服务不提供撤销，只对当前用户进程发送 `TERM`。
- `.playwright-mcp/` 是本地验证产物，不应纳入提交。

## 相关文档

- 用户指南：`../user/session-launcher.md`
- 架构地图：`../../.codestable/architecture/ARCHITECTURE.md`
- 快速访问验收：`../../.codestable/features/2026-06-24-quick-session-access/quick-session-access-acceptance.md`
