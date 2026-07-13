---
doc_type: feature-acceptance
feature: 2026-07-13-launch-preflight
status: accepted
date: 2026-07-13
---

# launch-preflight Acceptance

## Prerequisites

- design: approved
- design-review: passed
- implementation steps: all done
- code review: passed
- QA: passed

## Checks

| Check | Status | Evidence |
|-------|--------|----------|
| 未知 id 走 Ok+session_not_found block | passed | unit + review |
| 缓存窗 source_unverified warn 且不拦 launch | passed | unit |
| OpenCode 源=行非 db；共享 check_session_source | passed | unit |
| PATH 与 launcher login 缓存一致 | passed | `resolve_program_on_launch_path` + design |
| launch 有 block 不启动 | passed | `launch_session` 早返回 |
| 不写 wrapper / 不做全文索引 | passed | 只读模块 |

## Deliverables

- `session_source.rs` / `launch_preflight.rs`
- Tauri `preflight_launch`
- launch 门闩
- 前端 Preflight 类型与启动前展示

## Architecture note

扩展 launch 安全口径：除 CommandSpec 外增加 preflight block 门闩；源探测模块供 health 复用。完整 ARCHITECTURE 回写并入 `power-extend-harden-and-docs`。

## Verdict

**accepted**
