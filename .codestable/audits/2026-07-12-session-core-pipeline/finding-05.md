---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "security-01"
nature: security
severity: P1
confidence: medium
suggested_action: cs-issue
status: resolved
---

# Finding 05：Cursor cwd 取 blobs 中第一个可 canonicalize 的 Workspace Path

## 速答

无 ORDER BY、无 system 优先；用户/工具消息若先出现 `Workspace Path: /existing/dir` 会被采纳，导致在错误工作目录 resume 白名单程序。

## 关键证据

- `src-tauri/src/scanner/cursor.rs:137-154` — `SELECT data FROM blobs` 后第一个提取成功即胜出  
- 与 finding-01 叠加时，截断后的前缀路径若存在目录更容易误绑定  
- 删除仍按 chat 目录（安全），但**启动语义被污染**

## 影响

本地桌面威胁模型下是「错误 cwd 执行 CLI」；不是 RCE，但会破坏 resume 正确性与用户对「启动」的信任。

## 修复方向

优先 system/workspace 标记 blob；多命中时选最长合法绝对路径；fixture 覆盖 user 文本抢先。

## 建议动作

`cs-issue`（可与 finding-01 同 issue）。

## 处置

2026-07-12 已在代码中修复（P1 批量）。详见对应 scanner/launcher/ARCHITECTURE 改动。
