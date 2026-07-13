# Goal Feature Spec: session-bulk-delete

## Roadmap item
- roadmap: session-launcher-power-extend
- roadmap_item: session-bulk-delete
- depends_on: []

## Paths
- feature_dir: `.codestable/features/2026-07-13-session-bulk-delete`
- design: `.codestable/features/2026-07-13-session-bulk-delete/session-bulk-delete-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-session-bulk-delete/session-bulk-delete-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-session-bulk-delete/session-bulk-delete-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-session-bulk-delete/session-bulk-delete-review.md`
- qa: `.codestable/features/2026-07-13-session-bulk-delete/session-bulk-delete-qa.md`
- acceptance: `.codestable/features/2026-07-13-session-bulk-delete/session-bulk-delete-acceptance.md`

## Nature
functional

## One-liner
多选批量删除 + req 修订

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
