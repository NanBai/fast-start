---
doc_type: feature-design
feature: 2026-07-13-launch-history-and-preview
requirement:
roadmap: session-launcher-next-wave
roadmap_item: launch-history-and-preview
status: approved
summary: 成功启动写入最近记录；快捷再启动；可选预览 resume 命令
tags: [launch, history, preview]
---

# launch-history-and-preview 设计文档


## 1. 决策
- 仅后端 launch 成功后写 recent_launches（上限 20）
- sanitize 去掉不存在 session
- preview_launch_command 只读校验，不写 wrapper

## 2. 挂载点
1. preferences recent_launches
2. state.launch_session 成功路径写入
3. preview command
4. UI 最近列表 + 预览

## 3. 验收
- 启动成功后历史出现；失败不写
- 再启动可用
- 预览含 program/args/cwd
- 不做全局快捷键

