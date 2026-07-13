---
doc_type: feature-design
feature: 2026-07-13-port-protect-list
requirement:
roadmap: session-launcher-power-extend
roadmap_item: port-protect-list
status: approved
summary: port_protect_ports 偏好；terminate 命中保护端口整批失败
tags: [port, safety]
---

# port-protect-list 设计文档

## 1. 决策与约束

**目标**：保护端口不可被 terminate。  
**不做**：force 绕过；改变 ignore 仅隐藏的语义。

**决策**：

1. preferences key `port_protect_ports: u16[]`  
2. get/set 与 ignore 对称 sanitize  
3. `terminate_port_processes`：re-scan 后若任一目标 port∈protect → 整批 Err  

## 2. 名词与编排

**现状**：`port_ignore_ports` 过滤展示；terminate all-or-nothing + user_owned。  
**变化**：protect 拦截终止。

```text
terminate(ids)
  → re-scan + id 校验（现有）
  → 若 port 号 ∈ protect → Err(含端口列表)
  → 否则 TERM
```

### 挂载点

1. preferences load/save  
2. terminate 包装层  
3. Port UI 编辑保护列表  
4. cargo 单测  

### 2.5 结构健康度

preferences 增量；terminate 逻辑集中一处。

## 3. 验收

- 保护 3000 后杀含 3000 的多选 → 整批失败且进程仍在  
- ignore 仍只影响展示  
- 重启后 protect 保留  

**验证**：cargo test；手工  

## 4. 架构

Port 安全口径：protect ⊃ 用户多选。
