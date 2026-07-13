---
doc_type: feature-design
feature: 2026-07-13-ux-perf-harden-and-docs
requirement: quick-session-access
roadmap: session-launcher-ux-perf
roadmap_item: ux-perf-harden-and-docs
status: draft
summary: 文档对齐 + next-wave completed + 交叉回归
tags: [docs, harden]
---

# ux-perf-harden-and-docs 设计文档

## 1. 决策与约束

**依赖**：本 epic 未 drop 条目 done 后执行。  
**目标**：文档与代码一致；回归通过。  
**不做**：新功能。

**决策**：

1. README：WezTerm、批量删、预检、保护端口、按项目关端口（若仍缺失）  
2. `docs/user/session-launcher.md` 与本波按需 inspect / 虚拟列表说明  
3. ARCHITECTURE：App 壳/workspace、state 子模块、inspect 按需  
4. next-wave roadmap 主文档 `status: completed`（若仍 active）  
5. `pnpm build` + `cargo test --lib`  

## 2. 验收

- 文档字段与偏好 keys 无矛盾  
- 核心路径有回归证据  

## 3. 架构

ARCHITECTURE 升版记录 polish 边界。
