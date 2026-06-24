---
doc_type: feature-design
feature: 2026-06-24-quick-session-access
requirement: quick-session-access
status: approved
summary: 增加 session 搜索、键盘快速启动和项目级收藏置顶，让用户更快恢复高频工作现场。
tags: [frontend, session-list, search, favorites, productivity]
---

# quick-session-access 设计文档

## 0. 术语约定

| 术语 | 定义 | 防冲突结论 |
|---|---|---|
| 快速定位 | 用户输入关键词后，本地过滤当前 session 列表，并可用键盘移动焦点与启动 | 新增前端交互名词，不新增后端扫描语义 |
| 搜索查询 | 用户临时输入的关键词，匹配 `cliType / projectName / projectDir / summary` | 不持久化，不影响真实扫描缓存 |
| 活跃匹配项 | 搜索模式下当前被键盘选中的 session，回车启动它 | 新增前端局部状态，不写入后端 |
| 收藏项目 | 用户标记为高频入口的 `projectDir`，在列表中优先排序 | 本轮收藏粒度是项目，不是单条 session |

## 1. 决策与约束

### 需求摘要

用户目标：session 变多后，仍能快速找到并启动目标工作现场，同时让高频项目稳定排在更靠前的位置。

核心行为：

- 顶部提供搜索入口，按项目名、项目路径、session 简介和 agent 类型过滤当前列表。
- 搜索模式下自动展开有匹配结果的 agent / project，避免结果藏在折叠层里。
- 用户可以用键盘聚焦搜索、移动活跃匹配项，并回车启动当前匹配 session。
- 用户可以收藏 / 取消收藏项目；收藏项目在对应 agent 下优先排序，偏好重启后保留。

成功标准：

- 输入关键词后，列表只显示当前最近天数范围内的匹配 session，空结果有明确提示。
- `Cmd/Ctrl+K` 聚焦搜索；搜索有结果时，上下键切换活跃项，回车启动活跃 session。
- 收藏某个项目后，同一 `projectDir` 在各 agent 分组内排到非收藏项之前，刷新或重启后仍保留。
- 删除 session、刷新、最近天数筛选、主题、终端选择、打开方式不回退。
- `pnpm build` 和 `cd src-tauri && cargo test --lib` 通过。

明确不做：

- 不改 scanner、launcher、删除源载体和 `Session` 对外字段。
- 不做单条 session 收藏，不新增收藏 session 的数据模型。
- 不做跨设备同步、账号体系、云端存储或导入导出。
- 不做全局系统快捷键；app 未获得焦点时不响应。
- 不引入全文搜索数据库、索引服务或模糊搜索第三方依赖。
- 不改变最近天数筛选语义；搜索只在当前时间范围内继续过滤。

### 假设

- 假设收藏粒度选项目级。理由是 session 可能被删除或随 CLI 历史变化消失，项目目录更适合作为长期高频入口。
- 假设搜索使用大小写不敏感的包含匹配，不做拼音、分词、Levenshtein 距离或权重打分。
- 假设搜索查询不持久化；用户重启后保留收藏项目，但不保留上一次搜索词。

### 复杂度档位

走本地桌面工具默认档位，有两点偏离：

- 可测试性 = tested（偏离默认 testable）：搜索过滤、收藏排序和键盘选择属于核心交互，至少要有纯函数或组件级可验证覆盖。
- 安全性 = validated（偏离默认 trusted）：收藏的 `projectDir` 虽来自已扫描数据，但会持久化到本地偏好；保存前应限制为当前扫描结果中出现过的项目路径，避免偏好文件被污染后影响 UI。

### 关键决策

1. **搜索放在前端本地计算层，不改 Rust 扫描契约**
   这轮目标是更快定位已有 session，不是改变能扫描到什么。复用当前 `ScanResponse.sessions`，避免把查询参数下推到 scanner 造成第二套扫描路径。

2. **收藏项目复用 Tauri Store 偏好链路**
   终端、打开方式、主题已经走 `preferences.json`。收藏项目也是本机偏好，复用同一链路比前端 `localStorage` 更符合当前桌面应用结构。

3. **搜索模式强制展示匹配路径，清空搜索后恢复用户折叠状态**
   如果搜索结果仍被折叠层隐藏，快速定位失效。强制展开只在查询非空时生效，不覆盖用户原本的展开 / 折叠选择。

4. **收藏只改变项目组排序，不改变 session 内部时间排序**
   收藏表达“这个项目更重要”，不是重排项目里的历史记录。项目内仍按 `lastActiveAt` 倒序，保持恢复最近上下文的直觉。

5. **当前状态收尾作为实现前 gate，不混入功能主体**
   当前工作树已有大量未提交改动，且 `.codestable/attention.md` 的“非 git 仓库”说明已过期。实现前应先处理这两个状态问题；它们不是用户可见能力，不进入验收场景。

## 2. 名词与编排

### 2.1 名词层

#### 现状

- `SessionData` 在 `src/types.ts` 定义，包含 `id / cliType / sessionId / projectDir / projectName / lastActiveAt / summary`。
- `App` 持有 `sessions`、扫描错误、偏好、加载、启动、删除、右键菜单等状态；当前 `src/App.tsx` 约 363 行。
- 最近天数过滤在 `src/lib/sessionUtils.ts::filterSessionsByRecentDays`，只按 `lastActiveAt` 过滤。
- agent 分组在 `App` 中按 `CLI_ORDER` 聚合；项目分组在 `AgentGroup` 内部按 `projectDir` 聚合。
- 偏好链路在 `src-tauri/src/state.rs` / `commands.rs` 中已有 terminal、launch mode、theme 三组 get/set command。
- 前端没有 search query、active result、favorite project 相关类型或状态。

#### 变化

- **新增前端搜索状态**：`query: string`、`activeSessionId: string | null`，只存在于前端运行态。
- **新增收藏项目偏好**：`favoriteProjectDirs: string[]`，以 `projectDir` 精确字符串作为 key；后端持久化，前端用 `Set<string>` 参与排序。
- **新增快速访问计算结果**：从 `sessions + recentDays + query + favoriteProjectDirs` 推导出可渲染分组、匹配数量、首个/活跃 session。
- **新增偏好 command**：读取和保存收藏项目列表，不改变 `ScanResponse` 或 `SessionData`。
- **扩展项目组 UI 状态**：项目 header 增加收藏切换入口；搜索非空时 project/agent 接收 `forceOpen` 语义。

#### 接口示例

```ts
// 来源：src/lib/sessionUtils.ts 新增纯计算契约
type QuickAccessOptions = {
  recentDays: RecentDaysFilter;
  query: string;
  favoriteProjectDirs: Set<string>;
};

type QuickAccessResult = {
  sessions: SessionData[];
  activeSessionId: string | null;
  matchCount: number;
};

filterSessionsForQuickAccess(sessions, options);
```

```rust
// 来源：src-tauri/src/commands.rs 新增偏好 command
#[tauri::command]
pub fn get_favorite_project_dirs(state: State<'_, AppState>) -> Result<Vec<String>, String>;

#[tauri::command]
pub fn set_favorite_project_dirs(
    project_dirs: Vec<String>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String>;
```

```tsx
// 来源：src/components/ProjectBucket.tsx 扩展 props 语义
<ProjectBucket
  projectDir={group.projectDir}
  favorite={favoriteProjectDirs.has(group.projectDir)}
  forceOpen={query.trim().length > 0}
  onToggleFavorite={toggleFavoriteProject}
/>
```

### 2.2 编排层

```mermaid
flowchart TD
    A[App 启动] --> B[加载终端 / 打开方式 / 主题 / 收藏项目偏好]
    B --> C[scan_sessions 获取 Session 列表]
    C --> D[recentDays 过滤]
    D --> E{query 是否为空}
    E -- 空 --> F[收藏项目优先排序 + 保留用户折叠状态]
    E -- 非空 --> G[关键词过滤 + 计算 active 匹配项]
    G --> H[强制展开匹配 agent/project]
    F --> I[渲染 AgentGroup / ProjectBucket / SessionRow]
    H --> I
    J[用户收藏项目] --> K[更新本地 Set]
    K --> L[set_favorite_project_dirs 持久化]
    M[用户按 Enter] --> N{有 active session?}
    N -- 是 --> O[复用 launch_session(session.id)]
    N -- 否 --> P[不启动，只保持搜索状态]
```

#### 现状

- 启动编排：`App` mount 后调用 `loadPreferences()` 和 `loadSessions()`。
- 过滤编排：先用 `filterSessionsByRecentDays` 得到 `visibleSessions`，再按 `CLI_ORDER` 分到 `sessionsByCli`。
- 折叠状态：`AgentGroup` 和 `ProjectBucket` 各自持有局部 `expanded` 状态。
- 启动编排：所有启动最终走 `handleLaunch(session.id)`，后端仍由 `launch_session` 查缓存并启动终端。
- 偏好持久化：`AppState` 管理偏好值，`commands.rs` 注册 get/set command，`state.rs` 负责读写 `preferences.json`。

#### 变化

- 偏好加载阶段增加收藏项目列表；若偏好中包含当前扫描结果不存在的路径，前端不显示，但保存时可清理为当前合法集合。
- 过滤编排改为“最近天数范围 → 搜索查询 → 收藏排序 → agent/project 分组”。搜索不改变扫描缓存，只影响渲染结果。
- 折叠编排增加 `forceOpen`：查询非空时有匹配的 agent 和 project 必须展开；清空查询后恢复各组件原本的局部展开状态。
- 键盘编排增加搜索焦点：`Cmd/Ctrl+K` 聚焦搜索，`Escape` 清空查询或失焦，`ArrowUp/ArrowDown` 切换活跃匹配项，`Enter` 复用现有启动流程。
- 收藏编排增加 optimistic UI：先更新前端 Set，再调用保存；保存失败则回滚并显示错误，不能假成功。

#### 流程级约束

- 搜索必须是纯前端派生状态，不能触发重新扫描。
- 搜索匹配字段限制为 `cliType / projectName / projectDir / summary`；不得读取或索引 CLI 原始对话全文。
- 收藏项目只能来自当前已扫描 session 的 `projectDir`；保存列表需去重、排序稳定。
- 收藏保存失败时必须明确提示，并恢复保存前 UI 状态。
- 正在启动或删除中的 session 不响应 Enter 快速启动。
- 搜索空结果时不隐藏控制栏和状态提示，用户必须能直接清空查询或调整最近天数。

### 2.3 挂载点清单

| 挂载位置 | 动作 | 删除后效果 |
|---|---|---|
| 搜索入口 UI | 新增：控制栏中的搜索输入和键盘焦点入口 | 用户无法按关键词快速定位 session |
| 快速访问计算层 | 新增：最近天数、查询和收藏排序的统一派生结果 | 搜索 / 收藏无法稳定组合，结果容易与列表分组脱节 |
| 项目收藏入口 | 修改：项目 header 增加收藏切换按钮 | 用户无法标记高频项目 |
| 收藏偏好 command | 新增：get/set favorite project dirs | 收藏无法重启保留 |
| 键盘启动编排 | 新增：active session 与 Enter 启动 | 快速定位只能靠鼠标完成，命令面板式体验消失 |

### 2.4 推进策略

1. **实现前状态收尾 gate**
   退出信号：工作树状态已记录清楚；`.codestable/attention.md` 的 git 说明不再误导后续任务。

2. **微重构：拆出 App 编排 hook（只搬不改行为）**
   退出信号：偏好加载、session 加载 / 启动 / 删除行为不变，`pnpm build` 通过。

3. **快速访问纯计算层**
   退出信号：搜索过滤、空查询、大小写不敏感匹配、收藏排序都有可验证结果。

4. **静态 UI 接入**
   退出信号：控制栏出现搜索入口，项目 header 出现收藏按钮；无搜索时现有列表视觉和折叠默认行为不变。

5. **交互与键盘编排**
   退出信号：`Cmd/Ctrl+K`、上下键、回车、Escape 在搜索输入聚焦时按预期工作，不影响其他控件。

6. **收藏持久化接入**
   退出信号：收藏项目保存到 Tauri Store，刷新 / 重启后仍生效；保存失败有回滚和错误提示。

7. **联调与回归验证**
   退出信号：`pnpm build`、`cd src-tauri && cargo test --lib` 通过；搜索、收藏、刷新、删除、启动主路径均有 smoke 证据。

### 2.5 结构健康度与微重构

##### convention 检索

已执行：

```bash
python3 .codestable/tools/search-yaml.py --dir .codestable/compound \
  --filter doc_type=decision --filter category=convention \
  --query "目录组织 OR 命名 OR 归属 OR frontend OR component"
```

结果：未命中已归档的前端目录 convention。历史 `audit-p1-p2-remediation` acceptance 曾建议沉淀“组件放 `src/components/`、纯函数放 `src/lib/`、状态 hook 放 `src/hooks/`”，但尚未归档为 decision。

##### 评估

- 文件级 — `src/App.tsx`：约 363 行，已经承担偏好、扫描、启动、删除、菜单、主题和顶层渲染；本次继续加入搜索、收藏、键盘状态会让顶层编排继续膨胀。
- 文件级 — `src/components/Controls.tsx`：约 173 行，当前集中菜单 / segmented 控件；新增搜索输入属于控制栏职责延伸，但不应把搜索计算写入该文件。
- 文件级 — `src/components/ProjectBucket.tsx`：约 65 行，适合增加收藏按钮和 `forceOpen` 语义。
- 文件级 — `src/lib/sessionUtils.ts`：约 44 行，适合承载搜索过滤和收藏排序这类跨组件纯函数。
- 文件级 — `src-tauri/src/state.rs`：约 380 行，偏长；新增收藏偏好会继续扩大偏好职责，但现有 terminal/launch/theme 已集中在这里，本次只加同类 preference，不重构 Rust 状态层。
- 目录级 — `src/components/`：已有 8 个同层文件，本次可能新增搜索控件；仍属于业务组件目录，但需要避免把组件拆得过碎。
- 目录级 — `src/hooks/`：当前不存在；若抽出状态编排 hook，这是稳定模式，不是一次性临时目录。
- 目录级 — `src/lib/`：已有 `sessionUtils.ts`，适合继续放列表派生纯函数。

##### 结论：微重构（拆文件）

本 feature 前置一个只搬不改行为的前端微重构，目标是给搜索和收藏留出清晰挂载点，避免继续扩大 `App.tsx`。

- 搬什么：从 `App.tsx` 中搬出偏好加载 / 保存状态编排、session 加载 / 启动 / 删除状态编排。
- 搬到哪：`src/hooks/usePreferences.ts` 和 `src/hooks/useSessions.ts`；跨组件纯计算继续放 `src/lib/sessionUtils.ts`。
- 行为不变怎么验证：`pnpm build` 通过；刷新、终端选择、打开方式、主题选择、启动、删除 smoke 行为不变。
- 步骤序列：
  1. 抽出偏好 hook，保持 Tauri command 名称和状态值不变。
  2. 抽出 session hook，保持 `ScanResponse` 接入、启动和删除错误语义不变。
  3. App 只消费 hook 返回值并渲染现有组件，确认无用户可见变化。

##### 建议沉淀的 convention

- 是否稳定模式：稳定模式。
- 规则一句话：前端业务组件放 `src/components/`，跨组件纯函数放 `src/lib/`，状态编排 hook 放 `src/hooks/`，根目录只保留入口和顶层 App。
- 适用范围：frontend。
- 建议 implement 跑通后走 `cs-decide` 归档为 `category: convention`。

##### 超出范围的观察

- `src-tauri/src/state.rs` 已偏长，但本次新增的是同类偏好读写。若后续继续增加偏好项，建议单独走 `cs-refactor` 拆出 preference store 模块，本 feature 不处理。

## 3. 验收契约

### 关键场景清单

1. **关键词搜索**：输入项目名、路径片段、summary 片段或 agent 名 → 列表只显示当前最近天数范围内的匹配 session。
2. **空结果**：输入无匹配关键词 → 列表区域显示空结果提示，控制栏仍可清空搜索或调整最近天数。
3. **搜索强制展开**：agent / project 原本折叠时输入命中关键词 → 命中的分组自动展开；清空搜索后恢复原折叠状态。
4. **键盘聚焦**：按 `Cmd/Ctrl+K` → 搜索输入获得焦点；输入内容不触发重新扫描。
5. **键盘启动**：搜索有结果时按上下键切换活跃匹配项，按 Enter → 复用现有 `launch_session(session.id)` 启动对应 session。
6. **忙碌保护**：目标 session 正在启动或删除时按 Enter → 不重复触发启动，状态提示保持明确。
7. **收藏项目**：点击项目 header 的收藏按钮 → 当前 `projectDir` 被标记为收藏，同 agent 内排在非收藏项目之前。
8. **收藏持久化**：收藏项目后刷新或重启 app → 收藏状态和排序仍保留。
9. **收藏失败回滚**：保存收藏偏好失败 → UI 回到保存前状态，并显示错误。
10. **回归主路径**：刷新、最近天数筛选、主题、终端选择、打开方式、右键删除、鼠标点击启动仍按原行为工作。

### 明确不做的反向核对项

- `src-tauri/src/scanner*` 和 `src-tauri/src/launcher.rs` 不应出现本 feature 改动。
- `SessionData` 不新增收藏字段；收藏是本机偏好，不是扫描事实。
- 不新增单条 session 收藏相关类型，如 `favoriteSessionIds`。
- 不新增网络请求、云同步、账号或导入导出入口。
- 不新增全局系统快捷键权限或 Tauri global shortcut 插件。
- 不引入全文搜索 / 模糊搜索第三方依赖。

### 验证方式

- 必跑：`pnpm build`。
- 必跑：`cd src-tauri && cargo test --lib`。
- 前端 smoke：覆盖关键词搜索、空结果、收藏切换、收藏重启保留、键盘启动、搜索清空后的折叠恢复。
- 回归 smoke：覆盖刷新、最近天数筛选、右键删除取消、普通启动按钮。

## 4. 与项目级架构文档的关系

本 feature 不改变扫描、删除或启动的系统级 Rust 主流程；搜索是前端派生视图，收藏是本机偏好能力。

验收通过后建议在 `.codestable/architecture/ARCHITECTURE.md` 补充两点：

- 前端数据流里增加“最近天数过滤 → 搜索过滤 → 收藏排序 → agent/project 渲染”的派生视图说明。
- 偏好列表里补充 `favoriteProjectDirs` 与 terminal / launch mode / theme 同属本机 `preferences.json` 偏好。
