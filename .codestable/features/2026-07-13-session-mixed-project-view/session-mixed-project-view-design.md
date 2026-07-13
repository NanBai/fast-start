---
doc_type: feature-design
feature: 2026-07-13-session-mixed-project-view
requirement:
roadmap: session-launcher-next-wave
roadmap_item: session-mixed-project-view
status: approved
summary: Session 列表增加按 projectDir 跨 CLI 聚合视图，可与按 Agent 切换
tags: [session, ui, grouping]
---

# session-mixed-project-view 设计文档


## 0. 术语
| 术语 | 定义 |
|---|---|
| by-agent | 现有 CLI_ORDER → AgentGroup |
| by-project | 顶层按 projectDir 聚合，组内可再分 CLI |

## 1. 决策
- 纯前端派生，不改 ScanResponse
- 模式 preferences：`session_list_mode: "by-agent" | "by-project"` 默认 by-agent
- 不做实时 watch

## 2. 名词与编排
**现状**：`groupByProjectDir` 仅在 Agent 内；`AgentGroup` 按 cli。
**变化**：`groupSessionsByProject` 跨 CLI；App 按 mode 渲染。

### 2.3 挂载点
1. preferences key session_list_mode
2. sessionUtils 纯函数
3. App/控件切换 + 列表渲染

### 2.4 步骤
1. 纯函数 + 用例
2. 偏好读写
3. UI 切换与渲染
4. 文档一句

### 2.5 结构：抽纯函数到 sessionUtils，**不做大拆 App**

## 3. 验收
- 切换 by-project 同一 projectDir 下多 CLI 同组
- 切换回 by-agent 与现网一致
- 模式重启保留
- 启动/删除语义不变

