---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "security-02"
nature: security
severity: P2
confidence: high
suggested_action: cs-refactor
status: resolved
---

# Finding 06：validate_command_spec 只校验 args.last()

## 速答

session id 白名单只作用于最后一个参数，依赖「id 永远在 last」的隐式约定，纵深不足。

## 关键证据

- `src-tauri/src/security.rs:45-48` — 仅 `spec.args.last()` 做 `validate_session_id`  
- `AGENTS.md` Launch Safety 要求对来自 session 的 id 做字符集校验  
- 当前 command_spec 形状固定，短期风险低；未来 args 变形时易漏

## 影响

防御面偏窄；非当前可利用高危，但是安全契约与实现不完全对齐。

## 修复方向

按已知 argv 形状校验（`[resume, id]` / `[--resume, id]`），拒绝未知 flag；补单测。

## 建议动作

`cs-refactor`。

## 处置

2026-07-12 P2 批量已修。
