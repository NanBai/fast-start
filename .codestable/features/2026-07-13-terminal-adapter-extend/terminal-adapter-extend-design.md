---
doc_type: feature-design
feature: 2026-07-13-terminal-adapter-extend
requirement:
roadmap: session-launcher-power-extend
roadmap_item: terminal-adapter-extend
status: approved
summary: 至少一个新 macOS TerminalLauncher；wrapper+校验；偏好可选
tags: [terminal, launcher]
---

# terminal-adapter-extend 设计文档

## 1. 决策与约束

**目标**：偏好中可选新终端并成功 launch。  
**不做**：Windows 终端；AppleScript 直注业务命令。

**决策**：

1. **本机探测（2026-07-13）**：`/Applications` 无 `Warp.app` / `WezTerm.app`；PATH 无 `warp`/`wezterm`。Ghostty/iTerm 已有。  
2. **选型：WezTerm**（开源、CLI `wezterm start` / `cli` 文档清晰；本机未装时仍实现适配器，`is_available=false`，真实 smoke 可 drop 或装后补）  
3. 扩展 `TerminalType::WezTerm`（serde `wezterm`）；trait 含 `supports_tab` + `launch(&CommandSpec, LaunchMode)`  
4. 仅注入已校验 wrapper 路径  
5. 若实现期 WezTerm CLI/API 不可用导致无法安全 launch → **允许 drop** 并从 harden depends 移除  

### WezTerm tab/window 行为（写死）

| 模式 | 行为 |
|------|------|
| `supports_tab` | **true**（优先 `wezterm cli spawn --cwd ...` 在已有 domain；失败回退窗口） |
| `LaunchMode::NewTab` | 若 `wezterm cli list` 可连上 mux → `cli spawn` 执行 wrapper；否则回退 NewWindow |
| `LaunchMode::NewWindow` | `wezterm start --cwd <cwd> -- <wrapper>` 或等价：打开新窗口只跑 wrapper 路径（argv 不经 shell 拼接业务 resume） |
| 未安装 | `is_available()=false`（探测 `/Applications/WezTerm.app` 或 `which wezterm`） |

禁止：AppleScript/shell 把 `program + args` 业务串直注。

## 2. 名词与编排

**现状**：System / ITerm2 / Ghostty。  
**变化**：+ WezTerm；list_available 仅 available 时出现（或列出但禁用——实现选 list 只含 available，与现终端一致）。

```text
list_available → 含 wezterm 仅当 is_available
set preferred → 存枚举
launch → validate → wrapper → WezTermLauncher
```

### 挂载点

1. `TerminalType` + `WezTermLauncher`  
2. Controls 终端选择 labels  
3. `is_available=false` 单测  
4. 可选：本机安装后 smoke  

### 2.5 结构健康度

实现放 `launcher/terminals.rs` 或 `wezterm.rs`，不膨胀 state。

## 3. 验收

- 未装：is_available=false，不崩溃，偏好不误选强制  
- 已装：NewWindow 可 resume；NewTab 失败时回退窗口  
- 无业务命令直注  

**验证**：cargo test；本机若之后安装再 smoke  

## 4. 架构

终端表与 wrapper 安全口径扩展 WezTerm 一行。
