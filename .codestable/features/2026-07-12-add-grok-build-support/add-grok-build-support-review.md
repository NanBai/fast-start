---
doc_type: feature-review
feature: 2026-07-12-add-grok-build-support
status: passed
reviewer: subagent
reviewed: 2026-07-12
round: 1
---

# add-grok-build-support 代码审查报告

## 1. Scope And Inputs

- Design: none（feat-ff；契约见用户需求 + `add-grok-build-support-ff-note.md`）
- Checklist: none（fastforward）
- Evidence pack: none
- Gate results: none
- DoD results: none
- Implementation evidence: 对话实现 + 用户确认 UI OK + `cargo test --lib` / `pnpm build`
- Diff basis: 工作区 unstaged/untracked；**仅归因 Grok 相关文件**
- Baseline dirty files: port-monitor 全套（`port_monitor*`、`PortWorkspace`、`usePorts` 等）——本轮审查明确排除

### Independent Review

- Detection: 原生 Task agent（code-reviewer）可用；Paseo 不可用；`ocr` CLI 已安装但 LLM endpoint 未配置（`ocr llm test` 失败）
- 环节 A 独立隔离 Task agent: native-agent + completed（`019f55a9-40cf-7721-9fb9-91c2cde77ca0`）
- 环节 B OCR CLI: not-available（endpoint 未配置）
- OCR severity mapping: High→blocking/important, Medium→nit/suggestion, Low→discarded
- Merge policy: 子 agent findings 已逐条对照源码核验；#2/#3 在定稿前已修并复测
- Gate effect: `reviewer: subagent` 可放行；OCR 缺失不阻塞

## 2. Diff Summary

- 新增：`src-tauri/src/scanner/grok_build.rs`；ff-note / 本 review
- 修改：`models.rs`（CliType）、`scanner.rs`、`security.rs`、`launcher.rs`、`types.ts`、`AgentGroup.tsx`、`BrandMark.tsx`、`session-list.css`、`App.tsx` 副标题、`README.md`、`docs/user/session-launcher.md`、`AGENTS.md`
- 删除：none
- 未跟踪 / staged：`grok_build.rs` untracked；port-monitor 未跟踪但**排除**
- 风险热点：新 CLI 源、program 白名单、启动 PATH、Directory 删除

## 3. Adversarial Pass

- 假设的生产 bug：默认用户从 App 启动时 login PATH 不含 `~/.grok/bin` → `grok: command not found`
- 主动攻击过的反例：自定义 `GROK_HOME` 扫描/resume 不一致；percent-decode-only cwd 无测；单文件 IO 拖垮整批；删除边界
- 结果：PATH 实效与 percent-decode 测试升级为 findings 并已修；`GROK_HOME` 非默认场景进 residual-risk

## 4. Findings

### blocking

none

### important

- [x] REV-001 `src-tauri/src/launcher.rs` wrapper PATH 仅在 fallback 含 `~/.grok/bin`，login PATH 成功但漏目录时找不到 `grok`
  - Evidence: 独立 reviewer + `write_command_wrapper` 原逻辑；Grok 默认装在 `~/.grok/bin`
  - Impact: 打包/精简环境启动失败
  - Expected fix scope: wrapper 在解析 login PATH 后始终 prepend `$HOME/.grok/bin`（若目录存在）
  - **Round-1 修复**：已 prepend + wrapper 测试断言

- [x] REV-002 `src-tauri/src/scanner/grok_build.rs` percent-decode-only cwd 分支无 scan 级测试
  - Evidence: 仅有 `percent_decode` 单测；无「summary 无 cwd + 无 .cwd + 目录名含 %」fixture
  - Impact: 回退路径回归可静默坏
  - **Round-1 修复**：新增 `scanner_uses_percent_decoded_group_name_when_summary_cwd_missing`

- [ ] REV-003（延后 / residual）`GROK_HOME` 扫描与 resume 终端环境不一致
  - Evidence: scanner 读 env；wrapper 不 export `GROK_HOME`
  - Impact: 仅自定义 `GROK_HOME` 用户；默认 `~/.grok` 无影响
  - Expected fix scope: wrapper 透传或文档声明不支持 App 内自定义 home
  - **用户路径**：接受为 residual-risk（产品默认路径）

### nit

- [x] REV-004 `README.md` 删除说明未写 Grok session 目录 → 已补
- [ ] REV-005 `ARCHITECTURE.md` 仍写三家 CLI — 不在本次 scope；建议后续 `cs-docs-neat` / arch 刷新
- [ ] REV-006 launcher 注释仍提三家 CLI 名 — 不阻塞

### suggestion

- [ ] REV-007 单 `summary.json` 读失败用 `?` 会拖垮整 CLI 扫描（与 codex 同款）；可改为 skip
- [ ] REV-008 删除后 group 空壳 / `prompt_history.jsonl` 残留 — 可接受

### learning

- Grok session 布局与官方 `17-sessions.md` 一致；id 为 UUIDv7，兼容 `validate_session_id`
- resume：`grok --resume <id>` + `cd:true`

### praise

- 与现有 SessionScanner / delete Directory / 白名单模式一致
- fixture 测试覆盖主路径与无 cwd 跳过
- 前端 `Record<CliType, _>` 编译期防漏标

## 5. Test And QA Focus

- QA 必须重点复核：三终端启动 Grok；删除可丢弃 session；搜索/分组展示
- Evidence pack residual risks：自定义 `GROK_HOME` 未测
- 建议新增或加强的测试：已补 percent-decode scan + wrapper PATH；可选 `validate_command_spec` 绿路径
- 不能靠 review 完全确认的点：真机长路径 slug+hash + `.cwd`（逻辑有测）

## 6. Residual Risk

- 自定义 `GROK_HOME`：扫描与 resume 可能不一致 — 默认安装不受影响
- Architecture 文档未列 Grok — 后续文档收尾
- 工作树 port-monitor dirty：scoped-commit 必须排除

## 7. Verdict

- Status: **passed**
- Next: feat-ff 收尾提交（用户确认后 scoped-commit：仅 Grok 相关文件 + ff-note + review）
