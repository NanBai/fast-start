---
doc_type: feature-design
feature: 2026-07-13-wave-harden-and-docs
requirement:
roadmap: session-launcher-next-wave
roadmap_item: wave-harden-and-docs
status: approved
summary: epic 收口：交叉回归、ARCHITECTURE/用户文档/AGENTS 同步、残留风险
tags: [docs, harden, regression]
---

# wave-harden-and-docs 设计文档


## 1. 决策
- 依赖 1–8 均 done 或 dropped 后执行
- 回写 ARCHITECTURE（cache、出站、state 路径）
- AGENTS 偏好 key 列表更新
- 跑全量 cargo test + pnpm build；关键 smoke 清单

## 2. 挂载点
文档与验证，无新运行时能力（除非修回归 bug 另开 issue）

## 3. 验收
- 文档与代码一致
- 核心命令全绿
- residual risk 列表落盘（acceptance/qa）

