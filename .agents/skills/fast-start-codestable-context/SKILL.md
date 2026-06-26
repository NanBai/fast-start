---
name: fast-start-codestable-context
description: Use this skill in this repository when working with CodeStable docs, architecture, requirements, feature or issue records, audits, historical decisions, project context, or when a task asks "how is this project structured", "what did we decide before", "update architecture", "write docs", "acceptance", or "avoid breaking past decisions". Use it before relying on memory for project history.
---

# Fast Start CodeStable Context

## Purpose

Use this skill to retrieve project-specific history and constraints from `.codestable/` before changing code or docs.

## Read First

- `.codestable/attention.md`
- `.codestable/architecture/ARCHITECTURE.md`
- `.codestable/reference/shared-conventions.md`
- `.codestable/reference/system-overview.md`
- `.codestable/reference/maintainer-notes.md`
- The active feature, issue, requirement, or audit file related to the task.

## Repository Facts

- `.codestable/attention.md` is the startup note for CodeStable project constraints.
- `.codestable/architecture/ARCHITECTURE.md` records current architecture, not future plans.
- Requirements live under `.codestable/requirements/`.
- Feature records live under `.codestable/features/<date-slug>/`.
- Issue records live under `.codestable/issues/<date-slug>/`.
- Audits live under `.codestable/audits/`.
- Reusable learnings, decisions, and tricks live under `.codestable/compound/`.
- Helper scripts live under `.codestable/tools/`.

## Current Project Context

- Product: Session Launcher, a macOS-first Tauri desktop app for local AI CLI sessions.
- Core agents: Codex, Claude Code, Cursor.
- Core user workflows: scan sessions, filter/search, favorite project dirs, launch in external terminal, delete a local session source.
- Known high-risk areas: terminal launcher, Cursor store.db parsing, delete target safety, Tauri command contract, responsive control bar.

## Workflow

1. Read `.codestable/attention.md` before acting.
2. Locate the relevant feature/issue/audit/requirement with `rg --files .codestable`.
3. Prefer current architecture facts from `.codestable/architecture/ARCHITECTURE.md`.
4. If a previous design or acceptance exists for the task area, compare code against it before editing.
5. When updating docs, preserve existing valid content and append or revise narrowly.
6. If a new long-lived rule is discovered, store it in the correct CodeStable artifact instead of only mentioning it in chat.

## CodeStable Rules

- Feature work normally flows design -> checklist -> implementation -> acceptance.
- Issue work normally flows report -> analysis -> fix note.
- Architecture docs should describe the current implemented system.
- Requirements describe user-facing capability intent.
- Do not rewrite completed artifacts just to change style.
- On resume, check existing artifacts and continue from the first incomplete section.

## Commands

- List CodeStable files: `rg --files .codestable`
- Search CodeStable text: `rg "<term>" .codestable`
- Search frontmatter metadata: `python3 .codestable/tools/search-yaml.py --dir .codestable/compound --query "<term>"`
- Validate markdown frontmatter: `python3 .codestable/tools/validate-yaml.py --dir .codestable`
- Validate a YAML checklist: `python3 .codestable/tools/validate-yaml.py --file <path> --yaml-only`

## Verification

- For YAML checklist edits, run the relevant `.codestable/tools/validate-yaml.py` command when applicable.
- For doc-only changes, verify paths and referenced commands exist.
- For architecture or acceptance changes, cross-check code paths named in the document.

## Do Not

- Do not treat old audit findings as current without checking whether a remediation acceptance exists.
- Do not copy the whole CodeStable process into `AGENTS.md`; keep `AGENTS.md` as a compact entry index.
- Do not invent project decisions that are not backed by `.codestable/`, code, config, tests, or current user instruction.
