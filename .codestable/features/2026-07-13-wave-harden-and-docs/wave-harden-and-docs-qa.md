---
doc_type: feature-qa
feature: 2026-07-13-wave-harden-and-docs
status: passed
---

聚合验证：cargo test --lib 88 passed；pnpm build ok；release dry-run ok。

## Residual risks

- 桌面 GUI smoke（多终端 launch / 删除 disposable session）未在本会话跑通
- Grok 拉模型依赖真实上游可达；单测仅覆盖 URL/JSON 解析
- Port 规则 onBlur 保存，输入中途刷新可能用旧规则
