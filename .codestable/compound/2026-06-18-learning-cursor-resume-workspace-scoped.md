---
doc_type: learning
track: pitfall
slug: cursor-resume-workspace-scoped
title: cursor-agent 的 chatId 是 workspace 范围的——resume 必须先 cd 到正确目录，否则静默失败
component: scanner
tags: [cursor, cursor-agent, resume, workspace, cwd, cli-integration]
severity: high
created: 2026-06-18
source: feature 2026-06-18-cursor-cli-support
---

# cursor-agent resume 是 workspace 范围的，必须 cd

给 Session Launcher 加 cursor 支持时，以为 `cursor-agent --resume <chatId>` 像 codex/claude 那样"id 全局唯一、一条命令恢复"。实测完全不是——cursor 的 chatId 绑定 workspace，**必须在正确 cwd 下 resume 才生效，否则静默失败**（命令不报错，但恢复的不是目标 chat）。记录机制和找 cwd 的曲折过程。

## 现象

app 里点 cursor chat 启动，终端跑了 `cursor-agent --resume <chatId>`，但**会话没恢复**——工作目录不对，cursor 起来但不是那个 chat。

## 根因：cursor 的 resume 机制和 codex/claude 根本不同

| CLI | resume 命令 | id 范围 | 需要 cd？ |
|---|---|---|---|
| codex | `codex resume <id>` | 全局唯一 | ❌ 不需要 |
| claude | `claude --resume <id>` | 全局唯一 | ❌ 不需要 |
| **cursor** | `cursor-agent --resume <id>` | **workspace 范围** | ✅ **必须 cd 到 chat 所属目录** |

证据（实测）：
- `cursor-agent resume`（子命令形式）是"resume **the latest** chat session"，**不接受 chatId**——只恢复当前 workspace 的最近一个 chat
- `cursor-agent --resume <chatId>` 能精确恢复指定 chat，**但只在正确的 workspace/cwd 下**：在 `/tmp` 跑 ybb-interview 的 chatId → 失败（恢复不出来）
- `--workspace <path>` 参数默认就是当前工作目录

所以 cursor 没"一条命令恢复"这回事，必须 `cd <workspace> && cursor-agent --resume <chatId>`。

## 找 cwd 的曲折过程（试过但没用的路）

知道要 cd 后，难点是**怎么拿到每个 chat 的真实 cwd**——cursor 不像 codex/claude 在 session 文件里直接存 cwd。试了四条路：

1. **`~/.cursor/projects/<编码cwd>/` 目录名反向解码** → ❌ 编码规则 `/`→`-`，但路径里的 `-`（如 `fast-start`、`ai-test`）也被当分隔符，`fast-start`→`fast/start`，**全错**。canonicalize 验证把所有 chat 过滤成 0 条。
2. **agent-transcripts 锚点**（`projects/<编码>/agent-transcripts/<uuid>`）→ ❌ 锚点能建立 uuid→编码目录映射，但编码目录名还是要反解，回到歧义问题。
3. **正向匹配**（从 workspace.json/storage.json/worker.log 收集真实路径，正向编码匹配 projects 目录）→ ⚠️ 能工作但覆盖不全（workspace.json/storage.json 缺近期项目，worker.log 不是每个 project 都有），且要扫多个 cursor/VSCode 状态文件，复杂。
4. **worker.log 里的 workspacePath** → ⚠️ 有真实路径但不是每个 project 都有 worker.log。

## 最终解法：chat 自带的 Workspace Path

用户提醒"先搜一下"，一搜发现：**cursor 把 workspace 真实路径注入每个 chat 的 system prompt**，存在 chat 自己的 `store.db` 的 blobs 表里：

```
Workspace Path: /Users/xb/Desktop/codes/fast-start\nIf editing a git workspace...
```

这是 chat 自带的、无歧义的真实 cwd，不用反解编码、不用跨目录映射。`sqlite3 store.db "SELECT data FROM blobs;"` 然后提取 `Workspace Path:` 即可。实测覆盖 ~87% chat（empty-window 或极短 chat 没有，那些跳过）。

## 一个细节坑：Workspace Path 后面跟 JSON 转义的 \n

提取路径时，第一版用 `split_whitespace().next()` 取路径——结果路径后面紧跟 `\nIf`（**字面两字符 `\` + `n`，JSON 转义的换行，不是真空白**），candidate 变成 `/path/fast-start\nIf`，canonicalize 失败 → 所有 chat 提取不到 → 0 条。

修复：路径取到第一个空白 **或** 字面 `\`（JSON 转义起始）为止。因为 cursor 的内容是 JSON，换行都是字面 `\n`，按空白分词切不开。

## 下次怎么更早发现

1. **任何"某 CLI 的 resume/恢复命令"先搜它的 id 语义**——是全局唯一还是 workspace 范围。codex/claude 是全局，cursor 是 workspace 范围，这个差异决定了要不要 cd。不要假设和已知的 CLI 一致。
2. **要拿外部应用的某个元数据（cwd/path/config），先搜它存在哪**——别自己逆向猜存储格式。cursor 的 Workspace Path 注入 system prompt 这个设计，自己逆向很难想到，搜一下（论坛/issue/直接看 db 内容）就发现了。
3. **提取外部数据时警惕 JSON 转义**——路径/字段值后面可能跟 `\n`、`\"` 等字面转义，按真空白/引号分词会切断。提取时显式在转义字符处截断。

## 相关文档

- feature design: `.codestable/features/2026-06-18-cursor-cli-support/2026-06-18-cursor-cli-support-design.md`
- 代码: `src-tauri/src/scanner/cursor.rs`（extract_workspace_path）
- 架构: `.codestable/architecture/ARCHITECTURE.md` "CLI 覆盖"节
- 相关 learning: [[claude-code-cwd-decode-fallback-bug]]（claude 的 cwd 编码歧义，cursor 同源问题）
