---
doc_type: audit-finding
audit: 2026-06-19-fullstack-audit
finding_id: "maintainability-04"
nature: maintainability
severity: P1
confidence: high
suggested_action: cs-refactor
status: open
---

# Finding 04：前端主组件与样式文件过大，职责混杂

## 速答

`App.tsx` 和 `App.css` 已经远超项目约定的单文件规模，图标、业务状态、筛选、分组、会话行、骨架屏、启动控制和样式都集中在两个文件中。

## 关键证据

- `src/App.tsx:1` — 当前文件 737 行，超过项目 `AGENTS.md` 中“文件<=300行”的维护目标。
- `src/App.tsx:29` — 图标集合直接内联在主文件中。
- `src/App.tsx:318` — `SessionRow` 组件位于主文件。
- `src/App.tsx:384` — `ProjectBucket` 组件位于主文件。
- `src/App.tsx:436` — `AgentGroup` 组件位于主文件。
- `src/App.tsx:527` — `App` 主状态和命令调用也在同一文件。
- `src/App.css:1` — 当前样式文件 1110 行，组件样式、布局、状态和响应式规则集中维护。

## 影响

后续继续加筛选、折叠持久化、搜索、虚拟列表或 UI 微调时，改动容易在同一文件互相干扰；样式选择器也更难判断影响范围。

## 修复方向

按 UI 边界拆为 `components/SessionRow`、`components/ProjectBucket`、`components/AgentGroup`、`components/Toolbar`、`hooks/useSessions`，样式同步拆分或建立明确 section 目录。

## 建议动作

`frontend-design` + `cs-refactor`，因为这是前端结构治理，应该保持现有视觉和行为不变，只降低维护成本。
