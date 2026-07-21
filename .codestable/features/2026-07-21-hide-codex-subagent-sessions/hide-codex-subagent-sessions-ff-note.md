---
doc_type: feature-ff-note
feature: hide-codex-subagent-sessions
date: 2026-07-21
requirement:
tags: [codex, scanner, multi-agent, session-list]
---

## 做了什么
Codex multi-agent 一次对话会为子 agent 各写一份 rollout；列表现在默认隐藏这些 subagent 会话，只保留主会话。

## 改了哪些
- `src-tauri/src/scanner/codex.rs` — `is_codex_subagent` 识别 `thread_source=subagent` / `source.subagent`，扫描时跳过；补检测与 parent/child fixture 测试

## 怎么验证的
- `cd src-tauri && cargo test --lib scanner::codex::` → 8 passed
- 用户本机确认 `temp` 项目下只剩主会话（EffectAccepted）

## 顺手发现（可选，不阻塞）
- 无
