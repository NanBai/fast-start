use crate::models::{CommandSpec, LaunchMode, TerminalType};
use crate::security::{applescript_string, shell_escape, validate_command_spec};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub enum LaunchError {
    Validation(String),
    Spawn(String),
}

impl LaunchError {
    pub fn message(&self) -> String {
        match self {
            LaunchError::Validation(msg) => msg.clone(),
            LaunchError::Spawn(msg) => msg.clone(),
        }
    }
}

pub trait TerminalLauncher {
    fn terminal_type(&self) -> TerminalType;
    /// 探测该终端是否可用（如 Ghostty 未安装则 false）
    fn is_available(&self) -> bool;
    /// 该终端是否支持在已有窗口开新 tab。Terminal.app 不支持（AppleScript 硬限制）。
    fn supports_tab(&self) -> bool {
        true
    }
    /// 按 mode 开窗口或 tab：cd 到 spec.cwd 并执行 spec 程序。
    /// mode=NewTab 但终端不支持 tab（或无窗口可挂）时，实现自行回退到开窗口。
    fn launch(&self, spec: &CommandSpec, mode: LaunchMode) -> Result<(), LaunchError>;
}

pub fn launchers() -> Vec<Box<dyn TerminalLauncher + Send + Sync>> {
    vec![
        Box::new(SystemTerminalLauncher),
        Box::new(ITerm2Launcher),
        Box::new(GhosttyLauncher),
    ]
}

pub fn launcher_for(terminal: TerminalType) -> Option<Box<dyn TerminalLauncher + Send + Sync>> {
    launchers()
        .into_iter()
        .find(|launcher| launcher.terminal_type() == terminal)
}

fn build_shell_command(spec: &CommandSpec, cwd: &Path) -> String {
    let mut parts = vec![
        "cd".to_string(),
        shell_escape(&cwd.to_string_lossy()),
        "&&".to_string(),
        shell_escape(&spec.program),
    ];
    for arg in &spec.args {
        parts.push(shell_escape(arg));
    }
    parts.join(" ")
}

fn terminal_applescript(shell_command: &str) -> String {
    // Terminal.app 的 `do script` 无法干净地开新 tab：`make new tab` 字典不支持，
    // `do script in <tab/window>` 无法稳定复用（时序竞态），模拟 ⌘T 又需要
    // 辅助功能权限。因此 Terminal 多次启动时每次开新窗口（无法像 Ghostty/iTerm2
    // 那样堆 tab），这是 Terminal AppleScript 的硬限制。
    //
    // 冷启动时 `do script` 会让 Terminal 启动时多开一个空的默认窗口（无法从
    // AppleScript 侧避免，`activate` 与否都一样；尝试 `close window` 会误关含
    // 命令的窗口，风险更大）。因此保持最简单的 `do script`——保证含命令的窗口
    // 一定存在，多余的空窗口由用户手动关。Ghostty / iTerm2 才有干净的开 tab 体验。
    format!(
        "tell application \"Terminal\"\n\
         \x20\x20activate\n\
         \x20\x20do script {}\n\
         end tell",
        applescript_string(shell_command),
    )
}

/// iTerm2：有窗口开新 tab，无窗口（冷启动）等默认窗口出现并复用。
/// 注意：
/// - app 名是 "iTerm"（bundle 名），不是 "iTerm2"——用 "iTerm2" 不加载字典，
///   `create tab` 的 `tab` 会被当成未知 class name 报语法错。
/// - 冷启动不能 `create window`：会和 iTerm 自己启动时开的默认窗口叠加成两个。
/// - `create tab` 必须在 `tell current window` 块内（单行 tell ... to create 不被接受）。
fn iterm_open_tab_applescript(shell_command: &str) -> String {
    format!(
        "tell application \"iTerm\"\n\
         \x20\x20activate\n\
         \x20\x20if (count of windows) is 0 then\n\
         \x20\x20\x20\x20repeat until (count of windows) > 0\n\
         \x20\x20\x20\x20\x20\x20delay 0.1\n\
         \x20\x20\x20\x20end repeat\n\
         \x20\x20else\n\
         \x20\x20\x20\x20tell current window\n\
         \x20\x20\x20\x20\x20\x20create tab with default profile\n\
         \x20\x20\x20\x20end tell\n\
         \x20\x20end if\n\
         \x20\x20tell current session of current window\n\
         \x20\x20\x20\x20write text {}\n\
         \x20\x20end tell\n\
         end tell",
        applescript_string(shell_command)
    )
}

/// iTerm2：开新窗口。有窗口时 `create window`；冷启动（无窗口）时 iTerm 自己会开
/// 一个默认窗口，此时若再 `create window` 会叠加成两个——改为等默认窗口出现并复用。
fn iterm_open_window_applescript(shell_command: &str) -> String {
    format!(
        "tell application \"iTerm\"\n\
         \x20\x20activate\n\
         \x20\x20if (count of windows) is 0 then\n\
         \x20\x20\x20\x20repeat until (count of windows) > 0\n\
         \x20\x20\x20\x20\x20\x20delay 0.1\n\
         \x20\x20\x20\x20end repeat\n\
         \x20\x20else\n\
         \x20\x20\x20\x20create window with default profile\n\
         \x20\x20end if\n\
         \x20\x20tell current session of current window\n\
         \x20\x20\x20\x20write text {}\n\
         \x20\x20end tell\n\
         end tell",
        applescript_string(shell_command)
    )
}

fn run_osascript(script: &str) -> Result<(), LaunchError> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|err| LaunchError::Spawn(err.to_string()))?;

    command_output_result(output, "osascript")
}

fn command_output_result(
    output: std::process::Output,
    command_name: &str,
) -> Result<(), LaunchError> {
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err(LaunchError::Spawn(format!("{command_name} 启动失败")))
    } else {
        Err(LaunchError::Spawn(stderr))
    }
}

/// 为 Ghostty 生成一个临时 wrapper 脚本，返回脚本路径。
///
/// 为什么用 wrapper 而非直接 `-e <program> <args>`：
/// Ghostty 在 macOS 上把 `-e`/`--command` 的命令套进 `/usr/bin/login -flp`，
/// 多词命令（`codex resume <id>`）会让 login 解析失败，弹
/// "failed to launch the requested command" 误报。让 `-e` 只执行单脚本路径，
/// login 看到的是单个可执行文件，不会误报。
///
/// 同时 `-e` 自动设 `quit-after-last-window-closed=true`，agent 退出后
/// Ghostty 进程干净退出，不留孤儿（也不触发窗口复活）。
fn write_ghostty_wrapper(spec: &CommandSpec, cwd: &Path) -> Result<PathBuf, LaunchError> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let mut command = shell_escape(&spec.program);
    for arg in &spec.args {
        command.push(' ');
        command.push_str(&shell_escape(arg));
    }
    let cwd_escaped = shell_escape(&cwd.to_string_lossy());

    // Ghostty 的 `new tab` / `-e` 不走 login shell，PATH 不含用户自定义目录
    // （如 ~/.local/bin，codex/claude/cursor-agent 通常装在这里），会导致
    // "codex: not found"。脚本开头显式补 PATH：取当前进程 PATH + ~/.local/bin 兜底。
    let mut path = std::env::var("PATH").unwrap_or_default();
    if let Some(home) = std::env::var_os("HOME") {
        let local_bin = PathBuf::from(&home).join(".local/bin");
        if !path.split(':').any(|p| Path::new(p) == local_bin) {
            if !path.is_empty() {
                path.push(':');
            }
            path.push_str(&local_bin.to_string_lossy());
        }
    }

    // 脚本用 exec 替换 shell 进程，让 agent 直接成为 Ghostty 的子进程；
    // 退出码透传，Ghostty 据此干净退出。
    let script = format!(
        "#!/bin/sh\nexport PATH={path_env}\ncd {cwd_escaped} && exec {command}\n",
        path_env = shell_escape(&path),
    );

    let dir = std::env::temp_dir().join("fast-start-ghostty");
    fs::create_dir_all(&dir).map_err(|err| LaunchError::Spawn(err.to_string()))?;

    let wrapper = dir.join(format!("run-{}.sh", std::process::id()));
    fs::write(&wrapper, script).map_err(|err| LaunchError::Spawn(err.to_string()))?;

    let mut perms = fs::metadata(&wrapper)
        .map_err(|err| LaunchError::Spawn(err.to_string()))?
        .permissions();
    perms.set_mode(0o700);
    fs::set_permissions(&wrapper, perms).map_err(|err| LaunchError::Spawn(err.to_string()))?;

    Ok(wrapper)
}

pub struct SystemTerminalLauncher;

impl TerminalLauncher for SystemTerminalLauncher {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::System
    }

    fn is_available(&self) -> bool {
        Path::new("/System/Applications/Utilities/Terminal.app").exists()
    }

    /// Terminal.app 无法从 AppleScript 开新 tab（硬限制），始终开新窗口。
    fn supports_tab(&self) -> bool {
        false
    }

    fn launch(&self, spec: &CommandSpec, _mode: LaunchMode) -> Result<(), LaunchError> {
        let cwd = validate_command_spec(spec).map_err(LaunchError::Validation)?;
        let shell_command = build_shell_command(spec, &cwd);
        run_osascript(&terminal_applescript(&shell_command))
    }
}

pub struct ITerm2Launcher;

impl TerminalLauncher for ITerm2Launcher {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::ITerm2
    }

    fn is_available(&self) -> bool {
        Path::new("/Applications/iTerm.app").exists()
            || Path::new("/Applications/iTerm2.app").exists()
    }

    fn launch(&self, spec: &CommandSpec, mode: LaunchMode) -> Result<(), LaunchError> {
        let cwd = validate_command_spec(spec).map_err(LaunchError::Validation)?;
        let shell_command = build_shell_command(spec, &cwd);
        let script = match mode {
            LaunchMode::NewTab => iterm_open_tab_applescript(&shell_command),
            LaunchMode::NewWindow => iterm_open_window_applescript(&shell_command),
        };
        run_osascript(&script)
    }
}

/// Ghostty 是否已有窗口在运行（用于决定开 tab 还是开新窗口）。
fn ghostty_has_window() -> bool {
    let script = "tell application \"Ghostty\" to count windows";
    Command::new("osascript")
        .args(["-e", script])
        .output()
        .map(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse::<u32>()
                    .map_or(false, |n| n > 0)
        })
        .unwrap_or(false)
}

/// 在已运行的 Ghostty 窗口里开新 tab，运行指定 wrapper。
/// 用 AppleScript `new tab with configuration`（Ghostty sdef 提供），
/// 配置项 `command` 指向 wrapper 脚本（单路径，避免 login 误报），
/// `initial working directory` 设工作目录。
fn ghostty_new_tab(wrapper: &Path, cwd: &Path) -> Result<(), LaunchError> {
    let script = format!(
        "tell application \"Ghostty\"\n\
         \x20\x20set cfg to {{command:{cmd}, initial working directory:{wd}}}\n\
         \x20\x20new tab with configuration cfg in front window\n\
         end tell",
        cmd = applescript_string(&wrapper.to_string_lossy()),
        wd = applescript_string(&cwd.to_string_lossy()),
    );
    run_osascript(&script)
}

pub struct GhosttyLauncher;

impl TerminalLauncher for GhosttyLauncher {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::Ghostty
    }

    fn is_available(&self) -> bool {
        Path::new("/Applications/Ghostty.app").exists()
    }

    fn launch(&self, spec: &CommandSpec, mode: LaunchMode) -> Result<(), LaunchError> {
        let cwd = validate_command_spec(spec).map_err(LaunchError::Validation)?;
        let wrapper = write_ghostty_wrapper(spec, &cwd)?;

        // NewTab 且已有窗口 → 在该窗口开新 tab（AppleScript new tab）。
        // 其余情况（NewWindow，或 NewTab 但无窗口可挂）→ 开新窗口。
        if mode == LaunchMode::NewTab && ghostty_has_window() {
            return ghostty_new_tab(&wrapper, &cwd);
        }

        // `open -na Ghostty.app --args -e <wrapper>` 开新窗口。
        // `-e` 只传单脚本路径（避免 login 误报），且自动设
        // quit-after-last-window-closed=true，agent 退出后 Ghostty 干净退出。
        let output = Command::new("open")
            .args([
                "-na",
                "Ghostty.app",
                "--args",
                "-e",
                &wrapper.to_string_lossy(),
            ])
            .output()
            .map_err(|err| LaunchError::Spawn(err.to_string()))?;

        command_output_result(output, "open")
    }
}

#[cfg(test)]
mod tests {
    use super::{build_shell_command, write_ghostty_wrapper};
    use crate::models::CommandSpec;
    use std::path::PathBuf;

    fn codex_spec() -> CommandSpec {
        CommandSpec {
            cwd: PathBuf::from("/tmp/project with space"),
            program: "codex".to_string(),
            args: vec!["resume".to_string(), "abc-123".to_string()],
        }
    }

    #[test]
    fn shell_command_quotes_cwd_for_applescript_terminals() {
        let cwd = PathBuf::from("/tmp/project with space");

        assert_eq!(
            build_shell_command(&codex_spec(), &cwd),
            "cd '/tmp/project with space' && codex resume abc-123"
        );
    }

    #[test]
    fn ghostty_wrapper_cd_then_execs_command() {
        let cwd = PathBuf::from("/tmp/project with space");
        let wrapper = write_ghostty_wrapper(&codex_spec(), &cwd).unwrap();
        let content = std::fs::read_to_string(&wrapper).unwrap();

        assert!(
            content.contains("cd '/tmp/project with space'"),
            "wrapper should cd to escaped cwd"
        );
        assert!(
            content.contains("exec codex resume abc-123"),
            "wrapper should exec the command with its arguments"
        );
        let _ = std::fs::remove_file(&wrapper);
    }
}
