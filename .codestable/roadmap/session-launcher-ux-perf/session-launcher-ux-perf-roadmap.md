---
doc_type: roadmap
slug: session-launcher-ux-perf
status: active
created: 2026-07-13
last_reviewed: 2026-07-13
tags: [session-launcher, performance, ux, refactor, polish]
related_requirements:
  - quick-session-access
  - responsive-window-ui
related_architecture:
  - ARCHITECTURE
---

# Session Launcher 体验与性能 polish

## 1. 背景

`session-launcher-next-wave` 与 `session-launcher-power-extend` 均已 complete。产品能力面（五 CLI、预检/卫生/批量删、Port 保护与按项目关、Grok 诊断、WezTerm、扩展契约）已齐。

当前体感与可维护性瓶颈集中在：

| 信号 | 证据 |
|------|------|
| 前端编排过重 | `src/App.tsx` ≈ 1000 行：Session/Port/Grok 三页 + 快捷键 + 磁盘/批量/最近启动 |
| 后端 state 继续膨胀 | `state/mod.rs` ≈ 1100 行（ports 已拆出，session/launch/prefs 仍堆叠） |
| 列表探测成本 | full scan 后对最多 200 条 `inspect_session_health`（源 IO）；缓存窗已跳过，仍可按需再收紧 |
| 大列表渲染 | 无虚拟化；session 很多时 DOM 全量挂载 |
| 文档滞后 | README 仍写「三终端 / 无批量删」等，与 power-extend 现状不符 |

对象：日常 session 很多、频繁刷新/启动的 macOS 用户；以及要继续加功能而不继续堆 `App.tsx` 的维护者。

用户在 power-extend 收口后选定方向：**体验与性能 polish**（非 Windows、非新 CLI 产品类型）。

## 2. 范围与明确不做

### 本 roadmap 覆盖

1. **前端编排拆分**：Session / Port / Grok 工作区与 App 壳层解耦，偏好/快捷键边界清晰  
2. **后端 state 薄化**：session 删除/预检/健康/偏好相关从 `state/mod.rs` 再拆模块（行为等价优先）  
3. **健康探测按需化**：默认不全量 inspect；筛选陈旧 / 打开磁盘占用时再探测；可选空闲预热  
4. **列表性能**：session 列表虚拟化或窗口化渲染；保持键盘导航与收藏/多选  
5. **启动与扫描体感**：刷新/预检/启动的 loading 与状态文案一致；避免双次无意义 preflight 提示闪烁  
6. **文档与壳层对齐**：README / 用户文档补 WezTerm、批量删、预检、保护端口；next-wave 主文档标 completed（若未标）  
7. **收口**：交叉回归 + ARCHITECTURE 模块边界回写

### 明确不做

| 不做 | 理由 |
|------|------|
| Windows / Linux 一等 | 独立 v2 epic |
| 新 CLI 产品类型 / 动态插件 | power-extend 已交契约；本波不扩产品面 |
| 全文索引对话 | 隐私与 IO；另评估 |
| 删除回收站 / 撤销 | 仍确认即破坏 |
| 托盘 / 全局快捷键 / 菜单栏启动器 | 产品形态跃迁；另开 epic |
| 大范围 UI 视觉 redesign | 只做结构与性能，不换设计语言 |
| 行为不等价的「顺手功能」 | polish 默认行为等价；有用户可感变化须写进 item 验收 |

### Granularity Gate

| 判断项 | 结论 |
|--------|------|
| 为什么不是 single feature | 跨前端拆分、后端 state、列表虚拟化、探测策略、文档 多交付面；有依赖（拆分 → 虚拟化挂载点） |
| 为什么不是 brainstorm | 方向已选；成功信号可 yes/no |
| 最小闭环 | `app-shell-split`：App 只保留 tool 切换与壳，Session 页可独立渲染且 `pnpm build` 绿 |

### 方案深度 pre-pass

| 候选 | 取舍 |
|------|------|
| A. 拆分 + 按需 inspect + 虚拟列表 + 体感 + 文档 | **采用** |
| B. 只虚拟列表 | 拒 — App/state 债会继续挡迭代 |
| C. 含 Windows | 拒 — 明确不做 |
| D. 大 redesign | 拒 — 非本波 |

## 3. 模块拆分（概设）

```
session-launcher-ux-perf
├── AppShell：tool 切换、全局快捷键、主题/status 壳
├── SessionWorkspace：列表/筛选/批量/磁盘/最近启动（自 App 迁出）
├── StateModules：Rust AppState 按域拆文件（行为等价）
├── HealthOnDemand：inspect 触发策略
├── ListVirtualization：session 列表窗口化渲染
├── LaunchFeel：启动/刷新/预检 UI 反馈一致性
└── WaveClose：文档 + 回归
```

### AppShell · 应用壳

- **职责**：`activeTool`、全局 status、Cmd+K 分发、三工具页挂载  
- **承载**：`app-shell-split`  
- **Depth**：shallow 于壳；业务逻辑下沉 workspace

### SessionWorkspace · Session 工作区

- **职责**：原 App 内 sessions 分支：筛选、列表、批量、磁盘占用、最近启动、context menu  
- **承载**：`app-shell-split`（同 PR/同 feature 迁出）  
- **触碰**：`App.tsx`、新 `src/components/SessionWorkspace.tsx`（或等价）

### StateModules · 后端状态拆分

- **职责**：在不改变 Tauri command 契约的前提下拆 `state/mod.rs`（如 session_ops / prefs 已有 ports）  
- **承载**：`state-module-split`  
- **约束**：**行为等价**；`cargo test --lib` 全绿为门闩

### HealthOnDemand · 按需健康探测

- **职责**：默认不全量 inspect；`healthFilter≠all` 或磁盘面板打开时 invoke；可保留 full scan 后可选预热（item 内写死默认）  
- **承载**：`health-inspect-on-demand`  
- **触碰**：`useSessions`、`App`/`SessionWorkspace`

### ListVirtualization · 列表虚拟化

- **职责**：按 Agent / 按项目两种模式下，session 行窗口化渲染；键盘 ↑↓/Enter 仍可用  
- **承载**：`session-list-virtualize`  
- **约束**：不暴露 delete_target；不改 Session 契约

### LaunchFeel · 启动体感

- **职责**：减少启动路径双 preflight 的状态闪烁；刷新/删除/批量的 status 文案一致  
- **承载**：`launch-feedback-polish`  
- **约束**：不削弱后端 launch 门闩（仍必须 preflight）

### WaveClose · 收口

- **职责**：README/用户文档/ARCHITECTURE 模块边界；回归命令  
- **承载**：`ux-perf-harden-and-docs`

## 4. 接口契约（硬约束）

### 4.1 行为等价优先

- `state-module-split`、`app-shell-split`：**不得**改变 Tauri command 名、JSON 字段、删除/启动安全语义  
- 允许的用户可感变化仅限：inspect 触发时机、列表滚动性能、status 文案时序  

### 4.2 健康探测触发（写死默认）

```text
// 默认策略（feature 不得改语义，除非 roadmap update）
- fromCache=true 的 scan 结果：不调用 inspect（已落地，保持）
- full scan 后：默认不自动 inspect（本波改为按需；相对 power-extend 的 delta）
- 触发 inspect 的条件（满足任一即可，一次批量 ≤200 ids）：
  1. healthFilter ∈ {stale, missing_cwd, missing_source}
  2. 磁盘占用面板展开
  3. 用户显式点「重新探测」（若 UI 提供；可选）
- 列表角标：仅在已有 health 缓存时显示；无缓存不假装 missing
```

### 4.3 虚拟列表

```text
// 约束
- 虚拟化作用域：session 行（SessionRow）；分组 header（AgentGroup / ProjectBucket）可保留
- 键盘：可见结果序列上的 ↑↓/Enter 与现 quickAccess 语义一致
- 多选 checkbox / 收藏 / 启动按钮在虚拟窗口内仍可交互
- 禁止为虚拟化把 delete_target 或源路径塞进前端
```

### 4.4 拆分落点（建议路径，实现可微调）

| 模块 | 建议路径 |
|------|----------|
| Session 工作区 | `src/components/SessionWorkspace.tsx` |
| App 壳 | `src/App.tsx` 变薄 |
| state session ops | `src-tauri/src/state/session_ops.rs` 或 `launch.rs`（名称可调） |
| 虚拟列表 helper | `src/lib/sessionListVirtual.ts` 或组件内 |

## 5. 子 feature 清单与依赖

| # | slug | 一句话 | depends_on | 性质 |
|---|------|--------|------------|------|
| 1 | app-shell-split | App 壳与 SessionWorkspace 拆分 | [] | structural |
| 2 | state-module-split | Rust AppState 按域拆文件（行为等价） | [] | structural |
| 3 | health-inspect-on-demand | inspect 按需触发 | [app-shell-split] | functional |
| 4 | session-list-virtualize | session 行虚拟化 | [app-shell-split] | functional |
| 5 | launch-feedback-polish | 启动/刷新反馈一致、减少闪烁 | [health-inspect-on-demand] | functional |
| 6 | ux-perf-harden-and-docs | 文档对齐 + 回归收口 | 全部未 drop | non-functional |

依赖：3/4 依赖 1 的挂载点；2 与 1 可并行；5 在 3 之后避免探测策略与文案打架；6 最后。

### 核心验收路径（roadmap 级）

1. 冷启动缓存秒开 → full refresh 后列表仍可用；未开陈旧筛选时**不**批量 inspect（可用日志/断点/网络侧 invoke 计数验证）  
2. 切换到「陈旧」筛选 → 出现 inspect 调用 → 角标/过滤生效  
3. 展开磁盘占用 → inspect（若尚无缓存）→ 聚合数字或「未知」  
4. 100+ session fixture 或本机大列表滚动流畅；键盘 ↑↓/Enter 仍能启动  
5. 启动失败仍显示中文 preflight 原因；后端门闩仍在  
6. `pnpm build` + `cargo test --lib` 全绿；README 提及 WezTerm / 批量删 / 预检  

## 6. 验证策略

| 命令 | 用途 |
|------|------|
| `pnpm build` | 每前端 feature |
| `cd src-tauri && cargo test --lib` | 每后端 / 契约 feature |
| `pnpm tauri dev` smoke | 筛选/磁盘/滚动/启动（可丢弃数据） |

## 7. 观察项

- next-wave roadmap 主文档 status 仍可能为 `active` → 本波 harden 可顺带标 `completed`  
- 虚拟列表库选型：优先轻量自研窗口或现有依赖；**禁止**为虚拟化引入过重 UI 框架  
- `ProvidersWorkspace` / `PortWorkspace` 体积次优先；本波不强制拆，记观察  
- Windows、托盘、全文索引：仍另开 epic  

## 8. 变更日志

- 2026-07-13：用户在 power-extend 完成后选定「体验与性能 polish」；新建 draft epic。
