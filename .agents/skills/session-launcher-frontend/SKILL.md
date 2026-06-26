---
name: session-launcher-frontend
description: Use this skill in this repository when changing Session Launcher UI, session list rendering, search, recent-day filters, favorites, theme controls, launch controls, context menus, delete confirmation, keyboard behavior, responsive layout, frontend Tauri invoke contracts, or CSS. Use it for any task mentioning frontend, UI, React, controls, session row, favorite, search, theme, or responsive behavior.
---

# Session Launcher Frontend

## Purpose

Use this skill to modify the React UI without breaking the existing session-list workflow or duplicating frontend logic.

## Read First

- `docs/user/session-launcher.md`
- `src/App.tsx`
- `src/types.ts`
- `src/hooks/useSessions.ts`
- `src/hooks/usePreferences.ts`
- `src/lib/sessionUtils.ts`
- `src/components/AgentGroup.tsx`
- `src/components/ProjectBucket.tsx`
- `src/components/SessionRow.tsx`
- `src/components/Controls.tsx`
- `src/components/ConfirmDialog.tsx`
- `src/components/SessionContextMenu.tsx`
- `src/App.css`
- `src/styles/responsive.css`

## Architecture Facts

- `src/App.tsx` is the top-level interaction coordinator.
- Tauri calls are centralized in `src/hooks/useSessions.ts` and `src/hooks/usePreferences.ts`.
- Shared filtering, recent-day logic, favorite sorting, and favorite cleanup live in `src/lib/sessionUtils.ts`.
- Types and UI labels live in `src/types.ts`.
- Rendering hierarchy is `AgentGroup` -> `ProjectBucket` -> `SessionRow`.
- `src/App.css` imports the split CSS files from `src/styles/`.
- The default browser/WebView context menu is disabled in `src/main.tsx`.

## Existing UX Contract

- Search filters already loaded sessions; it does not trigger a scanner refresh.
- Search matches CLI type, agent label, project name, project path, and summary.
- `Cmd+K` focuses search.
- `Esc` clears search when non-empty, otherwise blurs the input.
- Up/down arrows move the active search result.
- `Enter` launches the active result.
- Favorites are project-directory based, not per-session.
- Terminal.app selected with new-tab mode shows a hint and opens a new window.
- Deletion uses a right-click context menu plus confirmation dialog.
- Delete failure must not pretend the row was removed.

## Workflow

1. Compare the requested UI change with `docs/user/session-launcher.md`.
2. Update `src/types.ts` first if command payloads, enum values, or labels change.
3. Prefer extending `src/lib/sessionUtils.ts` for list derivation rather than duplicating logic inside components.
4. Keep Tauri `invoke` calls inside the two hooks unless there is a strong reason to introduce a new hook.
5. Reuse `src/components/icons/Icon.tsx` and `BrandMark.tsx`; do not add a second icon system.
6. Place component-specific style changes in the existing CSS split and keep `src/App.css` as the import hub.
7. Update `docs/user/session-launcher.md` if user-visible behavior changes.

## Commands

- Build and type-check frontend: `pnpm build`
- Run desktop app for UI smoke: `pnpm tauri dev`
- Run frontend-only dev server: `pnpm dev`
- Run Rust tests when Tauri command payloads changed: `cd src-tauri && cargo test --lib`

## Verification

- Run `pnpm build` for TypeScript and frontend build verification.
- If Tauri command payloads changed, also run `cd src-tauri && cargo test --lib`.
- For responsive or layout changes, manually check the widths listed in `docs/dev/release-readiness.md`.

## Do Not

- Do not move search, favorites, or recent-day filtering into Rust unless the architecture changes deliberately.
- Do not pass backend-only delete source paths into React.
- Do not introduce visible help text for behaviors already documented outside the app.
- Do not add card-heavy or marketing-style UI; this is a compact desktop utility.
