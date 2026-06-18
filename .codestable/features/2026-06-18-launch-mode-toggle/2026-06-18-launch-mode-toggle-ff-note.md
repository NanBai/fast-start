---
doc_type: feature-ff-note
feature: 2026-06-18-launch-mode-toggle
date: 2026-06-18
tags: [launcher, launch-mode, terminal, preference]
---

## 做了什么

加了一个全局"打开方式"选项（新标签页 / 新窗口）。用户可选择启动 session 时是开新 tab 还是开新窗口，偏好持久化（关 app 重开记住）。Terminal.app 不支持开 tab（AppleScript 硬限制），选"新标签页"时自动回退到开新窗口并提示。

## 改了哪些

- `src-tauri/src/models.rs` — 新增 `LaunchMode` 枚举（NewTab / NewWindow，kebab-case 序列化）
- `src-tauri/src/launcher.rs` — `TerminalLauncher` trait 的 `launch` 加 `mode` 参数 + 新增 `supports_tab()` 默认方法；`SystemTerminalLauncher` 覆盖 `supports_tab()` 返回 false；iTerm2 的 applescript 拆成 `iterm_open_tab_applescript` / `iterm_open_window_applescript` 按 mode 选；Ghostty `launch` 按 mode 决定开 tab 还是窗口（NewTab+无窗口回退到窗口）
- `src-tauri/src/state.rs` — AppState 加 `launch_mode` 字段 + getter/setter；`launch_session` 读 mode 传入，Terminal 选 NewTab 时回退 NewWindow；新增 `load_launch_mode` / `save_launch_mode` 持久化（store key `launch_mode`，默认 NewTab）
- `src-tauri/src/commands.rs` — 新增 `get_launch_mode` / `set_launch_mode` 命令
- `src-tauri/src/lib.rs` — 启动加载 launch_mode + `AppState::new` 加参数 + 注册两个新命令
- `src/types.ts` — 加 `LaunchMode` 类型 + `LAUNCH_MODE_LABELS`
- `src/App.tsx` — 加 `launchMode` state + `handleLaunchModeChange` + `LaunchModeSelector` 组件（header 下拉）+ Terminal 选 NewTab 时的提示条

## 怎么验证的

- `cargo build` 绿灯 + `cargo test --lib` 4 测试全过
- `tsc --noEmit` 干净
- 用户人工验证通过 [2026-06-18]：切换下拉持久化生效；iTerm2/Ghostty 选"新窗口"开窗口、选"新标签页"开 tab；Terminal 选"新标签页"显示提示并实际开窗口

## 顺手发现（可选，不阻塞）

- 重启 app 时 `pnpm tauri dev` 偶发 "Port 1420 is already in use"——上次 dev 进程没完全退出。tauri watcher 会自动重编译恢复，但偶尔需要手动 `pkill -f "target/debug/tauri-app"` 再重启。不在本次范围。
