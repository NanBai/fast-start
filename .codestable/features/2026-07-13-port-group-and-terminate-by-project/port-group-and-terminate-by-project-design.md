---
doc_type: feature-design
feature: 2026-07-13-port-group-and-terminate-by-project
requirement:
roadmap: session-launcher-power-extend
roadmap_item: port-group-and-terminate-by-project
status: approved
summary: Port 按 workingDirectory 分组；组内多选后走既有 terminate
tags: [port, ui]
---

# port-group-and-terminate-by-project 设计文档

## 1. 决策与约束

**依赖**：`port-protect-list`。  
**目标**：按项目看端口并一键选中关闭。  
**不做**：新 `terminate_ports_for_project` command；未知目录组一键关。

**决策**：纯前端分组 + 收集 id → 现确认 → `terminate_port_processes`。

## 2. 名词与编排

分组键 = `PortUsage.workingDirectory`；空 →「未知目录」组（无默认一键关）。

```text
渲染分组
  → 用户点「关闭此项目端口」
  → 收集组内 id（仅 user 可选中项）
  → 确认 → terminate_port_processes
  → protect 拦截由后端保证
```

### 挂载点

1. PortWorkspace 分组 UI  
2. 复用确认与 terminate hook  

### 2.5 结构健康度

新增 `groupPortsByWorkingDirectory`（或等价名）；**禁止**改坏现有按端口号的 `groupPorts`。

## 3. 验收

- 同 cwd 两端口同组  
- 关闭项目组触发 terminate（可 mock/手工）  
- 未知目录无危险一键按钮  
- 无新后端 terminate 分叉  

**验证**：pnpm build；手工 http.server  

## 4. 架构

终止安全仍单点；UI 不直杀 PID。
