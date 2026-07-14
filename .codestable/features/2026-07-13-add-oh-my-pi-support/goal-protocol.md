---
doc_type: feature-goal-protocol
feature: 2026-07-13-add-oh-my-pi-support
---

# Goal Protocol: add-oh-my-pi-support

## 模式

本包在用户 **已确认 approved design** 后进入 goal 模式：

- 普通流程中「每阶段停等用户」的 checkpoint，改为写入报告 / `goal-state.yaml` / evidence。
- **只有** handoff 条件命中才停并向用户交接。
- 不得绕过 implementation TDD policy；行为代码 step 缺 RED/GREEN/VERIFY 且无 `TDD exception` → implementation gate 失败。

## 启动

1. 读本文件、`goal-state.yaml`、`goal-plan.md`
2. 读 `add-oh-my-pi-support-design.md`（必须 `status: approved`）
3. 读 `add-oh-my-pi-support-checklist.yaml`
4. 读 `.codestable/attention.md`
5. 以仓库事实校正 `goal-state.yaml`（产物存在则 stage 前进，不重复已完成 step）

## 执行 Loop

### A. Implementation（`stage: implementation`）

1. 写 `status: running`（若尚未）
2. 按 checklist steps 顺序推进；每步结束更新 step `status: done` 并留下证据块（命令输出、测试结果、diff 片段、前端验证描述）
3. 代码行为 step（2–4、6、8）默认 TDD micro-loop：
   - RED → GREEN → VERIFY（`cd src-tauri && cargo test --lib`）
4. 纯类型 / UI / 文档 step 按 goal-plan 的 TDD exception 规则（类型检查 + 手工 smoke + 必要 diff review）
5. 全部 steps done 且 CMD-001、CMD-002 通过后：
   - 生成 implementation evidence 摘要
   - 写 `stage: review` / `status: ready`

### B. Code Review（`stage: review`）

1. 运行 `cs-code-review`（或等价独立只读审查）针对本 feature 本轮 diff
2. 产出 `add-oh-my-pi-support-review.md`
3. blocking → `status: fixing`，修完回 `status: ready` 重跑 review
4. passed → 写 `stage: qa` / `status: ready`

### C. QA（`stage: qa`）

1. 按 design §3（验收契约）与 checklist checks 跑核心场景 + CMD-001/CMD-002
2. 产出 `add-oh-my-pi-support-qa.md`
3. failed/blocked → `status: fixing`，修完回 `stage: review` / `status: ready`（必须重跑 review + QA）
4. passed → 写 `stage: acceptance` / `status: ready`

### D. Acceptance（`stage: acceptance`）

1. 按 Acceptance Coverage Matrix 与 DoD 核对仓库事实（git diff、文件存在、测试绿、文档更新）
2. 回写 architecture / AGENTS.md / docs/user/session-launcher.md（按 design §4）
3. 更新 checklist 中 checks 状态
4. 写 `add-oh-my-pi-support-acceptance.md`
5. 全部通过后写 `stage: complete` / `status: passed`

## 证据记录规范

每完成一个 checklist step，必须在 goal-state 或独立 evidence 块记录：
- 触发命令 / 操作
- 关键输出（截断重要部分）
- 验证方式（cargo test 输出片段 / pnpm build 结果 / 手工观察）
- 相关文件 diff 引用

## 失败与回退

- 任何阶段失败 → 写 fixing + 问题描述 → 修复后回到上一 ready 状态
- 发现 scope 扩大或设计假设不成立 → 立即写 handoff 条件，更新 goal-state 并停止

## 退出

当 `stage: complete` 且 `status: passed` 时，feature 结束。更新 roadmap（如有）并执行 cs-docs-neat 相关同步。
