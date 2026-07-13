---
doc_type: feature-design
feature: 2026-07-13-session-favorites-and-empty-states
requirement:
roadmap: session-launcher-next-wave
roadmap_item: session-favorites-and-empty-states
status: approved
summary: session 级收藏置顶；CLI 空数据与扫描失败的空态/错误引导
tags: [session, favorites, empty-state]
---

# session-favorites-and-empty-states 设计文档


## 0. 术语
favorite_session_ids：Session.id 列表，sanitize 同 favorite_project_dirs

## 1. 决策
- 排序：session 收藏 > 项目收藏 > 时间（design 锁）
- 空态：无该 CLI 数据时展示简短引导（非营销页）
- 扫描失败：保留 scanErrors 展示并提示刷新

## 2. 名词
**现状**：仅 favorite_project_dirs；AgentGroup 空时简单。
**变化**：preferences + 前端排序 + 空/错 UI。

### 2.3 挂载点
1. favorite_session_ids load/save
2. sessionUtils 排序
3. SessionRow/Project 收藏控件
4. 空态/错误组件文案

### 2.5 结论：扩展现有 hooks/utils，不新子系统

## 3. 验收
- 收藏重启保留且置顶
- sanitize 去掉已删 session
- 空 CLI / 失败有可见引导
- 不做回收站

