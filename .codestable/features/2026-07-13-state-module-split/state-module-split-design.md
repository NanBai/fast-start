---
doc_type: feature-design
feature: 2026-07-13-state-module-split
requirement: quick-session-access
roadmap: session-launcher-ux-perf
roadmap_item: state-module-split
status: draft
summary: 行为等价拆分 state/mod.rs；不改 command 契约
tags: [backend, refactor, structure]
---

# state-module-split 设计文档

## 1. 决策与约束

**目标**：`state/mod.rs` 可维护；按域拆文件。  
**成功**：`cargo test --lib` 全绿；Tauri 表面不变。  
**不做**：改删除/预检/health 语义；大重构算法。

**决策**：

1. 优先抽出（名称可微调）：  
   - `state/session_ops.rs`：find/launch/preflight/inspect/delete/bulk/recent  
   - 或 `state/prefs_ports.rs` 若 prefs getter 仍挤在 mod  
2. `AppState` / `AppStateInner` 定义可留 `mod.rs` 或 `inner.rs`  
3. `pub use` 保持 `crate::state::AppState` 路径稳定  
4. **行为等价**：现有单测全部通过且不改断言语义  

## 2. 编排

仅模块边界移动；commands 仍调 `state.xxx`。

### 挂载点

1. 新 state 子模块  
2. `mod.rs` re-export  

## 3. 验收

- `cd src-tauri && cargo test --lib`  
- 无 command 签名变更  
- `state/mod.rs` 行数下降  

## 4. 架构

ARCHITECTURE 标明 state 子模块边界；harden 回写。
