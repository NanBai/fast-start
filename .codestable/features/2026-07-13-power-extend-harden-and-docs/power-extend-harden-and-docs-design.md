---
doc_type: feature-design
feature: 2026-07-13-power-extend-harden-and-docs
requirement:
roadmap: session-launcher-power-extend
roadmap_item: power-extend-harden-and-docs
status: approved
summary: epic 收口：交叉回归、ARCHITECTURE/用户文档/AGENTS、残留风险
tags: [docs, harden, regression]
---

# power-extend-harden-and-docs 设计文档

## 1. 决策与约束

**依赖**：本 epic 未 drop 的全部条目 done 后执行（或 drop 后从 depends 移除）。  
**目标**：代码/文档/偏好 key 一致；回归通过。  
**不做**：新功能；Windows。

## 2. 编排

1. 跑 `pnpm build` + `cargo test --lib`  
2. 对照 roadmap 矩阵手工 smoke（预检、批量删可丢弃、protect、Grok health、终端若在）  
3. 更新 ARCHITECTURE（preflight、bulk、protect、终端、health）  
4. 更新 docs/user、AGENTS preferences keys（`port_protect_ports` 等）  
5. 写残留风险清单  

### 挂载点

文档与验收记录；无新运行时 API 除非修回归。

### 2.5 结构健康度

仅文档与小修；发现大重构记观察项。

## 3. 验收

- 文档与代码字段一致  
- 核心矩阵场景有证据  
- items 全 done 或显式 dropped  

**验证**：acceptance 报告  

## 4. 架构

ARCHITECTURE 升版描述本波增量。
