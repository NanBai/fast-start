# Goal Feature Spec: session-favorites-and-empty-states

## Roadmap item
- roadmap: session-launcher-next-wave
- roadmap_item: session-favorites-and-empty-states
- depends_on: []

## Paths
- feature_dir: `.codestable/features/2026-07-13-session-favorites-and-empty-states`
- design: `.codestable/features/2026-07-13-session-favorites-and-empty-states/session-favorites-and-empty-states-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-session-favorites-and-empty-states/session-favorites-and-empty-states-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-session-favorites-and-empty-states/session-favorites-and-empty-states-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-session-favorites-and-empty-states/session-favorites-and-empty-states-review.md`
- qa: `.codestable/features/2026-07-13-session-favorites-and-empty-states/session-favorites-and-empty-states-qa.md`
- acceptance: `.codestable/features/2026-07-13-session-favorites-and-empty-states/session-favorites-and-empty-states-acceptance.md`

## Nature
functional

## One-liner
session收藏+空错态

## Core runtime path
见 design 验收场景

## Commands
pnpm build; cargo test --lib if prefs rust

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
