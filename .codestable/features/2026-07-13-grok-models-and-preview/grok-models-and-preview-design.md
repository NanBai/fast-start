---
doc_type: feature-design
feature: 2026-07-13-grok-models-and-preview
requirement:
roadmap: session-launcher-next-wave
roadmap_item: grok-models-and-preview
status: approved
summary: Grok 拉取模型列表、连通测试、启用前 config 预览
tags: [grok, network, preview]
---

# grok-models-and-preview 设计文档


## 0. 术语
fetch models / test connection / preview apply — 见 roadmap 4.5

## 1. 决策
- 仅用户触发；Rust HTTP；预览不写盘
- 依赖 reqwest 或 ureq 在实现选定

### 出站硬约束（必须实现）

| 项 | 规则 |
|---|---|
| 触发 | 仅用户点击；启动不请求 |
| 发起方 | 仅 Rust；前端不直连任意 URL |
| scheme | 仅 `http` / `https` |
| 超时 | ≤10s |
| 响应体 | 设大小上限（实现定，建议 ≤2MiB） |
| upstreamFormat | 既有枚举；非法 → Err |
| 密钥 | 不 log apiKey |
| CSP | 保持非 null |
| SSRF 默认 | 允许用户填写的 baseUrl（含内网）；错误收敛为 String；不跟随无限 redirect（最多有限次） |

## 2. 名词
新增 commands：grok_fetch_models、grok_test_connection、grok_preview_apply
复用 apply_profile_text 做预览

### 2.3 挂载点
1. commands + lib 注册
2. ProvidersWorkspace 编辑页按钮
3. Cargo 依赖（若新增）

### 2.5 网络逻辑可放 grok_provider/http.rs 新文件

## 3. 验收
- 合法 upstream 返回模型列表或明确错误
- 连通测试返回 latencyMs 或 Err
- 预览文本含 models_base_url/default
- 启动不自动请求
- CSP 非 null

