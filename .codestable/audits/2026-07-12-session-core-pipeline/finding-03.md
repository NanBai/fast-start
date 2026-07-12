---
doc_type: audit-finding
audit: 2026-07-12-session-core-pipeline
finding_id: "bug-03"
nature: bug
severity: P1
confidence: high
suggested_action: cs-issue
status: resolved
---

# Finding 03：Cursor 缺时间戳时 last_active_at=now()

## 速答

meta 缺少 `updatedAtMs/createdAtMs` 时用 `SystemTime::now()`，每次扫描都变成「刚刚」，长期绕过最近天数过滤并扰乱排序。

## 关键证据

- `src-tauri/src/scanner/cursor.rs:106-110` — `unwrap_or_else(|| DateTime::<Utc>::from(SystemTime::now()))`  
- 前端默认 `recentDays = "7"`（`App.tsx`），依赖 `lastActiveAt` 过滤  
- Grok 同类回退使用 `file_mtime`，更合理

## 影响

缺时间戳的 chat 每次刷新都置顶、永远落在「最近 7 天」内，快捷键 Enter 可能误选噪声项。

## 修复方向

回退到 `meta.json` 或 chat 目录 mtime；禁止用扫描时刻当业务时间。

## 建议动作

`cs-issue`。

## 处置

2026-07-12 已在代码中修复（P1 批量）。详见对应 scanner/launcher/ARCHITECTURE 改动。
