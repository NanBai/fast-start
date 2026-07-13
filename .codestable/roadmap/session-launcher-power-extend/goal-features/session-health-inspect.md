# Goal Feature Spec: session-health-inspect

## Roadmap item
- roadmap: session-launcher-power-extend
- roadmap_item: session-health-inspect
- depends_on: launch-preflight(shared source)

## Paths
- feature_dir: `.codestable/features/2026-07-13-session-health-inspect`
- design: `.codestable/features/2026-07-13-session-health-inspect/session-health-inspect-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-session-health-inspect/session-health-inspect-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-session-health-inspect/session-health-inspect-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-session-health-inspect/session-health-inspect-review.md`
- qa: `.codestable/features/2026-07-13-session-health-inspect/session-health-inspect-qa.md`
- acceptance: `.codestable/features/2026-07-13-session-health-inspect/session-health-inspect-acceptance.md`

## Nature
functional

## One-liner
inspect_session_health 陈旧筛选

## Core runtime path
见 design 验收场景；roadmap §4 硬约束不可改语义。

## Commands
cargo test --lib; pnpm build

## DoD / gates
1. implementation: checklist steps done + TDD evidence（UI/文档可 exception）
2. cs-code-review → passed
3. QA → passed
4. acceptance → passed；更新 items.yaml status done

## Failure recovery
- review blocking → review-fix → re-review
- QA fail → qa-fix → re-review + QA
- need scope change → CS_ROADMAP_GOAL_HANDOFF

## Cleanliness
- debug_output: forbidden
- 不暴露 delete_target 路径；不 log apiKey
