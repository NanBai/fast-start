---
doc_type: feature-review
feature: 2026-07-13-launch-preflight
status: passed
reviewed: 2026-07-13
---

# launch-preflight Code Review

## Scope

- `src-tauri/src/session_source.rs`（共享源探测）
- `src-tauri/src/launch_preflight.rs`（纯函数预检 + 类型）
- `src-tauri/src/launcher/mod.rs`（`launch_path_string` / `resolve_program_on_launch_path`）
- `src-tauri/src/state/mod.rs`（`preflight_launch` + launch 门闩）
- `src-tauri/src/commands.rs` / `lib.rs`（command 注册）
- `src/types.ts` / `src/hooks/useSessions.ts`（checks 展示）

## Verdict

**passed**（无 unresolved blocking）

> 注：本轮因 Grok Build `SubagentCoordinator` panic（`parent session must exist when spawning subagents`），无法稳定 spawn 独立 Task agent reviewer；审查由 goal 会话按 design/checklist 对照 diff 完成，并在 QA 用真实 `cargo test` / `pnpm build` 证据复核。

## Design alignment

| 契约点 | 结论 |
|--------|------|
| 共享 `check_session_source` | 新模块，OpenCode 按行 SELECT，非 db 文件存在性 |
| 未知 id → Ok + `session_not_found` block | `preflight_session(None, …)` 覆盖 |
| 缓存窗 `source_unverified` warn 不拦 launch | 单测 + severity=warn |
| `launch_session` 先 preflight，block 不启动 | `block_messages` 早返回 |
| PATH 与 wrapper 同源 | login PATH + `~/.grok/bin` |
| preview 同组装路径 | `command_spec_for_session` |
| 不写 wrapper / 不改 preferences | 预检只读 |

## Findings

无 blocking / important。

### Nits（non-blocking）

1. `launchSession` 会 preflight 两次（前端一次 + `launch_session` 一次）——可接受，保证后端门闩独立。
2. 目录体积估算为浅遍历 + cap，供后续 disk-usage/health 复用；preflight 仅关心 Present/Missing。

## Gate results

- `cd src-tauri && cargo test --lib` → 102 passed
- `pnpm build` → success

## Test And QA Focus

1. OpenCode 行删库在 → `source_missing` block
2. `ops_ready=false` → warn only
3. 坏 cwd → block 中文 message，launch Err
4. program 不在 PATH → `program_not_found`
5. 前端启动失败 status 含 checks message
6. 不暴露 delete_target 路径到 React JSON
