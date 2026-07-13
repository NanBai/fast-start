# Goal Feature Spec: port-group-and-terminate-by-project

## Roadmap item
- roadmap: session-launcher-power-extend
- roadmap_item: port-group-and-terminate-by-project
- depends_on: port-protect-list

## Paths
- feature_dir: `.codestable/features/2026-07-13-port-group-and-terminate-by-project`
- design: `.codestable/features/2026-07-13-port-group-and-terminate-by-project/port-group-and-terminate-by-project-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-port-group-and-terminate-by-project/port-group-and-terminate-by-project-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-port-group-and-terminate-by-project/port-group-and-terminate-by-project-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-port-group-and-terminate-by-project/port-group-and-terminate-by-project-review.md`
- qa: `.codestable/features/2026-07-13-port-group-and-terminate-by-project/port-group-and-terminate-by-project-qa.md`
- acceptance: `.codestable/features/2026-07-13-port-group-and-terminate-by-project/port-group-and-terminate-by-project-acceptance.md`

## Nature
functional

## One-liner
Port 按 cwd 分组 UI

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
