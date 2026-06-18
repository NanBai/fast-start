---
doc_type: feature-brainstorm
feature: 2026-06-17-cli-session-launcher
status: superseded-by-design
created: 2026-06-17
restored: 2026-06-18
restoration_note: |
  本文件在 2026-06-18 的一次文件丢失事件中遗失，由本会话上下文重建。
  design.md 的保真度高（多轮编辑留痕）；brainstorm.md 在丢失前仅被读过一次，
  此处基于 design 的调研结论与原始 brainstorm 的结构重建，细节可能与原版有出入。
  设计真源以 design.md 为准。
---

# cli-session-launcher 头脑风暴

## 背景与痛点

日常开发中频繁切换多个 AI CLI agent（codex / claude-code / cursor），每个 agent 的历史 session 散落在各自的本地存储里。需要恢复某个工作上下文时，痛点是：

- session 记录分散在三个不同位置、三种不同格式（jsonl / sqlite）
- 要恢复一个 session，得手动记起它在哪个项目目录、session id 是什么、再用对应 CLI 的 resume 命令
- 没有统一入口快速浏览"最近在哪些项目用过哪个 agent"

## 目标

一个轻量桌面应用，做三件事：

1. **聚合扫描**：扫三个 CLI 的本地 session 存储，统一展示
2. **一键恢复**：选中某条 session → 自动 cd 到工作目录 + 以 session id 启动对应 agent
3. **终端可选**：支持在系统 Terminal / iTerm2 / Ghostty 里打开

## 非目标

- 不做 session 持久化（每次冷扫描）
- 不做内置终端（始终调外部终端）
- 不做 session 编辑 / 删除
- 不做实时监控

## 候选方案

### 方案 A：Tauri 桌面应用（采纳）
- Tauri 2.x + React + TS
- Rust 后端扫描 + 拉起终端
- 优点：轻量、Rust 性能好、跨平台预留
- 选定理由：与"调用外部终端 + 读本地文件"的桌面工具定位最契合

### 方案 B：纯 CLI 工具
- 一个 Rust CLI，`fast-start list` / `fast-start resume <id>`
- 否决：列表浏览 + 终端选择的交互体验，GUI 更合适

### 方案 C：Electron
- 否决：体积大、资源重，对本应用过度

## 关键技术问题（头脑风暴阶段提出，design 阶段验证）

1. ❓ 三个 CLI 的 session 存在哪？格式是什么？→ design 阶段本机实测确认（见 design 调研结论小节）
2. ❓ 能否从 session 记录恢复工作目录（cwd）？→ design 确认 codex/claude 可，cursor 有 hash 反推风险
3. ❓ 各 CLI 是否支持 resume 命令？→ design 确认三家都支持
4. ❓ 如何安全地拼装终端启动命令（防注入）？→ design 定为 L2 安全档位 + 参数化构造

## 结论

进入 design 阶段，方案 A 采纳，v1 聚焦 codex + claude-code（可行性高），cursor 列 v2。
