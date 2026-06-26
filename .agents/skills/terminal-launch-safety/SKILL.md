---
name: terminal-launch-safety
description: Use this skill in this repository for terminal launching, Ghostty, iTerm2, Terminal.app, AppleScript, wrapper scripts, PATH resolution, CommandSpec validation, CSP/capabilities, session deletion, destructive file operations, or any change near launcher.rs, security.rs, session_delete.rs, delete_session, or Tauri permissions. Use it whenever the task touches local command execution or filesystem deletion.
---

# Terminal Launch Safety

## Purpose

Use this skill to preserve the security boundary around local command execution and destructive session deletion.

## Read First

- `.codestable/attention.md`
- `.codestable/compound/2026-06-18-learning-macos-terminal-launch-pitfalls.md`
- `.codestable/compound/2026-06-18-decision-ghostty-launch-via-wrapper-script.md`
- `.codestable/issues/2026-06-19-ghostty-env-node-not-found/ghostty-env-node-not-found-fix-note.md`
- `src-tauri/src/launcher.rs`
- `src-tauri/src/security.rs`
- `src-tauri/src/session_delete.rs`
- `src-tauri/src/scanner.rs`
- `src-tauri/src/state.rs`
- `src-tauri/capabilities/default.json`
- `src-tauri/tauri.conf.json`

## Launch Facts

- Allowed launch programs are `codex`, `claude`, and `cursor-agent`.
- `validate_command_spec` canonicalizes cwd when `cd` is true and validates the program.
- All current CLI resume flows are `cd <cwd> && resume <id>`.
- Terminal.app cannot reliably open a new tab from AppleScript; new-tab mode falls back to a new window.
- iTerm2's AppleScript application name is `iTerm`, not `iTerm2`.
- Ghostty on macOS must execute a single wrapper script path; do not pass multi-word commands directly to `-e` or `--command`.
- The wrapper script resolves login shell PATH before `exec`, which fixes packaged app PATH issues such as `env: node: No such file or directory`.
- Tauri CSP is explicitly non-null in `src-tauri/tauri.conf.json`.
- Capabilities currently include `core:default`, `opener:default`, and `store:default`.

## Delete Facts

- Frontend sends only `Session.id`; backend finds the cached session and internal delete target.
- `Session.delete_target` is skipped during serialization.
- Deletion canonicalizes both root and path.
- Deletion rejects root itself and paths outside root.
- Codex and Claude Code delete jsonl files.
- Cursor deletes the chat directory.
- Delete failure must surface as an error and must not fake-remove the row.

## Workflow

1. Identify whether the change affects launch, delete, or Tauri permissions.
2. Preserve `CommandSpec` and `validate_command_spec` as the launch boundary.
3. Preserve wrapper-only execution for Terminal.app, iTerm2, and Ghostty.
4. For delete changes, verify root/path/kind invariants in `session_delete.rs`.
5. Update `.codestable/attention.md` or a CodeStable decision/learning document if a new terminal or platform pitfall is discovered.

## Commands

- Run Rust safety tests: `cd src-tauri && cargo test --lib`
- Run desktop smoke: `pnpm tauri dev`
- Search launch/delete boundaries: `rg "CommandSpec|validate_command_spec|delete_session|delete_target|Ghostty|iTerm|Terminal.app|csp|capabilities" src-tauri .codestable docs`
- Inspect Tauri capability config: `sed -n '1,160p' src-tauri/capabilities/default.json`

## Verification

- Run `cd src-tauri && cargo test --lib`.
- For launcher changes, inspect generated wrapper behavior and perform a real terminal smoke when feasible.
- For delete changes, use disposable fixture/session data only.
- For CSP/capability changes, explain why the permission is needed and how it is bounded.

## Do Not

- Do not build `sh -c` command strings from session data.
- Do not reintroduce direct Ghostty multi-word command execution.
- Do not use `iTerm2` as the AppleScript application name.
- Do not attempt to make Terminal.app open tabs via accessibility keystrokes.
- Do not set `csp` to `null`.
- Do not expose source session paths to React.
- Do not run delete smoke on important real sessions.
