---
doc_type: feature-acceptance
feature: 2026-06-19-audit-p1-p2-remediation
status: accepted
summary: 全栈审计 P1/P2 整改验收通过，覆盖启动解耦、scanner fixture、Cursor rusqlite、前端拆分、wrapper 安全收敛和 CSP
tags: [audit, performance, maintainability, security]
---

# audit-p1-p2-remediation 验收报告

> 阶段：阶段 3（验收闭环）
> 验收日期：2026-06-19
> 关联方案 doc：`.codestable/features/2026-06-19-audit-p1-p2-remediation/audit-p1-p2-remediation-design.md`

## 0. 验收阶段补修记录

- [x] **无业务补修**：验收未发现 design / 实现偏差，无需补修业务代码。
- [x] **架构归并**：按 design 第 4 节实际更新 `.codestable/architecture/ARCHITECTURE.md`，内容包括启动扫描解耦、前端目录结构、Cursor 进程内 SQLite 读取、统一 wrapper 和 CSP 安全口径。
- [x] **状态规范化**：checklist checks 接手时为 29 条 `pending`，逐项核对后规范化为 29 条全 `passed`；steps 保持 implement 阶段的 7 条 `done`。

## 1. 接口契约核对

对照 design 2.1 名词层：

- [x] `SessionScanner` trait 对外契约保持不变：仍是 `cli_type()` / `scan_sessions() -> Result<Vec<Session>, ScanError>`。
- [x] scanner fixture 注入只在测试入口暴露：`CodexScanner::with_root` / `ClaudeCodeScanner::with_root` / `CursorScanner::with_root` 均为 `#[cfg(test)]`。
- [x] Cursor workspace 读取节点已从外部 `sqlite3` 子进程改为 `rusqlite::Connection::open` 进程内读取，仍返回 canonicalized cwd。
- [x] `CommandSpec` / `Session` JSON 契约未变；未新增 CLI agent 类型。
- [x] launcher 内部复用 `write_command_wrapper`，Terminal.app / iTerm2 / Ghostty 均通过 wrapper 执行实际命令。
- [x] 前端拆分为 `App` 顶层编排 + `components/` 展示交互 + `lib/` 纯函数 + `styles/` 样式分片；`SessionRow` 优先显示 `session.summary`，覆盖 Claude Code 简介展示。

流程图核对：design 2.2 中 `Tauri setup -> manage AppState -> scan_sessions/refresh_sessions -> scan_all -> scanner -> 前端展示` 和 `launch_session -> validate CommandSpec -> wrapper -> Terminal/iTerm/Ghostty` 均有代码落点。

## 2. 行为与决策核对

**需求摘要逐项验证**：

- [x] App 启动不再被全量扫描阻塞：`src-tauri/src/lib.rs` setup 只加载偏好、校正终端、`app.manage(state)`，无 `state.scan_all()`。
- [x] 前端仍通过 `scan_sessions` / `refresh_sessions` 加载和刷新：`commands.rs` 中 `scan_sessions -> cached_scan`，`refresh_sessions -> scan_all`。
- [x] Cursor 扫描不再每 chat 启动 `sqlite3`：`rg "Command::new(\"sqlite3\")|sqlite3" src-tauri/src src-tauri/Cargo.toml` 无子进程调用命中，仅 Cargo.toml 有 rusqlite 依赖。
- [x] scanner 单测不依赖真实 HOME：codex / claude / cursor fixture 测试均通过。
- [x] 前端主界面按职责拆分：`src/App.tsx` 从单体编排收敛为数据加载 + 顶层布局，展示逻辑下沉到组件。
- [x] Terminal/iTerm 安全收敛：AppleScript 只注入 wrapper 路径，不直接注入完整业务命令。
- [x] Codex 简介扫描不再卡在前 64 行：单测覆盖第 65 行后真实用户消息。
- [x] CSP 非 null：`tauri.conf.json` 已设置最小可用 CSP。

**明确不做反向核对**：

- [x] 未新增 CLI agent 类型，未改变 `Session` 对外字段。
- [x] 未新增 Windows / Linux launcher。
- [x] 未引入新的视觉主题、营销页、远程同步入口。
- [x] 未新增网络请求能力解决扫描性能问题。
- [x] 未改写历史 audit / learning 事实记录。

**关键决策落地**：

- [x] 四条 lane 并行整改：前端、启动/scanner、launcher、安全配置均小范围落地。
- [x] Tauri command 契约不变：前端仍调用既有命令名。
- [x] wrapper 思路从 Ghostty 扩展到 Terminal/iTerm。
- [x] 前端是结构拆分，不是重做视觉风格。

**挂载点反向核对与拔除沙盘**：

- [x] `src-tauri/src/lib.rs::run`：移除 setup 预扫描；回退该改动会恢复启动阻塞。
- [x] `src-tauri/src/scanner.rs` 与 `scanner/*`：fixture 数据源 + Cursor 读取节点；回退会恢复测试假绿和扫描开销。
- [x] `src-tauri/src/launcher.rs`：Terminal/iTerm 复用 wrapper；回退会恢复 AppleScript 业务命令注入。
- [x] `src/App.tsx` + `src/components/*` + `src/lib/*` + `src/styles/*`：前端拆分；删除这些挂载点会让 UI 回到单体。
- [x] `src-tauri/tauri.conf.json`：CSP 非 null；回退会恢复安全债务。

## 3. 验收场景核对

| 场景 | 结果 | 证据 |
|---|---|---|
| 启动不阻塞扫描 | passed | 代码审查：setup 无 `scan_all`；`scan_sessions` 首次触发扫描 |
| 刷新仍强制扫描 | passed | 代码审查：`refresh_sessions -> state.scan_all()` |
| Cursor 大量 chat 扫描 | passed | `cargo test --lib` 通过；grep 无 `Command::new("sqlite3")` |
| scanner 测试无本机依赖 | passed | `cargo test --lib` 14 passed，fixture 覆盖 codex / claude / cursor |
| Codex 64 行边界 | passed | 单测 `scanner_reads_real_user_message_after_sixty_four_lines` 通过 |
| cd 语义一致 | passed | 单测 `command_spec_always_cds_to_session_project_dir` 通过；源码注释为三家都 cd |
| Terminal/iTerm 安全 wrapper | passed | 单测 `applescript_terminals_receive_wrapper_path_only` / `command_wrapper_cd_then_execs_command` 通过 |
| 前端行为保持 | passed | `pnpm build` 通过；浏览器首屏渲染检查看到最近天数、agent 控制、终端选择和提示；普通浏览器缺 Tauri IPC 导致 `invoke` 报错，属工具环境限制，不是真实 Tauri 代码失败 |
| CSP 生效且不破坏构建 | passed | `pnpm build` 与 `pnpm tauri build --debug` 均通过 |

补充验证命令：

- `pnpm build`：通过。
- `cd src-tauri && cargo test --lib`：14 passed。
- `pnpm tauri build --debug`：通过，产物为 `src-tauri/target/debug/bundle/macos/Session Launcher.app` 和 debug DMG。
- `python3 .codestable/tools/validate-yaml.py --file .../audit-p1-p2-remediation-checklist.yaml --yaml-only`：通过。

## 4. 术语一致性与禁用词反向 grep

design 定义禁用短语：`cursor 不 cd`、`cursor resume 不校验 cwd`、`project_dir 是占位`。

**grep 命令**：

```bash
rg -n "cursor 不 cd|cursor resume 不校验 cwd|project_dir 是占位" \
  src-tauri/src .codestable/architecture .codestable/features/2026-06-19-audit-p1-p2-remediation
```

**命中分类**：

- **源码 / 当前架构**：无命中。
- **本 feature design/checklist**：命中 8 处，均位于术语守护、验收标准或“旧语义清理”说明中，属于本 feature 文档内明确标注的禁用旧语义，允许保留。

最终结论：所有必须修项为 0；源码和当前架构文档无禁用词残留。

## 5. 架构归并

已实际更新 `.codestable/architecture/ARCHITECTURE.md`：

- [x] 概览扫描流程：从“app 启动 / 用户刷新扫描”改为“前端首次加载 / 用户刷新触发扫描，AppState 缓存结果”。
- [x] 核心模块：前端从单个 App 描述更新为 `App + components + lib + styles`。
- [x] 技术栈：补充 `rusqlite` 用于 Cursor `store.db` 进程内读取。
- [x] 数据流：补充 `Tauri setup -> manage AppState -> scan_sessions cached_scan / refresh_sessions scan_all`，并加入 CursorScanner 和 wrapper 节点。
- [x] CLI 覆盖：Cursor `store.db` cwd 来源说明改为 rusqlite 读取。
- [x] 终端策略：Terminal/iTerm/Ghostty 均通过统一 wrapper；Terminal/iTerm AppleScript 只注入 wrapper 路径。
- [x] 安全口径：移除旧 warn，改为当前统一 wrapper + `validate_command_spec` + CSP 非 null 约束。

## 6. requirement 回写

design frontmatter `requirement` 为空，且本 feature 是审计整改 / 技术债收敛，不新增独立用户愿景能力。

**结论**：无 requirement 回写。

## 7. roadmap 回写

design frontmatter 无 `roadmap` / `roadmap_item` 字段，feature 非 roadmap 起头。

**结论**：非 roadmap 起头，跳过。

## 8. attention.md 候选盘点

候选 1：`.codestable/attention.md` 当前写“非 git 仓库”，但本次验收实际可执行 `git status` 并看到工作树改动。建议退出 acceptance 后用 `cs-note` 修正该条，避免后续 agent 把真实 git 状态误判为无需检查。

候选 2：Vite dev 固定 `1420` 且 `strictPort: true`；端口被占用时 `pnpm tauri dev` 会失败，需要先处理占用进程。这是用户已遇到的环境坑，建议用 `cs-note` 追加。

## 9. 遗留

- `src-tauri/src/launcher.rs` 仍偏长；未来新增 Windows / Linux launcher 时建议单独走 `cs-refactor` 按 terminal 拆文件。
- 前端目录 convention 已跑通，design 2.5 建议后续用 `cs-decide` 归档：前端业务组件放 `src/components/`，跨组件纯函数放 `src/lib/`，状态编排 hook 放 `src/hooks/`，根目录只保留入口和顶层 App。
- 真实桌面点击启动仍建议用户终审再确认一次，自动化侧已完成构建、单测、debug 打包和浏览器首屏渲染检查。
