---
doc_type: feature-acceptance
feature: 2026-06-20-delete-session-context-menu
status: accepted
summary: 右键删除单条 session 验收通过，覆盖前端确认流、后端源载体校验删除、缓存移除、架构与 requirement 回写
tags: [frontend, tauri-command, session-list, destructive-action]
---

# delete-session-context-menu 验收报告

> 阶段：阶段 3（验收闭环）
> 验收日期：2026-06-20
> 关联方案 doc：`.codestable/features/2026-06-20-delete-session-context-menu/delete-session-context-menu-design.md`
> 用户终审：2026-06-20 用户已确认

## 0. 验收阶段补修记录

- [x] **无业务补修**：验收未发现 design / 实现偏差，无需修改功能代码。
- [x] **架构归并**：已按 design 第 4 节更新 `.codestable/architecture/ARCHITECTURE.md`，补入 `session_delete.rs` 职责、删除数据流和 destructive action 安全口径。
- [x] **requirement 回写**：已将 `.codestable/requirements/delete-session-context-menu.md` 从 `draft` 升级为 `current`，并同步 `.codestable/requirements/VISION.md`。
- [x] **状态规范化**：checklist checks 接手时为 39 条 `pending`，逐项核对后规范化为 39 条全 `passed`；steps 保持 implement 阶段的 6 条 `done`。

## 1. 接口契约核对

对照 design 2.1 名词层：

- [x] `SessionDeleteKind` / `SessionDeleteTarget` 已在 `src-tauri/src/models.rs` 定义，记录 root / path / kind。
- [x] `Session.delete_target` 使用 `#[serde(skip)]`，前端 `SessionData` 未新增 `delete_target` / `deleteTarget` / path 字段。
- [x] `delete_session(session_id: String) -> Result<ScanResponse, String>` 已在 `commands.rs` 暴露并在 `lib.rs` 注册；参数使用前端当前行 `SessionData.id`。
- [x] Codex / Claude Code scanner 填充文件删除目标，Cursor scanner 填充 chat 目录删除目标。
- [x] 删除执行落在 `session_delete.rs`，不进入 scanner 或 launcher。

流程图核对：design 2.2 中“右键 SessionRow → 上下文菜单 → 确认弹窗 → invoke delete_session → AppState find_session → 校验目标 → 删除文件/目录 → 缓存移除 → 返回 ScanResponse”均有代码落点。

## 2. 行为与决策核对

**需求摘要逐项验证**：

- [x] 用户右键单条 session 行可打开“删除此 session”菜单：`SessionRow.onContextMenu` 逐层传到 `App`，由 `SessionContextMenu` 渲染。
- [x] 删除前必须确认：`requestDeleteSession` 只设置 `pendingDelete`，真正后端调用只在 `confirmDeleteSession` 中发生。
- [x] 删除成功后更新当前列表：前端用后端返回的 `ScanResponse` 调 `applyScanResult`。
- [x] 删除失败不假移除：`catch` 只设置错误状态，不改 `sessions`。

**明确不做反向核对**：

- [x] 不删除 `project_dir`：删除目标来自 scanner 填充的源载体，不使用 `project_dir` 执行 `remove_*`。
- [x] 不删除 CLI 全局目录 / 账号 / 缓存 / 其他 session：`session_delete.rs` 阻止 path 等于 root 或越出 root。
- [x] 不做批量、项目级、agent 级删除：前端只对单行打开菜单，后端只按一个 `Session.id` 删除。
- [x] 不做撤销、回收站、归档或云同步：代码无相关入口。
- [x] 不向前端暴露源路径：`src/types.ts` 无删除目标字段。
- [x] 不在 `launcher.rs` 加删除逻辑：grep 命中仅为既有 wrapper 临时文件清理。

**关键决策落地**：

- [x] 删除只通过 Tauri command 执行：前端只 invoke `delete_session`。
- [x] 扫描器负责产出删除目标：三家 scanner 均在构造 `Session` 时填充 `delete_target`。
- [x] 删除后更新内存缓存：`AppState.delete_session` 删除成功后 `retain(|item| item.id != session_id)`。
- [x] 路径校验按 CLI root 白名单：root/path canonicalize 后校验 starts_with，且 path 不能等于 root。

**挂载点反向核对与拔除沙盘**：

- [x] `Session.delete_target`：移除该字段会让后端缺少源载体。
- [x] 三个 scanner 的填充点：移除任一填充点会使对应 CLI 删除时报“删除目标缺失”。
- [x] `delete_session` command：移除 `commands.rs` / `lib.rs` 注册即可拔掉后端入口。
- [x] `SessionRow` / `SessionContextMenu`：移除后前端无右键触发入口。
- [x] `App` 删除状态和 `ConfirmDialog`：移除后无二次确认、调用和结果更新。
- [x] 反向 grep 覆盖 `delete_target|SessionDeleteTarget|delete_session|remove_file|remove_dir_all`，未发现清单外挂载点。

## 3. 验收场景核对

| 场景 | 结果 | 证据 |
|---|---|---|
| 右键菜单出现并可关闭 | passed | Playwright MCP mock IPC smoke：右键 `.session-row` 显示菜单；取消弹窗后列表保留 |
| 取消确认不调用后端 | passed | mock smoke 断言取消后 `delete_session` 调用数为 0 |
| Codex 删除源 jsonl | passed | `session_delete::deletes_file_target_inside_root` + `state::deleted_scanned_session_disappears_after_rescan` 通过 |
| Claude Code 删除源 jsonl | passed | scanner fixture 断言 delete_target 为 File；文件删除路径由同一 `delete_session_target` 单测覆盖 |
| Cursor 删除 chat 目录 | passed | `session_delete::deletes_directory_target_inside_root` + Cursor scanner fixture 断言 delete_target 为 Directory |
| 路径越界保护 | passed | `session_delete::rejects_target_outside_root` 通过 |
| 目标不存在 | passed | `session_delete::rejects_missing_target` 通过，错误消息提示刷新 |
| 启动并发保护 | passed | `App.handleSessionContextMenu` 在 `launchingId/deletingId` 命中当前行时直接返回；`SessionRow` busy 时启动按钮禁用 |
| 刷新一致性 | passed | 删除真实源载体后 scanner fixture 重扫为空；其他 session 不按 CLI `session_id` 模糊删除 |

补充验证命令：

- `cd src-tauri && cargo test --lib`：21 passed。
- `pnpm build`：通过，`tsc && vite build` 完成。
- `python3 .codestable/tools/validate-yaml.py --file .../delete-session-context-menu-checklist.yaml --yaml-only`：通过。
- `pnpm tauri dev`：当前进程存在，Vite `http://localhost:1420/` 返回 200。

说明：未删除真实用户 session，原因是该操作不可恢复。UI 交互用 mock IPC 覆盖右键、取消、确认和 `session.id` 参数；真实破坏性 smoke 需用户自行选择可丢弃 session 终审。

## 4. 术语一致性与禁用词反向 grep

design 未定义禁用词列表，跳过禁用词反向 grep。

术语一致性检查：

- `session 源载体` 在实现中对应 `SessionDeleteTarget`，只存在 Rust 后端内部。
- `删除 session` 对应删除源文件 / chat 目录，不对应项目工作目录。
- `右键菜单` 只挂在 `SessionRow`，未挂在 agent header 或 project header。

关键反向 grep：

```bash
rg -n "delete_target|SessionDeleteTarget|delete_session|remove_file|remove_dir_all" src src-tauri/src
rg -n "deleteTarget|delete_target|SessionDeleteTarget" src/types.ts src
rg -n "delete|remove_file|remove_dir_all|session_delete" src-tauri/src/launcher.rs src-tauri/src/security.rs src-tauri/src/scanner.rs
```

结论：源码中删除目标未进入前端类型；真实 session 删除只集中在 `session_delete.rs` 和 `AppState.delete_session`；`launcher.rs` 的 `remove_file` 为既有 wrapper cleanup，非本 feature 删除逻辑。

## 5. 架构归并

已实际更新 `.codestable/architecture/ARCHITECTURE.md`：

- [x] 概览新增本地 session 删除能力。
- [x] 名词层补 `Session.delete_target` 和 `SessionDeleteTarget`。
- [x] 核心模块新增 `session_delete.rs` 职责。
- [x] Tauri command 集合补 `delete_session`。
- [x] 数据流补“右键删除 session → delete_session → 校验源载体 → 删除文件/目录 → 更新缓存”链路。
- [x] CLI 覆盖补三家删除目标口径。
- [x] 安全口径补 root/path/kind 校验、不暴露路径、不删除工作目录和失败不假成功。

判据满足：没读过 design 的人打开 architecture 能知道系统现在有删除单条 session 的能力、删除目标来源和安全边界。

## 6. requirement 回写

design frontmatter 指向 `requirement: delete-session-context-menu`，原 requirement 状态为 `draft`。

已实际更新：

- [x] `.codestable/requirements/delete-session-context-menu.md`：`draft` → `current`，保留用户故事和边界，追加当前实现与变更日志。
- [x] `.codestable/requirements/VISION.md`：从 Draft 移到 Current。

## 7. roadmap 回写

design frontmatter 无 `roadmap` / `roadmap_item` 字段。

**结论**：非 roadmap 起头，跳过。

## 8. attention.md 候选盘点

候选 1：本机没有 Python `playwright` 包，`webapp-testing` 技能里的原生 Python Playwright 路径不可直接用；本次改用 Playwright MCP 完成 UI smoke。若后续频繁做本地前端验收，建议用 `cs-note` 记录“优先使用 Playwright MCP，除非先安装 Python playwright”。

## 9. 遗留

- 真实用户 session 删除未自动执行：这是不可恢复操作，验收只用临时 fixture 和 mock IPC smoke。用户终审时应选择一条可丢弃 session 手动右键删除，再刷新确认不再出现。
- `src-tauri/src/state.rs` 已接近偏长，当前只新增轻量编排和测试；后续如果继续增长，建议单独走 `cs-refactor` 拆分状态编排测试辅助。
