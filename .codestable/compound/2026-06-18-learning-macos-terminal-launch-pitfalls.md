---
doc_type: learning
track: pitfall
slug: macos-terminal-launch-pitfalls
title: macOS 外部终端拉起的三个坑（Ghostty login 误报 / iTerm app 名 / Terminal 无法开 tab）
component: launcher
tags: [macos, ghostty, iterm2, terminal-app, applescript, osascript]
severity: medium
created: 2026-06-18
source: feature 2026-06-17-cli-session-launcher
---

# macOS 外部终端拉起的三个坑

做 Session Launcher（一个 Tauri 桌面应用，点一下在某外部终端里 cd 到目录并跑命令）时，三个终端各踩一个坑，每个都耗了多轮试错才搞清根因。记录下来，下次做任何"从代码拉起 macOS 外部终端并执行命令"的工具都不必重走。

## 背景

需求：应用点击 → 在用户选定的终端（Terminal.app / iTerm2 / Ghostty）里开新 tab 或窗口 → cd 到工作目录 → 执行 `codex resume <id>` 这类命令。三个终端的拉起机制各不相同，且都有反直觉的坑。

## 坑 1：Ghostty 的 `-e` / `--command` 经 `/usr/bin/login` 套壳，多词命令弹误报

### 现象

用 `open -na Ghostty.app --args -e codex resume <id>` 启动，Ghostty 窗口里命令**实际执行成功了**，但窗口顶部一直弹红色错误：

```
Ghostty failed to launch the requested command:
/usr/bin/login -flp xb codex resume <id>
```

### 试过但没用的路径

- **`+new-window` action**：报 `+new-window is not supported on this platform`——macOS 不支持从 CLI 跑这个 action
- **直接调 Ghostty 二进制**：`/Applications/Ghostty.app/Contents/MacOS/ghostty ...`——官方文档明说 macOS 上从 CLI 启动终端不支持，只能用 `open -na`
- **`--command` 替代 `-e`**：文档说走 `/bin/sh -c`，但 macOS 上**仍然套 login**（文档没写但实际如此），同样弹误报
- **`--input=<cmd>\n` 发按键**：命令执行了，但 login 误报仍在（因为只要启动就套 login）

### 根因

Ghostty 文档里藏了一句（`abnormal-command-exit-runtime` 配置项附近）："on macOS, we allow any exit code because of the way shell processes are launched via the login command"。即 **macOS 上 Ghostty 一定把命令包进 `/usr/bin/login -flp <user>`**，这是平台机制不是 bug。`login` 解析多词命令（`codex resume abc`）时，把 `resume`/`abc` 当成自己的参数（用户名等），解析失败 → 弹误报。但 PTY 仍执行了命令，所以命令本身能跑、只是误报扰人。

`-e` flag 还顺带设 `quit-after-last-window-closed=true`（agent 退出后 Ghostty 干净退出，不留孤儿进程），这点是有用的。

### 解法（wrapper 脚本）

不让 `-e`/`--command` 直接执行多词命令，而是执行**单个 wrapper 脚本路径**。`login` 看到的是单个可执行文件，不解析多词参数，无误报：

```rust
// 生成临时脚本 $TMPDIR/fast-start-ghostty/run-<pid>.sh (0700)：
//   #!/bin/sh
//   export PATH=<含 ~/.local/bin>
//   cd '<cwd>' && exec codex resume <id>
// 然后：open -na Ghostty.app --args -e <脚本路径>
```

顺带要点：
- **补 PATH**：Ghostty 的 tab 不走 login shell，默认 PATH 不含 `~/.local/bin`（codex/claude 装在那），会 `codex: not found`。wrapper 里显式 `export PATH`。
- **开 tab**：Ghostty 有窗口时，用 AppleScript `new tab with configuration {command:<wrapper>, initial working directory:<cwd>} in front window` 开 tab（见 Ghostty 的 sdef）。Ghostty 在 macOS 上无 CLI 方式给运行实例开 tab，但有 AppleScript 字典。

### 下次怎么更早发现

看到 "failed to launch: /usr/bin/login -flp" 这种误报，立刻知道是 login 套壳问题，直接上 wrapper 脚本，不要在 `-e`/`--command`/`--input` 之间反复试——它们都会套 login。

---

## 坑 2：iTerm2 的 AppleScript app 名是 `iTerm`，不是 `iTerm2`

### 现象

`tell application "iTerm2" ... create tab with default profile` 报：

```
syntax error: Expected end of line but found class name. (-2741)
```

### 根因

iTerm2 的 **bundle 名 / AppleScript app 名是 `iTerm`**（应用叫 iTerm2，但脚本里要 `tell application "iTerm"`）。写成 `iTerm2` 时 AppleScript **不加载 iTerm 的脚本字典**，`create tab` 里的 `tab` 就成了未知的 class name → 语法错。

另外两个 iTerm2 的坑：
- **`create tab` 必须在 `tell current window` 块内**，单行 `tell current window to create tab ...` 不被接受
- **冷启动时不能 `create window`**：会和 iTerm 自己启动时开的默认窗口叠加成两个。改为 `activate` 后轮询等默认窗口出现再复用

### 解法

```applescript
tell application "iTerm"
    activate
    if (count of windows) is 0 then
        repeat until (count of windows) > 0  -- 等 iTerm 自己开的默认窗口
            delay 0.1
        end repeat
    else
        tell current window
            create tab with default profile  -- 必须在 tell current window 块内
        end tell
    end if
    tell current session of current window
        write text "<命令>"
    end tell
end tell
```

### 下次怎么更早发现

"Expected end of line but found class name" + 对象是第三方 app → 九成是 app 名写错导致字典没加载。先确认 bundle 名（`defaults read .../Info.plist CFBundleName` 或试 `iTerm` 而非 `iTerm2`）。

---

## 坑 3：Terminal.app 无法从 AppleScript 干净地开新 tab

### 现象

想让 Terminal 像 iTerm2/Ghostty 那样在已有窗口开 tab，试遍所有已知方法都不行。

### 试过但没用的路径

| 方法 | 结果 |
|---|---|
| `do script "cmd" in front window` | 在**当前 tab** 里追加执行（覆盖），不开新 tab |
| `do script "cmd" in window 1` | 同上，覆盖当前 tab |
| `do script "cmd" in (selected tab of front window)` | 复用当前 tab，不开新 tab |
| `make new tab at end of tabs` | `execution error: AppleEvent handler failed (-10000)`——Terminal 字典**不支持** make new tab |
| `tell front window: make new tab` | 同样不支持 |
| System Events 模拟 ⌘T | `osascript is not allowed to send keystrokes`——需要**辅助功能权限**，从 app 调用也一样卡权限墙 |

### 根因

Terminal.app 的 AppleScript 字典**没有"开新 tab"的命令**。`do script` 的行为：不带 `in` → 开新窗口；带 `in <window>` → 在该窗口**当前 tab** 执行（不开新 tab）；带 `in <tab>` → 在该 tab 执行。网上流传的"`in window X` 开新 tab"只在某些旧 macOS 版本成立，当前版本（Darwin 24.x）是覆盖。

### 解法（接受限制）

Terminal 无法开 tab，保持最简单的 `do script`（开新窗口）。**冷启动会多一个空默认窗口**——Terminal 启动时自己开一个 + 我们的 `do script` 开一个，共两个。这无法从 AppleScript 侧可靠解决（`close window`/`busy` 判断都试过，会随机关错窗口、把含命令的窗口关掉）。命令窗口一定存在，多余空窗口由用户手动关。

### 下次怎么更早发现

要做 Terminal.app 开 tab 的功能时，直接认定"做不到"，别浪费时间试 make new tab / do script in。需要 tab 体验就引导用户用 iTerm2 或 Ghostty。

---

## 总结：macOS 三终端拉起对照

| 终端 | 开 tab | 开窗口 | 关键机制 |
|---|---|---|---|
| iTerm2 | ✅ `create tab`（app 名 `iTerm`，块内） | ✅ 冷启动复用默认窗口 | osascript |
| Ghostty | ✅ AppleScript `new tab with configuration` | ✅ `open -na -e <wrapper>` | AppleScript 字典 + wrapper 脚本（规避 login 误报）|
| Terminal.app | ❌ 不可能 | ✅ `do script`（冷启动多一个空窗口） | osascript，硬限制 |

## 相关文档

- feature design: `.codestable/features/2026-06-17-cli-session-launcher/2026-06-17-cli-session-launcher-design.md` 2.1 节
- 架构：`.codestable/architecture/ARCHITECTURE.md` "终端拉起策略"节
