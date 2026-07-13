---
doc_type: feature-design
feature: 2026-07-13-app-shell-split
requirement: quick-session-access
roadmap: session-launcher-ux-perf
roadmap_item: app-shell-split
status: approved
summary: App 壳与 SessionWorkspace 拆分；行为等价
tags: [frontend, structure, ux]
---

# app-shell-split 设计文档

## 0. 术语

| 术语 | 定义 |
|------|------|
| App 壳 | tool 切换、全局 status、主题应用、Cmd+K 分发、挂载三工作区 |
| SessionWorkspace | Session 工具页全部 UI 与该页局部状态编排 |

## 1. 决策与约束

**目标**：`App.tsx` 变薄；Session 页可独立维护。  
**成功**：行为与拆分前一致；`pnpm build` 绿。  
**不做**：改 Tauri 契约；改 Port/Grok 业务逻辑（仅 props 传递）；视觉 redesign。

**决策**：

1. 新增 `src/components/SessionWorkspace.tsx`（或 `src/workspaces/SessionWorkspace.tsx`，二选一实现时定一处）  
2. 迁入：搜索/筛选/健康筛选/列表模式、session 列表、批量栏、磁盘占用、最近启动、command preview、scan errors、context menu、delete/bulk confirm  
3. App 保留：`activeTool`、preferences/ports/grok hooks 顶层、全局 keydown、status pill、Port/Providers 挂载  
4. **行为等价**：快捷键、启动、删除、收藏语义不变  

**复杂度**：默认档；微重构允许，不顺手加功能。

## 2. 编排

```text
App
  hooks: preferences, sessions, ports, grok
  → activeTool=sessions → SessionWorkspace(props)
  → ports → PortWorkspace
  → providers → ProvidersWorkspace
```

### 挂载点

1. 新 SessionWorkspace 组件  
2. App.tsx 删减 sessions 分支 JSX/局部 handler  

### 结构健康度

本 feature 的目的就是拆文件；禁止再往 App 堆 session 专用 state（可留在 hook）。

## 3. 验收

- 三工具页切换正常  
- 搜索/筛选/启动/删除/批量/磁盘/最近启动与拆分前一致  
- `pnpm build`  

## 4. 架构

ARCHITECTURE 前端入口：`App` 壳 + workspaces；收口 feature 回写。
