# Goal Protocol: extend-grok-providers-tool

## 模式

本包在用户 **已确认 approved design** 后进入 goal 模式：

- 普通流程中「每阶段停等用户」的 checkpoint，改为写入报告 / `goal-state.yaml` / evidence。
- **只有** handoff 条件命中才停并向用户交接。
- 不得绕过 implementation TDD policy；行为代码 step 缺 RED/GREEN/VERIFY 且无 `TDD exception` → implementation gate 失败。

## 启动

1. 读本文件、`goal-state.yaml`、`goal-plan.md`
2. 读 `extend-grok-providers-tool-design.md`（必须 `status: approved`）
3. 读 `extend-grok-providers-tool-checklist.yaml`
4. 读 `.codestable/attention.md`
5. 以仓库事实校正 `goal-state.yaml`（产物存在则 stage 前进，不重复已完成 step）

## 执行 Loop

### A. Implementation（`stage: implementation`）

1. 写 `status: running`（若尚未）
2. 按 checklist steps 顺序推进；每步结束更新 step `status: done` 并留下证据块
3. 代码行为 step（2–3）默认 TDD micro-loop：
   - RED → GREEN → VERIFY（`cd src-tauri && cargo test --lib`）
4. step 1 / 4 / 5 / 6 按 goal-plan 的 TDD exception 规则
5. 全部 steps done 且 CMD-001、CMD-002 通过后：
   - 生成 implementation evidence 摘要（可写在 checklist 旁笔记或 review 输入）
   - 写 `stage: review` / `status: ready`

### B. Code Review（`stage: review`）

1. 运行 `cs-code-review`（或等价独立只读审查）针对本 feature diff
2. 产出 `extend-grok-providers-tool-review.md`
3. blocking → `status: fixing`，修完回 `status: ready` 重跑 review
4. passed → 写 `stage: qa` / `status: ready`

### C. QA（`stage: qa`）

1. 按 design §3 与 checklist checks 跑核心场景 + CMD-001/002
2. 产出 `extend-grok-providers-tool-qa.md`
3. failed/blocked → `status: fixing`，修完回 `stage: review` / `status: ready`（重跑 review + QA）
4. passed → 写 `stage: acceptance` / `status: ready`

### D. Acceptance（`stage: acceptance`）

1. 按 Acceptance Coverage Matrix 与 DoD 核对仓库事实
2. 回写 architecture / user 文档（design §4）
3. 更新 checklist checks
4. 产出 `extend-grok-providers-tool-acceptance.md`
5. passed → **先**写 `stage: complete` / `status: passed`，**再**打印：

```text
CS_FEATURE_GOAL_COMPLETE
```

## Handoff

命中 goal-plan handoff 条件时：

1. 写 `stage: handoff` / `status: blocked` / `handoff_reason` / `handoff_next`
2. 打印：

```text
CS_FEATURE_GOAL_HANDOFF
Reason: <具体阻塞>
Next: <建议动作>
```

## 状态机（合法值）

| stage | status |
|---|---|
| implementation | ready-to-dispatch \| running |
| review | ready \| fixing |
| qa | ready \| fixing |
| acceptance | ready |
| complete | passed |
| handoff | blocked |

每次 stage/status 变化 **立即** 写回 `goal-state.yaml`。  
派发成功后写 `driver_kind` / `driver_id`。

## 技能入口提示

- Implementation：`cs-feat` `--stage impl` 协议 + project skills `session-launcher-backend` / `session-launcher-frontend` / `fast-start-dev-verify`
- Review：`cs-code-review`
- QA / Accept：`cs-feat` 对应 stage protocol
- 验证：`cd src-tauri && cargo test --lib`；`pnpm build`
