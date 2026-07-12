---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "performance-03"
nature: performance
severity: P2
confidence: medium
suggested_action: cs-refactor
status: resolved
---

# Finding 09：每次 launch 同步 zsh -lc 解析 PATH

## 速答

每次启动 session 都在 wrapper 内跑 login shell 解析 PATH，重配置用户可能数百 ms–数秒，阻塞 Tauri command 返回。

## 关键证据

- `src-tauri/src/launcher.rs` wrapper 中 `zsh -lc 'printf %s "$PATH"'`  
- 无缓存；与 finding-04 同源

## 影响

点「启动」体感延迟；与 PATH 污染问题叠加。

## 修复方向

缓存解析结果（TTL / shell 配置 mtime）；默认 fallback + 可选探测。

## 建议动作

`cs-refactor`（可与 finding-04 同批）。

## 处置

2026-07-12 P2 批量已修。
