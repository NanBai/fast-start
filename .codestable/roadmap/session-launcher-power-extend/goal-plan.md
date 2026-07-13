---
doc_type: roadmap-goal-plan
roadmap: session-launcher-power-extend
created: 2026-07-13
---

# Goal Plan: session-launcher-power-extend

## 输入

| 角色 | 路径 |
|---|---|
| Roadmap | `.codestable/roadmap/session-launcher-power-extend/session-launcher-power-extend-roadmap.md` |
| Items | `.codestable/roadmap/session-launcher-power-extend/session-launcher-power-extend-items.yaml` |
| Roadmap review | `session-launcher-power-extend-roadmap-review.md` (passed) |
| Protocols | `goal-protocol*.md` |
| State | `goal-state.yaml` |
| Feature specs | `goal-features/*.md` |

## 用户确认

- Roadmap：用户 `approved` → `status: active`
- 全部 11 份 feature design：用户 `approved` → 均已 `status: approved`
- Design-review：均 `passed`（含独立 Task agent 审查与 CR 修订）

## Feature 执行顺序

| # | slug | 交付物一句话 | 性质 |
|---|---|---|---|
| 1 | launch-preflight | 共享源探测 + 启动预检 + launch 门闩 | functional |
| 2 | session-health-inspect | 陈旧 inspect + 筛选（复用源探测） | functional |
| 3 | session-summary-enrichment | clean_summary≤160 + fixture | functional |
| 4 | session-bulk-delete | 多选批量删除（先 cs-req） | functional |
| 5 | session-disk-usage | 体积聚合 UI | functional |
| 6 | port-protect-list | 保护端口 + terminate 拦截 | functional |
| 7 | port-group-and-terminate-by-project | 按 cwd 分组关端口 | functional |
| 8 | grok-config-health | Grok issues 诊断无 secret | functional |
| 9 | terminal-adapter-extend | WezTerm 适配器 | functional |
| 10 | cli-extension-contract | CliType 注册测试 + docs | non-functional |
| 11 | power-extend-harden-and-docs | 回归与文档收口 | non-functional |

依赖：2 消费 1 的共享源模块；5 依赖 2；7 依赖 6；11 依赖全部未 drop。无循环。

## 核心验收路径（roadmap 级）

1. 坏 cwd session：preflight ok=false，launch 失败且中文原因；缓存窗 source 仅 warn 可 launch
2. OpenCode 行已删：source_missing block（非 db 文件存在性）
3. inspect 可筛 missing_cwd/source；无路径泄漏
4. 摘要超长截断 ≤160
5. 批量删除 partial success + failures 可见；仅可丢弃数据 smoke
6. 保护端口 terminate 整批失败
7. Port 按项目分组；未知目录无一键关
8. Grok health 有 issues[]，JSON 无 apiKey/备份绝对路径
9. WezTerm：未装 is_available=false；已装可 launch（可 drop）
10. CliType 缺注册测试红
11. ARCHITECTURE / 用户文档 / AGENTS keys 与代码一致

## 关键假设

- 轴 4 = macOS WezTerm + CLI 编译期契约，非 Windows
- 批量删除不可撤销，确认 + failures 足够
- 按项目杀端口以 workingDirectory 为准

## Top 3 风险

1. OpenCode 源语义误用 delete_target File — 共享 check_session_source 单测锁死
2. 批量误删 — 上限 50、二次确认、仅可丢弃 smoke
3. WezTerm 本机未装 — is_available=false 单测；允许 drop

## 必跑 / 聚合验证命令

| 命令 | 用途 |
|---|---|
| `cd src-tauri && cargo test --lib` | 每后端 feature + 最终聚合 |
| `pnpm build` | 每前端 feature + 最终聚合 |
| 手工 smoke（可丢弃数据） | 删除/杀端口/预检 |
| `python3 <cs-onboard>/tools/codestable-goal-consistency-gate.py --roadmap .codestable/roadmap/session-launcher-power-extend` | 最终审计 |

预检：每 feature 开始前若命令已红，先归因既有 vs 本次。

## 策略摘要

- **TDD**：行为代码 RED→GREEN→VERIFY；UI/文档可 TDD exception
- **DoD**：design approved + steps done + review/QA/accept passed
- **Gate**：goal-protocol-gates.md；缺脚本不假装 passed
- **Provider**：archguard/meta-cc 不可用记录 fallback，不自动阻塞
- **验证工具**：禁止 shim 伪造；只能装真实依赖
- **最终审计**：goal-audit.md + consistency gate + E/C/H summary

## 硬约束（goal 会话不可改）

- roadmap §4 接口契约
- OpenCode 源=行；preflight/inspect 同源
- bulk 走 AppState::delete_session 全路径
- port by project = UI-only
- Grok health 禁止 secret/绝对路径
