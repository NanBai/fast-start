---
doc_type: roadmap-goal-plan
roadmap: session-launcher-next-wave
created: 2026-07-13
---

# Goal Plan: session-launcher-next-wave

## 输入

| 角色 | 路径 |
|---|---|
| Roadmap | `.codestable/roadmap/session-launcher-next-wave/session-launcher-next-wave-roadmap.md` |
| Items | `.codestable/roadmap/session-launcher-next-wave/session-launcher-next-wave-items.yaml` |
| Roadmap review | `session-launcher-next-wave-roadmap-review.md` (passed) |
| Protocols | `goal-protocol*.md` |
| State | `goal-state.yaml` |

## 用户确认

- Roadmap：用户 `approved` → `status: active`
- 全部 9 份 feature design：用户 `approved` → 均已 `status: approved`
- Design-review：均 `passed`

## Feature 执行顺序

| # | slug | 交付物一句话 | 性质 |
|---|---|---|---|
| 1 | scan-cache-and-metrics | 扫描缓存秒开 + 立即 refresh + 耗时指标 | functional |
| 2 | session-mixed-project-view | 按项目跨 CLI 列表视图 | functional |
| 3 | session-favorites-and-empty-states | session 收藏 + 空/错态 | functional |
| 4 | grok-models-and-preview | 拉模型/连通/config 预览 | functional |
| 5 | grok-default-profile-policy | ensure_default 不强制 active | functional |
| 6 | port-power-ops | loopback 打开浏览器 + 端口规则 | functional |
| 7 | launch-history-and-preview | 最近启动 + 命令预览 | functional |
| 8 | release-automation | release 脚本 dry-run | mixed |
| 9 | wave-harden-and-docs | 文档/回归收口 | non-functional |

依赖：5 排程建议在 4 后；9 依赖 1–8 完成。无循环。

## 核心验收路径（roadmap 级）

1. 有 snapshot 冷启动：`scan_sessions` fromCache=true → 前端 refresh → 列表正确；缓存窗 delete 失败、full scan 后 delete 成功
2. 按项目视图可见多 CLI；session 收藏重启保留
3. Grok 用户触发拉模型/预览；无 API 上游 Default 不 active
4. Port loopback 打开；规则过滤；terminate 仍 all-or-nothing
5. 启动后最近历史可再启
6. release 脚本 dry-run 通过
7. 文档与 ARCHITECTURE 与代码一致

## 关键假设

- 缓存 + 立即 refresh 一致性模型用户可接受
- Port 不重写 terminate
- Grok 出站仅用户触发

## Top 3 风险

1. 缓存与 delete_target — 契约已锁失败语义
2. Grok 出站 — 硬约束在 design
3. 多 feature 并行改同一文件冲突 — 按顺序串行 goal loop

## 必跑 / 聚合验证命令

| 命令 | 用途 |
|---|---|
| `cd src-tauri && cargo test --lib` | 每 feature 后端后 + 最终聚合 |
| `pnpm build` | 每 feature 前端后 + 最终聚合 |
| `pnpm tauri build` | release-automation / 收口可选 |
| dry-run release 脚本 | release-automation |

预检：每 feature 开始前若命令已红，先归因既有 vs 本次。

## 策略摘要

- **TDD**：行为代码 RED→GREEN→VERIFY；UI/文档可 TDD exception
- **DoD**：design approved + steps done + review/QA/accept passed
- **Gate**：goal-protocol-gates.md；缺脚本不假装 passed
- **Provider**：archguard/meta-cc 不可用记录 fallback，不自动阻塞
- **验证工具**：禁止 shim 伪造；只能装真实依赖
- **最终审计**：`goal-protocol-audit.md` + 若存在则 `codestable-goal-consistency-gate.py --roadmap .codestable/roadmap/session-launcher-next-wave`

## 完成标记

- 每 feature accept：`CS_ROADMAP_GOAL_FEATURE_DONE`
- 全部 + 审计通过：`CS_ROADMAP_GOAL_COMPLETE`
- 阻塞：`CS_ROADMAP_GOAL_HANDOFF`
