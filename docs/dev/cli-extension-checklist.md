# CLI 扩展 Checklist

新增 `CliType` 时按下列挂点补齐（`cargo test --lib` 中 `cli_contract` 会校验注册完整性）。

## 必须挂点（7）

1. **`CliType` 枚举**（`src-tauri/src/models.rs` + `src/types.ts` + `CLI_LABELS` / `CLI_ORDER`）
2. **Scanner 实现**并在 `scanners()` 注册（`src-tauri/src/scanner.rs`）
3. **`command_spec_for_session`** 分支（program + resume args 形状）
4. **`security::ALLOWED_PROGRAMS` + `validate_resume_args`**
5. **删除路径**：`delete_target` 形态；OpenCode 必须走 SQLite **行**删除，禁止只删 db 文件
6. **源探测**：`session_source::check_session_source` 语义（File / Directory / SqliteRow）
7. **文档**：本 checklist + 用户文档 CLI 列表（如有）

## 可选 / 关联

- 启动预检 / 健康探测自动覆盖（复用共享源探测）
- 前端品牌图标 / 空态文案
- fixture 单测（不依赖真实 home 数据）

## 禁止

- 动态 `load_plugin` / dylib 插件运行时
- 前端暴露 `delete_target` 真实路径
- 用 `std::env::PATH` 单独解析 program 而与 launcher login PATH 分叉
