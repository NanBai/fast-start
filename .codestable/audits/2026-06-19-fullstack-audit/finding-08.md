---
doc_type: audit-finding
audit: 2026-06-19-fullstack-audit
finding_id: "security-08"
nature: security
severity: P2
confidence: medium
suggested_action: cs-refactor
status: open
---

# Finding 08：Tauri CSP 显式关闭，扩大前端注入风险面

## 速答

Tauri 配置中 `csp` 被设为 `null`，等于关闭前端内容安全策略；当前前端没有 `dangerouslySetInnerHTML`，但 CSP 仍是桌面 WebView 的重要纵深防护。

## 关键证据

- `src-tauri/tauri.conf.json:20` — `security` 配置块。
- `src-tauri/tauri.conf.json:21` — `"csp": null` 显式关闭 CSP。
- `src/App.tsx:2` — 前端通过 Tauri `invoke` 调用后端能力，WebView 注入风险会直接接近本地命令边界。
- `src/App.tsx:708` — 后端扫描错误会被渲染到页面；React 默认转义文本，但 CSP 仍能降低未来误用 HTML/资源加载时的风险。

## 影响

当前代码没有直接 XSS 证据，因此不是 P0/P1；但关闭 CSP 会让未来新增 HTML 渲染、外链资源、第三方脚本或调试代码时缺少兜底边界。

## 修复方向

恢复最小可用 CSP，例如限制 `default-src 'self'`、脚本/样式按 Vite/Tauri 需要配置；如果 dev 模式需要放宽，应区分 dev 与 build。

## 建议动作

`cs-refactor`，因为这是安全配置收敛，不应改变业务功能。
