---
doc_type: audit-finding
audit: 2026-06-19-fullstack-audit
finding_id: "performance-01"
nature: performance
severity: P1
confidence: high
suggested_action: cs-refactor
status: open
---

# Finding 01：Tauri setup 同步扫描会阻塞应用首屏

## 速答

应用初始化阶段直接执行全量 session 扫描，任一 scanner 慢都会拖住窗口启动和首屏反馈。

## 关键证据

- `src-tauri/src/lib.rs:20` — `.setup(|app| { ... })` 在 Tauri 启动期间执行初始化逻辑。
- `src-tauri/src/lib.rs:34` — `state.scan_all()?;` 在 `app.manage(state)` 前同步扫描全部 CLI。
- `src-tauri/src/state.rs:38` — `scan_all` 创建所有 scanner 并等待结果。
- `src-tauri/src/state.rs:53` — 主线程逐个 `join` scanner 线程，最慢 scanner 决定返回时间。

## 影响

用户本地 session 多、Cursor `store.db` 多、外部目录 IO 慢时，应用可能表现为启动慢或窗口迟迟不出现。这个问题与后续前端 loading 无关，因为前端还没拿到可渲染机会。

## 修复方向

首屏只加载持久化偏好并立刻 `manage` 状态；扫描改为前端 `scan_sessions` 首次调用触发，或后台线程异步扫描并通过事件通知前端。

## 建议动作

`cs-refactor`，因为这是行为不变的启动链路重排，核心是降低同步阻塞而不是改业务语义。
