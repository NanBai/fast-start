# Goal Feature Spec: release-automation

## Roadmap item
- roadmap: session-launcher-next-wave
- roadmap_item: release-automation
- depends_on: []

## Paths
- feature_dir: `.codestable/features/2026-07-13-release-automation`
- design: `.codestable/features/2026-07-13-release-automation/release-automation-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-release-automation/release-automation-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-release-automation/release-automation-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-release-automation/release-automation-review.md`
- qa: `.codestable/features/2026-07-13-release-automation/release-automation-qa.md`
- acceptance: `.codestable/features/2026-07-13-release-automation/release-automation-acceptance.md`

## Nature
mixed

## One-liner
release脚本

## Core runtime path
见 design 验收场景

## Commands
bash scripts/release.sh dry-run or documented

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
