---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "arch-drift-01"
nature: arch-drift
severity: P1
confidence: high
suggested_action: cs-refactor
status: resolved
---

# Finding 12：ARCHITECTURE 仍写三家 CLI，实现已是四家

## 速答

架构文档与安全口径仍描述 codex/claude/cursor 三家；代码已含 `CliType::GrokBuild`、`grok` 白名单、`scanner/grok_build.rs`。

## 关键证据

- `.codestable/architecture/ARCHITECTURE.md` — 「三个 AI CLI」「program 限 codex/claude/cursor-agent」、模块树无 grok  
- `src-tauri/src/models.rs` — `GrokBuild`  
- `src-tauri/src/security.rs:4` — `ALLOWED_PROGRAMS` 含 `grok`  
- `AGENTS.md` 已更新为四 CLI  

无独立 ADR 目录；以 ARCHITECTURE 为现状权威源时构成 **文档滞后于实现**。

## 影响

贡献者按文档会漏 Grok 删除（Directory）、PATH（`~/.grok/bin`）、白名单约束；与 CodeStable「architecture 只记现状」原则冲突。

## 修复方向

刷新 ARCHITECTURE：四 CLI 表、Grok 存储与 cwd 优先级、安全白名单、删除目标表。

## 建议动作

架构文档更新（可走 docs/arch 收尾，不必改业务代码）。

## 处置

2026-07-12 已在代码中修复（P1 批量）。ARCHITECTURE v1.3 同步四 CLI。
