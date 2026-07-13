---
doc_type: feature-design
feature: 2026-07-13-session-health-inspect
requirement: quick-session-access
roadmap: session-launcher-power-extend
roadmap_item: session-health-inspect
status: approved
summary: inspect_session_health 返回 cwd/源/flags/approxBytes；列表可筛陈旧
tags: [session, hygiene, inspect]
---

# session-health-inspect 设计文档

## 0. 术语约定

| 术语 | 定义 |
|------|------|
| SessionHealth | 单条探测结果；不含源路径 |
| cache_limited | ops_ready=false 或无法源探测 |
| size_capped | 目录有界 du 超限 |

## 1. 决策与约束

**目标**：对已加载 session 做只读健康探测，支持筛选 missing_cwd / missing_source。  
**不做**：不暴露 delete_target 路径；不递归 project_dir 工作区；不改扫描器输出契约核心字段。

**决策**：

1. command `inspect_session_health(ids)`，上限 200  
2. **源探测所有权**：复用 `launch-preflight` 落地的 `session_source::check_session_source`（同一函数）；本 feature **禁止**再写第二套源 IO。若 preflight 尚未合并，实现顺序上先合共享模块再合本 command（或同 PR 先文件后调用方）  
3. OpenCode：强制 `CliType::OpenCode` 分支做**行存在**探测；**忽略** `delete_target.kind==File` 与 db 路径存在性  
4. File/Directory：path + kind；Directory 体积 depth≤3、≤2000 files、≤50ms → 超限 null+`size_capped`  
5. `empty_summary` flag：当 `session.summary` 为 `None` 或 trim 后空串  
6. OpenCode `approxBytes` 恒 null

## 2. 名词与编排

### 2.1 名词

```text
SessionHealthReport { items: SessionHealth[] }
SessionHealth {
  sessionListId, cwdExists, sourceExists: bool|null,
  approxBytes: number|null,
  flags: missing_cwd|missing_source|empty_summary|cache_limited|size_capped[]
}
```

### 2.2 编排

```text
UI 筛选/可见行 → inspect_session_health(ids)
  → 逐 id find session → stat cwd + check_source + optional size
  → 返回 report（未知 id 可省略或空 flags，不 panic）
前端：角标/筛选仅消费 report + 现有 Session 字段
```

### 2.3 挂载点

1. command + 共享源探测模块  
2. 前端筛选 UI  
3. 单测：File/Dir/OpenCode/缓存窗  

### 2.5 结构健康度

新文件承载 inspect；禁止把路径字符串塞进前端类型。

## 3. 验收

- 删掉 cwd 后 inspect 标 missing_cwd  
- OpenCode 删行后 sourceExists=false  
- 缓存窗 sourceExists=null + cache_limited  
- 响应无绝对路径字段  

**验证**：cargo test；手工；pnpm build  

## 4. 架构

只读探测；与 delete 安全口径一致（前端无源路径）。
