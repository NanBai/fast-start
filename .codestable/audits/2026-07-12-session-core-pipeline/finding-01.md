---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "bug-01"
nature: bug
severity: P1
confidence: high
suggested_action: cs-issue
status: resolved
---

# Finding 01：Cursor Workspace Path 遇空格截断

## 速答

`extract_workspace_path_from_text` 在第一个空白处截断路径，macOS 上含空格的工程路径会漏扫或绑到错误前缀目录，导致 resume 失败。

## 关键证据

- `src-tauri/src/scanner/cursor.rs:169-173` — `take_while(|c| !c.is_whitespace() ...)` 明确在空白处停止  
- 注释写的是为了截断 JSON 字面 `\n`，但同时也截断了路径中的真实空格  
- Cursor resume 是 workspace 范围（compound `cursor-resume-workspace-scoped`）：cwd 错则静默失败

## 影响

- 路径如 `/Users/x/My Project` → candidate 变成 `/Users/x/My`  
- `canonicalize` 失败 → 整条 chat 被跳过  
- 若前缀恰好是已存在目录 → **错误 cwd 启动**

## 修复方向

按 JSON 边界（`\n` / `"`）截断，或使用更宽松的路径匹配；加含空格路径的 fixture。

## 建议动作

`cs-issue`，因为这是可复现的生产正确性缺陷。

## 处置

2026-07-12 已在代码中修复（P1 批量）。详见对应 scanner/launcher/ARCHITECTURE 改动。
