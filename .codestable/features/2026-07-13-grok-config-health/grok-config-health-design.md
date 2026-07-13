---
doc_type: feature-design
feature: 2026-07-13-grok-config-health
requirement: extend-grok-providers-tool
roadmap: session-launcher-power-extend
roadmap_item: grok-config-health
status: approved
summary: grok_config_health 产出可测 issues[]；无 secret/绝对路径
tags: [grok, health, diagnostics]
---

# grok-config-health 设计文档

## 1. 决策与约束

**目标**：Grok 页只读健康诊断。  
**不做**：写盘、restore、出站、log apiKey。

**决策**：

1. 新 command `grok_config_health`，**映射** status/list_backups 为白名单字段 + **issues[]**  
2. **禁止**原样返回 status（含 config_path/data_dir/backup path/size/api_key）  
3. issues codes 按 roadmap §4.6 可测生成  

## 2. 名词与编排

```text
GrokHealthReport {
  configPresent, authPresent, profilesCount,
  activeMode, activeProfileId, configMatchesActive,
  backups[{name, modifiedAt}],
  issues[{code, severity, message}]
}

// 字段映射（禁止原样 serde status/backups）
configPresent     ← status.config_exists
authPresent       ← status.official_logged_in（或 auth 文件存在）
profilesCount     ← list_profiles.len()
activeMode        ← official_active ? official : (active_profile ? profile : unknown)
activeProfileId   ← status.active_profile.map(id)  // 无 key
configMatchesActive ← status.config_matches_active
backups[].name    ← backup.file（文件名 only）
backups[].modifiedAt ← backup.created_at
// 丢弃：config_path, data_dir, backup.path, backup.size, profile.api_key
```

```text
UI 打开诊断
  → grok_config_health()
  → 内部读 status/backups/profiles
  → 生成 issues + 消毒字段
  → 面板展示
```

### 挂载点

1. grok_provider health 函数  
2. command 注册  
3. ProvidersWorkspace 面板  
4. 单测：issues 生成与无 key  

### 2.5 结构健康度

health 独立函数，不污染 switcher 写路径。

## 3. 验收

- official 无 auth → 有 auth_missing_official 类 issue  
- 响应 JSON 无 apiKey / 备份绝对路径  
- 不触发网络  

**验证**：cargo test；手工 Grok 页  

## 4. 架构

出站仍仅用户触发的 models/test；health 只读本地。
