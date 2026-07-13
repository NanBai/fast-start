---
doc_type: feature-qa
feature: 2026-07-13-session-mixed-project-view
status: passed
reviewed: 2026-07-13
---

# session-mixed-project-view QA

| 场景 | 结果 | 证据 |
|---|---|---|
| by-project 跨 CLI 同 projectDir | pass | `groupSessionsByProject` + App 渲染 ProjectBucket showCliLabel |
| by-agent 不变 | pass | 仍 CLI_ORDER + AgentGroup |
| 模式偏好 | pass | session_list_mode load/save commands |
| 不改 scanner | pass | 无 scanner/models Session 变更（仅 SessionListMode enum） |

Commands: cargo test --lib 76; pnpm build ok.

**passed**
