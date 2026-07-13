# Goal Feature Spec: cli-extension-contract

## Roadmap item
- roadmap: session-launcher-power-extend
- roadmap_item: cli-extension-contract
- depends_on: []

## Paths
- feature_dir: `.codestable/features/2026-07-13-cli-extension-contract`
- design: `.codestable/features/2026-07-13-cli-extension-contract/cli-extension-contract-design.md` (approved)
- checklist: `.codestable/features/2026-07-13-cli-extension-contract/cli-extension-contract-checklist.yaml`
- design-review: `.codestable/features/2026-07-13-cli-extension-contract/cli-extension-contract-design-review.md` (passed)
- review: `.codestable/features/2026-07-13-cli-extension-contract/cli-extension-contract-review.md`
- qa: `.codestable/features/2026-07-13-cli-extension-contract/cli-extension-contract-qa.md`
- acceptance: `.codestable/features/2026-07-13-cli-extension-contract/cli-extension-contract-acceptance.md`

## Nature
non-functional

## One-liner
CliType 注册完整性测试 + docs checklist

## Core runtime path
见 design 验收场景；roadmap §4 硬约束不可改语义。

## Commands
cargo test --lib

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
