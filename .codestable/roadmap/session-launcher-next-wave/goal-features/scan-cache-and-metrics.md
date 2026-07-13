# Goal Feature Spec: scan-cache-and-metrics

## Roadmap item
- roadmap: session-launcher-next-wave
- roadmap_item: scan-cache-and-metrics
- depends_on: []

## Paths
- feature_dir: `.codestable/features/2026-07-13-scan-cache-and-metrics`
- design: `.codestable/features/2026-07-13-scan-cache-and-metrics/scan-cache-and-metrics-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-scan-cache-and-metrics/scan-cache-and-metrics-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-scan-cache-and-metrics/scan-cache-and-metrics-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-scan-cache-and-metrics/scan-cache-and-metrics-review.md`
- qa: `.codestable/features/2026-07-13-scan-cache-and-metrics/scan-cache-and-metrics-qa.md`
- acceptance: `.codestable/features/2026-07-13-scan-cache-and-metrics/scan-cache-and-metrics-acceptance.md`

## Nature
functional

## One-liner
扫描缓存秒开+立即refresh+指标

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
