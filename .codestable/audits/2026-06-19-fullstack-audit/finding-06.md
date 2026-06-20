---
doc_type: audit-finding
audit: 2026-06-19-fullstack-audit
finding_id: "arch-drift-06"
nature: arch-drift
severity: P2
confidence: high
suggested_action: cs-arch check/update
status: open
---

# Finding 06：cd=false / cursor resume 注释与当前架构不一致

## 速答

多处注释仍保留“cursor resume 不 cd / cwd 是占位”的旧语义，但当前架构和代码已经明确三家 CLI 都应 cd 到工作目录。

## 关键证据

- `src-tauri/src/security.rs:38` — 注释写“cd=false（如 cursor resume）时不校验 cwd：project_dir 是占位”。
- `src-tauri/src/launcher.rs:48` — 注释写 `None（如 cursor resume）时直接跑 program`。
- `src-tauri/src/launcher.rs:189` — 注释写 `cd=false（cursor resume）时省略 cd`。
- `src-tauri/src/launcher.rs:328` — 注释写 `None（cursor resume）时省略`。
- `src-tauri/src/scanner.rs:92` — 当前代码说明三家 CLI 都是 `cd 到工作目录 && resume <id>`。
- `src-tauri/src/scanner.rs:95` — `CommandSpec { cd: true }`。
- `.codestable/architecture/ARCHITECTURE.md:96` — 架构文档明确“三家都是 cd <cwd> && resume <id> 模式”。

## 影响

当前行为没有错，但维护者后续看注释可能误以为 cursor 不需要 cwd 校验，从而引入回归。架构漂移类问题通常不会立刻炸，但会污染下一次实现决策。

## 修复方向

统一更新相关注释，删除 cursor 旧语义；如果未来还需要 `cd=false`，应把它描述为通用扩展能力，不绑定 cursor。

## 建议动作

`cs-arch check/update`，因为需要先确认当前架构文档为准，再同步代码注释或补充决策说明。
