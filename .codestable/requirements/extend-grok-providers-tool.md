---
doc_type: requirement
slug: extend-grok-providers-tool
pitch: 在 Grok 工具页同时管理官方账号登录与 API 供应商，并支持隐私保护与卡片布局偏好
status: current
---

# extend-grok-providers-tool

## 用户故事

作为在 macOS 上使用 Grok Build / Grok CLI 的开发者，我希望在 Session Launcher 的 Grok 工具页里：

1. 在 **官方账号（OAuth / auth.json）** 与 **API 供应商档案** 之间一键切换生效的 `config.toml` 上游；
2. 一键写入常见 **隐私保护** 本地配置（遥测关闭等）；
3. 把常用登录方式 **置顶 / 排序**，重启后仍保留；

这样不必再打开独立的 grok_switch 工具，也能完成 v0.2.0 级别的登录方式管理。

## 边界

- 只作用于 `~/.grok/config.toml` 与本机档案 `~/.grok_switch/profiles.json`；不改 session 扫描与终端 resume。
- 不实现 Windows 托盘 / 开机自启 / 独立 HTTP 服务（那是 grok-build-switch 宿主形态，不是本 app）。
- 隐私保护只写本地 config 段，不代替账号侧 Coding data sharing / `/privacy`。
