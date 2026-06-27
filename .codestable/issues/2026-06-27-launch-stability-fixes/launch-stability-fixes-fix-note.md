---
doc_type: issue-fix
issue: 2026-06-27-launch-stability-fixes
path: fast-track
fix_date: 2026-06-27
tags: [launcher, preferences, startup]
---

# 启动稳定性修复记录

## 1. 实际采用方案

基于代码 review 发现的 4 项 P1/P2 问题，采用最小改动：

1. **wrapper 并发覆盖**：`run-{pid}.sh` 改为 `run-{uuid}.sh`，每次 launch 独立脚本。
2. **首屏卡死**：`App.tsx` 启动时 `loadPreferences` 失败仍继续 `loadSessions`，并展示错误。
3. **iTerm 无限等待**：冷启动等待窗口改为最多 300 次 × 0.1s，超时 AppleScript 报错。
4. **偏好持久化顺序**：`commands.rs` / `lib.rs` 改为先 `save_*` 再更新内存 state。

## 2. 改动文件清单

| 文件 | 改动 |
|------|------|
| `src-tauri/src/launcher.rs` | UUID wrapper 文件名；iTerm 超时；新增并发路径单测 |
| `src/App.tsx` | 启动 effect 对偏好加载加 try/catch |
| `src-tauri/src/commands.rs` | 终端/打开方式/主题偏好先持久化再改 state |
| `src-tauri/src/lib.rs` | setup 中不可用终端回退同样先 save 再 set |

## 3. 验证结果

- [x] `cd src-tauri && cargo test --lib` — 24 passed
- [x] `pnpm build` — 通过
- [x] wrapper 单测 `command_wrapper_uses_unique_paths_for_concurrent_launches` 确认两次 launch 路径不同
- [ ] 桌面 smoke（并发双 session 启动、iTerm 冷启动）— 未在本机自动化执行

## 4. 遗留事项

（无）

---

## 5. 追加修复（2026-06-27，原「顺手发现」）

| 问题 | 方案 | 文件 |
|------|------|------|
| 最近 N 天过滤对无效日期放行 | 解析失败时排除，不再 `NaN \|\| >= cutoff` | `src/lib/sessionUtils.ts` |
| 每次扫描 session.id 重新 UUID | `Session::stable_id()` 用 UUID v5(cli + session_id + project_dir) | `models.rs` + 三个 scanner |

验证：`cargo test --lib` 26 passed；`pnpm build` 通过。
