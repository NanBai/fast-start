---
name: session-launcher-backend
description: Use this skill in this repository when changing Rust/Tauri backend commands, AppState, preferences, scanners for Codex/Claude Code/Cursor, command specs, session models, cached scans, or Tauri configuration. Use it for tasks mentioning scan_sessions, refresh_sessions, launch_session, preferences.json, scanner, Cursor store.db, Codex jsonl, Claude project jsonl, or adding a new CLI source.
---

# Session Launcher Backend

## Purpose

Use this skill to work on the Rust/Tauri backend while preserving the scanner-state-command boundaries.

## Read First

- `.codestable/architecture/ARCHITECTURE.md`
- `.codestable/attention.md`
- `src-tauri/src/models.rs`
- `src-tauri/src/scanner.rs`
- `src-tauri/src/scanner/codex.rs`
- `src-tauri/src/scanner/claude_code.rs`
- `src-tauri/src/scanner/cursor.rs`
- `src-tauri/src/state.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml`

## Architecture Facts

- Tauri command functions live in `src-tauri/src/commands.rs`.
- Application state, session cache, scan errors, terminal mode, theme, and favorite project dirs live in `src-tauri/src/state.rs`.
- `scan_sessions` returns cached results after the first scan; `refresh_sessions` forces `scan_all`.
- Scanner implementations are selected by `scanners()` in `src-tauri/src/scanner.rs`.
- `command_spec_for_session` maps each session to `CommandSpec`.
- All current CLI resume flows set `cd: true`.
- Codex sessions come from `~/.codex/sessions/**/*.jsonl`.
- Claude Code sessions come from `~/.claude/projects/<encoded>/<uuid>.jsonl`, with file `cwd` preferred over directory decode fallback.
- Cursor sessions come from `~/.cursor/chats/<hash>/<uuid>/{meta.json, store.db}` and require `Workspace Path:` from `store.db`.
- `favorite_project_dirs` is local preference data and affects frontend sorting, not scanner output.

## Workflow

1. Start from the command or data contract in `src-tauri/src/models.rs`.
2. Keep scanner parsing in the scanner module for that CLI.
3. Keep cache, preference, and command orchestration in `src-tauri/src/state.rs`.
4. Keep Tauri command wrappers thin in `src-tauri/src/commands.rs`.
5. Add fixture-backed tests for scanner changes; do not depend on the user's real home data.
6. If frontend payloads change, update `src/types.ts` and the relevant hook.

## Reuse Points

- `clean_summary` for list summaries.
- `decode_claude_project_dir` only as a fallback, never as the primary cwd source when file cwd exists.
- `command_spec_for_session` for all launch command generation.
- `normalize_project_dirs_for_sessions` for favorite cleanup before persistence.

## Commands

- Run Rust library tests: `cd src-tauri && cargo test --lib`
- Build frontend when serialized contracts changed: `pnpm build`
- Run desktop app for command smoke: `pnpm tauri dev`
- Search backend call sites: `rg "scan_sessions|refresh_sessions|launch_session|delete_session|CommandSpec" src-tauri src`

## Verification

- Run `cd src-tauri && cargo test --lib`.
- Run `pnpm build` when serialized models, Tauri command names, or frontend payloads change.

## Do Not

- Do not push search, recent-day filtering, or favorite sorting into scanner code.
- Do not rely on real `~/.codex`, `~/.claude`, or `~/.cursor` data in tests.
- Do not create a second source of truth for CLI labels or enum values.
- Do not pass delete source paths through frontend JSON; `Session.delete_target` is intentionally skipped during serialization.
