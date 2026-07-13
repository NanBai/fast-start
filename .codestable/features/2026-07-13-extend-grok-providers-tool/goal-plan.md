---
doc_type: feature-goal-plan
feature: 2026-07-13-extend-grok-providers-tool
created: 2026-07-13
---

# Goal Plan: extend-grok-providers-tool

## 输入路径

| 角色 | 路径 |
|---|---|
| Feature 目录 | `.codestable/features/2026-07-13-extend-grok-providers-tool/` |
| Design（approved） | `extend-grok-providers-tool-design.md` |
| Checklist | `extend-grok-providers-tool-checklist.yaml` |
| Design-review（passed） | `extend-grok-providers-tool-design-review.md` |
| Requirement | `.codestable/requirements/extend-grok-providers-tool.md` |
| Goal protocol | `goal-protocol.md` |
| Goal state | `goal-state.yaml` |

## 用户确认

- 时间：2026-07-13
- 依据：用户在 design-review `passed` 后明确回复 `approved`
- design frontmatter 已改为 `status: approved`

## 范围摘要

对齐 grok-build-switch v0.2.0 三类能力，落在现有 Grok 工具页：

1. 官方账号切换（清 API 覆盖 + clear_active + 可选 grok login）
2. 隐私保护写入 config.toml
3. 卡片排序 / 置顶（app preferences + Tauri layout 命令）

明确不做：托盘、HTTP 管理面、拉模型、完整 config 编辑器、账号侧 /privacy。

## 必跑验证命令

| ID | 命令 | 说明 |
|---|---|---|
| CMD-001 | `cd src-tauri && cargo test --lib` | 后端变换 / 编排 / 回归 |
| CMD-002 | `pnpm build` | 前端类型与打包 |

实现前轻量预检；红灯先归因既有 vs 本次。

## Implementation TDD policy

- **代码行为 step（checklist step 2–3）默认 RED → GREEN → VERIFY**
  - RED：先写失败单测（官方清理、隐私 merge、activate_official、loginRequired、无 config、backup 失败等）
  - GREEN：最小实现使测试通过
  - VERIFY：`cargo test --lib`（相关 + 全量）
- **step 1（纯函数）**：若仓库无 vitest，允许 `TDD exception`：导出函数 + 注释用例表 ≥3 组 + `pnpm build` 作为 VERIFY
- **step 4–5（UI/hook）**：允许 `TDD exception`：类型检查 + 手工 smoke 证据；行为关键路径以后端单测兜底
- **step 6（文档/harden）**：文档 diff + 命令 VERIFY；无 RED 要求
- 缺 RED/GREEN/VERIFY 且无 `TDD exception` → implementation gate 不通过

## 核心验收路径

| 场景 | 证据 |
|---|---|
| S1 官方清理 + clear_active | Rust 单测 |
| S2 loginRequired | Rust 单测 |
| S4 回到 API 供应商 | Rust 单测 |
| S5/S5b 隐私 | Rust 单测 |
| S6/S7 布局持久化 | 手工 / 偏好读写 |
| S11 空 profiles 仍显示官方卡 | 手工 |
| S12 无 config 官方 | Rust 单测 |
| S10 backup 失败 Err | Rust 单测或 FS 夹具 |

## DoD / gate 摘要

1. **Implementation**：checklist steps 全部 done；evidence 含命令输出与 TDD 记录；CMD-001/002 绿
2. **Code review**：`cs-code-review` passed，无 unresolved blocking
3. **QA**：核心场景 + 必跑命令；报告 `extend-grok-providers-tool-qa.md`
4. **Acceptance**：矩阵核对 + 文档回写；`extend-grok-providers-tool-acceptance.md` passed

## Handoff 条件

命中任一条则写 `goal-state.yaml` → `stage: handoff` / `status: blocked` 并输出 `CS_FEATURE_GOAL_HANDOFF`：

- 需改 approved design / 范围 / 公开契约
- 独立 review 不可用且无用户降级授权
- 同一失败项三轮仍不过
- 外部环境缺失导致核心行为无法判断
- 用户要求暂停 / 改方向 / 终止
