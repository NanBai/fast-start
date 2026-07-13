---
doc_type: feature-design
feature: 2026-07-13-session-summary-enrichment
requirement: quick-session-access
roadmap: session-launcher-power-extend
roadmap_item: session-summary-enrichment
status: approved
summary: 各 scanner 摘要质量与 clean_summary≤160 截断；fixture 锁定
tags: [scanner, summary]
---

# session-summary-enrichment 设计文档

## 1. 决策与约束

**目标**：降低列表摘要噪声；统一长度。  
**不做**：全文索引；暴露 jsonl 路径；改 resume。

**决策**：

1. **仅** `scanner::clean_summary` 做 ≤160 Unicode 标量截断  
2. 各 CLI 优先级按 roadmap §4.4；已满足的用 fixture 钉现状  
3. OpenCode 用 title，不用整段 directory 当摘要  

## 2. 名词与编排

**现状**：`clean_summary` trim/折行/压空白，无长度截断。  
**变化**：截断 + 各 scanner 缺口补齐 + fixture。

编排：扫描路径不变；summary 经 clean_summary 输出。

### 挂载点

1. `clean_summary`  
2. 各 `scanner/*` 取摘要点  
3. fixture 测试  

### 2.5 结构健康度

不新增大文件；改动限 scanner 模块。

## 3. 验收

- 超长摘要截断至 ≤160  
- Codex 噪声消息不进 summary（fixture）  
- 无全文索引代码路径  

**验证**：cargo test --lib  

## 4. 架构

搜索仍只匹配元数据字段；summary 质量提升不改变搜索边界。
