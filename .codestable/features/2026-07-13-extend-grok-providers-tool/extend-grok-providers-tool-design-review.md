---
doc_type: feature-design-review
feature: 2026-07-13-extend-grok-providers-tool
status: passed
reviewed: 2026-07-13
round: 2
---

# extend-grok-providers-tool feature design 审查报告

## 1. Scope And Inputs

- Design: `.codestable/features/2026-07-13-extend-grok-providers-tool/extend-grok-providers-tool-design.md`
- Checklist: `.codestable/features/2026-07-13-extend-grok-providers-tool/extend-grok-providers-tool-checklist.yaml`
- Intent / brainstorm: none
- Roadmap: none
- Related docs: `.codestable/requirements/extend-grok-providers-tool.md`、architecture（Grok/工具页相关）、既有 `add-grok-providers-tool` ff-note
- Code facts checked: `src-tauri/src/grok_provider/*`、`commands.rs`、`preferences.rs`、`src/types.ts`、`ProvidersWorkspace.tsx`、`useGrokProviders.ts`

### Independent Review

- Status: completed
- Detection: native-agent
- Provider / agent: general-purpose read-only Task agents（round 1 + round 2）
- Raw output: 会话内 subagent 回传（round 1: changes-requested；round 2: passed）
- Merge policy: 主 agent 已逐条核验 I-1～I-8 修订是否落盘；round 2 无新 blocking/important
- Gate effect: none（可进入用户整体 ConfirmDesign）

## 2. Design Summary

- Goal: 对齐 grok-build-switch v0.2.0 的官方账号切换、隐私保护写入、卡片排序/置顶；宿主仍为 Session Launcher Grok 工具页。
- Key contracts:
  - `grok_activate_official` → `GrokActivateOfficialResult`；login spawn 失败仍 Ok
  - `grok_apply_privacy_protection` → `GrokPrivacyResult`；无 config 可创建
  - `get/set_grok_provider_layout` + `buildProviderCards` 排序伪代码
  - status: `officialActive` / `officialLoggedIn`；auth 跟 `GROK_HOME`
- Steps: 6（纯函数 → config 变换 → 编排/命令 → 前端契约 → UI → harden/文档）
- Checks: 名词/编排/挂载/范围/验收齐全
- Baseline: `cargo test --lib` + `pnpm build`

## 3. Findings

### blocking

（无）

### important

（round 1 已全部关闭，见下）

#### Round 1 → 关闭记录

| ID | 主题 | 关闭方式 |
|---|---|---|
| I-1 | 布局 command | 命名 get/set_grok_provider_layout |
| I-2 | 默认排序 | §2.1 伪代码 |
| I-3 | Result DTO | 固定类型与 Ok/Err 分界 |
| I-4 | auth 路径 | GROK_HOME/auth.json |
| I-5 | 无 config backup | 跳过 backup + 空文本写出 |
| I-6 | ensure_default | 本轮保留并文档化 |
| I-7 | clear_active 半成功 | 失败返回 Err |
| I-8 | S2/S10 | 升 core + checklist |

### nit

- [x] FDR-N1 step-3 exit 补 S10（已修 checklist）
- [x] FDR-N2 Harden 与 sanitize 措辞（已改为渲染时过滤）

### residual-risk

- S10 备份失败路径依赖 FS 夹具，实现时需小心构造。
- auth 路径必须按 `GROK_HOME` 拼，禁止用 `config_path.parent()`。
- `ensure_default_profile` 保留导致纯 OAuth 首次可能 Default active（已接受）。
- `grok login` PATH 仍可能失败（已规定不回滚）。

### praise

- 术语守护清晰（官方 ≠ 删档案；隐私 ≠ 账号 /privacy）。
- 与现网 `grok_provider` 扩展而非平行子系统。
- login 不回滚与无 config 路径写清。

## 4. Evidence Confidence Ledger

| 检查项 | 结论 | 证据级 |
|---|---|---|
| Acceptance Matrix 覆盖核心场景 | 是 | E |
| 核心场景可追踪 step + 证据 | 是 | E |
| steps 可独立验证 / checks 可回溯 | 是 | E |
| Module/interface depth | 适用；invoke 不拼 TOML | C |
| dod.commands 字段完整 | 是 | E |
| 现状描述对代码 | 是（clear_active 缺失、backup 无文件 Err 等） | C |

## 5. Verdict

**status: passed**

可进入用户整体 design 确认（ConfirmDesign）。用户确认后将 `status: draft` → `approved`，再进 goal-package / 实现。

## 6. 用户 review 摘要（主 agent 附）

### 将做什么

1. 官方账号卡：一键清 API 覆盖，回退 OAuth（`auth.json`）
2. 隐私保护：一键写 telemetry/harness 本地开关
3. 卡片置顶 + 同组拖拽排序（app 偏好持久化）

### 明确不做

托盘、HTTP 管理面、拉模型、完整 config 编辑器、账号侧 /privacy、改 ensure_default 导入策略

### Top 3 风险

1. TOML 行级清理误删 → 单测锁清单  
2. `grok login` PATH 失败 → 不回滚 + 文案  
3. 布局死 key → 渲染过滤  

### 关键假设（可改）

- 布局存 app `preferences.json`，不存 `~/.grok_switch/settings.json`
- 缺 auth 时 best-effort 启动 `grok login`
- 拖动不跨置顶边界
- 保留 ensure_default_profile 行为
