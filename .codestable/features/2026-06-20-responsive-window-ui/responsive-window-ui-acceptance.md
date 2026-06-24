# responsive-window-ui 验收报告

> 阶段：阶段 3（验收闭环）
> 验收日期：2026-06-20
> 关联方案 doc：`.codestable/features/2026-06-20-responsive-window-ui/responsive-window-ui-design.md`

## 0. 验收阶段补修记录

- [x] **无补修**：验收未发现方案 / 实现偏差，无需补修。
- [x] **终审后补修**：用户发现 app 右键会出现 Reload / Inspect Element。根因是 Tauri WebView 默认上下文菜单和 devtools 暴露调试入口；已在 `src-tauri/tauri.conf.json` 显式设置 `devtools: false`，并在 `src/main.tsx` 阻止默认 `contextmenu`。复验：`pnpm build`、`pnpm tauri build --debug` 通过，Playwright 事件检查 `defaultPrevented: true`。
- [x] **状态规范化**：checklist checks 接手时为 30 条 `pending`，逐项核对后规范化为 30 条全 `passed`。
- [x] **工具失败分类**：首次 Playwright 脚本因 `evaluate` 传参方式错误失败，属于命令参数错误；修正为对象参数后复验通过，不计为应用失败。

## 1. 接口契约核对

### 接口示例逐项核对

- [x] 本 feature 无新增 TypeScript / Rust 对外接口示例；设计中的窗口适配示例已由 CSS / Tauri 配置落地并在第 3 节场景核对。

### 名词层"现状 → 变化"逐项核对

- [x] 窗口尺寸契约：`src-tauri/tauri.conf.json` 保留 `width: 800`、`height: 600`，新增 `minWidth: 360`、`minHeight: 420`。
- [x] 断点语义：`src/styles/responsive.css` 保留 compact `max-width: 560px`，新增 wide `min-width: 900px`。
- [x] app shell：`src/styles/base.css` 由固定 `max-width: 720px` 改为响应式宽度，800px 视口 shell 为 720px，1100px 视口 shell 为 1020px。
- [x] 控制栏：`src/styles/controls.css` 补充控件最小宽高，`src/styles/responsive.css` 在 compact 下让筛选、segmented、终端、主题稳定换行。
- [x] 列表密度：`src/styles/responsive.css` 在 compact 下收敛 agent header、project header、session row，在 wide 下放宽列表间距和内容宽度。

### 流程图核对

- [x] Tauri window config → `src-tauri/tauri.conf.json`。
- [x] React 同一组件树 → `src/App.tsx` 无 diff，未新增响应式状态。
- [x] CSS responsive 判断窗口宽度 → `src/styles/responsive.css`。
- [x] compact / regular / wide 三档 → compact `max-width: 560px`，regular 默认规则，wide `min-width: 900px`。

## 2. 行为与决策核对

### 需求摘要逐项验证

- [x] 默认窗口保持工作台式界面：800x600 Playwright smoke 中 shell 为 720px，控制栏单行，agent 默认折叠。
- [x] 窄窗口不横向溢出：360x520 / 560x600 自动化检查 overflow 为 0。
- [x] 宽窗口利用更多空间：1100x720 自动化检查 shell 为 1020px，长路径和 session 简介获得更宽区域。
- [x] 最小窗口边界：Tauri debug build 接受 `minWidth: 360` / `minHeight: 420` 配置。

### 明确不做逐项核对

- [x] 不改扫描、启动、偏好持久化和 Rust 后端业务逻辑：`git diff -- src-tauri/src` 无输出。
- [x] 不做移动端网页适配，不引入浏览器路由或独立 mobile 页面：无路由、页面或移动端入口新增。
- [x] 不改变主题选择控件形态，不重新做视觉风格：`src/components/Controls.tsx` 无 diff，主题仍为下拉菜单。
- [x] 不新增新 agent、不改变 session 展示字段含义：`src/types.ts` 无 diff。
- [x] 不把宽屏改成复杂 dashboard：宽屏仍是 agent 纵向排列。

### 关键决策落地

- [x] CSS 断点和现有组件结构优先：只改配置和样式，未新增 JS resize 逻辑。
- [x] Tauri 最小窗口尺寸作为布局契约：`tauri.conf.json` 已声明最小宽高。
- [x] 宽屏只提高信息承载：wide 规则只调整 shell、列表边距和 session 内容间距，未改变层级。

### 编排层"现状 → 变化"逐项核对

- [x] Tauri 层限制最小窗口：`minWidth` / `minHeight` 已落地。
- [x] React 层不感知窗口宽度：`src/App.tsx` 无 diff，未出现 resize state / listener。
- [x] CSS 三档布局变化：regular 默认，compact 收敛，wide 扩展。

### 流程级约束核对

- [x] 不用 JS 监听 resize：`rg "resize|ResizeObserver|matchMedia" src` 无新增命中。
- [x] 控件可点击：Playwright 验证四个视口控制项最低高度 40px。
- [x] 文本不覆盖按钮：Playwright 验证 session 文本与启动按钮最小间距 compact 为 8px、regular/wide 为 12px。
- [x] 宽屏不改变 agent 排序和默认折叠状态：页面刷新后 codex / claude-code / cursor 均为 `data-open="false"`。

### 挂载点反向核对

- [x] `src-tauri/tauri.conf.json`：新增最小窗口尺寸，删除该项会移除窗口边界能力。
- [x] `src/styles/base.css`：shell 宽度与边距适配，删除该项会恢复固定 720px 上限。
- [x] `src/styles/controls.css`：控制栏控件命中区，删除该项会降低 compact 可点稳定性。
- [x] `src/styles/session-list.css`：本次未直接修改；列表响应式通过 `responsive.css` 覆盖实现，未产生清单外挂载点。
- [x] `src/styles/responsive.css`：compact / wide 规则汇总，删除该项会移除主要响应式行为。
- [x] 反向核查：`git diff --name-only` 仅命中 `tauri.conf.json`、`base.css`、`controls.css`、`responsive.css` 和 CodeStable 文档产物，无清单外代码挂载点。
- [x] 拔除沙盘推演：还原上述配置和样式改动后，响应式窗口 UI 能力即消失，无残留业务状态或数据迁移。

## 3. 验收场景核对

- [x] **S1 最小窗口边界**：`pnpm tauri build --debug` 通过，说明 Tauri 配置字段有效；真实拖拽边界留给用户 GUI 终审。
- [x] **S2 compact 布局**：Playwright 360x520，overflow 0，shell 336px，控制项最低 40px，session 按钮间距 8px。
- [x] **S3 断点边界**：Playwright 560x600，overflow 0，shell 536px，控制项最低 40px，session 按钮间距 8px。
- [x] **S4 默认窗口**：Playwright 800x600，overflow 0，shell 720px，agent 默认折叠，控制栏单行。
- [x] **S5 wide 布局**：Playwright 1100x720，overflow 0，shell 1020px，session 按钮间距 12px。
- [x] **S6 主题兼容**：黑 / 白 / 跟随系统在 compact 和 wide 下 overflow 均为 0，body/text 颜色按 token 切换。
- [x] **S7 功能不回归**：最近天数筛选切到 30 天、刷新返回"已加载 4 个 session"、agent/project 可展开、启动按钮返回"终端启动成功"。

## 4. 术语一致性与禁用词反向 grep

- [x] `responsive-window-ui` 只作为 feature / requirement slug 和文档标题使用，未新增业务类型。
- [x] `compact` / `regular` / `wide` 只在设计、验收和 CSS 断点语义中使用；代码未新增同名数据模型。
- [x] `可视安全区` 未落为业务概念，只由 shell 宽度、padding 和断点规则表达。
- [x] design 未定义禁用词列表，跳过禁用词反向 grep。

## 5. 架构归并

- [x] `.codestable/architecture/ARCHITECTURE.md`：不需要更新。理由：本 feature 不新增模块、不改变 React ↔ Tauri ↔ Rust 交互、不改变扫描/启动/状态数据流；只新增 Tauri 窗口尺寸契约和前端 CSS 响应式规则。
- [x] 长期断点约定已有稳定价值，但属于前端 UI convention，不直接写入系统架构；如需长期约束，后续可走 `cs-decide` 归档。

## 6. requirement 回写

- [x] `requirement` 原为空，且本次是用户可感能力，已走 backfill。
- [x] 新增 `.codestable/requirements/responsive-window-ui.md`，状态为 `current`。
- [x] 新增 `.codestable/requirements/VISION.md`，索引当前能力。
- [x] 已回填 design frontmatter：`requirement: responsive-window-ui`。

## 7. roadmap 回写

- [x] 非 roadmap 起头：design frontmatter 无 `roadmap` / `roadmap_item` 字段，跳过 roadmap 回写。

## 8. attention.md 候选盘点

- [x] 无候选：本 feature 未暴露需要补入 `attention.md` 的新环境 / 工具 / 工作流信息。已有 `pnpm build`、`pnpm tauri dev`、`pnpm tauri build` 相关路径在 attention 中已覆盖。

## 9. 遗留

- 后续优化点：如后续要做宽屏 dashboard 或多列工作台，建议另起 feature；当前不纳入本次范围。
- 已知限制：真实 Tauri GUI 拖拽最小尺寸需要用户在本机窗口中终审确认。
- 实现阶段顺手发现：无。

## 10. 用户终审

- [x] 用户已于 2026-06-20 确认验收结果。

## 验证命令与结果

- `pnpm build`：通过。
- `pnpm tauri build --debug`：通过，产物位于 `src-tauri/target/debug/bundle/`。
- Playwright `contextmenu` 事件检查：通过，`defaultPrevented: true`，默认右键菜单被阻止。
- `python3 .codestable/tools/validate-yaml.py --file .codestable/features/2026-06-20-responsive-window-ui/responsive-window-ui-design.md`：通过。
- `python3 .codestable/tools/validate-yaml.py --file .codestable/features/2026-06-20-responsive-window-ui/responsive-window-ui-checklist.yaml --yaml-only`：通过。
- Playwright mock Tauri IPC smoke：通过，覆盖 360x520 / 560x600 / 800x600 / 1100x720、三种主题和基础交互。
