---
doc_type: feature-design
feature: 2026-07-13-add-oh-my-pi-support
status: approved
summary: 添加 Oh My Pi (omp) CLI 的本地 session 扫描、resume 启动、删除支持；同时在切换大模型供应商区域新增对 omp 供应商/模型切换的基础支持（与现有 Grok 工具页并存或扩展）。
tags: [scanner, cli-support, omp, oh-my-pi, providers, models]
---

# add-oh-my-pi-support 设计文档

## 0. 术语约定

| 术语 | 定义 | 防冲突结论 |
|---|---|---|
| Oh My Pi / omp / `omp` | 终端 AI 编码代理 CLI/TUI，binary 名 `omp`，会话存 `~/.omp/agent/sessions/` JSONL，模型配置在 `~/.omp/agent/models.yml` + `config.yml` | 新 CLI 类型；不与 grok-build 混淆 |
| session header | JSONL 第一行 `{"type":"session", "id": "...", "cwd": "...", ...}` | 解析入口 |
| 供应商切换 (for omp) | 通过编辑 `models.yml` 定义上游、通过 `config.yml` 的 `modelRoles` 设置角色默认模型 | 与 Grok 的 profiles.json + config.toml 机制不同，保持隔离 |
| providerKey (omp) | 仅用于 omp 布局偏好时稳定键；不复用 grok 的 | 局部定义 |
| resume 形状 | `omp -r <session-id>`（或 `--resume`）+ cd 到原 cwd | 类似 grok/cursor |

### 术语守护
- `omp` 作为程序名保留小写；用户可见 "Oh My Pi"
- 不把 omp session 路径暴露给前端（与现有契约一致）
- Grok 相关命名保持不变；omp 走独立路径

## 1. 决策与约束

### 需求摘要（来自用户）
快速启动场景下：
1. 支持 Oh My Pi（omp）的本地 session 聚合展示、搜索、收藏、启动恢复、删除。
2. 当前“切换大模型供应商”仅 Grok Build 可用，要给 oh-my-pi 也加上切换支持。

### 核心行为
- 扫描：从 `~/.omp/agent/sessions/`（支持 `OMP_HOME` 或 `PI_CODING_AGENT_DIR` 覆盖）发现分组目录下的 `*.jsonl`，解析 header 得 id/cwd/title，解析最近 message 或 mtime 得 last_active，抽取 summary（优先 title 或最后一条 user/assistant 文本）。
- 启动：`cd <project_dir> && omp -r <session_id>`（使用完整 id）。
- 删除：删除对应 session 的 `.jsonl` 文件（类似 Codex/Claude 的文件删除）。
- Provider 切换（omp 侧）：
  - 列出 `models.yml` 中定义的 providers + 其 models（消毒后返回前端）。
  - 支持查看/设置 `config.yml` 中 `modelRoles.default` 等角色默认模型（安全备份 + 最小合并写）。
  - 提供基础 health（文件存在性、是否可解析、当前 active model）。
  - UI：在 Providers 工作区或 Sessions 旁增加 Oh My Pi 供应商卡片/列表（不破坏现有 Grok 卡片流程）。

### 成功标准（可验证）
- 新增 `CliType::OhMyPi`（序列化 "oh-my-pi"），出现在 CLI_ORDER / CLI_LABELS。
- `scan_sessions` / `refresh_sessions` 能返回 omp sessions；无 omp 目录时不报错（与其他 CLI 一致，返回空 + scan_error? 视实现）。
- `command_spec_for_session` 对 OhMyPi 返回 `{program:"omp", args:["-r", id], cd:true}`。
- `validate_command_spec` 通过 "omp" + `-r <id>` 形状。
- 启动预检、health inspect、delete 对 omp 生效（source = 该 jsonl 文件）。
- 前端 Session 列表展示 "Oh My Pi" 标签，可过滤/搜索/收藏/启动。
- 对于 omp provider：
  - 存在 `grok_*` 风格的 Tauri 命令如 `omp_list_providers`、`omp_get_config_health`、`omp_set_role_model`（或统一但隔离实现）。
  - 前端可展示 omp 供应商列表 + 当前角色模型；切换动作写盘成功后状态刷新。
- `cd src-tauri && cargo test --lib` 通过；`pnpm build` 通过。
- 文档（AGENTS.md、architecture、user doc）提及 omp（在对应章节）。

### 明确不做
- 不完整复制 grok_provider 模块的 http fetch/test、大量 profile 编辑 UI、OAuth 官方账号流程给 omp（omp 无此概念，40+ 内置 + yml 自定义）。
- 不改现有 Grok 卡片排序/置顶逻辑或 preferences key 结构。
- 不把 providers tab 立刻重命名为支持多 CLI 的复杂 tab（先窄添加 omp section）。
- 不实现 omp 的 `/model` 运行时切换、fork/branch、compaction 等 TUI 内高级操作（launcher 只负责 resume）。
- 不支持 Windows（当前项目 macOS-first）。
- 不暴露 session 原始 jsonl 路径给前端。
- 不做 omp 的完整 models.yml 可视化编辑器（仅角色默认模型的受控切换 + 列表展示）。
- 不改 scan cache 格式（Session 结构不变）。

### 假设
- omp binary 名固定为 `omp`（用户通过 PATH 可用，install 后即是）。
- resume 使用 `-r <full-id>` 在 cd 后可靠；前缀匹配由 omp 自己处理。
- session 目录默认 `~/.omp/agent/sessions`；group 子目录内文件名为 `<ts>_<id>.jsonl` 或 `<id>.jsonl`（scanner 递归发现所有 *.jsonl 并从 header 取 cwd 兜底）。
- models.yml / config.yml 位于 `~/.omp/agent/` 下；可通过 env 覆盖根目录。
- 用户可能同时用多个 CLI；omp sessions 独立聚合，不共享 grok 的 provider layout。

### Top 3 风险 + 缓解
1. **session JSONL 格式演进或 group 布局变化导致解析不稳**  
   缓解：scanner 只依赖第一行 type=session 的已知关键字段（id,cwd,title,timestamp）；其余 entry 忽略；加宽松解析 + 单元测试用真实样例 fixture；mtime 兜底 last_active。
2. **config.yml / models.yml 写盘破坏用户格式或注释**  
   缓解：用最小 serde + 手动保留策略（类似 grok 隐私保护的“只改目标键”）；先备份到同级 .bak；仅操作 modelRoles.* 段；提供 health 报告漂移。
3. **resume 命令形状或 binary 名与用户实际安装不符**  
   缓解：security 严格校验 `-r` + id；文档写清“确保 `omp --version` 可用”；launch preflight 增加 program_not_found 提示。

### 非显然依赖
- omp 可能未安装：扫描返回 NotFound 错误，前端已有 scanErrors 处理。
- 目录权限或大文件：scanner 用与 codex 类似的 jsonl 流式读 + 预算。
- 现有 delete 安全（session_delete.rs）需支持 File kind 对 omp jsonl。

### 必跑验证命令
- `cd src-tauri && cargo test --lib`
- `pnpm build`
- 手动：安装 omp 后，在有 session 的项目跑 `pnpm tauri dev`，验证扫描到、能 launch（用可丢弃 session）、删除不误删其他文件。
- 基线风险：若当前有红灯，先分清。

### 交付物清单（用户/系统可见）
- Rust: 新 `scanner/oh_my_pi.rs`、models CliType 变体、scanner.rs 注册、security.rs 允许 "omp"、command_spec 匹配、session_delete 可能的小扩展、可能新增轻 omp_provider 模块或命令。
- TS: types.ts 更新 union + labels + order；可能小扩展 Providers* 组件或新增 omp 卡片逻辑。
- 文档：AGENTS.md（scanner 规则）、architecture/ARCHITECTURE.md、docs/user/session-launcher.md、.codestable/attention.md（如需新坑）。
- 测试：scanner 单元测试（fixture）。
- 无新持久化 key（除非 omp provider layout 必要，设计为可选）。

## 2. 现状与变化

### 2.1 名词层

**现状**（关键位置）：
- `src-tauri/src/models.rs:10`：`CliType` 枚举只有 Codex、ClaudeCode、Cursor、GrokBuild、OpenCode；`Session` 结构通用。
- `src-tauri/src/scanner.rs:39`：`scanners()` 硬编码 5 个；`command_spec_for_session` match 5 种 resume 形状。
- `src-tauri/src/security.rs:4`：`ALLOWED_PROGRAMS` + `validate_resume_args` 只有 5 个。
- `src-tauri/src/scanner/*.rs`：每个 CLI 一个 scanner 实现（Grok 用 summary.json + group cwd 解析；Codex/Claude 用 jsonl 流；OpenCode 用 rusqlite）。
- `src/types.ts:1`：`CliType` 字面 union + `CLI_ORDER` + `CLI_LABELS` + `CLI_LABELS`。
- `src-tauri/src/grok_provider/` 完整子系统 + `commands.rs` grok_* 命令 + `preferences.rs` grok_* keys + 前端 `hooks/useGrokProviders.ts`、`components/Providers*`、`lib/grokProviderCards.ts`。
- `AppTool` 只有 providers 对应 "Grok"。
- session delete / source check / preflight 都通过 `cli_type` + `delete_target` 区分。

**变化**：
- 在 `CliType` 增加 `OhMyPi`（#[serde(rename_all="kebab-case")] → "oh-my-pi"）。
- 新文件 `src-tauri/src/scanner/oh_my_pi.rs` 实现 `SessionScanner`，解析 JSONL header + 最近 entry 提取 summary/last_active，生成 `delete_target: File` 指向具体 `.jsonl`。
- `scanners()` 追加 `OhMyPi`；`command_spec_for_session` 追加 case 返回 program:"omp", args:["-r", id]。
- `security.rs`：ALLOWED 追加 "omp"；validate_resume_args 追加匹配 `["-r"|"--resume", id]`。
- `session_delete.rs` 与 `session_source.rs`：File kind 已支持，无需大改（OpenCode 特殊处理不影响）。
- 前端 types.ts：union 追加 "oh-my-pi"，ORDER 追加，LABELS 追加 `"oh-my-pi": "Oh My Pi"`。
- Providers 侧：**窄变更**——不新建完整 grok_provider 复制。新增轻量命令（`omp_list_providers`、`omp_get_config_health`、`omp_set_role_model` 等）在 commands.rs；后端可放 `grok_provider` 旁或新 `omp_provider/` 子模块（小）；前端在 ProvidersWorkspace 增加条件 section 或简单卡片列表（复用现有卡片渲染模式），不改 Grok 现有数据流。
- preferences.rs：若 omp 需要 order/pin 则加 `omp_provider_*` keys（设计时评估是否 MVP 必须；窄情况下先只读 health + set，不持久布局）。

### 2.2 编排层

**现状流程简图**（扫描 + launch）：
```
Tauri command scan/refresh
  → AppState.scan_sessions
    → parallel scanners() → 每 CLI 自己的 root()/parse → Session[]
  → 聚合排序
launch:
  preflight → validate_command_spec → command_spec_for_session → TerminalLauncher.launch
```

**本次变化**（在现有拓扑上插入 omp）：
```
scanners() 追加 OhMyPiScanner
command_spec 追加 OhMyPi case
validate 追加 omp case
delete / source check 自动覆盖（File）
```

**Providers 切换编排**（窄）：
```
Grok 路径不变（grok_provider state + 现有 commands）
新增：
  omp_list_providers() → 读 ~/.omp/agent/models.yml → 解析 providers[] + models[] → 消毒返回
  omp_get_config_health() → 读 config.yml + models.yml → 报告当前 modelRoles + 文件状态 + issues[]
  omp_apply_role_model(role, modelRef) → 备份 → 最小 patch config.yml 的 modelRoles.<role> → 返回新状态
前端：ProvidersWorkspace 根据当前 tool 或内部分段展示 Grok 区 + OhMyPi 区；切换动作调用对应 invoke。
```

推荐流程图：

```mermaid
flowchart TD
    A[scan_sessions] --> B{scanners}
    B --> C[CodexScanner]
    B --> D[Claude...]
    B --> E[GrokBuildScanner]
    B --> F[OpenCodeScanner]
    B --> G[OhMyPiScanner<br/>新]
    G --> H[~/.omp/agent/sessions/**/*.jsonl<br/>parse header + last msg]
    H --> I[Session {cli_type:OhMyPi, delete_target:File}]

    J[launch] --> K[command_spec_for_session]
    K --> L["omp -r <id> + cd"]
    L --> M[validate_program + validate_resume_args<br/>新增 omp 分支]
```

### 2.3 挂载点清单（删即消失判定）

1. `CliType::OhMyPi` 变体 + 所有 match 上的 case（models, scanner, security, command_spec, frontend types/labels/order）——删掉后 omp 不会被扫描/启动/展示。
2. `src-tauri/src/scanner/oh_my_pi.rs` 新文件 + scanner.rs 里的注册 —— 核心发现逻辑。
3. security.rs 的 "omp" + resume 形状 —— 启动 gate。
4. 前端 CLI_LABELS / CLI_ORDER 中 "oh-my-pi" 条目 —— 列表可见性。
5. 新增的 omp_* Tauri 命令 + 其后端实现 + 前端少量调用点（providers 区）—— 供应商切换能力消失。
（3-5 条为健康范围；若 providers 部分做成可选 section，可进一步隔离。）

### 2.4 推进策略

- 编排骨架：先改 models + scanner.rs + security + command_spec（窄、编译可测）。
- 计算节点：实现 oh_my_pi.rs 解析（参考 codex jsonl + grok cwd 兜底）。
- 持久化/删除：复用现有 File 路径，测试 delete_target 指向正确 jsonl。
- Provider 切换：后端轻命令先行（只读 health + 受控写），前端展示先行。
- 测试：每个 scanner 已有模式，oh_my_pi 加 with_root fixture 测试。
- 文档同步：在 design 批准后或 impl 末尾更新 AGENTS.md scanner 规则、architecture。
- 验证顺序：cargo test → pnpm build → 手工 smoke（有 omp 环境）。

### 2.5 结构健康度与微重构

**要改的文件评估**：
- models.rs：仅加 enum 变体，极小，健康。
- scanner.rs：match 追加 case，注册 vec 追加 → 文件已负责分发，接受追加（与之前 grok/opencode 做法一致）。
- security.rs：白名单 + validate 函数已按 program 分支，新增 omp 分支符合现有模式。
- 新 scanner/oh_my_pi.rs：新文件，职责单一，良好。
- commands.rs：若加 omp_* 命令，会在 grok 命令附近；考虑是否抽 provider 抽象？本次窄，不抽（避免 refactor）。
- types.ts / ProvidersWorkspace.tsx：前端类型追加 + UI 条件渲染 section。ProvidersWorkspace 当前较专为 Grok，追加 section 可能让它稍胖；若膨胀明显，impl 时可小拆但不阻塞。
- preferences.rs：若加 omp layout keys，类似 grok 两条，保持并列。

**目录评估**：
- scanner/ 下已有 5 个 CLI scanner，新增 1 个符合模式（不摊平）。
- grok_provider/ 是 Grok 专属，不把 omp 塞进去；若 provider 逻辑增多，未来可考虑 `provider/` 目录，但本次不做（超出窄范围）。
- 结论：**不做微重构**。现有结构能干净接纳（scanner 平行文件 + 中央注册），providers 变更控制在“追加 section + 新命令”不改变 Grok 路径。

**建议沉淀的 convention**（如 impl 后观察稳定）：无强制。本次保持与历史添加 CLI（grok-build, opencode）的节奏一致。

## 3. 验收契约

### 3.1 场景覆盖（输入/触发 → 期望可观察结果）

| ID | 场景 | 输入/触发 | 期望结果 | 证据类型 |
|----|------|-----------|----------|----------|
| S1 | 扫描到 omp session | 有 `~/.omp/agent/sessions/<g>/<s>.jsonl`（含合法 header + messages） | sessions 列表出现 cliType:"oh-my-pi"，project_dir 正确，summary 非空，last_active 合理 | cargo test + pnpm tauri dev 人工看列表 |
| S2 | 无 omp 目录 | 目录不存在 | scan 成功返回其他 sessions + 该 cli 的 scan_error 或空 | 同上 |
| S3 | resume 形状 | session.cliType=oh-my-pi | command_spec = {cwd, program:"omp", args:["-r", id], cd:true}；validate 通过 | 单元测试 + preflight preview |
| S4 | launch 预检 | 合法 omp session | preflight ok，无 block | 手工 launch 可丢弃 session |
| S5 | 删除 | 右键 omp session → 确认 | 该 jsonl 被删；列表刷新后消失；不删其他文件 | delete smoke（仅 disposable） |
| S6 | omp provider 列表 | 调用 omp 相关 command | 返回解析后的 providers 数组（含 name/base/models 等消毒字段） | 后端测试 + 前端展示 |
| S7 | omp 设置角色模型 | 选 role + modelRef → apply | config.yml 对应 modelRoles.<role> 更新；有备份；health 反映新值 | 集成测试 + 手工改后读回 |
| S8 | 类型/构建 | 改动后 | cargo test --lib 绿；pnpm build 绿 | CI 同命令 |
| S9 | 前端标签 | 列表渲染 | "Oh My Pi" 正确显示；CLI_ORDER 含它 | 视觉 + 快照或人工 |

### Acceptance Coverage Matrix
- 正常路径：S1,S3,S4,S6,S7
- 边界：S2（缺目录）、空 title、超长 summary（复用 clean_summary）
- 错误：非法 id（security）、目录外删除（已有 delete safety）、写盘失败（health 报 issue）
- 回归：其他 5 个 CLI 扫描/launch/delete 不受影响（全量 scanner 测试）

### DoD Contract
- design + checklist 批准后 impl。
- 所有 S* 有证据。
- 无临时 TODO/debug 输出。
- AGENTS.md scanner 规则更新提及 omp（~ "omp jsonl under ~/.omp/agent/sessions；cwd from header；resume -r <id>"）。
- 卸载路径清晰（见 2.3）。

## 4. 超出范围的观察 / 后续建议
- 完整 omp providers 管理（models.yml 可视编辑、多角色批量、从 env 发现 Ollama 等）可作为独立 follow-up feature。
- 如果 omp 未来提供类似 grok 的 "switch profiles" 官方机制，可再扩展。
- 考虑未来把 CLI resume 形状抽象成配置表，但本次不做（风险/范围）。
- 前端 Providers 工作区若持续膨胀，建议后续小 refactor 抽 "ProviderSection" 组件。

## 5. 基线与执行
- 验证基线：release-readiness.md、pnpm build、cargo test --lib。
- 本 feature 涉及 terminal-launch-safety 边界（新增 program），实现时触发对应 skill 意识。
- 清洁度：禁止在生产代码加 println!/console.log 调试 session 内容；测试 fixture 允许。

---
（本轮为 draft，等待 design-review 与用户整体确认后改为 approved）
