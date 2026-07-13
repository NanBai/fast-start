# Goal Feature Spec: wave-harden-and-docs

## Roadmap item
- roadmap: session-launcher-next-wave
- roadmap_item: wave-harden-and-docs
- depends_on: ['all prior']

## Paths
- feature_dir: `.codestable/features/2026-07-13-wave-harden-and-docs`
- design: `.codestable/features/2026-07-13-wave-harden-and-docs/wave-harden-and-docs-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-wave-harden-and-docs/wave-harden-and-docs-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-wave-harden-and-docs/wave-harden-and-docs-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-wave-harden-and-docs/wave-harden-and-docs-review.md`
- qa: `.codestable/features/2026-07-13-wave-harden-and-docs/wave-harden-and-docs-qa.md`
- acceptance: `.codestable/features/2026-07-13-wave-harden-and-docs/wave-harden-and-docs-acceptance.md`

## Nature
non-functional

## One-liner
文档回归收口

## Core runtime path
none — 文档/验证聚合；替代证据为命令输出与文档 diff

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
