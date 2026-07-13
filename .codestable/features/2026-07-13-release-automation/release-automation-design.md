---
doc_type: feature-design
feature: 2026-07-13-release-automation
requirement:
roadmap: session-launcher-next-wave
roadmap_item: release-automation
status: approved
summary: release 脚本：版本一致校验、构建 dmg、创建 GitHub Release
tags: [release, tooling]
---

# release-automation 设计文档


## 1. 决策
- 脚本放 `scripts/release.sh`（或 .ps1 不强制）
- 校验 package.json / Cargo.toml / tauri.conf.json 版本一致
- 调用 pnpm tauri build；gh release create
- 支持 DRY_RUN=1
- 不强制公证/签名自动化（文档说明人工门禁）

## 2. 挂载点
1. scripts/release.sh
2. docs/dev/release-readiness.md 引用

## 3. 验收
- dry-run 打印步骤不上传
- 版本不一致失败退出
- 文档可跟做

