---
name: fast-start-dev-verify
description: Use this skill for this repository whenever the task involves local setup, running the app, build failures, type checks, Rust tests, release readiness, smoke testing, or choosing the right validation command for Session Launcher. Use it even if the user only says "run it", "verify", "build", "test", "dev server", "Tauri dev", or "release check".
---

# Fast Start Dev Verify

## Purpose

Use this skill to run or explain the minimum reliable development and verification path for this Tauri + React project.

## Read First

- `README.md`
- `docs/dev/release-readiness.md`
- `package.json`
- `vite.config.ts`
- `tsconfig.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `.codestable/attention.md`

## Project Facts

- This is a macOS-first Tauri 2 desktop app named Session Launcher.
- Frontend code lives in `src/`; Rust/Tauri code lives in `src-tauri/`.
- Package scripts are `pnpm dev`, `pnpm build`, `pnpm preview`, and `pnpm tauri`.
- `pnpm build` runs `tsc && vite build`.
- There is no dedicated JS test or lint script in `package.json`.
- Vite dev server uses port `1420` with `strictPort: true`; `TAURI_DEV_HOST` enables HMR on `1421`.
- Rust library tests must be run from `src-tauri/` with `cargo test --lib`.
- `.playwright-mcp/` is a local verification artifact and should not be committed.

## Commands

- Install dependencies: `pnpm install`
- Run the desktop app: `pnpm tauri dev`
- Run frontend only: `pnpm dev`
- Build frontend: `pnpm build`
- Preview frontend build: `pnpm preview`
- Run Rust library tests: `cd src-tauri && cargo test --lib`

## Workflow

1. Check `git status --short` before changing or validating anything.
2. Choose the smallest sufficient verification:
   - frontend or type changes: `pnpm build`
   - Rust scanner/state/launcher/delete changes: `cd src-tauri && cargo test --lib`
   - Tauri command contract changes: run both commands above
   - user-facing desktop behavior: add manual `pnpm tauri dev` smoke notes
3. When validating release readiness, follow `docs/dev/release-readiness.md`.
4. Report commands exactly as run and whether they passed.

## Verification

- Documentation-only changes can use structural checks plus `git status --short`.
- Frontend or TypeScript changes should pass `pnpm build`.
- Rust backend changes should pass `cd src-tauri && cargo test --lib`.
- Tauri command contract changes should pass both frontend build and Rust tests.
- User-facing desktop behavior needs `pnpm tauri dev` smoke when feasible.

## Smoke Coverage

For desktop smoke checks, cover:

- app window opens on macOS
- Codex / Claude Code / Cursor groups render or show clear empty/error states
- refresh rescans
- recent-day filter changes visible sessions
- search supports `Cmd+K`, `Esc`, arrow navigation, and `Enter`
- favorites persist across refresh or restart
- theme persists across refresh or restart
- Terminal.app, iTerm2, and Ghostty launch behavior when available
- deletion only on disposable sessions
- responsive widths listed in `docs/dev/release-readiness.md`

## Do Not

- Do not claim JS tests or lint exist unless `package.json` changes.
- Do not run destructive deletion smoke on important real sessions.
- Do not treat `pnpm dev` as full desktop validation; it only starts the frontend server.
- Do not commit `.playwright-mcp/` artifacts.
