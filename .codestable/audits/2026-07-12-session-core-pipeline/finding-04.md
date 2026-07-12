---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "bug-04"
nature: bug
severity: P1
confidence: medium
suggested_action: cs-issue
status: resolved
---

# Finding 04：login shell PATH 解析易被 stdout 污染

## 速答

wrapper 用 `zsh/bash -lc 'printf %s "$PATH"'` 取 PATH，只检查非空；shell 启动脚本往 stdout 打字时 PATH 变成垃圾，agent 启动失败但前端可能仍显示「终端启动成功」。

## 关键证据

- `src-tauri/src/launcher.rs` `write_command_wrapper` 内 `resolve_login_path`  
- `PATH=$(resolve_login_path)` 后仅要求非空  
- osascript/`open` 成功 ≠ `exec grok/codex` 成功；前端 `useSessions.launchSession` 只看 invoke 是否抛错

## 影响

重度 shell 配置 / 打包 .app 用户：终端窗口打开但 CLI `command not found`，难排查。

## 修复方向

校验 PATH 形态（含 `:` 与已知目录）、取最后一行、失败强制 fallback；或缓存可靠 PATH。

## 建议动作

`cs-issue`。

## 处置

2026-07-12 已在代码中修复（P1 批量）。详见对应 scanner/launcher/ARCHITECTURE 改动。
