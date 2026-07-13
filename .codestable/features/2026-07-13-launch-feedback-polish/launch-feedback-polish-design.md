---
doc_type: feature-design
feature: 2026-07-13-launch-feedback-polish
requirement: quick-session-access
roadmap: session-launcher-ux-perf
roadmap_item: launch-feedback-polish
status: approved
summary: 启动/刷新 status 一致；减少预检闪烁；后端门闩保留
tags: [frontend, ux, launch]
---

# launch-feedback-polish 设计文档

## 1. 决策与约束

**目标**：启动路径 status 更稳、更少闪烁。  
**成功**：用户一次启动看到连贯文案；失败原因仍中文可读。  
**不做**：去掉后端 `launch_session` 内 preflight；改 resume 命令。

**决策**：

1. 前端启动：可 **只** 调 `launch_session`（后端已门闩），或保留 preflight 但 **合并** status（失败只报一次；成功不先刷「预检提示」再刷「启动成功」除非 warn 需展示）  
2. warn（如 source_unverified）：可附在成功文案后，或 info 一次，禁止连续三条 status 闪  
3. 刷新/删除/批量：统一「进行中 → 结果」两态，避免重复 formatScanStatus 覆盖错误  

## 2. 编排

```text
用户点启动
  → launchingId
  → invoke launch_session（或 preflight+launch 但单一结果 status）
  → success | error（含 block message）
```

### 挂载点

1. `useSessions.launchSession`  
2. 必要时 preview 路径 status  

## 3. 验收

- 正常启动：status 不出现无意义连闪  
- 坏 cwd：仍失败且中文原因  
- 后端门闩仍在（单测/代码路径）  

## 4. 架构

无新契约。
