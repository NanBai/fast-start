---
doc_type: feature-qa
feature: 2026-07-13-scan-cache-and-metrics
status: passed
reviewed: 2026-07-13
---

# scan-cache-and-metrics QA

## 1. Scope

对照 design 验收场景 S1–S4 与 checklist checks；命令证据来自本机实现会话。

## 2. Scenario Matrix

| ID | 场景 | 结果 | 证据 |
|---|---|---|---|
| S1 | 有 snapshot 时 scan_sessions 返回 fromCache=true | pass | `cached_scan_returns_from_disk_without_delete_target` |
| S2 | 缓存窗文件型 CLI delete 失败 | pass | `cache_window_delete_fails_for_file_cli`（含「刷新」） |
| S3 | ops 就绪后 delete 成功并写 snapshot | pass | `full_scan_write_cache_and_delete_succeeds` |
| S4 | 状态栏可见 fromCache / ms | pass | `useSessions.formatScanStatus` 拼接「缓存」「Nms」；`pnpm build` 通过 |
| R1 | 无 watcher / background worker | pass | 代码检索仅 scan_cache 读写 + 前端 refresh |
| R2 | snapshot 不含 delete_target | pass | `roundtrip_strips_delete_target` |

## 3. Commands

```text
cd src-tauri && cargo test --lib
# 76 passed

pnpm build
# tsc && vite build ok
```

## 4. Residual

- 桌面冷启动手测建议：`pnpm tauri dev` 第二次打开应先见缓存再 refresh（本轮未跑 GUI smoke，不阻塞单元契约）

## 5. Verdict

**status: passed**
