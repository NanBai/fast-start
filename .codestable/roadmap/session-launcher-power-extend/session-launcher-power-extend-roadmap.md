---
doc_type: roadmap
slug: session-launcher-power-extend
status: active
created: 2026-07-13
last_reviewed: 2026-07-13
tags: [session-launcher, ux, port-monitor, grok, terminal, extension, hygiene]
related_requirements:
  - quick-session-access
  - delete-session-context-menu
  - extend-grok-providers-tool
related_architecture:
  - ARCHITECTURE
---

# Session Launcher 能力加深与扩展面

## 1. 背景

`session-launcher-next-wave` 已全部落地：扫描缓存秒开、混合项目视图、session 收藏、Grok 模型/预览/Default 策略、Port 规则与 loopback、最近启动与命令预览、release 脚本与文档收口。

用户在下一波 brainstorm 中选定三条轴继续迭代（编号保留 brainstorm 原号；**轴 2 产品形态跃迁**见「明确不做」）：

1. **日常爽感**——找 / 启 / 清 session 更快、失败更可预期  
3. **工具页加深**——Port / Grok 在现有能力上再挖一层运维向能力  
4. **平台与扩展**——在 **仍限 macOS** 前提下扩展终端适配与 CLI 接入契约（**不是** Windows 一等支持，**不是**动态插件运行时）

对象：每天在多仓库、多 AI CLI 之间切换的 macOS 开发者；以及要继续加终端 / CLI 而不反复踩安全坑的维护者。

## 2. 范围与明确不做

### 本 roadmap 覆盖

1. **启动预检**：launch 前只读检查 cwd / program PATH / session 源是否仍在，给出 block/warn  
2. **Session 卫生**：标记陈旧（cwd 不存在 / 源载体缺失）、摘要质量、批量删除、磁盘占用概览  
3. **Port 加深**：保护端口名单、按项目目录分组、组内多选后调用既有 terminate 关闭同项目端口  
4. **Grok 运维加深**：config / auth / profiles 健康诊断与备份可回看（不重做模型拉取）  
5. **扩展面**：新增至少一种主流终端适配器；硬化「新增 CLI」的注册/白名单/删除/测试清单（契约 + 文档 + 回归测试，无动态插件加载）  
6. **收口**：交叉回归、ARCHITECTURE / 用户文档 / AGENTS 偏好 key 同步、残留风险清单

### 明确不做

| 不做 | 理由 |
|------|------|
| Windows / Linux 一等支持 | attention 与上波 roadmap 均标为独立 v2 epic；路径/终端/扫描根全换，会淹没本波 1/3 价值 |
| 系统托盘 / 全局快捷键 / 菜单栏迷你启动器 | brainstorm 轴 2 产品形态跃迁；另开 epic |
| 内置终端、云同步、账号体系 | 与「本机无云」产品定义冲突 |
| 动态插件框架 / 任意第三方 CLI 热加载 | 安全面过大；本波只做**编译期扩展契约**硬化 |
| 全文索引对话内容（默认开启） | 隐私与 IO 成本高；摘要 enrichment 先做；全文索引另评估 |
| 删除回收站 / 撤销删除 | 仍保持「确认即破坏」；批量删除只加二次确认与结果清单 |
| API Key 硬件钥匙串深度集成 | 可后续 security epic；本波 Grok 诊断不改密钥存储形态 |
| 再引入新 CLI 产品类型 | 本波交付扩展**清单与测试**，不强制上线第六家 CLI |
| 重做 next-wave 已验收能力 | 缓存/混合视图/收藏/模型预览等只做消费与增量 |

### Granularity Gate

| 判断项 | 结论 |
|--------|------|
| 为什么不是 single feature | 跨 Session 预检/卫生、Port 规则、Grok 诊断、Launcher 终端、扩展契约 5 个交付面；有共享契约与依赖 DAG |
| 为什么不是 brainstorm | 轴已选定，成功信号可写 yes/no；需的是拆解与接口，不是再分诊 |
| roadmap 边界 | 仅 macOS 本机工具加深 + 编译期扩展面；不含平台跃迁与云能力 |
| 最小闭环 | `launch-preflight`：对一条已知 session 返回可读的 block/warn 检查结果且不执行 resume |

### 方案深度 pre-pass

| 候选 | 取舍 |
|------|------|
| A. 1+3+4（预检/卫生 + Port/Grok 加深 + 终端与 CLI 契约） | **采用** — 用户 `/cs-epic 1,3,4` |
| B. 只做 Session 卫生 | 拒 — 用户明确含工具页与扩展 |
| C. 含 Windows 一等 | 拒 — 单独 epic；写入明确不做 |
| D. 动态插件运行时 | 拒 — 本波用契约+测试替代热加载 |

最小闭环选「最窄端到端路径」：预检直接挂在现有 launch/preview 旁路，不依赖批量删除或新终端即可演示价值。

## 3. 模块拆分（概设）

```
session-launcher-power-extend
├── LaunchPreflight：只读启动预检（cwd / PATH / source）
├── SessionHygiene：陈旧标记、摘要 enrich、批量删除、磁盘占用
├── PortDepth：保护端口、按项目分组、按项目终止
├── GrokOps：本地配置健康诊断与备份列表（只读为主）
├── ExtendSurface：新终端适配 + CLI 扩展契约/回归
└── WaveClose：本 epic 交叉回归与文档收口
```

### LaunchPreflight · 启动预检

- **职责**：在不写 wrapper、不执行 resume 的前提下，对 `Session.id`（列表稳定 id）做一组可序列化检查；供 UI 在启动前展示，也可被 `launch_session` 复用同一判定函数。
- **承载**：`launch-preflight`
- **触碰**：`commands.rs`、`security.rs` / `scanner::command_spec_for_session`、`state` 查缓存、`useSessions` / 启动 UI
- **Depth**：deep — callers 只拿 `PreflightResult`，检查细节藏在预检模块

### SessionHygiene · Session 卫生

- **职责**：对已扫描 session 做健康探测（cwd/源/体积）、改善列表摘要、支持多选批量删除（仍走 `session_delete` 校验）。
- **承载**：`session-health-inspect`、`session-summary-enrichment`、`session-bulk-delete`、`session-disk-usage`
- **触碰**：`session_delete.rs`、`scanner/*`（摘要）、`commands`、Session 列表 UI / `sessionUtils`
- **Depth**：deep 于 inspect + bulk 边界；摘要 enrich 分散在各 scanner 但共享 `clean_summary` 约定

### PortDepth · Port 加深

- **职责**：在现有 `port_ignore_ports` / `port_project_path_prefixes` / terminate all-or-nothing 之上，增加保护端口与按 `working_directory` 的项目向操作。
- **承载**：`port-protect-list`、`port-group-and-terminate-by-project`
- **触碰**：`port_monitor`、`preferences`、`PortWorkspace`、`usePorts`
- **Depth**：deep 于规则层；分组 UI 可浅但 terminate 必须走既有安全 re-scan

### GrokOps · Grok 运维诊断

- **职责**：只读汇总 `config.toml` / `auth.json` / `profiles.json` / 备份目录健康状态；可选列出可回看的备份文件名（恢复仍可手工或单命令，避免自动写盘扩大面）。
- **承载**：`grok-config-health`
- **触碰**：`grok_provider/*`、`ProvidersWorkspace`
- **Depth**：deep 于 grok_provider；不改出站模型拉取契约

### ExtendSurface · 扩展面

- **职责**：新增 ≥1 个 macOS 终端 `TerminalLauncher` 实现；把「新增 CLI」所需的枚举/白名单/scanner/command_spec/delete/测试挂点写成可执行契约与 checklist 测试（编译期，非动态加载）。
- **承载**：`terminal-adapter-extend`、`cli-extension-contract`
- **触碰**：`launcher/*`、`models::TerminalType`、`security` 白名单、`scanner` 注册表、docs
- **Depth**：terminal 实现 deep；contract 以测试锁定注册完整性

### WaveClose · 收口

- **职责**：全波次回归、ARCHITECTURE / 用户文档 / AGENTS 偏好 key、残留风险。
- **承载**：`power-extend-harden-and-docs`
- **触碰**：文档与跨模块 smoke，原则上少改行为

## 4. 模块间接口契约 / 共享协议（架构层详设）

### 4.1 启动预检

**方向**：UI / `launch_session` → LaunchPreflight → AppState 缓存 + FS/PATH  
**形式**：Tauri command + 可单测的纯函数

**契约**：

```text
// Tauri — 业务未知 id 不走 command Err，统一进 result
preflight_launch(session_list_id: String) -> Result<PreflightResult, String>
// command 层 Err 仅用于意外内部错误；未知 session 必须 Ok(PreflightResult)

PreflightResult {
  sessionListId: string
  ok: bool                    // 无 severity=block 时为 true
  checks: PreflightCheck[]
  preview: LaunchCommandPreview | null
  // preview 在能生成 CommandSpec 时填充，且必须与 preview_launch_command 同一组装路径
}

PreflightCheck {
  code: "session_not_found" | "cwd_missing" | "cwd_not_dir"
      | "program_not_found" | "source_missing" | "source_unverified"
      | "invalid_session_id" | "invalid_spec"
  severity: "block" | "warn"
  message: string             // 中文用户可读
}

fn preflight_session(
  session: Option<&Session>,
  ops_ready: bool,
  path_resolver: &impl ResolveProgram,
) -> PreflightResult
```

**判定矩阵（写死，feature-design 不得改语义）**：

| 条件 | check code | severity | launch 是否拦截 |
|------|------------|----------|-----------------|
| 缓存无此 `session_list_id` | `session_not_found` | block | 是 |
| cwd 路径不存在 | `cwd_missing` | block | 是 |
| cwd 存在但非目录 | `cwd_not_dir` | block | 是 |
| session_id / CommandSpec 校验失败 | `invalid_session_id` / `invalid_spec` | block | 是 |
| program 在 launcher 已缓存 PATH 上不可解析（与 wrapper 注入同一 PATH 语义；经可注入 `ResolveProgram`） | `program_not_found` | block | 是 |
| `ops_ready=true` 且源探测失败（**与 §4.2 同源**，见下「源探测」） | `source_missing` | block | 是 |
| `ops_ready=false` 或无法做源探测（缓存窗 / 无足够元数据） | `source_unverified` | **warn** | **否**（不因源拦截 launch） |

**源探测（preflight 与 inspect 必须共享同一函数，禁止两套 IO）**：

| 形态 | 代表 | `source_missing` 条件 |
|------|------|------------------------|
| File | Codex / Claude | `delete_target.path` 文件不存在或 kind 非文件 |
| Directory | Cursor / Grok | `delete_target.path` 目录不存在或 kind 非目录 |
| SqliteRow | **OpenCode** | **session 行不在 db**（与 `inspect_session_health` / 删除路径一致）；**禁止**用 `opencode.db` 文件存在性代替行存在性 |

**约束**：

- 只读：不得写 wrapper、不得改 preferences、不得删文件  
- `session_list_id` 语义与 `launch_session` / `delete_session` 一致（列表稳定 `Session.id`）  
- `launch_session` **必须**先跑同一 `preflight_session`：存在任一 `severity=block` → 返回 `Err(message)` 且不启动  
- 相对 next-wave：full scan 后源缺失会**新增** launch 失败面；缓存窗行为保持「可 launch、源仅 warn」  
- `preview` 字段与 `preview_launch_command` 共享 CommandSpec 组装，禁止两套逻辑  

**Interface 设计检查**：

- Module：`launch_preflight`（建议 `src-tauri/src/launch_preflight.rs`）  
- Seam：所有「能否启动」判定穿过 `preflight_session`  
- Depth：caller 只消费 ok/checks/preview  
- Dependency：in-process；PATH 解析复用现有 launcher PATH 缓存（`OnceLock`）  
- Adapter：生产 FS + 测试注入 cwd/source/program 存在性

### 4.2 Session 健康探测

**方向**：UI → SessionHygiene → 缓存 Session + FS  
**形式**：Tauri command

**契约**：

```text
inspect_session_health(session_list_ids: String[]) -> Result<SessionHealthReport, String>

SessionHealthReport {
  items: SessionHealth[]
}

SessionHealth {
  sessionListId: string
  cwdExists: bool
  sourceExists: bool | null   // null = 缓存窗无法判断（ops_ready=false 或无 delete_target）
  approxBytes: number | null  // 源载体近似字节；不可用则 null
  flags: ("missing_cwd" | "missing_source" | "empty_summary" | "cache_limited" | "size_capped")[]
}
```

**按 CLI 删除形态的源语义（写死）**：

| 形态 | 代表 CLI | `sourceExists` | `approxBytes` |
|------|----------|----------------|---------------|
| File | Codex / Claude Code | 源 `.jsonl` 文件存在 | `metadata().len()` |
| Directory | Cursor / Grok Build | chat/session 目录存在 | **有界**目录合计：max depth **3**、最多 **2000** 文件、单次目录预算 **50ms**；超限则 `null` + `size_capped` |
| SqliteRow | OpenCode | **session 行仍在 db**（不是「db 文件在不在」） | **固定 `null`**（不把整库当单 session 体积） |
| 缓存窗 | 任一 | `null` + `cache_limited` | `null` |

**约束**：

- 只读；单次调用 id 数量上限 **200**（超出 `Err`）  
- 不得把真实 `delete_target` 路径返回给前端  
- `approxBytes` **永不**递归整个 `project_dir` 工作区  
- 未知 id：该项可省略或 `flags` 不含有效探测；不得 panic  
- 前端过滤「陈旧」只基于本报告 + 已有 Session 字段  

**Interface 设计检查**：Seam 在 command 边界；scanner 不被迫每次全量 stat；同步 command 必须遵守有界 IO。

### 4.3 批量删除

**方向**：UI → commands → **与单条 `delete_session` 同一状态路径**  
**形式**：Tauri command

**契约**：

```text
delete_sessions(session_list_ids: String[]) -> Result<BulkDeleteResult, String>

BulkDeleteResult {
  deletedIds: string[]
  failures: { sessionListId: string, message: string }[]
  sessions: Session[]          // 当前缓存列表（camelCase 与 ScanResponse 对齐）
  scanErrors: CliScanError[]
  fromCache: boolean | null
  scanDurationMs: number | null
}
```

**约束**：

- 前端仍只传列表稳定 id；**禁止**传路径  
- **必须**循环调用与 `AppState::delete_session` / command `delete_session` **同一路径**（含 OpenCode SQLite 行删除分支），**禁止**只调 `session_delete::delete_session_target`（会漏 OpenCode）  
- **非** all-or-nothing：成功 id 进 `deletedIds` 并从缓存移除；失败进 `failures`  
- Side effects（写死；相对现单条 `AppState::delete_session` 的 **delta** 标出）：
  - 删除执行：循环同一状态路径（与单条相同；含 OpenCode 行删 + 文件/目录删）  
  - 每成功一条：内存缓存已更新且现路径会写 scan-cache（N≤50 次写盘可接受）  
  - 整批结束后 **额外必须** `sanitize_recent_launches`（现单条 **不**做此项 → bulk **强于**单条，避免 recent 幽灵；不要求本波回改单条）  
  - `favorite_session_ids`：本 command 不强制写 preferences（与单条一致）；UI 必须立刻从本地选中态去掉已删 id；下次 set/scan sanitize 清盘
- 空数组 → `Err`；上限 **50** ids / 次（超出 `Err`）  
- `ops_ready=false` 或无 `delete_target`：该 id failure「请等待完整刷新后再删除」  
- 不提供回收站；UI **必须**二次确认（展示将删数量），且结果默认展示 `failures` 列表（即使为空也明确「全部成功」）  
- **产品契约**：既有 requirement `delete-session-context-menu` 写「不做批量」——启动本 item 前须 `cs-req` 修订该 req（或新增 sibling req）允许批量 + 二次确认 + partial success；未改 req 不得标该 feature done  

**Interface 设计检查**：禁止复制删除逻辑；OpenCode 与文件/目录删除必须走现状态层分支。

### 4.4 摘要 enrichment（扫描期）

**方向**：各 `SessionScanner` → `Session.summary`  
**形式**：扫描实现约定（非新 command）

**契约**：

```text
Session.summary: Option<String>  // 列表展示优先 summary，空则回退 project_name

// 硬截断：仅在 scanner::clean_summary 内统一 Unicode 标量截断至 ≤ 160；
// 各 scanner 不得各自截断。clean_summary 仍负责 trim / 折行 / 压空白。

// 优先级（fixture 锁定；已满足的 CLI 以测试钉现状 + 只补缺口）
- Codex：优先真实用户消息；跳过 empty / is_system 类噪声（流式读）
- Claude：优先 aiTitle，其次 last user prompt
- Cursor：meta.title；空则 None
- Grok：summary.json 既有标题/摘要字段优先
- OpenCode：session title 字段优先，否则 None（不拿整段 directory 当摘要）
```

**约束**：不索引全文对话；不把原始 jsonl 路径暴露前端；完成信号以 fixture 断言为准，不写「体验更好」。

### 4.5 Port 保护与按项目终止

**方向**：preferences + PortDepth → terminate 路径  
**形式**：preferences keys + **UI 分组多选** + 现有 `terminate_port_processes`（**不新增**按项目专用 command）

**契约**：

```text
// preferences.json
port_protect_ports: number[]   // u16，去重排序；默认 []

get_port_protect_ports() -> number[]
set_port_protect_ports(ports: number[]) -> number[]  // 返回 sanitize 后

// terminate 扩展规则（保持 all-or-nothing）
terminate_port_processes(port_ids: string[]):
  1. re-scan + 校验 id 未变化（现有）
  2. 若任一目标 port 号 ∈ port_protect_ports → 整批 Err，message 含保护端口列表
  3. 否则现有 TERM 逻辑

// 按项目：纯前端
// - 按 PortUsage.workingDirectory 分组展示
// - 「关闭此项目端口」= 收集该组 id → 现有确认 → terminate_port_processes
// - 禁止新增 terminate_ports_for_project（本波写死，降低安全面分叉）
```

**约束**：

- protect **优先于**用户多选；不可 force 标志绕过  
- 不杀非 `user_owned`  
- `port_ignore_ports` 只影响展示；protect 影响终止  
- 分组键：空 `workingDirectory` 归入「未知目录」组，不提供「一键关未知组」默认按钮（防误杀）  

**Interface 设计检查**：终止安全仍集中在既有 terminate 包装层；UI 不得直杀 PID。

### 4.6 Grok 健康诊断

**方向**：UI → grok_provider  
**形式**：Tauri command（**delta 于既有** `grok_provider_status` / `grok_list_backups`，不是平行状态源）

**契约**：

```text
grok_config_health() -> Result<GrokHealthReport, String>

GrokHealthReport {
  configPresent: bool
  authPresent: bool
  profilesCount: number
  activeMode: "official" | "profile" | "unknown"
  activeProfileId: string | null
  configMatchesActive: bool | null   // 可对齐既有 status.config_matches_active
  backups: { name: string, modifiedAt: string }[]  // ≤20；仅 name + 时间，禁止 path/size 泄漏非必要
  issues: GrokHealthIssue[]          // 验收核心：至少覆盖下列 code 的可测生成
}

GrokHealthIssue {
  code: "auth_missing_official"
     | "config_missing"
     | "config_mismatch_active"
     | "profiles_empty"
     | "backup_dir_unreadable"
     | "active_profile_missing"
  severity: "info" | "warn" | "error"
  message: string
}

// 禁止字段：apiKey、secret 原文、备份绝对路径、profiles 内 key
```

**相对现有 API 的 delta（验收以 issues 为准）**：

- 可内部复用 `status()` / `list_backups()` 读路径，但 **必须** 产出可测 `issues[]`  
- 不得只做「再包一层 status」而无 issues  
- 本 command **不**触发 restore / apply / 出站  

**约束**：只读；不写 config / 不删备份；错误与日志不带 key。

### 4.7 终端扩展

**方向**：`TerminalType` + `TerminalLauncher`  
**形式**：枚举扩展 + trait 实现

**契约**：

```text
// 对齐现码 trait（不得抄旧签名）
trait TerminalLauncher {
  fn terminal_type(&self) -> TerminalType;
  fn is_available(&self) -> bool;
  fn supports_tab(&self) -> bool;
  fn launch(&self, spec: &CommandSpec, mode: LaunchMode) -> Result<(), LaunchError>;
}

// TerminalType: system | iterm2 | ghostty | {new kebab-case}
// list_available_terminals：available 的新类型必须出现
// preferred_terminal 反序列化未知值 → 回退 System（可不写回磁盘）
```

**约束**：

- 禁止 AppleScript / CLI 直注业务 command 字符串；只注入已校验 wrapper 路径  
- 必须先 `validate_command_spec`  
- design 阶段本机探测 Warp / WezTerm，至少落地 **1** 个；**若本机均不可用且 CI 无法冒烟，允许 drop 本 item**，并从 harden `depends_on` 移除  
- 新终端 tab/window 行为必须在 design 写明（含不支持 tab 时的回退）

### 4.8 CLI 扩展契约（编译期）

**方向**：维护者 / CI 测试 → 注册表完整性  
**形式**：文档 checklist + Rust 测试 + 前端 labels 挂点，非运行时插件 API

**契约**：

```text
// 对每个 CliType 变体，下列挂点必须存在（测试枚举强制）：
1. scanners() 中有对应 SessionScanner
2. command_spec_for_session 分支
3. security program 白名单含 resume program
4. validate_command_spec 形状（resume / --resume / --session 等）
5. session_delete / OpenCode 删除映射有测试
6. ARCHITECTURE 或 docs/user CLI 表有一行
7. 前端 types/labels（agent 展示名）覆盖该 CliType

// 不提供：
load_plugin(path) / 外部 .dylib 扫描器
```

**约束**：新增 CLI 仍需发版编译；契约只降低遗漏。

### 4.9 共享 preferences keys（本波新增）

```text
port_protect_ports: u16[]
// 可选（若 bulk UI 需持久化，默认不做）：无新 key
// 终端枚举扩展可能影响 preferred_terminal 合法集合
```

前端 `usePreferences` / AGENTS.md keys 列表必须在收口条目同步。

### 4.x 共享不变量

- 前端永不接收 `delete_target` 路径  
- 搜索仍默认不读对话全文（摘要 enrich 除外）  
- CSP 不得 null；capabilities 不因本波扩大到任意 shell  
- 破坏性操作：删除 / 杀端口必须确认或既有确认对话框；预检与 health 只读  

## 5. 子 feature 清单

1. **launch-preflight** — 新增 `preflight_launch`；launch 路径复用 block 判定；UI 展示检查结果  
   - 所属模块：LaunchPreflight  
   - 依赖：无  
   - 状态：planned  
   - 对应 feature：未启动  
   - 备注：最小闭环  

2. **session-health-inspect** — `inspect_session_health`；列表可按 missing_cwd / missing_source 筛选或角标  
   - 所属模块：SessionHygiene  
   - 依赖：无  
   - 状态：planned  

3. **session-summary-enrichment** — 各 scanner 摘要质量与长度约定；fixture 测试锁定  
   - 所属模块：SessionHygiene  
   - 依赖：无  
   - 状态：planned  

4. **session-bulk-delete** — 多选 + `delete_sessions` 批量删除与结果清单  
   - 所属模块：SessionHygiene  
   - 依赖：无（UI 可与 health 筛选组合，但无硬依赖）  
   - 状态：planned  

5. **session-disk-usage** — 按 CLI / 项目聚合 `approxBytes` 展示（可基于 inspect 或专用汇总 command）  
   - 所属模块：SessionHygiene  
   - 依赖：`session-health-inspect`  
   - 状态：planned  

6. **port-protect-list** — `port_protect_ports` 偏好 + terminate 拦截  
   - 所属模块：PortDepth  
   - 依赖：无  
   - 状态：planned  

7. **port-group-and-terminate-by-project** — Port 列表按项目目录分组；支持选中项目批量 terminate（走保护规则）  
   - 所属模块：PortDepth  
   - 依赖：`port-protect-list`  
   - 状态：planned  

8. **grok-config-health** — `grok_config_health` 诊断面板（无 secret）  
   - 所属模块：GrokOps  
   - 依赖：无  
   - 状态：planned  

9. **terminal-adapter-extend** — 至少一个新 `TerminalLauncher` + 偏好可选  
   - 所属模块：ExtendSurface  
   - 依赖：无  
   - 状态：planned  

10. **cli-extension-contract** — CliType 注册完整性测试 + 维护者 checklist 文档  
    - 所属模块：ExtendSurface  
    - 依赖：无  
    - 状态：planned  

11. **power-extend-harden-and-docs** — 交叉回归、ARCHITECTURE/用户文档/AGENTS、残留风险  
    - 所属模块：WaveClose  
    - 依赖：上述全部未 drop 条目  
    - 状态：planned  

**最小闭环**：第 1 条 `launch-preflight` 完成后，对 cwd 已删的 session 点启动会得到明确 block 原因且不拉起终端。

### Goal Coverage Matrix

| Goal / completion signal | Covered by item(s) | Verification entry | Evidence type | Core? |
|---|---|---|---|---|
| 启动前可对失效 cwd/program/源给出 block | launch-preflight | cargo test + 手工坏 cwd | test + manual | yes |
| 可识别并筛选陈旧 session | session-health-inspect | 手工删目录后 inspect | manual + unit | yes |
| 列表摘要噪声下降且可测 | session-summary-enrichment | scanner fixture tests | test | yes |
| 可一次删除多条并看到失败项 | session-bulk-delete | 可丢弃 session 批量删 | manual + unit | yes |
| 可知各 CLI session 近似体积（可 null / size_capped） | session-disk-usage | UI 展示 + unit | manual + test | no |
| 保护端口不可被 terminate | port-protect-list | cargo test | test | yes |
| 可按项目查看并关闭端口 | port-group-and-terminate-by-project | 临时 http.server | manual + test | yes |
| Grok 页可见无 secret 健康报告 | grok-config-health | 手工打开 Grok 页 | manual + unit | yes |
| 偏好中可选新终端且能 launch | terminal-adapter-extend | 本机装对应终端 smoke | manual + unit | yes |
| 新增 CLI 挂点有测试防漏 | cli-extension-contract | cargo test | test | yes |
| 文档/架构/偏好 key 与代码一致 | power-extend-harden-and-docs | diff 核对 | acceptance | yes |

## 6. 排期思路

- **先最小闭环** `launch-preflight`：每次启动路径受益，且建立 `session_list_id` 只读检查样板。  
- **Session 卫生可并行**：inspect / summary / bulk-delete 无环；disk-usage 跟在 inspect 后。  
- **Port**：先 protect 再 group/terminate-by-project，避免无保护批量杀。  
- **Grok / 终端 / CLI 契约**与 Session 并行。  
- **最后收口**，不提前标 epic 完成。

### Top 3 风险与缓解

1. **批量删除误伤** — 上限 50、二次确认、复用单条校验、failures 可见、仅可丢弃数据做 smoke。  
2. **预检与缓存窗不一致** — 无 `delete_target` 时 source 仅 warn；launch 在 full scan 前对源的 block 策略写清。  
3. **新终端 AppleScript/CLI 行为差异** — 强制 wrapper + `validate_command_spec`；design 含本机探测 spike；失败则换另一终端或 drop 该终端变体。

### 非显然依赖

- 新终端本机是否安装影响 smoke（CI 可只测 `is_available` false 路径 + 单元）。  
- Grok 备份目录布局依赖现有 `grok_provider` 备份实现，health 只读枚举。  
- `App.tsx` / `state` 已偏大：feature-design 应优先新文件/hooks，避免无意义膨胀（观察项，不单开重构 epic）。

### 关键假设

- 用户接受「批量删除不可撤销」，只要确认与结果清单清晰。  
- 轴 4 的「平台与扩展」在本 epic 中解释为 **macOS 终端扩展 + CLI 编译期契约**，不是 Windows。  
- 按项目杀端口以 `working_directory` 匹配为准，cwd 解析不到的进程只能手选。  
- 摘要 enrichment 不引入可配置的全文搜索引擎。

### 基线与验证入口

- `pnpm build`  
- `cd src-tauri && cargo test --lib`  
- 桌面：`pnpm tauri dev` smoke（预检 block、批量删可丢弃 session、protect 端口、Grok health、新终端若已装）  
- 删除 / 杀端口 smoke **仅**可丢弃数据  

### 交付物落点

| 条目 | 主要落点 |
|------|----------|
| launch-preflight | `src-tauri` 预检模块 + command + 前端启动 UI |
| session-health-inspect | command + sessionUtils/UI 筛选 |
| session-summary-enrichment | `scanner/*` + tests |
| session-bulk-delete | command + 多选 UI + 确认框 |
| session-disk-usage | UI 聚合 + 可能薄 command |
| port-protect-list | preferences + terminate 拦截 + UI |
| port-group-and-terminate-by-project | PortWorkspace UI 分组多选 + 既有 terminate |
| grok-config-health | grok_provider + ProvidersWorkspace |
| terminal-adapter-extend | launcher + TerminalType + Controls |
| cli-extension-contract | tests + docs/dev checklist |
| power-extend-harden-and-docs | ARCHITECTURE / user guide / AGENTS / acceptance |

### 知识回写点

- 新终端启动坑 → compound learning / attention  
- 批量删除上限与 partial success 语义 → ARCHITECTURE 安全口径  
- CLI 扩展 checklist → docs/dev 与 AGENTS scanner 规则  

## 7. 观察项

- `session-launcher-next-wave` 主文档 status 仍为 `active` 但 items 全 done → 建议标 `completed`（不在本 epic 自动改，除非用户授权 update）。  
- 空壳 `devpilot-port-monitor` roadmap 仍可归档删除。  
- `VISION.md` requirements 索引落后于已落地能力，建议 `cs-docs-neat` / `cs-req` 另刷。  
- Windows 一等、托盘/全局快捷键、Keychain、全文索引：明确不做，可各自另开 epic。  
- `App.tsx` / `state/mod.rs` 体积：本波要求新代码优先拆文件，不单开大重构条目。  
- 若用户坚持轴 4 = Windows：本 roadmap 需 superseded 并新开 Windows epic。

## 8. 变更日志

- 2026-07-13：新建 draft epic，收敛 brainstorm 轴 1/3/4（Windows 排除；扩展=终端+CLI 契约）。
- 2026-07-13：独立 review round-1 → 锁死 preflight/ops_ready 矩阵、bulk 走 AppState 全路径、health/OpenCode 语义、Grok health delta、port 按项目 UI-only、summary 截断与 trait 签名。
- 2026-07-13：独立 review round-2 → preflight 源探测与 inspect 同源（OpenCode=行）；bulk recent sanitize 标为相对单条的 delta；交付物表去掉 port thin command。
