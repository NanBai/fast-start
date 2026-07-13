---
doc_type: feature-qa
feature: 2026-07-13-extend-grok-providers-tool
status: passed
reviewed: 2026-07-13
---

# extend-grok-providers-tool QA

## Commands

| ID | 结果 |
|---|---|
| CMD-001 `cd src-tauri && cargo test --lib` | 69 passed |
| CMD-002 `pnpm build` | tsc + vite ok |

## Core scenarios

| Scenario | 证据 | 结果 |
|---|---|---|
| S1 官方清理 | `activate_official_clears_provider_and_active` | pass |
| S2 loginRequired | 同上 + 无 auth | pass |
| S3 有 auth | `activate_official_with_auth_not_login_required` | pass |
| S4 回 API | `activate_then_official_then_api_again` | pass |
| S5/S5b 隐私 | `privacy_with_config_backs_up` / `privacy_without_config_creates_file` | pass |
| S10 backup 失败 | `activate_official_backup_failure_returns_err` 等 | pass |
| S12 无 config 官方 | `activate_official_without_config_skips_backup` | pass |
| S6/S7/S11 UI | 代码路径：layout 命令 + buildProviderCards 始终含 official；手工建议 `pnpm tauri dev` 点验 | pass（契约级） |

## Scope guard

- 无 systray / HTTP 管理面 / 拉模型列表：grep 无新增

## Review-fix recheck

- CR-001 Skeleton 门闩已改为 `grokStatus == null`
- clear_active 失败单测已加；S12 断言已固定
- `cargo test --lib grok_provider` 16 passed；`pnpm build` ok

## Verdict

**passed**
