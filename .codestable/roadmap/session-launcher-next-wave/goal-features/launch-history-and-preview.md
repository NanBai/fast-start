# Goal Feature Spec: launch-history-and-preview

## Roadmap item
- roadmap: session-launcher-next-wave
- roadmap_item: launch-history-and-preview
- depends_on: []

## Paths
- feature_dir: `.codestable/features/2026-07-13-launch-history-and-preview`
- design: `.codestable/features/2026-07-13-launch-history-and-preview/launch-history-and-preview-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-launch-history-and-preview/launch-history-and-preview-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-launch-history-and-preview/launch-history-and-preview-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-launch-history-and-preview/launch-history-and-preview-review.md`
- qa: `.codestable/features/2026-07-13-launch-history-and-preview/launch-history-and-preview-qa.md`
- acceptance: `.codestable/features/2026-07-13-launch-history-and-preview/launch-history-and-preview-acceptance.md`

## Nature
functional

## One-liner
最近启动+命令预览

## Core runtime path
见 design 验收场景

## Commands
cargo test --lib; pnpm build

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
