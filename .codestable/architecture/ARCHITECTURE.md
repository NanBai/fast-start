---
type: architecture
project: 快开CLI
version: 1.1
last_updated: 2026-06-18
---

# 快开CLI 架构文档

**项目简介**：一个 Tauri 桌面应用，聚合展示 codex / claude-code / cursor 三个 AI CLI agent 的最近 session，选中后一键拉起外部终端、cd 到工作目录、以 session ID resume 对应 agent，快速恢复工作上下文。

---

## 概览

桌面端三层结构：React 前端（展示与交互）↔ Tauri command 边界 ↔ Rust 后端（扫描 + 终端拉起）。

- **扫描**：app 启动 / 用户刷新时，并行调用各 `SessionScanner` 实现，从各 CLI 的本地存储读 session 元数据，聚合按时间排序。
- **恢复**：用户点"启动"→ 后端按 session_id 反查 → 对应 Scanner 产出 `CommandSpec` → 按 `preferred_terminal` 选 `TerminalLauncher` 实现 → 开窗、cd、执行 resume 命令。

详细设计见首个 feature：`.codestable/features/2026-06-17-cli-session-launcher/`。

---

## 名词层（核心类型）

| 类型 | 职责 | 定义位置 |
|---|---|---|
| `Session` | 一条 session 记录（id / cli_type / session_id / project_dir / project_name / last_active_at），`#[serde(rename_all="camelCase")]` 对齐前端 | `src-tauri/src/models.rs` |
| `CliType` | 枚举：codex / claude-code / cursor | `src-tauri/src/models.rs` |
| `TerminalType` | 枚举：System / ITerm2 / Ghostty | `src-tauri/src/models.rs` |
| `CommandSpec` | 纯数据：cwd + program + args，Scanner 输出给 Launcher，解耦"知道 resume 命令"与"知道怎么开窗" | `src-tauri/src/models.rs` |
| `SessionScanner` trait | `cli_type()` / `scan_sessions() -> Result<Vec<Session>, ScanError>`；每 CLI 一实现 | `src-tauri/src/scanner.rs` |
| `TerminalLauncher` trait | `terminal_type()` / `is_available()` / `launch(&CommandSpec)`；每终端一实现，跨平台预留 | `src-tauri/src/launcher.rs` |
| `AppState` | Tauri 管理的全局状态：sessions 缓存 + preferred_terminal | `src-tauri/src/state.rs` |

---

## 核心模块

```
src-tauri/src/
├── models.rs      Session / CliType / TerminalType / CommandSpec
├── scanner.rs     SessionScanner trait + CodexScanner / ClaudeCodeScanner / CursorScanner
├── launcher.rs    TerminalLauncher trait + SystemTerminalLauncher / ITerm2Launcher / GhosttyLauncher
├── security.rs    validate_command_spec / validate_session_id（注入防护）
├── state.rs       AppState（tauri-plugin-store 持久化偏好）
└── commands.rs    Tauri commands（scan_sessions / refresh_sessions / launch_session /
                   list_available_terminals / get/set_preferred_terminal）

src/               React 前端：App / TerminalSelector / SessionList / CliGroup / SessionRow
```

---

## 技术栈

- **桌面框架**：Tauri 2.x
- **后端**：Rust（扫描并行；tauri-plugin-store 持久化偏好）
- **前端**：React + TypeScript
- **平台**：v1 优先 macOS（Terminal.app / iTerm2 / Ghostty 三种外部终端，均走 osascript 拉起，开 tab 还是开窗见下"终端拉起策略"）；架构上 `TerminalLauncher` trait 为后续 Windows 兼容预留

---

## 数据流

```mermaid
flowchart TD
    A[App 启动] --> B[scan_sessions]
    B --> S1[CodexScanner.scan]
    B --> S2[ClaudeCodeScanner.scan]
    S1 --> D[聚合 & 按 last_active_at 排序]
    S2 --> D
    D --> E[前端渲染 SessionList]

    F[用户点击启动] --> G[launch_session]
    G --> H0[读 AppState.preferred_terminal]
    H0 --> H[Scanner 查 session_id → CommandSpec{cwd, program, args}]
    H --> HL[选 TerminalLauncher 实现]
    HL --> L[launch: 开窗 + cd + resume]
    L --> K{成功?}
    K -- 是 --> OK[前端显示成功]
    K -- 否 --> ERR[前端显示错误]
```

---

## CLI 覆盖

| CLI | session 存储 | cwd 来源 | resume 命令 |
|---|---|---|---|
| codex | `~/.codex/sessions/**/*.jsonl`（payload.cwd） | jsonl 直接读 | `codex resume <id>`（id 全局唯一，可不 cd） |
| claude-code | `~/.claude/projects/<编码>/<uuid>.jsonl` | jsonl 的 cwd 字段（优先，decode 歧义见 learning） | `claude --resume <id>` |
| cursor | `~/.cursor/chats/<hash>/<uuid>/{meta.json, store.db}` | store.db 里 cursor 注入的 `Workspace Path:`（system prompt，无歧义） | `cursor-agent --resume <id>`（**id 是 workspace 范围，必须 cd 到对应目录**） |

三家都是 `cd <cwd> && resume <id>` 模式（cursor 的 cd 必须准确，否则 resume 失败）。cursor 不用 `cursor-agent ls`（要 TTY）。

---

## 终端拉起策略（macOS）

三种终端开窗/tab 行为各异（实现期实测，详见 feature design 的 2.1 节）：

| 终端 | 已有窗口 | 无窗口 | 机制 |
|---|---|---|---|
| **iTerm2** | 开新 tab（`create tab with default profile`） | activate 后复用默认窗口 | osascript；app 名 `iTerm`（非 `iTerm2`，否则字典不加载报语法错） |
| **Ghostty** | 开新 tab（AppleScript `new tab with configuration`） | `open -na ... -e <wrapper>` 开首个窗口 | AppleScript 字典 + 临时 wrapper 脚本规避 login 误报；`-e` 自动 `quit-after-last-window-closed=true` |
| **Terminal.app** | 开新窗口（无法开 tab） | 开新窗口（冷启动会多一个空默认窗口，硬限制） | osascript `do script`；`make new tab` 不支持，模拟 ⌘T 需辅助功能权限 |

**Ghostty wrapper 脚本**：Ghostty 在 macOS 上把 `-e`/`--command` 套进 `/usr/bin/login`，多词命令弹误报。`GhosttyLauncher::launch` 生成临时脚本 `$TMPDIR/fast-start-ghostty/run-<pid>.sh`（`cd <cwd> && exec <program> <args>`，补 PATH 含 `~/.local/bin`），让 `-e` 只执行单脚本路径。

---

## 部署架构

纯本地桌面应用，无后端服务、无网络。偏好终端经 `tauri-plugin-store` 落地为 app 数据目录下 JSON。session 数据只冷扫描读取各 CLI 的本地存储，不持久化、不上传。

---

## 安全口径

launch 会拼装并执行 shell 命令，`project_dir` / `session_id` 来源于扫描外部 CLI 存储（不可信数据）：

- `CommandSpec.cwd` 必须 `canonicalize` 且为目录
- `program` 限白名单：`codex` / `claude` / `cursor-agent`
- `args` 中来自 session 数据的 id 字段做 UUID 字符集白名单校验
- **禁止**裸拼成 shell 字符串丢给 `sh -c`；osascript 脚本内动态片段用 `quoted form of` 转义

> 验收发现：实现侧 cwd/program/id 三道校验已扎实生效。Ghostty 经二次验收已端到端实测通过（wrapper 脚本规避 login 误报）。Terminal/iTerm2 的 osascript 调用仍走自定义转义 + shell 字符串拼接（间接经 shell），因上游已校验实际不可利用，记 warn；后续建议改 `std::Command` 参数化构造进一步收敛。
