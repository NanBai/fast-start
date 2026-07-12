---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "maintainability-02"
nature: maintainability
severity: P2
confidence: high
suggested_action: cs-refactor
status: resolved
---

# Finding 11：高风险大文件持续膨胀

## 速答

`state.rs`（~628 行，混 session+port）、`App.tsx`（~522 行，双工具页编排）、`launcher.rs`（~450 行）均在 AGENTS「高风险大文件」名单内且继续增长。

## 关键证据

- 行数统计：`state.rs` 628、`App.tsx` 522、`launcher.rs` 457  
- `AGENTS.md` High Risk：prefer narrow edits；do not unrelated refactors  
- 2026-06-19 audit finding-04 同类问题仍在，且已叠加 Port 工具

## 影响

改启动/删除/扫描时 diff 噪声大、回归面难控；与「窄改动」目标冲突。

## 修复方向

launcher 按终端拆分；state 拆 port 子系统；App session/port 壳再收一层。单独重构，不与功能混做。

## 建议动作

`cs-refactor`。

## 处置

2026-07-12 P2 批量已修。
