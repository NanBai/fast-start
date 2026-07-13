---
doc_type: roadmap-goal-plan
roadmap: session-launcher-ux-perf
created: 2026-07-13
---

# Goal Plan: session-launcher-ux-perf

## 用户确认

- Roadmap：`approved` → active
- 全部 6 份 feature design：`approved`

## 执行顺序

1. app-shell-split  
2. state-module-split  
3. health-inspect-on-demand  
4. session-list-virtualize  
5. launch-feedback-polish  
6. ux-perf-harden-and-docs  

## 核心验收

见 roadmap §5；命令：`pnpm build`、`cd src-tauri && cargo test --lib`。

## DoD / Gate

与 power-extend goal-protocol 相同：steps done → review → QA → accept → scoped-commit。
