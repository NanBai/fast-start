---
doc_type: feature-design
feature: 2026-07-13-port-power-ops
requirement:
roadmap: session-launcher-next-wave
roadmap_item: port-power-ops
status: approved
summary: loopback 打开浏览器、端口规则偏好；复用现有多 id terminate
tags: [port, browser, preferences]
---

# port-power-ops 设计文档


## 1. 决策
- **不重写** terminate_port_processes（已 Vec + all-or-nothing）
- 打开浏览器：仅 loopback URL + opener
- 规则：port_project_path_prefixes、port_ignore_ports 叠在 is_project_service 上
- 若缺跨组多选 UI 则补；组内关闭全部已有

## 2. 挂载点
1. preferences 两 key
2. is_project_service / 前端过滤接入
3. PortWorkspace 打开按钮
4. 可选跨组多选

## 3. 验收
- loopback 可开浏览器；非 loopback 禁用
- ignore 端口不展示或降权（design 定不展示）
- prefix 扩大项目服务
- 批量关闭仍 all-or-nothing 回归

