---
doc_type: feature-design
feature: 2026-06-20-delete-session-context-menu
requirement: delete-session-context-menu
status: accepted
summary: 在单条 session 行上右键打开菜单，确认后删除该 session 在对应 CLI 存储中的源文件或目录，并从当前列表移除。
tags: [frontend, tauri-command, session-list, destructive-action]
---

# delete-session-context-menu 设计文档

## 0. 术语约定

| 术语 | 定义 | 防冲突结论 |
|---|---|---|
| session 源载体 | 扫描器读到一条 session 时对应的真实本地文件或目录 | 新增后端内部名词，不暴露到前端 JSON |
| 删除 session | 删除选中 session 的源载体，并从当前 UI 列表移除 | 不是删除项目工作目录，也不是删除所有同名历史 |
| 右键菜单 | 用户在单条 `SessionRow` 上触发的上下文菜单 | 只挂在 session 行，不挂在 agent 或 project header |

## 1. 决策与约束

### 需求摘要

用户目标：右键点击单条 session，可以选择删除这条 session，并从相应文件目录中删除它。

核心行为：

- 用户在某一条 session 行右键，看到“删除此 session”操作。
- 用户确认后，后端删除该 session 对应的本地存储载体。
- 删除成功后，当前列表立即移除这条 session；下次刷新也不会再扫到它。
- 删除失败时前端显示明确错误，不从列表假移除。

成功标准：

- Codex session 删除后，对应 `~/.codex/sessions/**/*.jsonl` 源文件不存在。
- Claude Code session 删除后，对应 `~/.claude/projects/<encoded-project>/<uuid>.jsonl` 源文件不存在。
- Cursor session 删除后，对应 `~/.cursor/chats/<hash>/<uuid>/` chat 目录不存在。
- 右键只作用于被点击的单条 session，不影响同一项目下其他 session。
- `pnpm build` 和 `cd src-tauri && cargo test --lib` 通过。

明确不做：

- 不删除 `project_dir` 指向的工作目录。
- 不删除 CLI 全局目录、账号配置、缓存目录或其他 session。
- 不做批量删除、按项目删除、按 agent 删除。
- 不做撤销、回收站或归档；删除是不可恢复动作，必须先确认。
- 不把源文件路径序列化给前端展示或调试输出。
- 不在删除失败时静默刷新或假装成功。

### 假设

- 假设用户接受删除前二次确认；这是本地文件删除，误触成本高，不能右键后直接删。
- 假设删除目标以本次扫描命中的源载体为准。Codex 若存在多个文件记录同一个 `session_id`，本 feature 只删除列表当前这条对应的源文件，不做同 ID 全量清理。

### 复杂度档位

走本地桌面工具默认档位，但安全性上调到 `validated`：

- 删除目标来自外部 CLI 本地存储，属于不可信输入，必须校验路径仍位于对应 CLI 存储根目录下。
- 删除动作是 destructive action，必须显式失败、显式确认，不允许隐藏 fallback。

### 关键决策

1. **删除只通过后端 Tauri command 执行**
   前端只传 `session.id`。真实路径留在 Rust 内存态，避免把用户本机路径作为前端可见契约扩散。

2. **扫描器负责产出删除目标**
   扫描器最清楚一条 session 来自哪个文件或目录。`Session` 增加后端内部 `delete_target`，用 serde skip，不改变前端 `SessionData` 字段。

3. **删除后更新内存缓存，不强制全量扫描**
   成功删除源载体后，`AppState` 从 `sessions` 缓存移除该 id 并返回新的 `ScanResponse`。这样反馈快；用户点刷新时仍按磁盘重新扫描。

4. **路径校验必须按 CLI 根目录做白名单**
   删除前 canonicalize 源载体和 CLI 根目录，确认源载体在根目录内；Codex / Claude 只允许删文件，Cursor 只允许删目录。

## 2. 名词与编排

### 2.1 名词层

#### 现状

- `Session` 定义在 `src-tauri/src/models.rs`，当前字段包括 `id / cli_type / session_id / project_dir / project_name / last_active_at / summary`，会序列化给前端。
- Codex 扫描器在 `src-tauri/src/scanner/codex.rs` 递归解析 `~/.codex/sessions/**/*.jsonl`，但解析结果没有保留源文件路径。
- Claude Code 扫描器在 `src-tauri/src/scanner/claude_code.rs` 遍历 `~/.claude/projects/<encoded-project>/*.jsonl`，但解析结果没有保留源文件路径。
- Cursor 扫描器在 `src-tauri/src/scanner/cursor.rs` 遍历 `~/.cursor/chats/<hash>/<uuid>/`，但解析结果没有保留 chat 目录路径。
- `commands.rs` 只有 `scan_sessions / refresh_sessions / launch_session / preferences`，没有删除 command。
- 前端 `SessionRow` 只有启动按钮，没有右键菜单或删除确认状态。

#### 变化

- **新增内部删除目标类型**：

```rust
// 来源：src-tauri/src/models.rs，字段用 serde skip，不进入前端 JSON
pub enum SessionDeleteKind {
    File,
    Directory,
}

pub struct SessionDeleteTarget {
    pub root: PathBuf,
    pub path: PathBuf,
    pub kind: SessionDeleteKind,
}
```

- **Session 增加内部字段**：`delete_target: Option<SessionDeleteTarget>`，不序列化给前端。三家 scanner 都必须为真实可删除 session 填充。
- **新增 Tauri command**：`delete_session(session_id: String) -> Result<ScanResponse, String>`。参数沿用前端已有的 `SessionData.id`，不是 CLI 原始 `session_id`。
- **前端新增局部交互状态**：右键菜单状态、待删除 session、删除中 id。状态只影响 UI，不成为删除事实源。

#### 删除目标示例

| CLI | 输入 session | 删除目标 |
|---|---|---|
| Codex | `session.id = <ui-id>` | 扫描命中的某个 `~/.codex/sessions/.../*.jsonl` 文件 |
| Claude Code | `session.id = <ui-id>` | `~/.claude/projects/<encoded-project>/<uuid>.jsonl` 文件 |
| Cursor | `session.id = <ui-id>` | `~/.cursor/chats/<hash>/<uuid>/` 目录 |

### 2.2 编排层

```mermaid
flowchart TD
    A[用户右键 SessionRow] --> B[前端打开行级上下文菜单]
    B --> C[用户点 删除此 session]
    C --> D[前端显示确认弹窗]
    D -- 取消 --> E[关闭弹窗，不调用后端]
    D -- 确认 --> F[invoke delete_session(session.id)]
    F --> G[AppState find_session]
    G --> H[validate delete_target root/path/kind]
    H --> I{目标有效?}
    I -- 否 --> J[返回明确错误]
    I -- 是 --> K[删除文件或目录]
    K --> L[从 AppState.sessions 移除该 id]
    L --> M[返回新的 ScanResponse]
    M --> N[前端更新列表和状态提示]
```

#### 现状

- 扫描流程已经由 `scan_sessions` / `refresh_sessions` 返回 `ScanResponse`。
- 启动流程通过 `launch_session(session.id)` 找缓存中的 `Session`，再生成 `CommandSpec`。
- 前端 session 列表由 `App` 维护，`AgentGroup -> ProjectBucket -> SessionRow` 逐层传 `onLaunch`。

#### 变化

- 扫描流程多记录删除目标，但返回给前端的 JSON 契约不变。
- 删除流程复用 `AppState.find_session` 的缓存查找语义；找到后只执行删除，不进入 launcher。
- 前端给 `SessionRow` 增加 `onContextMenu` 和 `onDeleteRequest`，`App` 负责统一确认、调用 `delete_session`、接收 `ScanResponse` 更新状态。

#### 流程级约束

- 删除命令必须先校验目标路径仍存在、仍在对应 CLI root 下、文件/目录类型匹配。
- 删除失败不得从前端列表移除该 session。
- 删除成功后只移除当前 `session.id`；不按 `session_id` 模糊删除其他缓存项。
- 删除 command 不输出真实路径到错误消息；用户可见错误只描述“删除目标不存在 / 不在允许目录内 / 删除失败”。
- 正在启动中的 session 不允许同时删除；前端按钮和菜单项按 `launchingId` / `deletingId` 禁用。

### 2.3 挂载点清单

| 挂载位置 | 动作 | 删除后效果 |
|---|---|---|
| `Session.delete_target` 内部字段 | 记录源载体 root/path/kind | 后端无法知道应删哪个文件或目录 |
| 三个 scanner 的 session 构造点 | 填充各自删除目标 | 某 CLI 的 session 无法删除或会报缺少目标 |
| `delete_session` Tauri command | 执行校验、删除和缓存更新 | 前端没有安全删除入口 |
| `SessionRow` 右键菜单 | 暴露单条 session 的删除入口 | 用户无法从列表触发删除 |
| `App` 删除确认与结果接入 | 管理确认、调用 command、更新列表 | 删除操作缺少二次确认和状态反馈 |

### 2.4 推进策略

1. **删除目标名词接入**
   退出信号：`Session` 能携带不序列化的删除目标，前端 `SessionData` 字段不变。

2. **扫描器填充删除目标**
   退出信号：Codex / Claude / Cursor fixture 单测能断言删除目标分别指向文件或目录。

3. **后端删除 command**
   退出信号：删除文件、删除目录、路径越界、目标不存在四类单测覆盖；删除成功返回移除后的 `ScanResponse`。

4. **前端右键菜单和确认流**
   退出信号：右键单条 session 出现菜单；取消不调用后端；确认调用 `delete_session` 并按返回结果更新列表。

5. **联调与回归验证**
   退出信号：`pnpm build`、`cd src-tauri && cargo test --lib` 通过；本地 app 中至少对一个 fixture 或真实可丢弃 session 完成删除 smoke。

### 2.5 结构健康度与微重构

#### convention 检索

已执行：

```bash
python3 .codestable/tools/search-yaml.py --dir .codestable/compound --query "目录组织 文件归属 命名约定 session 删除 storage scanner"
python3 .codestable/tools/search-yaml.py --dir .codestable/features --query "session list context menu delete remove scanner source path"
```

结果：未命中删除 session、右键菜单或 scanner 删除目标相关 convention / 历史 design。

#### 文件级评估

- `src-tauri/src/models.rs`：当前约 70 行，适合新增小型内部删除目标类型。
- `src-tauri/src/scanner.rs`：当前约 120 行，适合放删除目标辅助校验入口或 trait 级共享类型；真正删除逻辑不应塞进 scanner。
- `src-tauri/src/state.rs`：当前约 230 行，接近 300 行；新增删除编排会继续增长，建议把文件删除校验/执行拆到新模块。
- `src-tauri/src/commands.rs` / `lib.rs`：都是 command 挂载层，新增一个 command 属于正常扩展。
- `src/components/SessionRow.tsx`：当前很小，右键事件入口可放这里，但菜单和确认弹窗不应全部塞进行组件。
- `src/App.tsx`：当前约 270 行，继续追加删除状态仍可接受，但需避免把菜单渲染细节写成长块。

#### 目录级评估

- `src-tauri/src/` 已按 models / scanner / launcher / state / commands / security 分层；删除本地文件属于新的 destructive file operation，不完全属于 scanner 或 launcher。
- `src/components/` 已有行、桶、分组组件；适合新增 `SessionContextMenu` / `ConfirmDialog` 这类展示组件。
- `src/styles/` 已按列表与控件拆分；删除菜单样式可放 `session-list.css` 或新建轻量 `menu.css`，实现期以行数控制决定。

#### 结论：做微重构（新增后端删除模块，不搬旧行为）

本次不移动现有文件，但建议新增 `src-tauri/src/session_delete.rs` 承载删除目标校验和删除执行，避免把 `state.rs` 推成混合“状态 + 文件系统危险操作”文件。

- 搬什么：不搬既有代码；新增删除校验/执行节点。
- 放到哪：`src-tauri/src/session_delete.rs`。
- 怎么验证行为不变：新增模块前后，现有扫描和启动单测保持通过；删除模块单测只覆盖新增行为。

#### 超出范围的观察

如果后续要做“批量删除 / 按项目删除 / 删除前预览占用空间”，需要单独设计批量操作和更强确认机制，不应塞进本 feature。

## 3. 验收契约

### 关键场景清单

1. **右键菜单出现**：右键某条 session 行 → 只在该行附近出现“删除此 session”，左键其他区域或 Esc 关闭。
2. **取消确认**：右键删除 → 弹出确认 → 点击取消 → 不调用 `delete_session`，列表不变。
3. **Codex 删除**：选择 Codex session → 确认删除 → 对应源 `jsonl` 文件被删除，当前列表少一条。
4. **Claude 删除**：选择 Claude Code session → 确认删除 → 对应源 `jsonl` 文件被删除，当前列表少一条。
5. **Cursor 删除**：选择 Cursor session → 确认删除 → 对应 chat 目录被删除，当前列表少一条。
6. **错误路径保护**：删除目标不在对应 CLI root 下 → 后端返回错误，不执行删除，不移除列表项。
7. **目标不存在**：缓存里有 session 但源文件/目录已被外部删除 → 后端返回明确错误，提示用户刷新，不假成功。
8. **启动并发保护**：某 session 正在启动时 → 该行删除入口不可用或确认按钮禁用。
9. **刷新一致性**：删除成功后点击刷新 → 被删除 session 不再出现；其他 session 排序和分组保持。

### 明确不做的反向核对项

- `project_dir` 指向的工作目录不能被删除。
- 不能按 `session_id` 删除多个同 ID 文件；只删扫描命中的源载体。
- 不能把 `delete_target.path` 加到前端 `SessionData`。
- 不能在 `launcher.rs` 里加入删除逻辑。
- 不能把删除失败处理成成功状态提示。
- 不新增网络、云同步、回收站或撤销能力。

### 验证方式

- Rust：`cd src-tauri && cargo test --lib`，重点覆盖删除目标校验、文件删除、目录删除、越界拦截、目标不存在。
- 前端：`pnpm build`，覆盖 TypeScript 契约和构建。
- Smoke：运行 Tauri dev，选一条可丢弃 session 右键删除，确认 UI 移除；随后刷新验证不再出现。

## 4. 与项目级架构文档的关系

本 feature 改变架构文档中的核心命令集合：新增 `delete_session` Tauri command，并让 scanner 记录“不序列化给前端的源载体”。实现验收通过后，应在 `.codestable/architecture/ARCHITECTURE.md` 补充：

- 核心模块里新增 `session_delete.rs` 的职责。
- 数据流里补一条“右键删除 session → delete_session → 校验源载体 → 删除文件/目录 → 更新缓存”的本地 destructive 操作链路。
- 安全口径里补充删除目标路径校验要求。
