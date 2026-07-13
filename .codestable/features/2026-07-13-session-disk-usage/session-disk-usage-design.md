---
doc_type: feature-design
feature: 2026-07-13-session-disk-usage
requirement: quick-session-access
roadmap: session-launcher-power-extend
roadmap_item: session-disk-usage
status: approved
summary: 按 CLI/项目聚合 session 源载体近似体积并展示
tags: [session, disk, hygiene]
---

# session-disk-usage 设计文档

## 1. 决策与约束

**依赖**：`session-health-inspect`（复用 approxBytes）。  
**目标**：展示各 CLI / 项目 session 源近似占用。  
**不做**：递归整个 git 工作区；精确计费级统计。

**决策**：聚合 inspect 结果；允许 null/size_capped；非核心可 drop。

## 2. 名词与编排

前端按 `cliType` / `projectDir` 对 `approxBytes` 求和；null 不计入并显示「未知」。

**刷新时机**：用户打开体积面板或点刷新时，**一次** `inspect_session_health(当前列表 id，≤200)`，结果缓存于 hook 状态再聚合；禁止对每行单独 invoke。

### 挂载点

1. UI 体积面板/折叠区  
2. 一次批量 inspect + 前端聚合

### 2.5 结构健康度

纯前端聚合优先；无必要不新 command。

## 3. 验收

- 有 File 源 session 时体积 >0 或可见数字  
- OpenCode 显示未知/不占假体积  
- 不扫描 project_dir 工作区文件树  

**验证**：pnpm build；手工  

## 4. 架构

无跨模块新契约；消费 health 报告。
