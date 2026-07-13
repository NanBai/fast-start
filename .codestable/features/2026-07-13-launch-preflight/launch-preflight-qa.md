---
doc_type: feature-qa
feature: 2026-07-13-launch-preflight
status: passed
date: 2026-07-13
---

# launch-preflight QA

## Commands

| ID | Command | Result |
|----|---------|--------|
| CMD-001 | `cd src-tauri && cargo test --lib` | **passed** — 102 tests |
| CMD-002 | `pnpm build` | **passed** — tsc + vite build |

## Scenario coverage

| 场景 | 证据 | 结果 |
|------|------|------|
| 未知 id → Ok+session_not_found | `unknown_session_is_ok_result_with_block` | passed |
| 缓存窗 source_unverified warn 不拦 | `cache_window_source_unverified_warn_does_not_block_when_rest_ok` | passed |
| OpenCode 源=行非 db | `opencode_uses_row_not_db_file` + `opencode_row_missing_blocks_even_if_db_exists` | passed |
| cwd_missing / cwd_not_dir | 对应 unit tests | passed |
| program_not_found via ResolveProgram | `program_not_found_blocks` | passed |
| 健康 session ok + preview | `healthy_session_ok_with_preview` | passed |
| 前端类型/构建 | `pnpm build` | passed |

## Manual smoke

桌面端坏 cwd 全链路（`pnpm tauri dev` 点启动）本轮未跑——属可延后手工；核心矩阵已由单测锁死，launch 门闩为同步纯函数调用，无异步竞态。

## Residual risks

- 生产 PATH 解析依赖 login shell 一次缓存；与 wrapper 一致，但极慢机器首次 launch 仍可能触发 shell 探测（既有行为）。
- 手工 smoke 未覆盖真实终端不启动（block 路径不调用 launcher）。

## Verdict

**passed**
