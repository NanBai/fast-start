---
doc_type: feature-design
feature: 2026-07-13-cli-extension-contract
requirement:
roadmap: session-launcher-power-extend
roadmap_item: cli-extension-contract
status: approved
summary: CliType 注册完整性 cargo 测试 + 前端 labels + docs checklist
tags: [cli, contract, maintainability]
---

# cli-extension-contract 设计文档

## 1. 决策与约束

**目标**：新增 CLI 时测试防漏挂点。  
**不做**：动态插件 / dylib 加载。

**决策**：对每个 `CliType` 变体断言：scanner 注册、command_spec、program 白名单、args 形状、删除映射、文档表、前端 labels。

## 2. 名词与编排

编译期测试枚举所有 CliType；缺一 fail。  
文档：`docs/dev/` 下 CLI 扩展 checklist（或 release-readiness 小节）。

### 挂载点

1. `cargo test` 注册完整性  
2. 前端 labels 覆盖测试或静态检查  
3. docs checklist  

### 2.5 结构健康度

测试新文件；不改运行时架构。

## 3. 验收

- 人为去掉某 scanner 注册则测试红（可临时验证后恢复）  
- checklist 文档存在且列 7 挂点  
- 无 load_plugin API  

**验证**：cargo test；文档 diff  

## 4. 架构

强化「编译期扩展」而非插件运行时。
