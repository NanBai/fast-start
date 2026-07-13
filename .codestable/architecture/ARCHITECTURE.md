---
type: architecture
project: 快开CLI
version: 1.3
last_updated: 2026-07-12
---

# 快开CLI 架构文档

**项目简介**：一个 Tauri 桌面应用，聚合展示 codex / claude-code / cursor / grok-build / opencode 五个 AI CLI agent 的最近 session，选中后一键拉起外部终端、cd 到工作目录、以 session ID resume 对应 agent，快速恢复工作上下文。另提供本机开发端口监控（Port 工具页，见 `port_monitor`），以及 Grok 登录方式管理（Grok 工具页，见 `grok_provider`：API 供应商档案 / 官方账号切换 / 隐私保护本地配置 / 卡片排序置顶）。

---

## 概览

桌面端三层结构：React 前端（展示与交互）↔ Tauri command 边界 ↔ Rust 后端（扫描 + 终端拉起 + 本地 session 删除）。

- **扫描**：前端首次加载 / 用户刷新时，通过 Tauri command 触发 `AppState` 扫描；`scan_sessions` 复用内存缓存，冷启动可先读本机 `scan-cache-v1.json` 秒开列表（`fromCache=true`，无 `delete_target`），前端随即 `refresh_sessions` 全量扫描；`refresh_sessions` 强制 full scan 并原子写回 snapshot。后端并行调用各 `SessionScanner` 实现，从各 CLI 的本地存储读 session 元数据，聚合按时间排序；响应可带 `scanDurationMs`。
- **快速访问**：搜索、最近天数和项目收藏都在前端从扫描结果派生；稳定顺序是“最近天数过滤 → 搜索过滤 → 收藏排序 → agent/project 渲染”。搜索不下推到 scanner，不触发重新扫描。
- **恢复**：用户点"启动"→ 后端按 session_id 反查 → 对应 Scanner 产出 `CommandSpec` → 按 `preferred_terminal` 选 `TerminalLauncher` 实现 → 开窗、cd、执行 resume 命令。
- **删除**：用户右键单条 session → 前端确认 → `delete_session` 按前端 `Session.id` 反查缓存里的后端内部删除目标 → 校验 root/path/kind → 删除对应 CLI session 源文件或 chat 目录 → 从缓存移除当前行。

详细设计见首个 feature：`.codestable/features/2026-06-17-cli-session-launcher/`。

---

## 名词层（核心类型）

| 类型 | 职责 | 定义位置 |
|---|---|---|
| `Session` | 一条 session 记录（id / cli_type / session_id / project_dir / project_name / last_active_at），`#[serde(rename_all="camelCase")]` 对齐前端；内部 `delete_target` 用 `#[serde(skip)]` 保留源载体，不进入前端 JSON | `src-tauri/src/models.rs` |
| `CliType` | 枚举：codex / claude-code / cursor / grok-build / opencode | `src-tauri/src/models.rs` |
| `TerminalType` | 枚举：System / ITerm2 / Ghostty | `src-tauri/src/models.rs` |
| `CommandSpec` | 纯数据：cwd + program + args，Scanner 输出给 Launcher，解耦"知道 resume 命令"与"知道怎么开窗" | `src-tauri/src/models.rs` |
| `SessionDeleteTarget` | 后端内部删除目标：root / path / kind；Codex 和 Claude Code 指向 jsonl 文件，Cursor / Grok Build 指向 session/chat 目录 | `src-tauri/src/models.rs` |
| `SessionScanner` trait | `cli_type()` / `scan_sessions() -> Result<Vec<Session>, ScanError>`；每 CLI 一实现 | `src-tauri/src/scanner.rs` |
| `TerminalLauncher` trait | `terminal_type()` / `is_available()` / `launch(&CommandSpec)`；每终端一实现，跨平台预留 | `src-tauri/src/launcher.rs` |
| `QuickAccessOptions` / `QuickAccessResult` | 前端列表派生契约：用 sessions、recentDays、query、favoriteProjectDirs 算出当前可见 sessions、匹配数量和活跃项 | `src/lib/sessionUtils.ts` |
| `favorite_project_dirs` | 本机偏好里的收藏项目列表，key 是精确 `projectDir` 字符串；只影响前端排序，不进入 `Session` / `ScanResponse` | `src-tauri/src/state.rs` |
| `AppState` | Tauri 管理的全局状态：sessions 缓存 + scan_errors + preferred_terminal + launch_mode + theme_mode + favorite_project_dirs + scan-cache 路径 / ops_ready | `src-tauri/src/state/` |
| `scan-cache-v1.json` | 本机扫描快照（app_data）；仅可序列化 Session 字段，永不写 `delete_target`；缓存窗删除文件型 CLI 需先 full scan | `src-tauri/src/state/scan_cache.rs` |

---

## 核心模块

```
src-tauri/src/
├── models.rs      Session / CliType / TerminalType / CommandSpec
├── scanner.rs     SessionScanner trait + 分发
├── scanner/
│   ├── codex.rs
│   ├── claude_code.rs
│   ├── cursor.rs
│   ├── grok_build.rs
│   └── opencode.rs
├── launcher.rs    TerminalLauncher trait + System / iTerm2 / Ghostty
├── session_delete.rs
│                  session 源载体删除前的 root/path/kind 校验 + 文件/目录删除执行
├── security.rs    validate_command_spec / validate_session_id（注入防护）
├── port_monitor/  本机端口扫描与进程终止（Port 工具页）
├── grok_provider/ Grok config 供应商档案、官方账号清理、隐私保护、备份
├── state.rs       AppState（sessions 缓存 + port 缓存 + 偏好）
└── commands.rs    Tauri commands（session / port / grok / preferences）

src/               React 前端：App 负责 Session/Port/Grok 三工具页与键盘编排；
                   hooks/ 承载 usePreferences / useSessions / usePorts / useGrokProviders；
                   components/ 承载 AgentGroup / ProjectBucket / SessionRow /
                   Controls / PortWorkspace / ProvidersWorkspace / Skeleton / icons；
                   lib/ 承载跨组件纯函数（含 grokProviderCards）；styles/ 按组件边界拆分样式。
```

---

## 技术栈

- **桌面框架**：Tauri 2.x
- **后端**：Rust（扫描并行；tauri-plugin-store 持久化偏好；rusqlite 进程内读取 Cursor store.db）
- **前端**：React + TypeScript
- **平台**：v1 优先 macOS（Terminal.app / iTerm2 / Ghostty 三种外部终端，均走 osascript 拉起，开 tab 还是开窗见下"终端拉起策略"）；架构上 `TerminalLauncher` trait 为后续 Windows 兼容预留

---

## 数据流

```mermaid
flowchart TD
    A[Tauri setup] --> A1[加载终端 / 启动方式 / 主题 / 收藏项目偏好]
    A1 --> A2[manage AppState]
    A2 --> B[前端首次加载调用 scan_sessions]
    B --> C{AppState 已扫描?}
    C -- 是 --> D0[返回内存缓存]
    C -- 否 --> C1{有合法 scan-cache?}
    C1 -- 是 --> D1[fromCache 秒开 + 前端 refresh]
    C1 -- 否 --> S0[scan_all 并行扫描]
    D1 --> S0
    S0 --> S1[CodexScanner.scan]
    S0 --> S2[ClaudeCodeScanner.scan]
    S0 --> S3[CursorScanner.scan]
    S0 --> S4[GrokBuildScanner.scan]
    S0 --> S5[OpenCodeScanner.scan]
    S1 --> D[聚合 & 按 last_active_at 排序]
    S2 --> D
    S3 --> D
    S4 --> D
    S5 --> D
    D --> E0[前端 recentDays 过滤]
    D0 --> E0
    E0 --> E1[搜索查询过滤]
    E1 --> E2[收藏项目排序]
    E2 --> E[前端按 agent / 工作目录渲染]
    R[用户点击刷新] --> R1[refresh_sessions]
    R1 --> S0

    F[用户点击启动] --> G[launch_session]
    G --> H0[读 AppState.preferred_terminal]
    H0 --> H[Scanner 查 session_id → CommandSpec{cwd, program, args}]
    H --> HL[选 TerminalLauncher 实现]
    HL --> W[生成临时 wrapper]
    W --> L[launch: 开窗/tab + 执行 wrapper]
    L --> K{成功?}
    K -- 是 --> OK[前端显示成功]
    K -- 否 --> ERR[前端显示错误]

    X[用户右键单条 session] --> X1[SessionContextMenu]
    X1 --> X2[ConfirmDialog 二次确认]
    X2 -- 取消 --> X3[关闭弹窗，不调用后端]
    X2 -- 确认 --> X4[delete_session(session.id)]
    X4 --> X5[AppState.find_session]
    X5 --> X6[session_delete 校验 root/path/kind]
    X6 --> X7[remove_file 或 remove_dir_all]
    X7 --> X8[AppState.sessions 只移除当前 id]
    X8 --> X9[返回 ScanResponse，前端更新列表]

    Y[用户收藏项目] --> Y1[set_favorite_project_dirs]
    Y1 --> Y2[按当前 sessions project_dir 过滤]
    Y2 --> Y3[写入 preferences.json]
    Y3 --> Y4[更新 AppState.favorite_project_dirs]
```

## 快速访问约束

- 搜索只匹配 `cliType` / agent 展示名 / `projectName` / `projectDir` / `summary`，不读取或索引 CLI 原始对话全文。
- `favorite_project_dirs` 保存前按当前扫描到的 `Session.project_dir` 过滤，去重并保留用户选择顺序；偏好文件被手工污染时，前端派生层也会按当前 session 再过滤。
- 收藏粒度是项目目录，不是单条 session；`Session` / `ScanResponse` 不携带收藏事实。
- 查询非空时前端通过 `forceOpen` 展开有匹配结果的 agent/project，清空查询后回到组件自身的展开状态。

---

## CLI 覆盖

| CLI | session 存储 | cwd 来源 | resume 命令 |
|---|---|---|---|
| codex | `~/.codex/sessions/**/*.jsonl`（payload.cwd） | jsonl 直接读 | `codex resume <id>`（仍 cd 到原工作目录，方便恢复上下文） |
| claude-code | `~/.claude/projects/<编码>/<uuid>.jsonl` | jsonl 的 cwd 字段（优先，decode 歧义见 learning） | `claude --resume <id>` |
| cursor | `~/.cursor/chats/<hash>/<uuid>/{meta.json, store.db}` | store.db 里 cursor 注入的 `Workspace Path:`（可含空格；多命中取最长 canonicalize 目录；rusqlite 过滤相关 blob） | `cursor-agent --resume <id>`（**id 是 workspace 范围，必须 cd 到对应目录**） |
| grok-build | `~/.grok/sessions/<url-encoded-cwd>/<session-id>/summary.json`（`GROK_HOME` 可覆盖根目录） | 优先 `summary.info.cwd`，其次 group `.cwd`，再对含 `%` 的目录名 percent-decode | `grok --resume <id>` |
| opencode | `$XDG_DATA_HOME/opencode/opencode.db` 或 `~/.local/share/opencode/opencode.db` 表 `session` | `session.directory` | `opencode --session <id>` |

五家都是 `cd <cwd> && continue <id>` 模式（cursor 的 cd 必须准确，否则 resume 失败）。cursor 不用 `cursor-agent ls`（要 TTY）。

删除目标和扫描来源一致：codex / claude-code 删除扫描命中的 `.jsonl` 文件；cursor 删除 chat 目录；grok-build 删除 session 目录；opencode 删除 `session` 表对应行（不删 db 文件）。删除动作不按 CLI 原始 `session_id` 批量匹配，只处理 UI 当前行对应的 `Session.id`。

---

## 终端拉起策略（macOS）

三种终端开窗/tab 行为各异（实现期实测，详见 feature design 的 2.1 节）：

| 终端 | 已有窗口 | 无窗口 | 机制 |
|---|---|---|---|
| **iTerm2** | 开新 tab（`create tab with default profile`） | activate 后复用默认窗口 | osascript；app 名 `iTerm`；`write text` 只注入临时 wrapper 路径 |
| **Ghostty** | 开新 tab（AppleScript `new tab with configuration`） | `open -na ... -e <wrapper>` 开首个窗口 | AppleScript 字典 + 临时 wrapper 脚本规避 login 误报；`-e` 自动 `quit-after-last-window-closed=true` |
| **Terminal.app** | 开新窗口（无法开 tab） | 开新窗口（冷启动会多一个空默认窗口，硬限制） | osascript `do script` 只注入临时 wrapper 路径；`make new tab` 不支持，模拟 ⌘T 需辅助功能权限 |

**统一 wrapper 脚本**：三种终端启动前都会生成临时脚本 `$TMPDIR/fast-start-ghostty/run-<uuid>.sh`（权限 `0700`），脚本负责：解析登录 shell PATH（取最后一行并校验形态，失败回退 fallback）、prepend `~/.grok/bin`、`cd <cwd>`、`exec <program> <args>`。Terminal.app / iTerm2 的 AppleScript 只注入 wrapper 路径，不再直接写入完整业务命令；Ghostty 也继续通过 wrapper 规避 `/usr/bin/login` 对多词命令的误报。

---

## 部署架构

纯本地桌面应用，无后端服务、无网络。终端、打开方式、主题和收藏项目偏好经 `tauri-plugin-store` 落地为 app 数据目录下 `preferences.json`。session 数据只冷扫描读取各 CLI 的本地存储，不持久化、不上传。Tauri CSP 使用最小可用策略，禁止回退到 `csp: null`。

---

## 安全口径

launch 会生成本地 wrapper 并执行 CLI resume，`project_dir` / `session_id` 来源于扫描外部 CLI 存储（不可信数据）：

- `CommandSpec.cwd` 必须 `canonicalize` 且为目录
- `program` 限白名单：`codex` / `claude` / `cursor-agent` / `grok` / `opencode`
- `args` 中来自 session 数据的 id 字段做 UUID 字符集白名单校验
- Terminal.app / iTerm2 / Ghostty 都必须通过 `validate_command_spec` 后再写 wrapper
- Terminal.app / iTerm2 AppleScript 只允许注入 wrapper 路径，不允许重新引入完整业务命令字符串
- **禁止**裸拼成 shell 字符串丢给 `sh -c`；wrapper 内容使用 `shell_escape` 拼接，AppleScript 动态片段用 `quoted form of` 转义

> 当前边界：五家 CLI 都是 `cd <cwd> && continue <id>` 语义；安全校验在 `CommandSpec` 层统一完成，终端实现只负责选择窗口 / tab 并启动 wrapper。

session 删除是本地 destructive action，`delete_target.root/path/kind` 同样来源于扫描外部 CLI 存储（不可信数据）：

- 前端只传 `Session.id`，不接收、不展示、不调试输出真实源文件路径
- 删除前必须 `canonicalize` root 和 path，并确认 path 在 root 内，且 path 不能等于 root
- Codex / Claude Code 只允许删除文件；Cursor / Grok Build 只允许删除目录；OpenCode 删除 SQLite session 行
- 删除失败必须显式报错，不能从列表假移除；删除成功只移除当前缓存 id，不按 `session_id` 清理其他项
- 删除能力不得删除 `project_dir` 指向的工作目录、CLI 全局目录、账号配置、缓存目录或其他 session

## 变更日志

### 2026-07-12 - OpenCode CLI

- 新增 OpenCode：`opencode.db` 扫描、`opencode --session <id>`、SQLite 行删除、白名单与 PATH（`~/.bun/bin` / `~/.opencode/bin`）。

### 2026-07-12 - 四 CLI + 扫描/启动硬化

- CLI 覆盖与安全白名单补齐 Grok Build（`grok` / `summary.json` / Directory 删除 / `~/.grok/bin` PATH）。
- Cursor cwd：支持路径空格；多候选取最长合法路径；缺时间戳回退 mtime。
- Codex/Claude：流式读 jsonl；Codex 坏行 skip 不拖垮整 CLI。
- wrapper PATH：login shell 输出取末行并做形态校验，失败回退 fallback。

### 2026-06-24 - 快速访问归并

- 前端数据流补充快速访问派生链路：最近天数过滤、搜索过滤、收藏排序、agent/project 渲染。
- 偏好范围补充 `favorite_project_dirs`，明确和终端、打开方式、主题同属本机 `preferences.json`。
- 核心模块补充 `src/hooks/` 状态编排目录，`src/lib/sessionUtils.ts` 继续承载跨组件纯函数。
