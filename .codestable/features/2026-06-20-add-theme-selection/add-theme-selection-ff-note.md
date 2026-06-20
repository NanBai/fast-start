---
doc_type: feature-ff-note
feature: add-theme-selection
date: 2026-06-20
requirement:
tags: [frontend, tauri, preference, theme]
---

## 做了什么
新增主题选择能力，用户可以在黑、白、跟随系统之间切换；主题偏好会持久化，重启应用后继续生效。

## 改了哪些
- `src-tauri/src/models.rs` — 新增 `ThemeMode` 偏好枚举。
- `src-tauri/src/state.rs` / `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` — 复用现有 Tauri Store 偏好链路，新增主题读取、保存和命令注册。
- `src/types.ts` / `src/App.tsx` / `src/components/Controls.tsx` — 前端新增主题类型、加载与切换状态、控制栏主题菜单。
- `src/styles/base.css` / `src/styles/controls.css` — 增加强制黑白主题覆盖和主题菜单视觉状态。

## 怎么验证的
已执行 `pnpm build`，前端 TypeScript 与 Vite 构建通过。
已执行 `cd src-tauri && cargo test --lib`，14 个 Rust lib 测试通过。
已执行 `cd src-tauri && cargo fmt --check`，Rust 格式检查通过。
