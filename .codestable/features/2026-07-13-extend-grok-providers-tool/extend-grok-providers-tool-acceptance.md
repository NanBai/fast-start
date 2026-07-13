---
doc_type: feature-acceptance
feature: 2026-07-13-extend-grok-providers-tool
status: passed
accepted: 2026-07-13
---

# extend-grok-providers-tool acceptance

## Design / gates

- design: approved
- design-review: passed
- implementation: checklist steps all done
- code review: passed（round 3 独立 subagent 复审；round 2 changes-requested 已 review-fix）
- QA: passed

## Deliverables

| 交付物 | 仓库事实 |
|---|---|
| 官方激活命令 | `grok_activate_official` in commands/lib |
| 隐私命令 | `grok_apply_privacy_protection` |
| 布局命令 | `get/set_grok_provider_layout` |
| status 字段 | `officialActive` / `officialLoggedIn` |
| 前端卡片 | `grokProviderCards.ts` + ProvidersWorkspace |
| 文档 | ARCHITECTURE + docs/user/session-launcher.md |

## Matrix

核心场景均有单测或契约级证据（见 QA）。明确不做项未引入托盘/HTTP。

## Verdict

**passed**

CS_FEATURE_GOAL_COMPLETE 条件满足（review/QA/acceptance 均为 passed，无 handoff）。
