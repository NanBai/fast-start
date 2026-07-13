---
doc_type: feature-design
feature: 2026-07-13-health-inspect-on-demand
requirement: quick-session-access
roadmap: session-launcher-ux-perf
roadmap_item: health-inspect-on-demand
status: draft
summary: inspect 按需触发；角标仅有缓存时显示
tags: [session, performance, health]
---

# health-inspect-on-demand 设计文档

## 1. 决策与约束

**目标**：降低 full scan 后默认 IO。  
**成功**：未开陈旧筛选且未开磁盘面板时不自动 inspect。  
**不做**：改 inspect 后端语义/上限 200；假装无缓存时 source missing。

**决策**（对齐 roadmap §4.2）：

1. full scan 后 **默认不** `inspect_session_health`  
2. 触发（任一）：`healthFilter` 为 stale/missing_*；磁盘占用面板展开；可选「重新探测」按钮  
3. `fromCache` 仍跳过（已有）  
4. 角标：仅 `healthById` 有条目时显示  
5. 切换到触发筛选时若无缓存 → 一次批量 inspect（≤200）  

## 2. 编排

```text
scan/refresh → 更新 sessions（不 inspect）
用户选陈旧筛选 / 开磁盘面板
  → inspectHealthForSessions(ids)
  → 更新 healthById → 过滤/角标/聚合
```

### 挂载点

1. `useSessions` 去掉 applyScanResult 内自动 inspect（full scan）  
2. SessionWorkspace/App 在 filter/磁盘 open 时调用  

## 3. 验收

- full refresh 后无陈旧筛选：不发起 inspect（可用临时计数或 dev 断言）  
- 选「陈旧」→ 有探测 → 过滤生效  
- 磁盘面板 → 有聚合或未知  

## 4. 架构

health 为按需能力；ARCHITECTURE 探测策略一句。
