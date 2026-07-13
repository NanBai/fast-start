---
doc_type: roadmap
slug: session-launcher-next-wave
status: active
created: 2026-07-13
last_reviewed: 2026-07-13
tags: [session-launcher, performance, ux, grok, port-monitor, polish]
related_requirements:
  - quick-session-access
  - extend-grok-providers-tool
  - responsive-window-ui
related_architecture:
  - ARCHITECTURE
---

# Session Launcher 下一波产品与性能迭代

## 1. 背景

Session Launcher 已具备：五 CLI session 扫描/启动/删除、搜索收藏、Port 监控、Grok 供应商（含官方账号/隐私/布局）、主题与响应式布局。2026-07-13 起完成启动扫描加速与 header tab 布局修复。

用户确认「以上分析都做」——将产品分析中的**高价值日常迭代**收敛为一份 epic：让「秒开、好找、好管 Grok、好关端口、好发布」成为可验收的一串子 feature，而不是散落的愿望清单。

对象：继续在 macOS 上用多 AI CLI 切换仓库的开发者；维护者按依赖 DAG 推进即可。

## 2. 范围与明确不做

### 本 roadmap 覆盖

1. **扫描体验**：本地缓存 + 立即 refresh + 扫描耗时可见
2. **Session 工作流**：按项目混合 Agent 视图、session 级收藏、空态/错误引导
3. **Grok 工具页打磨**：拉模型/连通测试、启用前 config 预览、Default 导入策略
4. **Port 工具页增强**：批量关闭、打开浏览器、可配置「项目服务」规则
5. **启动与历史**：最近启动记录、可选命令预览
6. **发布与收口**：release 脚本化、全波次 polish/harden/文档归并

### 明确不做

| 不做 | 理由 |
|------|------|
| Windows / Linux 一等支持 | attention 标为 v2；终端/路径/扫描根全换，单独 epic |
| 系统托盘 / 全局快捷键 | 改变产品形态，需单独决策 |
| 内置终端、云同步、账号体系 | 与「本机无云」产品定义冲突 |
| 插件化任意新 CLI 框架 | 过大；本波仍按现有 scanner 扩展模式 |
| API Key 硬件级加密 / 钥匙串深度集成 | 可后续安全 epic；本波最多文档化明文风险 |
| 删除回收站 / 撤销删除 | 既有「确认即破坏」契约，本波不改语义 |
| 再引入新 CLI 类型 | 除非本波收口后另开 epic |

### Granularity Gate

| 判断项 | 结论 |
|--------|------|
| 为什么不是 single feature | 跨扫描缓存、Session UI、Grok、Port、偏好、发布 6 个独立交付面；有共享契约与依赖 DAG |
| 为什么不是 brainstorm | 目标/边界/成功信号已在对话分析中收敛，可直接拆执行 |
| roadmap 边界 | 仅 macOS 本机工具迭代；不含平台扩展与云能力 |
| 最小闭环 | `scan-cache-and-metrics`：冷启动读缓存秒开列表 + 立即 refresh + 状态栏可见耗时 |

### 方案深度 pre-pass

| 候选 | 取舍 |
|------|------|
| A. 全量做分析中的日常迭代（缓存/混合视图/Grok 打磨/Port 增强/历史/发布） | **采用** — 用户明确「都做」 |
| B. 只做三条优先 | 用户拒绝缩范围 |
| C. 含 Windows/托盘 | 拒 — 与 v2/产品形态冲突，写入明确不做 |

最小闭环选「最窄端到端路径」：缓存扫描直接影响每次打开，不依赖其他新 UI 模式。

## 3. 模块拆分（概设）

```
session-launcher-next-wave
├── ScanCache：扫描结果持久化缓存、失效、立即 refresh、耗时指标
├── SessionWorkflow：混合项目视图、session 收藏、空态/错误引导
├── GrokProvidersX：模型拉取/探测、config 预览、Default 策略
├── PortOps：批量关闭、打开浏览器、服务规则偏好
├── LaunchHistory：最近启动记录与可选命令预览
└── ReleaseOps：发布脚本与波次收口文档
```

### ScanCache · 扫描缓存

- **职责**：在 `AppState` 扫描链路旁提供「磁盘缓存快照 → 即时返回 → 同流程立即 refresh / 用户显式刷新」；暴露耗时指标给前端状态栏。
- **承载**：`scan-cache-and-metrics`
- **触碰**：`state/mod.rs`、`commands.rs`、`useSessions`、偏好或独立 cache 文件
- **Depth**：deep — 调用方仍只调 `scan_sessions`/`refresh_sessions`，缓存藏在状态层

### SessionWorkflow · Session 工作流

- **职责**：前端列表派生与偏好扩展，不改 scanner 输出契约中的核心字段（可扩展可选字段）。
- **承载**：`session-mixed-project-view`、`session-favorites-and-empty-states`
- **触碰**：`sessionUtils`、`App.tsx`、`preferences`、列表组件
- **Depth**：deep 于前端派生层

### GrokProvidersX · Grok 增强

- **职责**：在现有 `grok_provider` 上增加非破坏性命令与 UI；不改 profiles 存储路径约定。
- **承载**：`grok-models-and-preview`、`grok-default-profile-policy`
- **触碰**：`grok_provider/*`、`ProvidersWorkspace`、`commands`/`lib`
- **Depth**：deep 于 grok_provider 模块

### PortOps · 端口运维

- **职责**：端口列表上的批量与导航操作；规则过滤可配置。
- **承载**：`port-power-ops`
- **触碰**：`port_monitor`、`usePorts`、`PortWorkspace`、preferences
- **Depth**：deep 于 port 子系统

### LaunchHistory · 启动历史

- **职责**：记录成功启动的 session 摘要，提供快捷再启动；可选展示将执行命令（脱敏）。
- **承载**：`launch-history-and-preview`
- **触碰**：`state` launch 路径、preferences、Session UI
- **Depth**：中等 — 偏好 + 薄 UI

### ReleaseOps · 发布与收口

- **职责**：可重复的 release 脚本；本 epic 完成后的文档/架构/回归收口。
- **承载**：`release-automation`、`wave-harden-and-docs`
- **触碰**：`docs/`、`.codestable`、仓库根脚本（如 `scripts/`）
- **Depth**：浅层工具 + 文档

## 4. 模块间接口契约 / 共享协议（架构层详设）

### 4.1 ScanCacheStore

**方向**：ScanCache → AppState / commands  
**形式**：Rust 模块内函数 + 既有 Tauri command 语义扩展

**契约（硬锁定，禁止 design 再「二选一」）**：

```
// 缓存路径（本 epic 锁定）
// {app_data_dir}/scan-cache-v1.json
// app_data_dir = Tauri app data dir（实现用 tauri path API）；测试可注入路径

ScanCacheSnapshot {
  version: u32,                 // 当前 = 1；不匹配则忽略
  saved_at: ISO8601,
  sessions: Session[],          // 仅前端可见字段语义；delete_target 永不写入此文件
  scan_errors: CliScanError[],
  total_ms: number              // 写出该 snapshot 的那次 full scan 耗时
}

// 冷启动路径（最窄 E2E，无独立 background worker）
scan_sessions / cached_scan():
  1. 若内存未 ops_ready：
     a. 若磁盘 snapshot version 匹配 → 构造 ScanResponse {
          sessions, scanErrors,
          fromCache: true,
          scanDurationMs: 0 或省略
        } 并立即返回（展示用；此时 ops 可能未就绪）
     b. 否则走 full scan_all，写 snapshot，fromCache=false，scanDurationMs=实测
  2. 前端约定：若 fromCache===true，在同一启动流程中**立即**再调 refresh_sessions()
     （无需后端 event bus；禁止「只读缓存且永不 refresh」标 done）

refresh_sessions / scan_all():
  - 并行五 CLI full scan（保持现逻辑）
  - 成功后 atomic write snapshot
  - 返回 fromCache=false + scanDurationMs
  - single-flight：若已有 scan 在途，后续 refresh 等待同一次结果（design 实现细节，必须有）

// ops 就绪
// - full scan 完成后内存 sessions 带 delete_target（现模型），ops_ready=true
// - 仅缓存展示阶段：delete_session 若缺 delete_target → Err 明确「请等待刷新完成/请刷新」
// - launch_session 仅需 cliType/sessionId/projectDir，缓存窗内允许启动；失败时提示刷新
// - OpenCode 删除走 session_id，不依赖 delete_target 文件路径；仍建议 full scan 防陈旧列表

// 失效
// - version 不匹配 → 忽略
// - 用户点刷新 → 强制 scan_all
// 不做 fs watcher
```

**验收硬条（可测）**：
1. 有合法 snapshot 时，`scan_sessions` 可不跑完整五 CLI 扫描线程即返回且 `fromCache=true`（unit/integration seam）。  
2. 缓存命中后、full scan 完成前：对需 `delete_target` 的 CLI，`delete_session` 必须明确失败。  
3. full scan 完成后：`delete_session` 在有效目标上成功。  
4. full scan 后 snapshot 文件更新。

**Interface 设计检查**：
- Module：`state` 拥有缓存；scanner 不感知缓存
- Seam：`scan_all` / `cached_scan` + 可注入 cache path
- Dependency：local file I/O
### 4.2 ScanTimingStatus

**方向**：ScanCache → 前端 status  
**形式**：扩展 `ScanResponse` 或并列 invoke 字段

```
ScanResponse {
  // 既有
  sessions, scanErrors
  // 新增（camelCase）
  scanDurationMs?: number       // 本次 full scan 耗时；缓存命中可为 0 或省略
  fromCache?: boolean           // true = 本包来自磁盘缓存
}
```

**约束**：旧前端忽略未知字段应可工作；本波前端必须展示 `fromCache`/`scanDurationMs` 之一于状态栏。

### 4.3 SessionFavoriteIds

**方向**：SessionWorkflow ↔ preferences  
**形式**：`preferences.json` key

```
key: favorite_session_ids
value: string[]   // Session.id（列表稳定 id），去重保序
sanitize: 仅保留当前 sessions 中存在的 id（与 favorite_project_dirs 同模式）
```

**约束**：与 `favorite_project_dirs` 并存；排序优先级 design 中定义（建议：session 收藏 > 项目收藏 > 时间）。

### 4.4 MixedProjectGrouping

**方向**：纯前端  
**形式**：`sessionUtils` 纯函数

```
groupSessionsByProject(sessions: SessionData[], options) ->
  ProjectGroup[] {
    projectDir, projectName, sessionsByCli: Map<CliType, SessionData[]>, sessions: SessionData[]
  }
```

**约束**：不改 `ScanResponse`；视图模式用前端 state 或 preferences key `session_list_mode: "by-agent" | "by-project"`。

### 4.5 GrokModelsAndPreview

**方向**：GrokProvidersX → UI  
**形式**：Tauri commands

```
grok_fetch_models(baseUrl, apiKey, upstreamFormat) -> { models: string[] }
grok_test_connection(baseUrl, apiKey, upstreamFormat, model?) -> { ok: true, latencyMs: number }
grok_preview_apply(profileId | profile draft) -> { text: string }  // 不写盘

// Default 策略见 grok-default-profile-policy item
// ensure_default：profiles 空且 config 存在时导入；
// 无 models_base_url → active=false；有 → active=true；不删档案
```

**出站安全（硬约束）**：
- 仅用户点击触发；启动不请求
- 仅 Rust 发 HTTP；前端不直连任意 URL
- scheme 仅 `http`/`https`；超时（建议 ≤10s）；响应体大小上限
- 不 log apiKey；CSP 保持非 null
- `upstreamFormat` 使用既有枚举语义（openai_chat / responses / …），非法值 Err
- 依赖（如 reqwest）在 feature design 论证，不默认放开 SSRF 到 link-local 以外策略可在 design 再收紧

### 4.0 Architecture deltas（相对当前 ARCHITECTURE 声明）

| 现状文档说法 | 本 epic 变更 |
|---|---|
| session 扫描结果不持久化 | 允许 **本机** scan-cache 文件加速展示（非云同步） |
| 应用无网络 | Grok 工具页在**用户触发**下允许 Rust 出站测模型/连通；仍无账号云同步 |
| 部署无网络依赖 | 核心 session/port 启动路径仍离线可用 |

### 4.6 PortPowerOps

**方向**：PortOps → 后端 / 前端  
**形式**：复用现有 terminate + 新增偏好与 UI

**现码事实（本 epic 不得假装新做）**：
- `terminate_port_processes(port_ids: Vec<String>)` **已支持多 id**
- 校验失败 **整批 all-or-nothing**（不引入部分成功明细，除非单独 RFC）
- Port 组级「关闭全部」UI 已部分存在

**本条目 delta 契约**：

```
// 1) 打开浏览器（loopback only）
// 优先 tauri-plugin-opener；URL 仅允许
//   http://127.0.0.1:{port} | http://localhost:{port} | http://[::1]:{port}
// 非 loopback → 前端禁用 + 若走后端则 Err

// 2) 服务规则 preferences（叠在现有 is_project_service 启发式之上）
key: port_project_path_prefixes: string[]  // 可执行路径前缀命中 → 视为项目服务
key: port_ignore_ports: number[]           // 列表中端口不展示或降权（design 定）
// 优先级：ignore 优先于展示；prefix 扩大「项目服务」集合；默认均为空

// 3) 批量关闭
// 复用 terminate_port_processes；若缺跨组多选 UI 则只补 UI，不重写 kill 语义
```

**约束**：保持 re-scan + 仅当前用户；默认 all-or-nothing。

### 4.7 LaunchHistory

**方向**：LaunchHistory ↔ preferences + launch 成功路径

```
key: recent_launches
value: RecentLaunch[] {
  sessionId: string,      // Session.id
  cliType: string,
  projectDir: string,
  projectName: string,
  summary?: string,
  launchedAt: ISO8601
}
// 上限 N=20，成功 launch_session 后写入；启动时 sanitize 掉已不存在的 session
```

**命令预览**（可选 UI）：

```
// 后端已有 command_spec 内部能力；可新增
preview_launch_command(sessionListId: string) -> { program: string, args: string[], cwd: string }
// 不执行；供 UI 展示
```

### 4.8 共享约束（全 epic）

- 平台：仅 macOS  
- 安全：不暴露 delete 源路径；program 白名单不变  
- 验证基线：`pnpm build` + `cd src-tauri && cargo test --lib`  
- 文档：用户可见能力变更更新 `docs/user/session-launcher.md`；架构现状更新 `ARCHITECTURE.md`

> 无「跨模块 HTTP 微服务」；均为进程内 Tauri 命令与前端派生。

## 5. 子 feature 清单

1. **scan-cache-and-metrics** — 扫描结果本地缓存、缓存命中秒开列表、同启动流程立即 refresh 全量扫描、状态栏展示耗时与是否缓存  
   - 模块：ScanCache  
   - 依赖：无  
   - 状态：planned · feature：未启动 · **最小闭环**

2. **session-mixed-project-view** — Session 列表增加「按项目」视图：同一 `projectDir` 下聚合多 CLI session，可切换回按 Agent  
   - 模块：SessionWorkflow  
   - 依赖：无（可与 1 并行）  
   - 状态：planned

3. **session-favorites-and-empty-states** — session 级收藏置顶；CLI 空数据/扫描失败空态与错误引导  
   - 模块：SessionWorkflow  
   - 依赖：无  
   - 状态：planned

4. **grok-models-and-preview** — 供应商编辑页拉取模型列表、连通测试；启用前 config 预览  
   - 模块：GrokProvidersX  
   - 依赖：无  
   - 状态：planned

5. **grok-default-profile-policy** — 修正 `ensure_default_profile`：无 API 上游时不强制 Default active，与官方模式一致  
   - 模块：GrokProvidersX  
   - 依赖：无（排程建议在 4 之后；items 不设硬 depends_on）  
   - 状态：planned  

6. **port-power-ops** — 在现有多 id terminate 之上：loopback 打开浏览器、可配置路径前缀/忽略端口；必要时补跨组多选 UI  
   - 模块：PortOps  
   - 依赖：无  
   - 状态：planned  
   - 备注：不重写 terminate；保持 all-or-nothing

7. **launch-history-and-preview** — 成功启动写入最近记录并提供快捷再启动；可选预览 resume 命令  
   - 模块：LaunchHistory  
   - 依赖：无  
   - 状态：planned

8. **release-automation** — 仓库内脚本：校验版本一致、构建 dmg、创建 gh release（文档说明用法）  
   - 模块：ReleaseOps  
   - 依赖：无  
   - 状态：planned

9. **wave-harden-and-docs** — 本 epic 收口：交叉回归、ARCHITECTURE/用户文档/AGENTS 偏好 key 同步、残留风险清单  
   - 模块：ReleaseOps  
   - 依赖：1–8 全部 done 或 dropped  
   - 状态：planned  
   - 备注：polish/harden/回归扫尾

**最小闭环**：第 1 条 `scan-cache-and-metrics` 做完后，冷启动可先展示缓存列表，刷新后更新，状态栏可见耗时。

### Goal Coverage Matrix

| Goal / completion signal | Covered by item(s) | Verification | Evidence | Core? |
|---|---|---|---|---|
| 有 snapshot 时 scan_sessions 返回 fromCache=true 且不跑完整五 CLI 扫描 | scan-cache-and-metrics | unit/integration seam | test | yes |
| 缓存窗内 delete 明确失败；full scan 后 delete 成功 | scan-cache-and-metrics | cargo test + 可选手工 | test | yes |
| 全量刷新写回 snapshot | scan-cache-and-metrics | 文件更新断言 / 手工 | test + manual | yes |
| 状态栏展示 fromCache 或 scanDurationMs | scan-cache-and-metrics | 手工 / UI | manual | yes |
| 按项目看到多 CLI session | session-mixed-project-view | 切换视图手工 | manual | yes |
| session 收藏重启保留 | session-favorites-and-empty-states | 重启 app | manual | yes |
| 空 CLI / 扫描失败有引导 | session-favorites-and-empty-states | 模拟错误/空 | manual | no |
| 拉模型与连通测试可用 | grok-models-and-preview | 对测试 upstream | manual + unit mock | yes |
| 启用前可见 config 预览 | grok-models-and-preview | 预览含 base_url | unit + manual | yes |
| 无 API 上游不强制 Default active | grok-default-profile-policy | unit + 手工 | cargo test | yes |
| loopback 打开浏览器可用；规则过滤生效 | port-power-ops | 临时 http.server + 改 preferences | manual | yes |
| 批量关闭仍 all-or-nothing 且 re-scan | port-power-ops | 复用现 API 回归 | cargo test + manual | no |
| 最近启动可再启 | launch-history-and-preview | 启动后列表出现 | manual | yes |
| 一键/脚本可发 release | release-automation | dry-run 文档 | command | no |
| 文档与架构与代码一致 | wave-harden-and-docs | diff 核对 | acceptance | yes |

## 6. 排期思路

- **先安全网/性能闭环**（1）：每次打开都受益，且为后续 UI 提供稳定数据节奏。  
- **Session 与 Grok/Port/Launch 可并行**（2–7）：模块边界清晰，接口以 preferences + 既有 command 扩展为主。  
- **Grok 两条串行**（4→5）：同改 `switcher`/状态，降低冲突。  
- **发布脚本可随时插空**（8）。  
- **最后收口**（9）：全量回归与文档，不提前标完成。

### Top 3 风险与缓解

1. **缓存与 delete_target 不一致** — 已锁定：缓存窗 delete 失败；前端缓存命中后立即 refresh；full scan 后 ops 就绪。  
2. **Grok 出站扩大攻击面** — 仅用户触发；Rust 发请求；scheme/超时/不 log key；CSP 不放宽。  
3. **Port 规则误配置藏端口** — 默认规则为空；文档说明；ignore 列表可清空。

### 关键假设

- 用户接受「先看缓存快照，启动流程内紧接一次 refresh」的一致性模型（非无限独立 worker）。  
- 混合项目视图不改变启动/删除语义。  
- release 脚本默认需本机已登录 `gh` 与签名环境（与现手工 release 一致）。
### 基线与验证入口

- `pnpm build`  
- `cd src-tauri && cargo test --lib`  
- 桌面：`pnpm tauri dev` smoke（session 启动、Grok 切换、Port 关闭仅可丢弃进程）  
- 发布：`pnpm tauri build`（收口/release 条目）

## 7. 观察项

- `devpilot-port-monitor` roadmap 目录为空壳，可归档或删除（不在本 epic 处理）。  
- API Key 钥匙串加密可另开 security epic。  
- Windows epic 仍独立。  
- scan 缓存路径已锁为 app data `scan-cache-v1.json`；design 只实现与测试路径注入，acceptance 回写 ARCHITECTURE。

## 8. 变更日志

- 2026-07-13：新建 epic，覆盖产品分析中的日常迭代全集（排除 v2/托盘/云）。
- 2026-07-13：独立 review 修订——锁定 ScanCache 冷启动路径（缓存 + 立即 refresh）；port-power-ops 改为现码 delta；补 Architecture deltas 与 Grok 出站约束；硬化完成信号。
