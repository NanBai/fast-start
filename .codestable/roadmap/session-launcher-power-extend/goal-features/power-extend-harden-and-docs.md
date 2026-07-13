# Goal Feature Spec: power-extend-harden-and-docs

## Roadmap item
- roadmap: session-launcher-power-extend
- roadmap_item: power-extend-harden-and-docs
- depends_on: all prior

## Paths
- feature_dir: `.codestable/features/2026-07-13-power-extend-harden-and-docs`
- design: `.codestable/features/2026-07-13-power-extend-harden-and-docs/power-extend-harden-and-docs-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-power-extend-harden-and-docs/power-extend-harden-and-docs-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-power-extend-harden-and-docs/power-extend-harden-and-docs-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-power-extend-harden-and-docs/power-extend-harden-and-docs-review.md`
- qa: `.codestable/features/2026-07-13-power-extend-harden-and-docs/power-extend-harden-and-docs-qa.md`
- acceptance: `.codestable/features/2026-07-13-power-extend-harden-and-docs/power-extend-harden-and-docs-acceptance.md`

## Nature
non-functional

## One-liner
交叉回归与文档收口

## Core runtime path
见 design 验收场景；roadmap §4 硬约束不可改语义。

## Commands
pnpm build; cargo test --lib

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
