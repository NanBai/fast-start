---
doc_type: feature-acceptance
feature: 2026-06-24-quick-session-access
status: pending-user-review
summary: quick-session-access 已完成验收核对，搜索、键盘快速启动、项目收藏置顶、架构归并和 requirement 回写均已就绪，待用户终审
tags: [frontend, session-list, search, favorites, productivity]
---

# quick-session-access 验收报告

> 阶段：阶段 3（验收闭环）
> 验收日期：2026-06-24
> 关联方案 doc：`.codestable/features/2026-06-24-quick-session-access/quick-session-access-design.md`
> 用户终审：待用户确认

## 0. 验收阶段补修记录

- [x] **业务补修**：验收发现后端 `set_favorite_project_dirs` 可被直接 command 调用写入非当前扫描项目路径；已补 `AppState::sanitize_favorite_project_dirs`，保存前按当前 `Session.project_dir` 过滤、去重并保序，再写 Store 和内存状态。复验：`cargo test --lib` 23 passed。
- [x] **用户终审布局补修**：用户发现新增搜索框后默认窗口内右侧终端和主题控件被挤出窗口；根因是桌面态 `.control-bar` 不换行，搜索框、最近天数、启动方式、终端、主题控件最小宽度合计超过 800px 窗口内容区。已修 `src/styles/controls.css` 和 `src/styles/responsive.css`：控制栏允许换行、搜索框 flex 基准收敛到 220px、`max-width: 860px` 时搜索框独占一行。复验：Playwright 多宽度布局 smoke 覆盖 1200 / 1040 / 900 / 800 / 720 / 640 / 560 / 480 / 360，均无水平溢出且控件在视口内。
- [x] **测试补充**：新增 Rust 单测 `favorite_project_dirs_are_limited_to_scanned_sessions_before_save`，覆盖非法路径、重复路径和空字符串过滤。
- [x] **架构归并**：已按 design 第 4 节更新 `.codestable/architecture/ARCHITECTURE.md`，补入快速访问派生链路、`favorite_project_dirs` 偏好、`src/hooks/` 分工和快速访问约束。
- [x] **requirement 回写**：已将 `.codestable/requirements/quick-session-access.md` 从 `draft` 升级为 `current`，并同步 `.codestable/requirements/VISION.md`。
- [x] **工作树说明**：当前工作树包含此前 responsive / delete-session 等未提交改动；本报告的范围核对以 quick-access 相关符号、设计清单和代码落点为准，不把既有 scanner / delete 改动误判为本 feature 越界。
- [x] **状态规范化**：checklist checks 接手时为 37 条 `pending`，逐项核对后规范化为 37 条全 `passed`；steps 保持 implement 阶段的 7 条 `done`。

## 1. 接口契约核对

对照 design 2.1 名词层：

- [x] `filterSessionsForQuickAccess(sessions, options)` 已在 `src/lib/sessionUtils.ts` 实现，输入为 `sessions + recentDays + query + favoriteProjectDirs + activeSessionId`，输出为可见 sessions、activeSessionId 和 matchCount。
- [x] 搜索查询是前端运行态：`src/App.tsx` 只维护 `searchQuery` / `activeSessionId`，不持久化，不写入后端扫描缓存。
- [x] 收藏项目使用 `projectDir` 精确字符串：前端用 `favoriteProjectDirs: string[]` / `Set<string>`，后端 Store key 为 `favorite_project_dirs`。
- [x] `favoriteProjectDirs` 是本机偏好：`src/types.ts::SessionData` 和 `ScanResponse` 未新增收藏字段。
- [x] 后端 command 已暴露并注册 `get_favorite_project_dirs` / `set_favorite_project_dirs`，注册点在 `src-tauri/src/lib.rs`。
- [x] `ProjectBucket` 已支持 `favorite`、`forceOpen`、`activeSessionId` 和 `onToggleFavorite` 语义；`AgentGroup` 向下传递 favorite set 和 forceOpen。

流程图核对：design 2.2 中“加载偏好 → scan_sessions → 最近天数过滤 → 搜索过滤 → 收藏排序 → agent/project 渲染 → 收藏持久化 → Enter 复用 launch_session”均有代码落点。

## 2. 行为与决策核对

**需求摘要逐项验证**：

- [x] 顶部搜索入口已接入控制栏，匹配项目名、项目路径、summary 和 agent 类型。
- [x] 搜索模式下命中 agent/project 强制展开；清空搜索后恢复组件自身展开状态。
- [x] `Meta/Ctrl+K` 聚焦搜索框；上下键切换 active session；Enter 复用 `launch_session(session.id)`。
- [x] 项目 header 收藏按钮可切换收藏；收藏项目在同 agent 内排在非收藏项之前；重载后仍可读回。

**明确不做反向核对**：

- [x] 不改扫描 / 启动主流程：quick-access 相关 grep 未命中 `src-tauri/src/launcher.rs`；`src-tauri/src/scanner*` 中命中的 `user_query` 属 Cursor 摘要提取既有语义，不是搜索下推。
- [x] 不做单条 session 收藏：`favoriteSessionIds` / `favorite_session` 无命中。
- [x] 不做跨设备同步、账号、导入导出：无新增网络请求、同步入口或账号入口。
- [x] 不做全局系统快捷键：无 Tauri global shortcut 插件或权限；快捷键只由前端 `window.keydown` 在 app 窗口内处理。
- [x] 不引入全文搜索 / 模糊搜索依赖：`package.json` 无新增搜索依赖，反向 grep 未命中 Fuse / fuzzy / search-index。

**关键决策落地**：

- [x] 搜索在前端本地计算层：`filterSessionsForQuickAccess` 只处理已扫描 sessions；搜索输入不调用 `scan_sessions` / `refresh_sessions`。
- [x] 收藏项目复用 Tauri Store：`preferences.json` 统一承载 terminal、launch mode、theme 和 favorite project dirs。
- [x] 搜索时强制展开但不覆盖局部状态：`open = forceOpen || expanded`。
- [x] 收藏只改变项目组排序：排序在 session 列表派生层按收藏项目稳定提升，不改变项目内原始时间顺序。
- [x] 保存失败回滚：`usePreferences.handleFavoriteProjectDirsChange` 先乐观更新，catch 后恢复 previous 并显示错误。
- [x] 忙碌保护：Enter 快速启动命中 `launchingId` / `deletingId` 时只提示“当前 session 正在处理中”，不重复启动。

**挂载点反向核对与拔除沙盘**：

- [x] 搜索入口 UI：移除 `SearchBox` 及 `App` 中 `searchQuery` 编排即可拔除关键词定位能力。
- [x] 快速访问计算层：移除 `filterSessionsForQuickAccess` 后搜索、最近天数和收藏排序无法统一组合。
- [x] 项目收藏入口：移除 `ProjectBucket` 收藏按钮和 `onToggleFavorite` 后无法标记高频项目。
- [x] 收藏偏好 command：移除 get/set favorite command 后收藏无法重启保留。
- [x] 键盘启动编排：移除 `activeSessionId` 和 `handleSearchKeyDown` 后只剩鼠标启动。
- [x] 反向 grep 覆盖 `favorite|query|search|forceOpen|activeSession|favorite_project`，quick-access 相关引用均落在上述挂载点内。

## 3. 验收场景核对

| 场景 | 结果 | 证据 |
|---|---|---|
| 关键词搜索 | passed | Playwright mock IPC smoke：搜索 `api` 只显示 `Review API workflow`，隐藏非匹配 session |
| 空结果 | passed | 搜索 `none-match` 显示“没有匹配的 session”，控制栏仍存在 |
| 搜索强制展开 | passed | `.cli-group[data-cli="claude-code"]` 在搜索命中时 `data-open=true`；Escape 清空后恢复 collapsed |
| 键盘聚焦 | passed | `Meta+K` 后搜索输入成为 `document.activeElement` |
| 键盘启动 | passed | 搜索 `api` 后 Enter 调用 `launch_session(claude-1)`；搜索 `codes` 后 ArrowDown + Enter 调用 `launch_session(cursor-1)` |
| 忙碌保护 | passed | mock `launch_session` 延迟期间连续按 Enter，只产生 1 次 launch 调用 |
| 收藏项目 | passed | 点击项目 header 收藏按钮后调用 `set_favorite_project_dirs`，保存 `/Users/xb/Desktop/codes/fast-start` |
| 收藏持久化 | passed | mock reload 后搜索 `fast-start`，项目以 `data-favorite=true` 渲染 |
| 收藏失败回滚 | passed | 代码审查：`usePreferences` catch 分支恢复 previous 并报错；后端补修保证 Store 保存成功后才更新内存 |
| 回归主路径 | passed | mock smoke 覆盖刷新；`pnpm build` 覆盖主题 / 控件类型；`cargo test --lib` 覆盖后端状态与删除回归 |

补充验证命令：

- `pnpm build`：通过，`tsc && vite build` 完成。
- `cd src-tauri && cargo test --lib`：23 passed。
- Playwright mock IPC smoke：通过，console error 0；覆盖搜索、强制展开、`Meta+K`、Enter、忙碌保护、收藏保存、reload 保留、刷新、空结果、Escape、方向键切换。
- Playwright 多宽度布局 smoke：通过；1200 / 1040 / 900 / 800 / 720 / 640 / 560 / 480 / 360 下 `documentScrollWidth === documentClientWidth`，终端和主题控件均在视口内。
- 工具失败处理：前两次 smoke 失败分别是等待默认折叠下不可见 `.session-row`、dev StrictMode 初始 effect 双调用、浏览器环境无 `process`；均为测试脚本问题，修正后通过。

说明：dev 模式下 `React.StrictMode` 会让初始 `scan_sessions` 发生 2 次；验收只断言“输入搜索不新增 scan 调用”。reload 后 scan 总数增加属于页面重载，不是搜索触发。

## 4. 术语一致性与禁用词反向 grep

design 未定义禁用词列表，跳过禁用词反向 grep。

术语一致性检查：

- `快速定位` 落在前端搜索输入、active session 和键盘启动编排，不新增后端扫描语义。
- `搜索查询` 只作为 `searchQuery` / `query` 运行态存在，不持久化。
- `活跃匹配项` 对应 `activeSessionId`，只在搜索模式下参与 Enter 启动。
- `收藏项目` 对应 `favoriteProjectDirs` / `favorite_project_dirs`，粒度为 `projectDir`，不是 session id。

反向 grep：

```bash
rg -n "favoriteSessionIds|favorite_session|globalShortcut|global-shortcut|plugin-global-shortcut|Fuse|fuse.js|fuzzy|search-index|全文搜索|cloud sync|账号|导入|导出" src src-tauri package.json pnpm-lock.yaml
rg -n "favorite|quick|query|search|forceOpen|activeSession|favoriteProject|favorite_project|set_favorite|get_favorite" src-tauri/src/scanner.rs src-tauri/src/scanner src-tauri/src/launcher.rs src-tauri/tauri.conf.json
```

结论：禁用范围无命中；scanner 中 `user_query` 命中属于 Cursor 摘要来源解析，不属于本 feature 搜索能力。

## 5. 架构归并

### 归并建议 diff

**目标文档**：`.codestable/architecture/ARCHITECTURE.md`

| 归并类别 | design 来源 | 建议操作 | 理由 |
|---|---|---|---|
| 名词层 | 第 2.1 节 `QuickAccessOptions` / `QuickAccessResult` | 归并 | 前端列表派生契约稳定，后续搜索 / 排序类功能会复用理解 |
| 名词层 | 第 2.1 节 `favoriteProjectDirs` / `favorite_project_dirs` | 归并 | 本机偏好新增长期字段，影响 Store 口径 |
| 主流程 | 第 2.2 节“最近天数过滤 → 搜索过滤 → 收藏排序 → agent/project 渲染” | 归并 | 改变前端数据流，后续 feature 需要知道搜索不下推 scanner |
| 挂载点 | 第 2.3 节搜索入口、计算层、收藏入口、偏好 command、键盘启动 | 归并 | 是可卸载的长期入口和模块交互边界 |
| 已知约束 | 第 2.2 节搜索字段限制、收藏路径合法性、失败回滚、忙碌保护 | 归并 | 属于跨 feature 稳定约束 |
| 不归并 | 第 2.4 节推进策略 | 不归并 | 单次实现节奏，不属于系统现状 |
| 不归并 | 第 3 节具体 smoke 场景 | 不归并 | 验收证据留在 acceptance，不进入架构地图 |

已实际写入：

- [x] 概览新增“快速访问”能力和前端派生顺序。
- [x] 名词层补 `QuickAccessOptions` / `QuickAccessResult`、`favorite_project_dirs` 和扩展后的 `AppState` 职责。
- [x] 核心模块补 `src/hooks/` 状态编排目录。
- [x] 数据流补前端过滤 / 搜索 / 收藏排序链路，以及收藏保存到 `preferences.json` 的链路。
- [x] 新增“快速访问约束”节。
- [x] 部署架构补充 terminal / launch mode / theme / favorite project dirs 同属本机 `preferences.json`。
- [x] 变更日志新增 2026-06-24 快速访问归并记录。

判据满足：没读过 design 的人打开 architecture 能知道系统现在有快速访问能力、数据流位置、偏好归属和边界约束。

## 6. requirement 回写

design frontmatter 指向 `requirement: quick-session-access`，原 requirement 状态为 `draft`。

已实际更新：

- [x] `.codestable/requirements/quick-session-access.md`：`draft` → `current`，保留用户故事、痛点、解决方式和边界，追加变更日志。
- [x] `implemented_by` 补 `.codestable/architecture/ARCHITECTURE.md` 对应入口。
- [x] `.codestable/requirements/VISION.md`：`quick-session-access` 从 Draft 移到 Current。

## 7. roadmap 回写

design frontmatter 无 `roadmap` / `roadmap_item` 字段。

**结论**：非 roadmap 起头，跳过。

## 8. attention.md 候选盘点

- [x] **无候选**：本 feature 未暴露需要补入 `.codestable/attention.md` 的新启动规则。已有注意事项已经覆盖本项目测试命令、Tauri / React / TypeScript 项目性质和根目录 git 口径。

## 9. 遗留

- 后续优化点：`src/App.tsx` 和 `src-tauri/src/state.rs` 仍偏长；本轮已抽出前端 hooks，但继续扩展偏好或顶层编排时，建议单独走 `cs-refactor` 拆分状态层和 App 顶层职责。
- 已知限制：本轮不做全局系统快捷键、模糊搜索、跨设备同步或单条 session 收藏，符合 design 边界。
- 验收证据限制：浏览器交互用 mock IPC 验证前端行为；真实 Tauri Store 写入链路由 Rust command / state 代码审查和 lib 测试覆盖，未额外打开原生 Tauri 窗口做人工终审。
