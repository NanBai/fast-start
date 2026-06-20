---
doc_type: audit-index
audit: 2026-06-19-fullstack-audit
scope: Tauri 后端扫描/启动链路、React 前端会话列表与项目配置
created: 2026-06-19
status: active
total_findings: 8
---

# fullstack-audit 审计报告

## 范围

本次审计覆盖 `src-tauri/src/` 后端核心模块（scanner / launcher / state / commands / security / models）、`src/` 前端主界面、Tauri/Vite/package 配置，以及 `.codestable/architecture/ARCHITECTURE.md` 中记录的架构约束。

## 总评

共发现 8 条优化项：P1 4 条、P2 4 条；性质上性能 3 条、安全 2 条、可维护性 2 条、架构漂移 1 条。最值得优先处理的是启动阶段同步扫描、Cursor 每个 chat 启一个 `sqlite3` 子进程、环境依赖型测试假绿、前端单文件过大。整体代码主路径清晰，校验和错误返回比早期实现更完整，但扫描链路与 UI 组件组织已经开始进入维护成本上升区。

## 发现清单

| # | 性质 | 严重度 | 置信度 | 标题 | 文件 |
|---|---|---|---|---|---|
| 1 | performance | P1 | high | Tauri setup 同步扫描会阻塞应用首屏 | [finding-01.md](finding-01.md) |
| 2 | performance | P1 | high | Cursor 扫描对每个 chat 启动一次 sqlite3 子进程 | [finding-02.md](finding-02.md) |
| 3 | maintainability | P1 | high | 扫描测试依赖本机真实 CLI 数据，容易假绿 | [finding-03.md](finding-03.md) |
| 4 | maintainability | P1 | high | 前端主组件与样式文件过大，职责混杂 | [finding-04.md](finding-04.md) |
| 5 | security | P2 | medium | Terminal/iTerm 仍通过 shell 字符串注入终端 | [finding-05.md](finding-05.md) |
| 6 | arch-drift | P2 | high | cd=false / cursor resume 注释与当前架构不一致 | [finding-06.md](finding-06.md) |
| 7 | bug | P2 | medium | Codex 简介只扫描前 64 行，可能漏掉真实用户输入 | [finding-07.md](finding-07.md) |
| 8 | security | P2 | medium | Tauri CSP 显式关闭，扩大前端注入风险面 | [finding-08.md](finding-08.md) |

## 按维度分布

| 性质 | P0 | P1 | P2 | 合计 |
|---|---|---|---|---|
| bug | 0 | 0 | 1 | 1 |
| security | 0 | 0 | 2 | 2 |
| performance | 0 | 2 | 0 | 2 |
| maintainability | 0 | 2 | 0 | 2 |
| arch-drift | 0 | 0 | 1 | 1 |
| **合计** | **0** | **4** | **4** | **8** |

## 下一步建议

- **P1 本迭代修**：finding-01、finding-02 建议走 `cs-refactor` 拆异步/增量扫描；finding-03 建议走 `cs-refactor` 改成临时 fixture 测试；finding-04 建议走 `frontend-design` + `cs-refactor` 分组件与样式边界。
- **P2 排队处理**：finding-05、finding-08 建议纳入安全收敛；finding-06 先做 `cs-arch check/update` 判定文档还是注释要改；finding-07 可作为小型 `cs-issue` 修摘要边界。
