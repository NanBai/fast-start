---
doc_type: feature-acceptance
feature: 2026-06-18-cursor-cli-support
status: accepted
summary: cursor CLI 支持验收通过——扫描真实 chat、Workspace Path 恢复 cwd、cd + resume 端到端可用
tags: [cursor, scanner, cli-integration]
---

# cursor CLI 支持验收报告

> 阶段：阶段 3（验收闭环）
> 验收日期：2026-06-18
> 关联方案 doc：`.codestable/features/2026-06-18-cursor-cli-support/2026-06-18-cursor-cli-support-design.md`

## 0. 验收阶段补修记录

- [x] **有补修**：design frontmatter `summary` 过时——还写"不 cd，靠 resume 自带上下文"，实际方案是 cd + resume（impl 阶段实测推翻"不 cd"）。已改 summary 为"从 store.db 提取 Workspace Path 恢复 cwd，cd 到 workspace 并 resume"。
- [x] **历史状态规范化**：checklist checks 接手时为 `pending`，逐项核对后规范化为 19 条全 `passed`（场景核对见第 3 节，挂载点见第 2 节，均通过）。

> 注：本 feature 的方案在 implement 阶段经历三次实质修正（不 cd → 必须 cd + agent-transcripts → Workspace Path），每次修正都已回填 design 第 1/2.1 节关键决策段。验收确认 design 当前状态与实现一致。

## 1. 接口契约核对

对照 design 2.1 名词层：

- [x] CursorScanner 实现 SessionScanner（`scanner/cursor.rs`）：cli_type 返回 Cursor，scan_sessions 遍历 chats 读 meta.json ✓
- [x] Session 字段：project_name=meta.title，project_dir=Workspace Path 提取的真实 cwd，session_id=chat-uuid ✓
- [x] cursor-agent 在 ALLOWED_PROGRAMS 白名单（首 feature 预留），chat-uuid 走 validate_session_id ✓
- [x] CommandSpec 加 cd 字段（三家 v1 都 true），design 已记此为预留字段 ✓

无偏差。

## 2. 行为与决策核对

**需求摘要**（design 1）：
- [x] cursor 分组展示真实 chat 列表 ✓（用户实测）
- [x] 点 cursor chat → cd 到 workspace + resume 恢复会话 ✓（用户实测）

**明确不做反向核对**（grep）：
- [x] 不解析 store.db 对话内容——只 SELECT data 提取 Workspace Path 字符串，不读对话 ✓
- [x] 不用 cursor-agent ls——代码无该调用 ✓（grep 无命中）
- [x] 不反推 workspace hash / 反向解码——靠 Workspace Path，无哈希运算 ✓
- [x] 拿不到 Workspace Path 的 chat 不显示——extract 返回 None 时 continue 跳过 ✓

**关键决策落地**：
- [x] cwd 来源 = Workspace Path（design 关键决策段）：`extract_workspace_path` 从 store.db blobs 提取 ✓
- [x] 三家同模式 cd + resume：cursor cd=true，和 codex/claude 一致 ✓

**挂载点反向核对**：
- [x] 挂载点1 scanners() 注册 CursorScanner（`scanner.rs:40`）→ grep 确认唯一引用 ✓
- [x] 挂载点2 前端 cursor 分组——已删 v2 特判，走通用展示逻辑 ✓
- [x] **反向 grep**：CursorScanner 在代码仅 scanners() 注册一处引用，无清单外挂载点 ✓
- [x] **拔除沙盘**：移除 cursor mod + scanners() 条目，cursor 分组自动走通用逻辑显示空，无残留 ✓

## 3. 验收场景核对

| 场景 | 结果 | 证据 |
|---|---|---|
| 1 cursor 扫描 | passed | 用户人工验证通过 [2026-06-18，验证范围: cursor 分组显示真实 chat 列表] |
| 2 启动恢复 | passed | 用户人工验证通过 [2026-06-18，验证范围: 点 cursor chat 能恢复指定会话] |
| 3 cd 验证 | passed | 用户实测（cursor 也 cd，resume 成功）；design 已从"不 cd"修正为"cd" |
| 4 无 title 跳过 | passed | 代码 `meta.title.filter` 跳过（类型保证） |
| 5 cursor 未装 | passed | `chats` 不存在返回 ScanError::NotFound，不影响其他 CLI |
| 6 手动刷新 | passed | 走既有 refresh_sessions ✓ |
| 7 三家共存 | passed | 用户实测三家分组都有数据 |

补充证据：cargo test 5 测试全过（含 `extract_workspace_path_strips_json_escape_suffix` 断言 Workspace Path 提取正确）；tsc --noEmit 干净。

## 4. 术语一致性

design 未定义禁用词列表，跳过反向 grep。术语 CursorScanner / Workspace Path 在代码与 design 一致。

## 5. 架构归并

对照 design 第 4 节，已实际写入 `ARCHITECTURE.md`：
- [x] 原"v1/v2 边界"节改为"CLI 覆盖"节，cursor 从 v2 升 v1，记录三家 session 存储/cwd 来源/resume 命令对照（含 cursor 必须 cd 的约束）✓
- [x] 核心模块注释 CursorScanner（首 feature 已预留）✓

判据满足：没读 design 的人看 architecture 能知道 cursor 现在可用、机制是 Workspace Path、resume 要 cd。

## 6. requirement 回写

design frontmatter 无 `requirement` 字段，cursor 支持是首 feature（cli-session-launcher）愿景里"三个 CLI"的落地。首 feature 验收时未建独立 req，本次保持跳过（项目愿景在 ARCHITECTURE.md 体现）。

**结论**：无 requirement 回写。

## 7. roadmap 回写

design frontmatter 无 `roadmap`/`roadmap_item` 字段，非 roadmap 起头。

**结论**：跳过。

## 8. attention.md 候选盘点

本 feature 暴露一个"下个 feature 还会撞"的工作流经验：
- **候选1**：逆向猜测外部应用（cursor/其他 CLI）的本地存储格式前，先搜（web/文档/github）——cursor 的 `Workspace Path:` 注入 system prompt 这个机制，是自己逆向碰壁多次后才搜到的。建议加 attention.md 一行"逆向外部数据格式前先搜"。

本节只登记，落不落由退出后环节定。

## 9. 遗留

- **Workspace Path 覆盖率 ~87%**：极少数 chat（empty-window 或极短 chat）无 Workspace Path，不显示。属可接受边界。
- **CommandSpec.cd 字段 v1 未使用**：三家都 cd=true，字段是为未来"不 cd 的 CLI"预留。design 2.5 已记，建议未来有需要时再启用，或走 cs-refactor 移除。
- **顺手发现**：无。
