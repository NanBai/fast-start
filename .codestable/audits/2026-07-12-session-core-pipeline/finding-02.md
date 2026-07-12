---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "bug-02"
nature: bug
severity: P1
confidence: high
suggested_action: cs-issue
status: resolved
---

# Finding 02：Codex 单行 JSON 损坏拖垮整 CLI 扫描

## 速答

Codex jsonl 任意一行解析失败会通过 `?` 向上传播，导致整个 Codex CLI 扫描失败、0 条 session；Claude/Grok 策略更软。

## 关键证据

- `src-tauri/src/scanner/codex.rs:102-103` — `serde_json::from_str(line).map_err(...)?`  
- `collect_jsonl_files` 对 `parse_codex_file` 使用 `?`，单文件失败即整树失败  
- 对比：`claude_code.rs` 行解析 `Err(_) => continue`；`grok_build.rs` summary 坏 JSON → `Ok(None)`

## 影响

用户侧表现为「Codex 扫描失败」、该 CLI 全部 session 不可见/不可启动，即使仅一条损坏日志。

## 修复方向

坏行 skip（或单文件 skip 并记 warning），禁止单点拖垮整 CLI；fixture：中部插坏行仍返回其它 session。

## 建议动作

`cs-issue`。

## 处置

2026-07-12 已在代码中修复（P1 批量）。详见对应 scanner/launcher/ARCHITECTURE 改动。
