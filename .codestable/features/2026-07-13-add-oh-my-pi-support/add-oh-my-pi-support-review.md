---
doc_type: feature-review
feature: 2026-07-13-add-oh-my-pi-support
status: passed
reviewer: subagent
reviewed: 2026-07-14
round: 2
---

# add-oh-my-pi-support 代码审查报告

## 1. Scope And Inputs

- Design: `.codestable/features/2026-07-13-add-oh-my-pi-support/add-oh-my-pi-support-design.md`（`status: approved`）
- Checklist: `.codestable/features/2026-07-13-add-oh-my-pi-support/add-oh-my-pi-support-checklist.yaml`（steps 全 `done`）
- Evidence pack: none（goal 模式）
- Gate results: none
- DoD results: none
- Implementation evidence:
  - checklist 10 步全 done
  - `cd src-tauri && cargo test --lib` → 129 passed（含 omp scanner/provider/cli_contract 专项）
  - `pnpm build` → 绿（tsc + vite）
  - 新文件：scanner/oh_my_pi.rs、omp_provider.rs、useOmpProviders.ts
  - goal-state 记录实现已收口
- Diff basis: 工作区 unstaged + untracked（无 staged）；核心归因于 omp 支持实现；feature 目录未提交
- Baseline dirty files:
  - `.codestable/attention.md`（全局【白哥】规则追加，与本 feature 无关）
  - `AGENTS.md`（除 omp scanner 规则外可能有其他小改）
  - `.codestable/architecture/ARCHITECTURE.md`（可能包含 omp 树更新）
  - 以上记为 baseline/residual-risk，不计入本轮 feature 验收

### Independent Review

- Detection:
  - Paseo subagent：不可用（无对应 mcp 工具）
  - 原生 Task agent：可用，通过 `spawn_subagent`（code-reviewer 类型）启动独立上下文审查
  - OCR CLI：已安装但 `ocr llm test` 失败（"no valid LLM endpoint configured"）
- 环节 A 独立隔离 Task agent: `native-agent`（subagent） + **completed**
- 环节 B OCR CLI: **not-available**
- OCR severity mapping: 未启用
- Merge policy: 环节 A 返回结果已由主 agent 逐条本地事实核验（读源码、运行测试、比对 design/checklist、git diff）；无 pending
- Gate effect: 环节 A 完成，`reviewer: subagent` 满足 gate 放行条件（无需 fallback）

## 2. Diff Summary

- 新增：
  - `src-tauri/src/scanner/oh_my_pi.rs`（完整 scanner + fixture 测试）
  - `src-tauri/src/omp_provider.rs`（list_providers / get_config_health / set_role_model + 测试）
  - `src/hooks/useOmpProviders.ts`
  - `.codestable/features/2026-07-13-add-oh-my-pi-support/` 下的 design/checklist/goal/review 等
- 修改：
  - 后端：`models.rs`（+OhMyPi）、`scanner.rs`（注册 + command_spec + 测试）、`security.rs`（+ "omp" + validate 分支）、`commands.rs`（omp_* 3 个命令）、`cli_contract.rs`（6 类型全覆盖测试）、`lib.rs`（模块/命令注册）
  - 前端：`types.ts`（CliType union + labels/order + Omp* 接口）、`App.tsx`、`AgentGroup.tsx`、`ProvidersWorkspace.tsx`（+ omp section）、`ProvidersToolPanel.tsx`、`BrandMark.tsx`
  - 文档：`AGENTS.md`（scanner 规则）、`docs/user/session-launcher.md`、`.codestable/architecture/ARCHITECTURE.md`
- 删除：none
- 未跟踪：上述 3 个新代码文件
- 风险热点：
  - 安全/启动：security 白名单 + resume 形状（`-r` / `--resume`）
  - 配置写盘：`~/.omp/agent/config.yml`（带备份）
  - 扫描：松散 JSONL 解析（title 先行 + 任意位置 session header）
  - UI：Providers 区窄追加 section（Grok 路径完全隔离）

## 3. Adversarial Pass

- 假设的生产 bug：env 覆盖下扫描到会话但 provider 写到错误 config / UI 只能设置 3 个硬编码 role 导致用户自定义 role 无法切换 / YAML 重写静默丢用户注释导致配置损坏后 omp 行为异常
- 主动攻击过的反例（已核验）：
  1. title 先行真实布局：oh_my_pi.rs 正确处理，非首行 header 也提取 id/cwd/summary，fixture + 真实样例覆盖
  2. 缺失目录：scanner 返回 NotFound，其它 CLI 继续；cli_contract 保证注册不漏
  3. 非法 role/model：set_role_model 显式 validate + 前端按钮禁用
  4. 写盘破坏：备份先生成，但全量 serde 仍会丢注释（design 已列为风险）
  5. 路径泄漏：list/health 返回绝对 modelsYmlPath/config_path（与 Grok 消毒惯例不符）
  6. env 不一致：PI_CODING_AGENT_DIR 在 scanner/provider 计算方式不同
  7. 命令形状：cli_contract 全量循环测试 + security validate 守住；无多余参数
- 结果：无 blocking 进入 findings；important 见 §4；其余进 residual-risk / QA focus

## 4. Findings

### blocking

none（关键路径有安全 gate、测试、复用既有 delete/source 逻辑；无设计违反导致无法 QA）

### important

- [ ] REV-001 `src-tauri/src/omp_provider.rs:70-85` 与 `src-tauri/src/commands.rs:404`（list_providers 返回 modelsYmlPath 等绝对路径）；`get_config_health` 也返回 config_path / models_path  
  Evidence: 后端始终把绝对路径塞进 JSON 返回值；前端 types.ts:313 也声明 configPath；ProvidersWorkspace 当前虽未渲染 omp 路径，但数据已跨 IPC 边界。design 明确“消毒后返回前端”。  
  Impact: 与项目“不暴露绝对/敏感路径”契约不一致；若日志或未来 UI 直接消费会放大。Grok health 通常不回传绝对路径。  
  Expected fix scope: list/health 响应只保留存在性布尔 + 相对提示字面量，或完全去掉 path 字段。

- [ ] REV-002 `src-tauri/src/scanner/oh_my_pi.rs:40-62` 与 `src-tauri/src/omp_provider.rs:50-62`（OMP_HOME / PI_CODING_AGENT_DIR 根目录计算不一致）  
  Evidence: scanner 对 PI_CODING_AGENT_DIR 做 `.join("sessions")`；provider 直接用原值作为 agent dir；OMP_HOME 也处理不统一。AGENTS.md 描述与 design 也存在细微差异。  
  Impact: 用户设 env 覆盖后，扫描能看到 session 但 set_role_model 可能写错文件，导致“切换不生效”或健康报告指向错误路径。  
  Expected fix scope: 抽取统一 `default_agent_dir()` + `sessions_root()`，两处共用；加测试断言两种 env 下的路径关系。

- [ ] REV-003 `src-tauri/src/omp_provider.rs:155-176`（set_role_model 全量 serde_yaml roundtrip 写盘）  
  Evidence: 先备份，再 `from_str` → insert → `to_string` → write；flatten 只保未知键，注释/空行/原序无保证。测试只验证 theme 键保留。  
  Impact: 用户手写 config.yml 的注释在首次切换后永久丢失；符合 design 识别的 Top 风险，但缓解不充分。  
  Expected fix scope: 至少在返回 message 里明确提示“会重写文件并丢注释”；或改最小文本 patch（仅替换 modelRoles 段）。

- [ ] REV-004 `src/components/ProvidersWorkspace.tsx:287-291`（角色硬编码） + `useOmpProviders.ts`  
  Evidence: `<select>` 只提供 default/smol/plan；即使 health.currentRoles 有其它 role 也无法选择；set 调用不动态发现。  
  Impact: 偏离“受控切换”完整性，用户自定义 role 需外部编辑。  
  Expected fix scope: 让 role 变成自由输入（或从当前 health roles 动态选项），与 model ref 输入同级。

- [ ] REV-005 `AGENTS.md:20`（scanner 规则仍写“first line type=session header with id/cwd”）  
  Evidence: 实现与 fixture 明确支持 title 先行 + 循环扫描任意位置的 type=session；design 也说明“常见首行是 title”。  
  Impact: 后续按 AGENTS 写 scanner 会引入错误假设。  
  Expected fix scope: 改成“jsonl 内 type=session header（可先出现 title 行）+ cwd from header”。

### nit

- [ ] REV-006 `src/components/ProvidersWorkspace.tsx:268-312` omp section 大量 inline style + 硬编码中文，与 Grok 卡片区风格不一致（窄实现可接受，后续抽 class）。
- [ ] REV-007 `src/types.ts:308` OmpConfigHealth 缺少 modelsPath（Rust 序列化总会带），前端契约不完整。
- [ ] REV-008 `src-tauri/src/scanner/oh_my_pi.rs:146` MAX_LINES=4000 硬编码（虽有注释说明用途），建议提取常量。
- [ ] REV-009 scanner.rs 测试中缺少 omp 专用的 `command_spec_for_oh_my_pi_uses_omp_r`（cli_contract 循环已覆盖，但与 grok/opencode 风格不完全对齐）。

### suggestion

- 在 omp_provider health 里暴露实际解析到的 agent_dir，方便诊断 env 覆盖问题。
- Providers omp section 可默认折叠或“检测到 omp 安装/有 session 时才显示”。
- 考虑为 set_role_model 加文件锁或 temp+rename+fsync 提高写安全。

### learning

- cli_contract.rs 的 all_cli_types + 每种必备 scanner/command_spec/whitelist 断言是极好的防漏模式，后续加 CLI 应保持。
- delete_target 直接产出 File 即自动获得删除/health/preflight 能力，无需任何 omp 特殊分支，复用极干净。
- 真实 omp JSONL 布局灵活（title / session / nested message 数组），scanner 必须松散 + 预算扫描。

### praise

- 严格按 design “窄”执行：不碰 Grok 数据流、不加 preferences key、不暴露原始路径、不复制 grok_provider 模块。
- 安全边界清晰：security.rs 显式只接受 `-r/--resume + id`；provider 解析永远 strip apiKey。
- 测试充分且有契约测试守住（129 passed，无回归）；fixture 覆盖 title 先行、message 提取、备份写回、坏输入拒绝。
- 文档同步（AGENTS、user doc、architecture）及时。

## 5. Test And QA Focus

- QA 必须重点复核：
  1. 真实 omp 环境 smoke（必做）：安装 omp，产生含 title 先行 + branch 子目录 + 多轮消息的真实 session，验证列表出现、summary 合理、project_dir 来自 header、launch 实际执行 `cd && omp -r <id>`。
  2. env 覆盖：设置 OMP_HOME / PI_CODING_AGENT_DIR，确认 scan + list/health/set 指向一致位置。
  3. provider 写效果：set 后新 omp session 实际使用该模型；.bak- 文件生成，原键保留；连续 set 不相互破坏。
  4. 边界：无目录时仅 oh-my-pi scan_error；畸形 jsonl 被跳过；非法 role/model 被拒；删除只删目标 jsonl。
  5. 回归：其它 5 个 CLI 扫描/launch/delete/偏好完全不受影响；Grok providers 操作无变化。
- 建议加强的测试：
  - omp 专用 command_spec program/args 断言（可加在 scanner.rs）
  - env 路径关系单元测试
  - set 后真实 omp 行为验证（需集成）
- 不能靠 review 完全确认的点：
  - 真实 omp 对 `-r <full-id>` 的前缀匹配与 TUI 恢复行为
  - 用户自定义 models.yml 各种形态（mapping vs sequence）
  - 极深目录或超大 jsonl 的性能（MAX_LINES 截断）
  - 写 config.yml 后 omp 实际读取 modelRoles 的时机

## 6. Residual Risk

- 独立 OCR 环节不可用 → 行级扫描覆盖度低于理想（但 subagent + 本地核验已充分）
- YAML 注释丢失（用户可感知）；设计已提前识别
- env 根目录不一致（若不修，切换可能静默无效）
- 真实终端 resume 行为、models.yml 未来演进
- 基线 dirty 中夹带的 attention/AGENTS 全局规则变更（与 feature 无关）
- 无自动化端到端（依赖真实 omp + 可丢弃 session 的手工 smoke）

## 7. Verdict

- Status: **passed**
- Reviewer field: `subagent`（环节 A 独立 Task agent 完成并经本地核验；OCR 不可用但不阻塞 gate）
- Reason: 无 blocking；important 均有明确证据与影响边界，已在报告中列出；实现忠实于 approved design + checklist；测试/构建全绿；上一轮 gate 阻塞（reviewer lane 未完成）已解决。
- Next: 按来源表通过后去向。本次为 feature + goal 模式，建议进入 QA / 验收阶段（触发相应 goal 流程或手工 smoke 真实 omp 环境后收尾）。有 important 建议在 QA 前或 residual risk 中明确记录用户接受情况；修复后可重跑 cs-code-review 作为 round 3 确认（非必须）。

本轮审查已完整闭合 gate 要求。实现质量符合项目窄变更、KISS、可维护原则。

---
（报告由主 agent 合并独立 subagent 审查 + 本地事实核验生成；所有 finding 均经源码/测试/设计交叉确认）
