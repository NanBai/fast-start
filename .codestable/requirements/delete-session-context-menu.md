---
doc_type: requirement
requirement: delete-session-context-menu
status: current
summary: 用户可以从 Session Launcher 列表里删除单条历史 session，并同步移除对应 CLI 的本地 session 文件或目录。
---

# delete-session-context-menu requirement

## 用户故事

作为 Session Launcher 用户，我希望在列表里右键某条 session 后可以删除它，让过期、误建或不再需要的历史会话从列表和对应 CLI 本地存储里消失。

## 边界

- 只删除用户选中的单条 session 的扫描源载体。
- Codex / Claude Code 的删除目标是对应 `jsonl` 文件。
- Cursor 的删除目标是对应 chat 目录。
- 删除前必须有确认步骤，避免右键误触直接造成不可恢复的数据删除。
- 前端只传当前行 `SessionData.id`，不展示、不持久化真实源文件路径。
- 后端必须校验删除目标仍在对应 CLI 存储 root 内，且文件 / 目录类型匹配。
- 不删除项目工作目录，不删除 CLI 程序，不做批量清理。
- 不做回收站、撤销、归档或云端同步。

## 当前实现

- 单条 session 行支持右键打开上下文菜单，菜单项为“删除此 session”。
- 删除前显示确认弹窗；取消不会调用后端。
- 确认后调用 Tauri command `delete_session(session_id)`，其中 `session_id` 是前端行级 `SessionData.id`。
- 删除成功后，后端删除源载体并返回新的 `ScanResponse`，前端立即更新列表。
- 删除失败时，前端显示明确错误，并保留原列表项。

## 变更日志

- 2026-06-20：feature `2026-06-20-delete-session-context-menu` 验收通过，能力状态由 `draft` 升级为 `current`；落地 Codex / Claude Code 文件删除、Cursor chat 目录删除、路径白名单校验和前端二次确认。
