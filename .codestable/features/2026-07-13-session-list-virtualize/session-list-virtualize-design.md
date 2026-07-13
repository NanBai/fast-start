---
doc_type: feature-design
feature: 2026-07-13-session-list-virtualize
requirement: quick-session-access
roadmap: session-launcher-ux-perf
roadmap_item: session-list-virtualize
status: draft
summary: Session 行窗口化渲染；保留键盘与多选
tags: [frontend, performance, list]
---

# session-list-virtualize 设计文档

## 1. 决策与约束

**目标**：大列表滚动与挂载成本可控。  
**成功**：可见行窗口渲染；↑↓/Enter/多选/收藏/启动可用。  
**不做**：重型 UI 框架；虚拟化 Agent 级以外无关页面；暴露源路径。

**决策**：

1. 优先 **轻量自研** 窗口（按行高估算 + overscan）或已有依赖内最小方案；**禁止**为虚拟化新引大型表格库  
2. 作用域：`SessionRow` 列表；`AgentGroup` / `ProjectBucket` header 可保持  
3. 键盘导航基于 **可见结果序列**（与 quickAccess 一致），虚拟窗口须能 scrollIntoView 活跃行  
4. 行高尽量稳定（现有 session-row CSS）；动态高度若难则固定估算 + overscan  

## 2. 编排

```text
visibleSessions / projectGroups
  → VirtualSessionList 或 Bucket 内虚拟化
  → 仅 mount 窗口内 SessionRow
```

### 挂载点

1. 新 helper/组件  
2. SessionWorkspace 列表渲染点  

## 3. 验收

- 本机或构造 100+ 行可滚动；DOM 中 SessionRow 数明显少于总数  
- 键盘启动仍可用  
- `pnpm build`  

## 4. 架构

列表渲染策略记一笔；无新后端契约。
