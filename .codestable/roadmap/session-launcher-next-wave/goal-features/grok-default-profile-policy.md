# Goal Feature Spec: grok-default-profile-policy

## Roadmap item
- roadmap: session-launcher-next-wave
- roadmap_item: grok-default-profile-policy
- depends_on: ['grok-models-and-preview(schedule)']

## Paths
- feature_dir: `.codestable/features/2026-07-13-grok-default-profile-policy`
- design: `.codestable/features/2026-07-13-grok-default-profile-policy/grok-default-profile-policy-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-grok-default-profile-policy/grok-default-profile-policy-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-grok-default-profile-policy/grok-default-profile-policy-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-grok-default-profile-policy/grok-default-profile-policy-review.md`
- qa: `.codestable/features/2026-07-13-grok-default-profile-policy/grok-default-profile-policy-qa.md`
- acceptance: `.codestable/features/2026-07-13-grok-default-profile-policy/grok-default-profile-policy-acceptance.md`

## Nature
functional

## One-liner
ensure_default策略

## Core runtime path
见 design 验收场景

## Commands
cargo test --lib

## DoD / gates
1. implementation: checklist steps done + TDD evidence
2. cs-code-review → passed
3. QA → passed
4. acceptance → passed；更新 items.yaml status done

## Failure recovery
- review blocking → review-fix → re-review
- QA fail → qa-fix → re-review + QA
- need scope change → CS_ROADMAP_GOAL_HANDOFF

## Cleanliness
无 debug 输出、临时 TODO、注释死代码、无用 import。
