---
doc_type: feature-design
feature: 2026-07-13-grok-default-profile-policy
requirement:
roadmap: session-launcher-next-wave
roadmap_item: grok-default-profile-policy
status: approved
summary: ensure_default_profile：无 API 上游时 import active=false
tags: [grok, auth, default-profile]
---

# grok-default-profile-policy 设计文档


## 1. 决策
- profiles 空且 config 存在时仍可导入 Default
- 无 models_base_url（或等价 endpoints 空）→ active=false（官方模式）
- 有 models_base_url → active=true
- 不删档案、不改 auth.json

## 2. 现状
`ensure_default_profile` → import_current("Default", true)

## 2.3 挂载点
仅 switcher ensure_default + 单测

## 3. 验收
- 纯 OAuth 样 config → 导入后 officialActive
- 含 models_base_url → Default active
- 已有 profiles 不导入

