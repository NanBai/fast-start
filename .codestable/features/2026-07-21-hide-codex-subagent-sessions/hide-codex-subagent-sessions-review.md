---
doc_type: feature-review
feature: 2026-07-21-hide-codex-subagent-sessions
status: passed
reviewer: subagent
reviewed: 2026-07-21
round: 1
lane_a_state: completed
lane_a_ref: "80e893b0-bec1-45de-8a18-bf89988376eb"
lane_a_reason: ""
lane_b_state: skipped
lane_b_ref: ""
lane_b_reason: "skipped-scope-ambiguous：工作区含无关 dirty，ocr 裸扫会越界；本轮 scope 仅 src-tauri/src/scanner/codex.rs"
---

# hide-codex-subagent-sessions 代码审查报告

## 1. Scope And Inputs

- Design: none（Quick / ff-note）
- Checklist: none
- Evidence pack: none
- Gate results: none
- DoD results: none
- Implementation evidence: `hide-codex-subagent-sessions-ff-note.md`；用户 EffectAccepted
- Diff basis: `git diff -- src-tauri/src/scanner/codex.rs`（+69/-1）
- Review mode: initial
- Baseline dirty files: `.codestable/attention.md`、`src-tauri/Cargo.lock`、`.codestable/issues/2026-07-15-login-path-space-node-not-found/`（本轮不归因）

### Independent Review

- Detection: Task agent（generalPurpose）可用；ocr CLI 可用但 scope 歧义故跳过裸扫
- 环节 A 独立隔离 Task agent: independent-agent + completed
- 环节 B OCR CLI: skipped
- OCR severity mapping: High→blocking/important, Medium→nit/suggestion, Low→discarded
- Merge policy: 环节 A findings 已逐条本地核验后合并；主 agent 复跑 `cargo test --lib scanner::codex::` → 8 passed
- Gate effect: none（passed，`reviewer: subagent`）

## 2. Diff Summary

- 新增：无新文件
- 修改：`src-tauri/src/scanner/codex.rs`
- 删除：无
- 未跟踪 / staged：ff-note + 本 review
- 风险热点：none

## 3. Adversarial Pass

- 假设的生产 bug：漏隐（缺信号子会话）或误隐（畸形 source / 多条 meta）
- 主动攻击过的反例：仅 parent_thread_id、source.subagent null、多条 meta 顺序、大小写漂移、收藏残留、测试假阳性
- 结果：无升级为 blocking/important；格式漂移与缓存闪回进 residual-risk；多 meta「先 subagent 后 user」早退判定为正确策略（learning）

## 4. Findings

### blocking

none

### important

none

### nit

- [ ] REV-001 `src-tauri/src/scanner/codex.rs:197-200` `contains_key("subagent")` 对 `null`/空对象也会命中
  - Evidence: 逻辑推演；本机语料未见该畸形
  - Impact: 极端畸形 meta 可能误隐藏
  - Expected fix scope: 可选要求 `subagent` 为 object；不扩到 UI/偏好（本次不修）

- [ ] REV-002 `src-tauri/src/scanner/codex.rs:125` 注释写「重复行无妨」，与「任一行判定 subagent 则整文件跳过」不完全一致
  - Evidence: 注释 vs 早退行为
  - Impact: 后续读者误判可幂等合并
  - Expected fix scope: 改注释（本次不修）

### suggestion

- [ ] REV-003 集成测试补「仅 thread_source」/「仅 source.subagent」单信号路径；可选多条 meta fixture
- [ ] REV-004 隐藏后无法从 UI 删子 rollout 磁盘文件——产品范围外，需另开能力

### learning

- 本机语料：`thread_source=subagent` 与 `source.subagent` 同集；主会话 `source` 为 `"cli"|"exec"` 字符串
- 多条 meta 常见形态为首条 subagent、次条像 parent；对首条早退应保留，不能改成「最后一条 meta 为准」

### praise

- 过滤落在 `parse_codex_file` 的 `session_meta` 早退，不污染 `SessionScanner` 契约
- 双信号 OR 与真实 Codex schema 对齐；fixture 不依赖 `~/.codex`
- 改动面极窄

### residual-risk

- 未来 Codex 格式漂移可能导致漏隐/误隐
- 扫描缓存未 refresh 前可能短暂看到旧子会话（既有行为）

## 5. Test And QA Focus

- QA 必须重点复核：multi-agent 项目 refresh 后只见主会话；普通主会话不误减；收藏/最近启动子会话被 sanitize；冷启动/二次 refresh 稳定
- Evidence pack residual risks / gate warnings：none
- 建议新增或加强的测试：单信号 scanner fixture；多条 session_meta 早退
- 不能靠 review 完全确认的点：旧版/异机无双信号子会话；是否需要「显示子会话」开关

## 6. Residual Risk

- Codex schema 漂移 → 观察真实语料后再收紧/放宽判定
- 缓存闪回 → 用户 refresh 即可；非本 diff 引入

## 7. Verdict

- Status: passed
- Next: Quick 收尾——是否代为 scoped-commit（代码 + ff-note + review）；可选 `cs-keep` 沉淀

## 8. Focused Closure（无则写 none）

none
