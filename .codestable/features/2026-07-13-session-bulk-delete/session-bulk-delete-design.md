---
doc_type: feature-design
feature: 2026-07-13-session-bulk-delete
requirement: delete-session-context-menu
roadmap: session-launcher-power-extend
roadmap_item: session-bulk-delete
status: approved
summary: 多选批量删除；走 AppState::delete_session 全路径；partial success
tags: [session, delete, bulk]
---

# session-bulk-delete 设计文档

## 0. 术语

| 术语 | 定义 |
|------|------|
| BulkDeleteResult | deletedIds + failures + 列表视图字段 |
| 全路径删除 | 与单条 delete_session 相同状态分支（含 OpenCode 行删） |

## 1. 决策与约束

**目标**：一次确认删除多条 session。  
**成功**：部分失败时成功项已删、failures 可见。  
**不做**：回收站；force 绕过校验；传路径。

**决策**：

1. **实现前** `cs-req` 修订 `delete-session-context-menu`：允许批量 + 二次确认 + partial success（闸门；未改 req 不得 accept）  
2. `delete_sessions(ids)` 循环 **AppState::delete_session** 语义（含 OpenCode）；**禁止**只调 `delete_session_target`  
3. 上限 50；空数组 Err  
4. 整批结束后 **必须** `sanitize_recent_launches` + 写盘（**代码事实**：单条 `delete_session` command 已 sanitize；bulk 批末仍必须再做一次，防止循环中途失败遗漏写盘/幽灵 id）  
5. UI 多选 + 确认文案含数量 + **默认展示 failures**（全成功也明示）

## 2. 名词与编排

```text
BulkDeleteResult {
  deletedIds, failures[{sessionListId,message}],
  sessions, scanErrors, fromCache, scanDurationMs
}
```

```text
确认对话框
  → delete_sessions(ids)
  → for id: 同单条删除路径
  → sanitize recent + 返回结果
  → UI 更新列表与 failures
```

### 挂载点

1. command `delete_sessions`  
2. Session 列表多选 UI + ConfirmDialog  
3. 单测：OpenCode 分支、缓存窗 failure、上限  

### 2.5 结构健康度

多选状态进 hook/新组件，避免 App.tsx 无界膨胀。

## 3. 验收

- 2 条可丢弃 session 批量删后列表消失  
- 1 条无效 id：deleted 有成功项，failures 有一项  
- 缓存窗：failure 提示刷新  
- 无路径进前端  

**验证**：cargo test；仅可丢弃数据 smoke；pnpm build；req 已修订  

## 4. 架构

删除安全口径扩展为「单条与批量同一校验」；仍不删 project_dir。
