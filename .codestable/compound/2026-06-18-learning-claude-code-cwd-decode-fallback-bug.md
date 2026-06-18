---
doc_type: learning
track: pitfall
slug: claude-code-cwd-decode-fallback-bug
title: claude-code session 的 cwd 别信目录名 decode——优先读 jsonl，否则被 encode 歧义 + fallback 覆盖坑
component: scanner
tags: [claude-code, session-parsing, cwd, encode, fallback]
severity: high
created: 2026-06-18
source: feature 2026-06-17-cli-session-launcher
---

# claude-code session 的 cwd 解析：别信目录名 decode

做 Session Launcher 扫描 claude-code 的 session 时，一批 session 启动报"工作目录不存在"，根因是 cwd 解析逻辑同时踩了**编码歧义**和**fallback 覆盖真实值**两个坑。

## 现象

扫描 `~/.claude/projects/` 下的 claude session，对含 `-` 或 `.` 的项目目录（如 `fast-start`、`ai-test`、`.coze`）启动时报"工作目录不存在"。`validate_cwd` 的 `canonicalize` 失败。

## 根因 1：目录名 encode 有歧义，无法无损反解

claude-code 把项目 cwd 编码成目录名：`/` → `-`。所以：

| 真实 cwd | 编码后 |
|---|---|
| `/Users/xb/Desktop/codes/fast-start` | `-Users-xb-Desktop-codes-fast-start` |
| `/Users/xb/Desktop/codes/ai-test` | `-Users-xb-Desktop-codes-ai-test` |
| `/Users/xb/.coze/agents/123/workspace` | `-Users-xb--coze-agents-123-workspace` |

`decode` 时把 `-` 全换回 `/`，**无法区分"路径分隔符的 `-`"和"目录名里原本的 `-`"**：

- `fast-start` → 错解成 `fast/start`
- `ai-test` → 错解成 `ai/test`
- `.coze`（`/` 后跟 `.`）→ `.` 还会丢，变成 `//coze`

所以**目录名 decode 不可信**，只能当粗略猜测。

## 根因 2：fallback 是 `Some` 导致 jsonl 真实 cwd 永不被读取（真正的 bug）

jsonl 文件**每一行都带 `cwd` 字段**（真实路径，无歧义）。解析函数本应"优先用 jsonl 的 cwd，decode 只作 fallback"：

```rust
// ❌ 出 bug 的写法
fn parse_claude_file(path, fallback_cwd: Option<PathBuf>) {
    let mut cwd = fallback_cwd;  // fallback_cwd 是 Some(解码结果)
    for line in content.lines() {
        ...
        if cwd.is_none() {        // ← 永远是 false！fallback 是 Some
            cwd = parsed.cwd.map(PathBuf::from);
        }
    }
}
```

`fallback_cwd` 从 decode 来的是 `Some`（哪怕解错了），`cwd.is_none()` 永不成立，**jsonl 里的真实 cwd 永远没机会被读取**。结果：所有 session 的 cwd 都用了错解的 decode 值，含 `-`/`.` 的目录全挂。

## 解法

让 fallback 真正只作 fallback——初始 `None`，jsonl 有真实值就用，没有才退回 decode：

```rust
// ✅ 正确写法
fn parse_claude_file(path, fallback_cwd: Option<PathBuf>) {
    let mut cwd: Option<PathBuf> = None;  // 初始 None，不是 fallback
    for line in content.lines() {
        ...
        if cwd.is_none() {
            cwd = parsed.cwd.map(PathBuf::from);  // 用 jsonl 真实值
        }
    }
    Ok((latest, cwd.or(fallback_cwd)))  // jsonl 没有才退回 decode fallback
}
```

## 验证修复

修复后，含 `-`/`.` 的项目目录 session 都能正确解析 cwd（来自 jsonl 真实值），启动正常。

## 下次怎么更早发现

- **怀疑编码方案时**：先看编码是否**可逆**。claude 的 `/`→`-` 把分隔符和合法字符混为一谈，注定不可逆 → 任何 decode 结果都不可信，必须找原文（jsonl）。
- **写"优先 A，fallback B"逻辑时**：检查 fallback 的初始值。如果 fallback 是 `Some`，`if X.is_none()` 这类条件会失效——典型的"fallback 反客为主"bug。更稳妥的写法是初始 `None` + 末尾 `.or(fallback)`。

## 相关文档

- feature design: `.codestable/features/2026-06-17-cli-session-launcher/2026-06-17-cli-session-launcher-design.md` 调研结论（claude-code 行）
- 代码：`src-tauri/src/scanner/claude_code.rs::parse_claude_file`
