---
doc_type: feature-design
feature: 2026-06-17-cli-session-launcher
status: accepted
created: 2026-06-17
updated: 2026-06-18
summary: 快开CLI——聚合展示 codex / claude-code / cursor 三个 AI CLI agent 的最近 session，选中后一键拉起外部终端、cd 到工作目录、以 session ID resume 对应 agent。
tags: [tauri, react, cli-integration, desktop]
---

# cli-session-launcher 设计文档

## 0. 术语表

| 术语 | 定义 | 防冲突结论 |
|---|---|---|
| **Session** | 一次 CLI agent 进程的运行记录，包含 session ID、工作目录、最后活动时间 | 新建概念，无冲突 |
| **CLI 类型** | 三种受支持的 agent：codex、claude-code、cursor | 新建概念，无冲突 |
| **Session Scanner** | 负责从指定 CLI 的存储位置发现并解析 session 记录的模块 | 新建概念，无冲突 |
| **会话恢复** | 拉起终端 → cd 到工作目录 → 用 session ID 启动对应 agent 的完整动作链 | 新建概念，无冲突 |
| **终端类型** | 可选的终端应用：系统 Terminal、iTerm2、Ghostty | 新建概念，无冲突 |
| **TerminalLauncher** | 封装"在指定终端里 cd 到某目录并执行某命令"的抽象，每种终端一个实现 | 新建概念，无冲突 |

## 1. 明确不做

- **不做 session 持久化**：扫描结果 app 关闭后不保留，不写入任何 DB 文件
- **不做内置终端**：不引入 xterm.js / 终端组件，始终调用外部终端进程
- **不做 session 编辑**：无删除 / 重命名 / 修改 session 的 UI 或 API
- **不做实时监控**：无 fs-watcher / polling / 后台扫描线程
- **不做跨设备同步**
- **v1 仅 codex + claude-code**；cursor 列 v2（sqlite 解析 + workspace hash 反推成本高）

## 1.5 复杂度档位

| 维度 | 档位 | 理由 |
|---|---|---|
| 并发模型 | 不需要 | 冷扫描，无共享状态竞争 |
| 数据竞争 | 不适用 | 无多线程写共享状态 |
| IO 模式 | 同步阻塞 | 文件扫描，量小 |
| 通信模式 | 单向请求 | 前端 invoke → 后端执行 |
| 失败语义 | 自愈 | 单个 Scanner 失败不影响其他 |
| 可观测性 | opaque | 初期不做 logging 框架 |
| 可测试性 | testable | 结构支持注入，但初期不写测试 |
| 安全性 | L2 | 纯本地无网络，但 launch 会拼装 shell 命令执行；`project_dir` / `session_id` 来源于扫描外部 CLI 存储的不可信数据，存在命令注入面，须参数化构造、禁止裸字符串拼接 |

## 2.1 关键决策

- **桌面框架**：Tauri 2.x + React + TypeScript — 已确认
- **Session 发现策略**：冷扫描（app 启动时扫描一次），不做持久化 — 已确认
- **终端拉起**：支持三种终端可选 — 系统 Terminal.app、iTerm2、Ghostty。通过 AppleScript 或各终端的 CLI 协议实现目录跳转和命令执行 — 已确认
- **偏好终端持久化**：通过 Tauri store plugin（`tauri-plugin-store`）保存用户选择，落地为 app 数据目录下 JSON — 已确认
- **跨平台**：优先 macOS，架构上为后续 Windows 兼容预留（`TerminalLauncher` trait 封装终端拉起逻辑，见 2.1）

### 各 CLI session 调研结论（2026-06-17，本机实测）

> feature 成立的前提是"能扫到 session + 能恢复 cwd + 能 resume"。以下基于本机三个 CLI 的真实存储实测，不是文档猜测。

| CLI | session 存储路径 | 格式 | cwd 可恢复性 | resume 命令 | 可行性 |
|---|---|---|---|---|---|
| **codex** | `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl` | jsonl，每行 `{timestamp, type, payload}`；`session_meta` 和 `turn_context` 两类 payload 含 `cwd` 字段 | ✓ 直接读 `payload.cwd` | `codex resume`（picker）或 `codex resume --last` | **高** |
| **claude-code** | `~/.claude/projects/<编码后的 cwd>/<uuid>.jsonl` | jsonl；**cwd 编码在目录名里**（`/` → `-`，如 `-Users-xb-Desktop-codes-ai-test`）；jsonl 内首行也含 `cwd` 字段 | ✓ 从 jsonl 的 cwd 字段（**优先**，多数行都带真实 cwd）；目录名 decode 仅作 fallback（**注意**：decode 有歧义——路径分隔符 `/` 和路径里的 `-` 都编成 `-`，含 `-` 或 `.` 的目录名如 `fast-start`/`ai-test`/`.coze` 无法无损反解，故 fallback 结果可能错误，必须优先用 jsonl 真实 cwd） | `claude --resume <session-id>` 或 `claude -c`（当前目录最近） | **高** |
| **cursor** | `~/.cursor/chats/<workspace-hash>/<uuid>/store.db` | **sqlite 二进制**，非 jsonl | ⚠️ workspace hash 是单向哈希，无公开算法反推 cwd；需额外建立 hash→cwd 映射 | `cursor-agent --resume <chat-id>` 或 `cursor-agent resume` | **中**（见风险） |

**codex session 文件真实样例**（节选关键字段）：
```
{"type":"session_meta","payload":{"cwd":"/Users/xb/Desktop/codes/ybb-ai-platform", ...}}
{"type":"turn_context","payload":{"cwd":"/Users/xb/Desktop/codes/ybb-ai-platform", ...}}
```

**claude-code 目录命名真实样例**：
```
~/.claude/projects/-Users-xb-Desktop-codes-fast-start/<uuid>.jsonl
                       └─ 解码即 cwd：/Users/xb/Desktop/codes/fast-start
```

**cursor 的隐藏风险**（实现阶段重点验证）：
1. `cursor-agent ls`（本意用它列 session）**要交互式 TTY**，非 TTY 环境直接报 `Raw mode is not supported` 退出 —— 在 Tauri 后端进程里 spawn 它拿不到列表
2. chat 存储是 `store.db`（sqlite），需要直接读 sqlite 提取 session 列表，而非依赖 `cursor-agent ls`
3. `~/.cursor/chats/<workspace-hash>/` 的 hash 是 workspace 路径单向哈希，**没有公开反推算法**；cwd 恢复必须额外维护一个 hash→cwd 映射（来源候选：`~/.cursor/projects/` 子目录名，或扫描期让用户标注）

**结论**：codex / claude-code **可行性高**，作为 v1 首批目标；cursor 可行但实现成本和不确定性显著更高，**建议 v1 先做 codex + claude-code，cursor 列为 v2**，避免 cursor 的 sqlite 解析和 hash 反推拖垮 v1 交付。

## 2.1 名词层（核心类型）

**Session** — 一条 session 记录

```rust
// 来源：src-tauri/src/models.rs (新增)

// 所有暴露给前端的 struct/enum 统一加 #[serde(rename_all = "camelCase")]，
// 与前端 TS 的 camelCase 对齐，避免手工逐字段 rename 出错。
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Session {
    id: String,            // 唯一标识（UUID）
    cli_type: CliType,
    session_id: String,    // CLI 本身的 session ID
    project_dir: PathBuf,  // 工作目录绝对路径
    project_name: String,  // 目录名，用于显示
    last_active_at: chrono::DateTime<Utc>,
}
```

**CliType** — 枚举 CLI 类型

```rust
// 来源：src-tauri/src/models.rs (新增)
enum CliType {
    Codex,
    ClaudeCode,
    Cursor,
}
```

**TerminalType** — 枚举支持的终端

```rust
// 来源：src-tauri/src/models.rs (新增)
enum TerminalType {
    System,   // macOS Terminal.app
    ITerm2,   // iTerm2
    Ghostty,  // Ghostty
}
```

> 终端是否真实可用（如 Ghostty 未安装）属于运行期检测，不进枚举。`TerminalLauncher::is_available()`（见下）在扫描期探测，前端据此灰显不可选项。

**SessionScanner trait** — 每种 CLI 一个实现

```rust
// 来源：src-tauri/src/scanner.rs (新增)
trait SessionScanner {
    fn cli_type(&self) -> CliType;
    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError>;
}
```

三种实现：
- `CodexScanner` — 扫描 codex CLI 的 session 存储
- `ClaudeCodeScanner` — 扫描 claude-code CLI 的 session 存储
- `CursorScanner` — 扫描 cursor CLI 的 session 存储（v2）

**CommandSpec** — 描述"在某目录执行某 CLI 的 resume 命令"，是 Scanner 输出给 Launcher 的纯数据，避免 Launcher 知道每种 CLI 的 resume 细节

```rust
// 来源：src-tauri/src/models.rs (新增)
struct CommandSpec {
    cwd: PathBuf,              // cd 到这里（已校验为存在且为目录，见安全约束）
    program: String,           // CLI 可执行名，如 "codex" / "claude" / "cursor-agent"
    args: Vec<String>,         // resume 参数，如 ["resume", "--last"] / ["--resume", "<id>"]
}
```

**TerminalLauncher trait** — 每种终端实现这个 trait，封装"开窗口 + cd + 执行命令"。这是 2.2 编排里原本缺失的抽象层，对应关键决策的跨平台预留

```rust
// 来源：src-tauri/src/launcher.rs (新增)
trait TerminalLauncher {
    fn terminal_type(&self) -> TerminalType;
    /// 探测该终端是否可用（如 Ghostty 未安装则 false）
    fn is_available(&self) -> bool;
    /// 在新窗口/标签里 cd 到 spec.cwd 并执行 spec 程序
    fn launch(&self, spec: &CommandSpec) -> Result<(), LaunchError>;
}
```

三种实现（macOS v1）：
- `SystemTerminalLauncher` — Terminal.app，走 `osascript`，`tell application "Terminal" to do script "..."`。**开新窗口**（Terminal 的 AppleScript 无法干净开新 tab：`make new tab` 字典不支持，`do script in <tab/window>` 无法稳定复用，模拟 ⌘T 需辅助功能权限）。冷启动 `do script` 会触发 Terminal 启动默认窗口 + 命令窗口共两个，这是 Terminal 固有行为无法从 AppleScript 侧避免，命令窗口一定存在。
- `ITerm2Launcher` — iTerm2，走 `osascript`（app 名是 `iTerm` 不是 `iTerm2`，否则不加载字典导致 `create tab` 语法错）。**有窗口时开新 tab**（`tell current window: create tab with default profile`）；冷启动时 `activate` 后轮询等默认窗口出现并复用（避免 `create window` 叠加成两个窗口），再 `write text` 命令。
- `GhosttyLauncher` — Ghostty。**有窗口时用 AppleScript `new tab with configuration` 在已有窗口开 tab**；无窗口时 `open -na Ghostty.app --args -e <wrapper>` 开首个窗口。Ghostty 在 macOS 上无 CLI 方式给运行实例开 tab，但提供 AppleScript 字典（`new tab` / `surface configuration` 记录含 `command` / `initial working directory`）。`command` 配置项指向一个临时 wrapper 脚本（见下），单脚本路径避免 login 误报。

**Ghostty wrapper 脚本机制**（实现期实测得出，原 design 未写）：
Ghostty 在 macOS 上把 `-e`/`--command` 的命令套进 `/usr/bin/login -flp`，多词命令（`codex resume <id>`）会让 login 解析失败弹"failed to launch"误报（命令其实已执行）。解决：`launch` 时生成一个临时 wrapper 脚本到 `$TMPDIR/fast-start-ghostty/run-<pid>.sh`（`0700`），内容 `export PATH=<含 ~/.local/bin> && cd <cwd> && exec <program> <args>`，让 `-e`/`--command` 只执行单脚本路径（login 不解析多词命令，无误报）。同时 `-e` 自动设 `quit-after-last-window-closed=true`，agent 退出后 Ghostty 干净退出不留孤儿。PATH 显式补 `~/.local/bin` 是因为 Ghostty tab 不走 login shell，默认 PATH 不含用户自定义目录（codex/claude/cursor-agent 装在那里）。

> 注入防护（对应安全档位 L2）：`CommandSpec` 的 `cwd` / `program` / `args` **绝不在 Rust 侧拼成 shell 字符串再交给 osascript**。osascript 那侧仍是一段脚本——`cwd` 必须先用 `std::fs::canonicalize` 校验存在且为目录、`program` 必须是白名单（`codex`/`claude`/`cursor-agent`）、`args` 里来自 session 数据的部分（如 session id）做字符白名单（UUID 字符集）。详见 2.2 流程级约束。

**Tauri Commands** — 暴露给前端调用的 Rust 函数

```rust
// 来源：src-tauri/src/commands.rs (新增)

/// 扫描所有 CLI 的 session，返回聚合列表。读 AppState 缓存，无则触发一次扫描
#[tauri::command]
fn scan_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String>;

/// 刷新：强制重新扫描（用户点"刷新"按钮时）
#[tauri::command]
fn refresh_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String>;

/// 启动一个 session：拉起终端 → cd → 运行 agent。
/// terminal 不由前端传——统一从 AppState.preferred_terminal 读，避免"下拉值"与"本次传参"语义分裂
#[tauri::command]
fn launch_session(session_id: String, state: State<'_, AppState>) -> Result<(), String>;

/// 探测当前机器上哪些终端可用（Ghostty 可能未装）
#[tauri::command]
fn list_available_terminals(state: State<'_, AppState>) -> Result<Vec<TerminalType>, String>;

/// 获取用户偏好的终端
#[tauri::command]
fn get_preferred_terminal(state: State<'_, AppState>) -> Result<TerminalType, String>;

/// 设置用户偏好的终端，返回 Result 以便向前端报持久化错误
#[tauri::command]
fn set_preferred_terminal(terminal: TerminalType, state: State<'_, AppState>) -> Result<(), String>;
```

## 2.2 编排

```mermaid
flowchart TD
    A[App 启动] --> B[scan_sessions]
    B --> S1[CodexScanner.scan]
    B --> S2[ClaudeCodeScanner.scan]
    S1 --> D[聚合 & 按 last_active_at 排序]
    S2 --> D
    D --> E[前端渲染 SessionList]

    F[用户点击"启动"] --> G[launch_session]
    G --> H0[读 AppState.preferred_terminal]
    H0 --> H[Scanner 查 session_id → 产出 CommandSpec{cwd, program, args}]
    H --> HL[选对应 TerminalLauncher 实现]
    HL -- Terminal.app --> J1[SystemTerminalLauncher.launch]
    HL -- iTerm2 --> J2[ITerm2Launcher.launch]
    HL -- Ghostty --> J3[GhosttyLauncher.launch]
    
    J1 --> K{launch 成功?}
    J2 --> K
    J3 --> K
    K -- 是 --> L[前端显示启动成功]
    K -- 否 --> M[前端显示错误提示]
```

- Rust 后端编排顺序：
  1. `scan_sessions` 并行调用三个 Scanner
  2. 合并结果按 `last_active_at` 降序排列
  3. 返回前端渲染
  4. `launch_session` 由 session_id 反查 → 对应 Scanner 产出 `CommandSpec`（cwd + program + resume args）→ 按 `preferred_terminal` 选 `TerminalLauncher` 实现执行 `.launch(spec)`

- 前端编排顺序：
  1. 组件 mount 后立即调用 `invoke("scan_sessions")` 和 `invoke("get_preferred_terminal")`
  2. 渲染：顶部终端选择下拉（选项来自 `list_available_terminals`，未装的灰显），下方按 CLI 类型分组的可展开列表
  3. 用户点击"启动"按钮 → `invoke("launch_session", { sessionId })`（不传 terminal，后端读偏好）
  4. 根据返回结果显示成功 / 失败状态

### 流程级约束

| 约束 | 规则 |
|---|---|
| **失败语义** | 某一个 Scanner 失败不应影响其他 Scanner；失败 CLI 在列表中标记"扫描失败" |
| **命令构造安全** | `CommandSpec.cwd` 须 `canonicalize` 校验存在且为目录；`program` 限白名单（`codex`/`claude`/`cursor-agent`）；`args` 中来自 session 数据的 id 字段做 UUID 字符集白名单校验。**禁止**把 cwd/program/args 裸拼成 shell 字符串后丢给 `sh -c`——osascript 脚本里的动态片段用 AppleScript 转义（`quoted form of`）注入 |
| **终端启动幂等性** | 多次点击同一 session 会多次打开新终端窗口，不做去重（终端窗口天然隔离） |
| **并发** | 三个 Scanner 可并行扫描（`tokio::join!` 或 `futures::join_all`） |
| **扫描时机** | 仅在应用 launch / 用户手动刷新时触发，不做 fs watcher |
| **终端可用性** | 启动时调各 `TerminalLauncher::is_available()` 探测，不可用的在 UI 灰显；用户选了不可用终端时 `launch_session` 返回明确错误 |
| **可观测点** | 前端显示扫描状态（loading / 成功 N 条 / 失败） |
| **扩展点** | `SessionScanner` trait（新增 CLI）和 `TerminalLauncher` trait（新增终端）都设计为可加实现 |

## 2.3 挂载点

| 挂载点 | 实现方式 | 操作 | 移除时影响 |
|---|---|---|---|
| Tauri command 注册 | `tauri::Builder::invoke_handler(tauri::generate_handler![scan_sessions, refresh_sessions, launch_session, list_available_terminals, get_preferred_terminal, set_preferred_terminal])` | 新增 | feature 消失 |
| Tauri 窗口创建 | `tauri::Builder::new().run()` 主窗口配置 | 新增 | feature 消失 |
| 前端入口 | `src/App.tsx` 组件渲染 | 新增 | feature 消失 |
| 终端命令执行 | 各 `TerminalLauncher` 实现的 osascript / CLI 调用 | 新增 | 终端拉起不可用 |
| 偏好终端持久化 | `tauri-plugin-store` 写入 app 数据目录 JSON | 新增 | 终端选择回退为默认 |

## 2.4 推进策略

1. 编排骨架 — 用 create-tauri-app 初始化 + 确认可编译
   退出信号：`pnpm tauri dev` 能看到默认 Tauri 窗口
2. 编排骨架 — Rust 后端 models + SessionScanner trait + TerminalLauncher trait 骨架 + 六个 Tauri command（空实现）+ invoke_handler 注册
   退出信号：`scan_sessions` 返回 mock 数据聚合列表，前端 invoke 能拿到
3. Rust 后端 — 各 CLI Scanner 实现 + session 路径调研
   退出信号：扫描到真实 CLI session 数据（v1 先聚焦 **codex 和 claude-code**；cursor 因 sqlite 解析 + workspace hash 反推成本高，列 v2）
4. Rust 后端 — 三种 TerminalLauncher 实现 + launch_session 串联
   退出信号：点击启动能在选定终端正确 cd 并以 session id resume 对应 agent
5. 接通持久化 — tauri-plugin-store 持久化 preferred_terminal + is_available 探测
   退出信号：关 app 重开后偏好终端记忆生效；未装的终端在下拉灰显
6. 联调与收尾 — v1 两种 CLI（codex + claude-code）的端到端验证 + 错误处理 + 样式收尾
   退出信号：codex 和 claude-code 都能正常扫描和启动，失败场景有提示；cursor 分组显示"v2 开发中"

## 3. 验收场景清单

| # | 场景 | 输入/触发 | 期望可观察结果 |
|---|---|---|---|
| 1 | 正常扫描 | 启动 app | 1. 列表展示各 CLI 的 session（v1: codex + claude-code）<br>2. 按最后活动时间降序排列<br>3. 每条显示：CLI 类型、项目目录名、最后活动时间 |
| 2 | 手动刷新 | 点击刷新按钮 | 重新扫描并更新列表 |
| 3 | 选择终端 | 下拉切换终端类型 | 更新偏好；下次启动 session 时使用对应终端 |
| 4 | 启动 session（Terminal.app） | 偏好设为 System → 某 codex session → 点击"启动" | 1. 系统 Terminal.app 窗口打开<br>2. cd 到 session 的工作目录<br>3. 执行 `codex resume`（对应 session）。**Terminal 无法开 tab**，每次启动开新窗口；冷启动可能多一个空默认窗口（Terminal AppleScript 硬限制） |
| 5 | 启动 session（iTerm2） | 偏好设为 iTerm2 → 某 claude-code session → 点击"启动" | iTerm2 **在已有窗口开新 tab**（无窗口时开窗口），cd 到目录，执行 `claude --resume <id>` |
| 6 | 启动 session（Ghostty） | 偏好设为 Ghostty（且已安装）→ 某条 session → 点击"启动" | Ghostty **在已有窗口开新 tab**（无窗口时开窗口），cd 到目录并执行 agent 启动命令；无 login 误报，agent 退出后窗口/tab 干净关闭不留孤儿 |
| 7 | 偏好终端记忆 | 设置 Terminal → 关 app → 重新打开 | 终端选择下拉显示上次选中的值 |
| 8 | 单个 CLI 扫描失败 | codex 目录不存在/损坏 | 1. 其他 CLI 的 session 正常显示<br>2. 该 CLI 分组显示"扫描失败"提示 |
| 9 | 没有 session | 某 CLI 无历史 session | 该 CLI 的分组显示"暂无 session" |
| 10 | 关闭重开 | 关闭 app 后重新打开 | session 列表为新扫描结果（不依赖持久化） |
| 11 | 终端未安装 | Ghostty 未装 | 下拉里 Ghostty 选项灰显不可选；`list_available_terminals` 不返回它 |
| 12 | cwd 校验失败 | session 记录的 project_dir 已被删除 | 启动返回明确错误（"工作目录不存在"），不尝试执行 shell |
| 13 | cursor v2 边界 | v1 阶段 cursor 分组 | 显示"cursor 支持开发中（v2）"，不尝试扫描 |

## 4. 验收通过后

- **架构入口**：`ARCHITECTURE.md` 当前为占位模板，acceptance 通过后应回填：
  - 名词层：`Session`、`CLIType`、`TerminalType`、`CommandSpec`、`SessionScanner`、`TerminalLauncher` 类型定义
  - 技术栈：Tauri + React + TypeScript
  - 架构图：当前 feature 的主流程图
  - v1/v2 边界：v1 = codex + claude-code；v2 = cursor（sqlite + workspace hash 反推）

## 验收结论（2026-06-18）

acceptance 已完成：31 条 checks 中 `pass 32 / warn 3 / fail 0`，无 P0/P1 阻断项。3 条 warn（命令构造走 shell 字符串、并发用 thread::spawn 而非 tokio、Ghostty 未装未端到端实测）均不阻塞验收。详见 `.codestable/audits/2026-06-17-cli-session-launcher-accept/`。

### 二次验收（2026-06-18，终端 tab 化 + bug 修复）

首轮验收后针对真实使用反馈做了以下改动，已回填本 design 对应章节：
1. **Ghostty 开 tab**：改用 AppleScript `new tab with configuration`（design 原写 `-e`/`--working-directory`），实测 macOS Ghostty 无 CLI 开 tab 但有 AppleScript 字典。
2. **Ghostty wrapper 脚本**：解决 `-e`/`--command` 被 login 套壳导致的多词命令误报；`-e` 自动 `quit-after-last-window-closed=true` 让 agent 退出后干净关闭。**此改动直接解决首轮 warn 3**（Ghostty 已端到端实测通过）。
3. **iTerm2 开 tab**：design 原写开新窗口，改为有窗口开 tab（app 名修正为 `iTerm`）。
4. **Terminal 开新窗口**：确认 AppleScript 无法开 tab（`make new tab` 不支持），保持开新窗口，冷启动双窗口为硬限制。
5. **claude cwd bug 修复**：`parse_claude_file` 原逻辑因 fallback 是 `Some` 导致 jsonl 真实 cwd 永不被读取，改用 `cwd.or(fallback_cwd)` 优先真实值。

这些改动经真实终端实测验证：Ghostty/iTerm2 在已有窗口开 tab、Terminal 开新窗口（双窗口）、codex/claude 在对应终端 cd + resume 成功。详见二次验收报告。
