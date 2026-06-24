---
doc_type: requirement
slug: responsive-window-ui
pitch: 窗口变窄或变宽时，Session Launcher 仍然清晰可用
status: current
last_reviewed: 2026-06-20
implemented_by:
  - ARCHITECTURE.md
tags: [frontend, responsive, desktop]
---

# 不同窗口下都能稳定使用 Session Launcher

## 用户故事

- 作为把 app 放在侧边窄窗口里的用户，我希望仍能看清筛选、终端、主题和启动按钮，而不是每次都要把窗口拉大。
- 作为在大屏上整理多个 agent 会话的人，我希望列表能利用更多横向空间，而不是长路径和简介一直被固定窄宽截断。
- 作为频繁切换主题和窗口尺寸的人，我希望界面不会在 resize 后出现文字重叠、按钮难点或横向滚动。

## 为什么需要

Session Launcher 是一个常驻桌面工具，用户会根据当前工作区把它放大、缩小或贴在屏幕边缘。窗口适配不好时，真正影响的不是美观，而是能不能快速判断会话、点到控件、恢复工作现场。

## 怎么解决

让同一套界面根据窗口宽度自动调整密度：窄窗口优先保证控件可点、文字不压按钮；默认窗口保持熟悉的工作台视图；宽窗口让列表和路径获得更多空间。

## 边界

- 不提供独立移动端网页，也不把桌面 app 改成移动页面。
- 不改变 agent 类型、会话字段或启动流程。
- 不负责重新设计视觉风格，只保证现有界面在不同窗口下稳定可用。
