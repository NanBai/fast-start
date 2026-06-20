---
doc_type: audit-finding
audit: 2026-06-19-fullstack-audit
finding_id: "security-05"
nature: security
severity: P2
confidence: medium
suggested_action: cs-refactor
status: open
---

# Finding 05：Terminal/iTerm 仍通过 shell 字符串注入终端

## 速答

Terminal.app 和 iTerm 启动链路仍先拼 shell 命令字符串，再通过 AppleScript `do script` / `write text` 注入终端执行；当前有白名单和转义，实际风险受控，但仍是架构文档标注过的安全债务。

## 关键证据

- `src-tauri/src/launcher.rs:48` — `build_shell_command` 明确“拼成 shell 命令字符串”。
- `src-tauri/src/launcher.rs:51` — cwd 被拼入 `cd <cwd> && ...`。
- `src-tauri/src/launcher.rs:56` — program 被拼入同一 shell 字符串。
- `src-tauri/src/launcher.rs:281` — Terminal.app 使用 `build_shell_command`。
- `src-tauri/src/launcher.rs:300` — iTerm 使用 `build_shell_command`。
- `.codestable/architecture/ARCHITECTURE.md:127` — 架构安全口径写明禁止裸拼 shell 字符串给 `sh -c`，动态片段要收敛转义。
- `.codestable/architecture/ARCHITECTURE.md:129` — 文档已把 Terminal/iTerm 的 shell 字符串拼接记录为 warn。

## 影响

目前 `validate_command_spec`、program 白名单和 session id 校验降低了可利用性；但这条链路依旧对未来字段扩展、转义遗漏、非预期参数更敏感。安全边界靠多处约定维持，维护成本偏高。

## 修复方向

参考 Ghostty wrapper，把 Terminal/iTerm 也改成执行受控 wrapper 路径，AppleScript 只传单个脚本路径，命令参数由 Rust 生成并校验。

## 建议动作

`cs-refactor`，因为这是安全债务收敛，目标是减少 shell 层暴露面。
