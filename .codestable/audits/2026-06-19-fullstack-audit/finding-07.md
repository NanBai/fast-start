---
doc_type: audit-finding
audit: 2026-06-19-fullstack-audit
finding_id: "bug-07"
nature: bug
severity: P2
confidence: medium
suggested_action: cs-issue
status: open
---

# Finding 07：Codex 简介只扫描前 64 行，可能漏掉真实用户输入

## 速答

Codex scanner 为了提取 session 元数据和首条真实用户消息，只读取 jsonl 前 64 行；如果前置系统/环境注入较长，真实用户消息可能落在 64 行之后，导致简介为空。

## 关键证据

- `src-tauri/src/scanner/codex.rs:81` — `for line in content.lines().take(64)` 固定只看前 64 行。
- `src-tauri/src/scanner/codex.rs:118` — 简介目标是第一条“真实”用户消息。
- `src-tauri/src/scanner/codex.rs:121` — 只有在前 64 行内遇到 `response_item` 才会尝试提取 summary。
- `src-tauri/src/scanner/codex.rs:128` — 若 id/cwd 拿到了但 summary 没拿到，仍返回 session，只是 `summary` 为 `None`。
- `src/App.tsx:329` — 前端 summary 为空时会回退。
- `src/App.tsx:334` — 回退展示 `projectName`，用户看到的就不是会话简介。

## 影响

用户会看到 Codex 会话简介缺失或大量回退到项目名，和 Cursor / Claude 的“每条显示会话简介”体验不一致。触发概率取决于 Codex jsonl 开头注入内容长度。

## 修复方向

改为扫描到同时拿到 `session_meta` 和首条真实用户消息再停止，并设置最大字节数或最大行数上限；补一个真实用户消息位于第 65 行之后的单元测试。

## 建议动作

`cs-issue`，因为这是可见行为缺陷，修复范围较小且需要新增边界测试。
