---
doc_type: feature-goal-plan
feature: 2026-07-13-add-oh-my-pi-support
created: 2026-07-13
---

# Goal Plan: add-oh-my-pi-support

## 输入路径

| 角色 | 路径 |
|---|---|
| Feature 目录 | `.codestable/features/2026-07-13-add-oh-my-pi-support/` |
| Design（approved） | `add-oh-my-pi-support-design.md` |
| Checklist | `add-oh-my-pi-support-checklist.yaml` |
| Design-review（candidate） | `add-oh-my-pi-support-design-review.md` |
| Requirement | (none — 本 feature 直接来自用户对话，无独立 req 文档) |
| Goal protocol | `goal-protocol.md` |
| Goal state | `goal-state.yaml` |

## 用户确认

- 时间：2026-07-13
- 依据：用户在设计文档呈现后明确回复 `approved`
- design frontmatter 已改为 `status: approved`
- 注意：本轮 design-review 为 candidate stub（未执行独立 Task agent review），用户直接整体确认 design 进入 goal 阶段。后续 impl/code-review/qa/acceptance 仍需完整执行。

## 范围摘要

为 Session Launcher 添加对 oh-my-pi（omp） 的完整支持：

1. **Session 支持（核心 MVP）**：
   - 新 CliType::OhMyPi
   - OhMyPiScanner：解析 `~/.omp/agent/sessions/**/*.jsonl`（header + 消息条目）
   - resume 命令：`omp -r <id>`（cd 项目目录）
   - delete：删除对应 .jsonl 文件
   - 前端完整展示、搜索、收藏、启动

2. **切换大模型供应商支持（窄实现）**：
   - 轻量 omp provider 命令：list providers（来自 models.yml）、config health、set role model（安全写 config.yml modelRoles）
   - 前端在 Providers 工作区增加 Oh My Pi section（Grok 路径完全隔离）

**明确不做**（见 design §1）：
- 不复制 grok_provider 全套（http、复杂 profile UI、OAuth）
- 不做完整 models.yml 可视编辑
- 不改现有 Grok 任何行为
- 不暴露原始路径
- macOS-first

## 必跑验证命令

| ID | 命令 | 说明 |
|---|---|---|
| CMD-001 | `cd src-tauri && cargo test --lib` | 后端 scanner、security、command spec、provider 命令回归 |
| CMD-002 | `pnpm build` | 前端类型、labels、providers 区构建 |

实现前轻量预检；红灯先归因既有 vs 本次。

## Implementation TDD policy

- **代码行为 step（checklist step 2–4、6、8）默认 RED → GREEN → VERIFY**
  - RED：先写失败单测（scanner 解析、各种 resume 形状、security 校验、provider 读写）
  - GREEN：最小实现使测试通过
  - VERIFY：`cd src-tauri && cargo test --lib`（相关 + 全量）
- **step 1（enum）、5（前端类型）、7（UI section）、9（文档）**：允许 TDD exception，使用类型检查 + 手工 smoke + diff review 作为主要证据。
- **step 10（构建回归）**：全量命令绿灯。
- 缺 RED/GREEN/VERIFY 且无 TDD exception → implementation gate 不通过。

## 核心验收路径

| 场景 | 证据 |
|---|---|
| S1 扫描 omp session | Rust 单元测试（with_root fixture）+ 手工列表 |
| S3 resume 形状 + validate | 单元测试 + preflight preview |
| S4/S5 launch + delete | 手工（disposable session）+ delete smoke |
| S6/S7 omp provider list + set role | 后端测试 + 前端切换后 health 刷新 |
| S8/S9 类型构建 + 文档 | CMD-001/002 + grep 确认 AGENTS.md 等更新 |
| 回归其他 CLI | 全量 cargo test |

## DoD / gate 摘要

1. **Implementation**：checklist steps 全部 done；evidence 含命令输出与 TDD 记录；CMD-001/002 绿
2. **Code review**：`cs-code-review` passed，无 unresolved blocking
3. **QA**：核心场景 + 必跑命令；报告 `add-oh-my-pi-support-qa.md`
4. **Acceptance**：矩阵核对 + 文档回写（AGENTS.md、architecture、user doc）；`add-oh-my-pi-support-acceptance.md` passed

## Handoff 条件

命中任一条则写 `goal-state.yaml` → `stage: handoff` / `status: blocked` 并输出 `CS_FEATURE_GOAL_HANDOFF`：
- 无法安全解析 omp session 格式（格式重大变化）
- 写 config.yml 风险超出设计窄范围
- 安全/launcher 边界出现新阻塞问题
- 用户在 impl 中要求扩大 scope（例如完整 omp provider UI）
