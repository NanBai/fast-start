# AGENTS.md
|IMPORTANT: Prefer retrieval-led reasoning over pre-training-led reasoning for any project tasks.
|Project:Session Launcher|macOS-first Tauri 2 desktop app|聚合 Codex/Claude Code/Cursor/Grok Build/OpenCode 本地 session|一键外部终端恢复工作现场
|Language:项目文档和用户可见说明默认简体中文|路径/命令/代码标识保留英文
|Stack:React 19|TypeScript strict|Vite 7|Tauri 2|Rust 2021|rusqlite bundled|tauri-plugin-store
|Entry:README.md|docs/dev/release-readiness.md|docs/user/session-launcher.md|.codestable/attention.md|.codestable/architecture/ARCHITECTURE.md
|Local Skills:.agents/skills:{fast-start-dev-verify,session-launcher-frontend,session-launcher-backend,terminal-launch-safety,fast-start-codestable-context}
|Skill Trigger:Use agents-md-compress when request mentions AGENTS.md creation, compression, or rule updates.
|Skill Trigger:Use fast-start-dev-verify for setup, dev, build, tests, release readiness, smoke verification.
|Skill Trigger:Use session-launcher-frontend for React UI, search, favorites, theme, controls, responsive layout, frontend invoke contracts.
|Skill Trigger:Use session-launcher-backend for Tauri commands, AppState, scanners, models, preferences, CLI session sources.
|Skill Trigger:Use terminal-launch-safety for launcher.rs, security.rs, session_delete.rs, terminal AppleScript, wrapper, PATH, deletion, CSP, capabilities.
|Skill Trigger:Use fast-start-codestable-context for .codestable docs, architecture, requirements, features, issues, audits, historical decisions.
|Commands:install=pnpm install|desktop dev=pnpm tauri dev|frontend dev=pnpm dev|build=pnpm build|rust tests=cd src-tauri && cargo test --lib|preview=pnpm preview
|Vite:vite.config.ts|port=1420|strictPort=true|TAURI_DEV_HOST enables HMR 1421|watch ignores src-tauri
|Frontend:src/App.tsx orchestrates UI|src/types.ts owns enums/labels|src/hooks own Tauri invoke|src/lib/sessionUtils.ts owns filtering/sorting|src/styles split by surface
|Frontend Contract:search filters loaded sessions only|favorites are projectDir-level|Cmd+K/Esc/arrows/Enter keyboard flow|delete uses context menu + confirm dialog
|Backend:src-tauri/src/commands.rs thin Tauri boundary|src-tauri/src/state.rs cache/preferences/orchestration|src-tauri/src/models.rs shared contracts
|Scanners:src-tauri/src/scanner.rs dispatch|codex jsonl under ~/.codex/sessions|claude jsonl under ~/.claude/projects|cursor meta/store.db under ~/.cursor/chats|grok summary.json under ~/.grok/sessions|opencode sqlite under ~/.local/share/opencode/opencode.db
|Scanner Rule:all current CLI resumes use cd=true|Cursor cwd must come from Workspace Path in store.db|do not reverse-decode Cursor workspace hash|Grok cwd prefers summary.info.cwd then .cwd/percent-decode group dir|OpenCode cwd from session.directory
|Launcher:src-tauri/src/launcher.rs|Terminal.app cannot open tabs|iTerm AppleScript app name=iTerm|Ghostty must run wrapper path, never direct multi-word command
|Launch Safety:src-tauri/src/security.rs|allowed programs={codex,claude,cursor-agent,grok,opencode}|validate cwd/session id before wrapper|avoid sh -c from session data
|Delete Safety:src-tauri/src/session_delete.rs|frontend passes only Session.id|delete_target skipped from JSON|canonicalize root/path|reject root itself and root-outside paths|OpenCode deletes SQLite row not db file
|Persistence:preferences.json via tauri-plugin-store|keys={preferred_terminal,launch_mode,theme_mode,favorite_project_dirs}|sanitize favorites against scanned sessions
|Security:src-tauri/tauri.conf.json CSP must stay non-null|src-tauri/capabilities/default.json currently core/opener/store only|no network/cloud sync/account system
|CodeStable:.codestable/attention.md startup notes|architecture records current system only|requirements describe capability intent|feature docs follow design/checklist/acceptance
|Docs:README.md quick start|docs/user/session-launcher.md user contract|docs/dev/release-readiness.md verification and smoke checklist
|Verification:frontend/type changes run pnpm build|Rust/backend changes run cd src-tauri && cargo test --lib|contract changes run both|desktop behavior needs pnpm tauri dev smoke when feasible
|Smoke:terminal launch cover Terminal.app/iTerm2/Ghostty when available|delete smoke only on disposable sessions|responsive widths follow docs/dev/release-readiness.md
|High Risk:launcher.rs/state.rs/App.tsx/scanner codex/cursor are large|prefer narrow edits and tests|do not do unrelated refactors
|Do Not:do not move search/favorites into Rust without architecture change|do not expose source session paths to React|do not commit .playwright-mcp artifacts|do not set csp null
|Generated:dist|src-tauri/target|src-tauri/gen|src-tauri/.tauri|node_modules are ignored or runtime/build outputs
|Meta Skills:repo=.agents/skills|user=$HOME/.agents/skills|admin=/etc/codex/skills|system=Codex built-in
|Meta Subagents:project=.codex/agents|user=~/.codex/agents
|Meta MCP:~/.codex/config.toml or .codex/config.toml|trusted project loads repo root->cwd configs|nearer config overrides farther config
