---
doc_type: feature-acceptance
feature: 2026-06-17-cli-session-launcher
status: accepted
summary: 二次验收——首轮通过后针对真实使用反馈做的终端 tab 化 + bug 修复，回填 design 后核对通过
tags: [tauri, react, cli-integration, desktop]
---

# cli-session-launcher 二次验收报告

> 阶段：阶段 3（验收闭环）— 二次
> 验收日期：2026-06-18
> 关联方案 doc：`.codestable/features/2026-06-17-cli-session-launcher/2026-06-17-cli-session-launcher-design.md`
> 首轮验收：`.codestable/audits/2026-06-17-cli-session-launcher-accept/index.md`（pass 32 / warn 3 / fail 0）

## 0. 验收阶段补修记录

首轮验收后，用户在真实终端上试用暴露了一系列问题，本次针对这些做改动并补修：

| 发现 → 偏差 → 已修 → 复验 |
|---|
| **Ghostty 多窗口 + login 误报**：原 `-e <program> <args>` 经 `/usr/bin/login` 套壳，多词命令弹 "failed to launch" 误报，且每次点击开独立窗口、关不掉会"复活" → 引入 wrapper 脚本机制（`GhosttyLauncher` 生成 `$TMPDIR/fast-start-ghostty/run-<pid>.sh`，`-e`/`--command` 只执行单脚本路径）+ 有窗口时改 AppleScript `new tab` 开 tab → **已修 `launcher.rs`，回填 design 2.1，用户实测无误报、开 tab、agent 退出干净关闭** |
| **Ghostty `codex not found`**：wrapper 用 `/bin/sh` 跑，PATH 不含 `~/.local/bin` → wrapper 显式 `export PATH`（补 `~/.local/bin`） → **已修 `launcher.rs`，用户实测 codex 正常启动** |
| **iTerm2 报语法错 + 开新窗口**：app 名误写 `iTerm2`（实际 bundle 名 `iTerm`，不加载字典导致 `create tab` 报 "Expected end of line but found class name"）；且原本开新窗口 → 改 app 名为 `iTerm` + 有窗口时 `create tab with default profile`、冷启动 activate 等默认窗口复用（避免叠加两窗口） → **已修 `launcher.rs`，回填 design 2.1，用户实测开 tab、冷启动单窗口** |
| **Terminal 双窗口（尝试开 tab 失败）**：尝试让 Terminal 开 tab，但 `make new tab` 字典不支持、`do script in <tab>` 无法稳定复用、模拟 ⌘T 需辅助功能权限 → 回退到最简 `do script` 开新窗口；冷启动双窗口为 Terminal AppleScript 硬限制（无法从 AppleScript 侧可靠区分命令窗口与空窗口） → **已修 `launcher.rs`，回填 design 2.1/场景4，用户接受此限制** |
| **claude cwd 解析错误**：`parse_claude_file` 原逻辑 `let mut cwd = fallback_cwd`，fallback 是 `Some` 导致 `if cwd.is_none()` 永不成立，jsonl 真实 cwd 被忽略；fallback 来自目录名 decode 有歧义（`/` 和 `-` 都编成 `-`，`fast-start`→`fast/start`），导致含 `-`/`.` 目录报"工作目录不存在" → 改为 `let cwd = None` + `cwd.or(fallback_cwd)`，优先用 jsonl 真实 cwd → **已修 `scanner/claude_code.rs`，回填 design 调研结论，用户实测 claude session 正常启动** |

**补修边界判定**：以上均为 bug 修复 + 实现细节纠偏（终端启动策略、cwd 解析），未改接口签名、未新增功能模块，属验收阶段可补修范围。所有改动同步回填 design 对应章节。

**历史状态规范化**：checklist checks 接手时为 `pass`/`warn`（首轮遗留非标准状态），逐项核对后规范化为 `passed`（35 条全 passed）；其中首轮 2 条 warn（命令构造走 shell 字符串、并发用 thread::spawn）功能上不阻塞，场景 6 Ghostty warn 因本次实测升 passed。

## 1. 接口契约核对

对照 design 2.1 名词层（已回填 wrapper/终端策略）：

- [x] **Session**（`models.rs`）：六字段 + `#[serde(rename_all="camelCase")]`，与 design 一致 ✓
- [x] **CliType**（`models.rs`）：`kebab-case` 序列化，含 Cursor（v2 灰显） ✓
- [x] **TerminalType**（`models.rs`）：`lowercase` 序列化，System/ITerm2/Ghostty ✓
- [x] **CommandSpec**（`models.rs`）：cwd + program + args 纯数据 ✓
- [x] **SessionScanner trait**（`scanner.rs`）：CodexScanner / ClaudeCodeScanner 实现 ✓
- [x] **TerminalLauncher trait**（`launcher.rs`）：三实现，`launch` 签名一致 ✓
- [x] **wrapper 脚本生成**（`launcher.rs::write_ghostty_wrapper`）：回填后的 design 2.1 描述一致 ✓

无偏差。

## 2. 行为与决策核对

**需求摘要逐项**（design 1）：
- [x] 聚合展示 codex + claude-code session（v1） ✓
- [x] 按时间降序、显示 CLI 类型/目录名/时间 ✓
- [x] 终端可选（Terminal/iTerm2/Ghostty） ✓
- [x] 选中 → 开终端 → cd → resume ✓

**明确不做核对**（grep 确认）：
- [x] 无 session 持久化 / DB ✓
- [x] 无内置终端（无 xterm.js 依赖） ✓
- [x] 无 session 编辑 UI ✓
- [x] 无 fs-watcher / 后台扫描 ✓
- [x] cursor v1 灰显不扫描 ✓

**挂载点反向核对**（grep 已执行）：
- [x] Tauri command 注册（`lib.rs:37` generate_handler） ✓
- [x] Tauri 窗口创建 ✓
- [x] 前端入口（`main.tsx`） ✓
- [x] TerminalLauncher 三实现（System/ITerm2/Ghostty） ✓
- [x] tauri-plugin-store 持久化（`state.rs` preferences.json） ✓

**拔除沙盘**：移除 invoke_handler 注册 + 前端 App + launcher 模块 + store plugin 即 feature 消失，无清单外残留引用。

## 3. 验收场景核对

二次验收聚焦被改动的场景 4/5/6，其余场景首轮已验、本次无改动维持 passed：

| 场景 | 结果 | 证据 |
|---|---|---|
| 4 Terminal.app 启动 | passed | 用户人工验证通过 [2026-06-18，验证范围: Terminal 开新窗口、cd、codex resume；冷启动双窗口为硬限制已接受] |
| 5 iTerm2 启动 | passed | 用户人工验证通过 [2026-06-18，验证范围: iTerm2 已有窗口开新 tab、冷启动单窗口、claude --resume] |
| 6 Ghostty 启动 | passed | 用户人工验证通过 [2026-06-18，验证范围: Ghostty 已有窗口开新 tab、无 login 误报、codex/claude 在对应目录 resume、PATH 修复后 codex 可找到] |
| 1-3, 7-13 | passed | 首轮已验，本次无改动 |

**前端验证**：用户已人工验证三种终端的启动行为（第一等证据）。

## 4. 术语一致性

design 未定义禁用词列表，跳过反向 grep。术语核对：
- TerminalType / CliType / Session / CommandSpec 命名与代码一致 ✓
- 终端类型 `iTerm`（非 iTerm2）已统一（launcher 内 AppleScript 用 `iTerm`，对外枚举/前端仍 `iterm2` 供显示） ✓

## 5. 架构归并

对照 design 4，已实际写入 `architecture/ARCHITECTURE.md`：
- [x] 名词层类型表 ✓（首轮）
- [x] 主流程 mermaid 图 ✓（首轮）
- [x] v1/v2 边界 ✓（首轮）
- [x] **终端拉起策略表（二次新增）**：iTerm2/Ghostty 开 tab、Terminal 开窗口 + Ghostty wrapper 机制 ✓（本次写入）
- [x] 技术栈更新（Ghostty 改为 AppleScript） ✓（本次）

## 6. requirement 回写

design frontmatter 无 `requirement` 字段，且这是项目首个 feature。首轮未建 req，本次保持跳过（项目愿景已在 ARCHITECTURE.md 体现）。如需正式 req，可后续 `cs-req draft`。

**结论**：无 requirement 回写。

## 7. roadmap 回写

design frontmatter 无 `roadmap`/`roadmap_item` 字段，feature 非从 roadmap 起头。

**结论**：非 roadmap 起头，跳过。

## 8. attention.md 候选盘点

本 feature 暴露的"每个 feature 都会撞"的环境/工作流信息：
- 跑 Rust 测试要在 `src-tauri/` 目录（根目录 `cargo test` 找不到 Cargo.toml）
- 项目非 git 仓库，`git status` 报错属正常
- 三个外部终端的 AppleScript 坑（app 名 `iTerm`、Ghostty login 误报、Terminal 无法开 tab）

**已写入 attention.md**（重建时一并补入，因 attention.md 在文件丢失事件中遗失）。

## 9. 遗留

- **Terminal.app 冷启动双窗口**：AppleScript 硬限制，无法可靠解决；命令窗口一定存在，多余空窗口用户手动关。已接受。
- **命令构造走 shell 字符串**（首轮 warn 1）：Terminal/iTerm2 的 osascript 仍间接经 shell，上游校验扎实实际不可利用；建议后续改 `std::Command` 参数化。
- **并发用 `std::thread::spawn`**（首轮 warn 2）：偏离 design 的 tokio 措辞，功能等价；非必须改。
- **cursor v2**：单独开 feature 做可行性验证（sqlite schema + workspace hash 反推 + `cursor-agent ls` TTY 问题）。
