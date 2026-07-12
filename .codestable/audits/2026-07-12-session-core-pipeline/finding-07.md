---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "performance-01"
nature: performance
severity: P1
confidence: high
suggested_action: cs-refactor
status: resolved
---

# Finding 07：Codex/Claude 全量 read_to_string 大 jsonl

## 速答

每个 session 文件整文件读入内存；summary 靠后时几乎读完整文件。本机样本约 588 个 codex jsonl。

## 关键证据

- `src-tauri/src/scanner/codex.rs:92` — `fs::read_to_string(path)?`  
- `src-tauri/src/scanner/claude_code.rs` 同样整文件读  
- 刷新路径 `state.scan_all` 并行扫全部 CLI，内存与 IO 叠加

## 影响

session 多、单文件大时刷新卡顿、内存尖峰。

## 修复方向

流式按行读；meta/cwd 早停；summary 行数上限；与 finding-02 一并做失败隔离。

## 建议动作

`cs-refactor`。

## 处置

2026-07-12 已在代码中修复（P1 批量）。详见对应 scanner/launcher/ARCHITECTURE 改动。
