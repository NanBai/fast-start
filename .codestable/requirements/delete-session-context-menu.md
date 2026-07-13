---
doc_type: requirement
requirement: delete-session-context-menu
status: current
summary: 用户可以从 Session Launcher 列表里删除单条或批量历史 session，并同步移除对应 CLI 的本地 session 文件、目录或 OpenCode 行。
---

# delete-session-context-menu requirement

## 用户故事

作为 Session Launcher 用户，我希望在列表里右键某条 session 后可以删除它，或在多选后批量删除，让过期、误建或不再需要的历史会话从列表和对应 CLI 本地存储里消失。

## 边界

- 删除目标是用户选中的 session 的扫描源载体（文件 / 目录 / OpenCode SQLite 行）。
- Codex / Claude Code 的删除目标是对应 `jsonl` 文件。
- Cursor / Grok Build 的删除目标是对应 chat/session 目录。
- OpenCode 的删除目标是 `opencode.db` 中的 session **行**（不删整个 db 文件）。
- 删除前必须有确认步骤：单条右键确认；批量删除须二次确认并展示将删数量。
- 前端只传列表稳定 `SessionData.id`（或 id 列表），不展示、不持久化真实源文件路径。
- 后端必须校验删除目标仍在对应 CLI 存储 root 内，且文件 / 目录类型匹配（OpenCode 走行删除分支）。
- 不删除项目工作目录，不删除 CLI 程序。
- **允许批量删除**：单次上限 50；走与单条相同的状态删除全路径；允许 partial success（成功 id 已删，失败 id 返回 failures 列表，UI 必须展示）。
- 不做回收站、撤销、归档或云端同步。

## 当前实现

- 单条 session 行支持右键打开上下文菜单，菜单项为“删除此 session”。
- 删除前显示确认弹窗；取消不会调用后端。
- 确认后调用 Tauri command `delete_session(sessionListId)`，其中 id 是前端行级 `SessionData.id`。
- 批量：`delete_sessions(sessionListIds)` 返回 `deletedIds` + `failures` + 当前列表视图字段。
- 删除成功后，后端删除源载体并更新缓存列表；批量结束须 sanitize 最近启动记录。
- 删除失败时，前端显示明确错误；批量时展示 failures 明细。

## 变更日志

- 2026-06-20：feature `2026-06-20-delete-session-context-menu` 验收通过，能力状态由 `draft` 升级为 `current`；落地 Codex / Claude Code 文件删除、Cursor chat 目录删除、路径白名单校验和前端二次确认。
- 2026-07-13：power-extend `session-bulk-delete` 修订边界——允许批量（≤50）、二次确认、partial success + failures 展示；OpenCode 行删除纳入同一全路径。
