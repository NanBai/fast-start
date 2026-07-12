//! 外部终端拉起：校验 CommandSpec → 写临时 wrapper → 各终端实现开窗/tab。
//!
//! 终端实现见 `terminals`；PATH 在进程内缓存，避免每次 launch 同步跑 login shell。

mod terminals;

use crate::models::{CommandSpec, LaunchMode, TerminalType};
use crate::security::shell_escape;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use terminals::{GhosttyLauncher, ITerm2Launcher, SystemTerminalLauncher};

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

/// 给 AppleScript 终端注入的短命令：只执行受控 wrapper，不直接写入业务命令。
pub(crate) fn wrapper_shell_command(wrapper: &Path) -> String {
    shell_escape(&wrapper.to_string_lossy())
}

pub(crate) fn run_osascript(script: &str) -> Result<(), LaunchError> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|err| LaunchError::Spawn(err.to_string()))?;

    command_output_result(output, "osascript")
}

pub(crate) fn command_output_result(
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

/// 为终端生成一个临时 wrapper 脚本，返回脚本路径。
///
/// 为什么用 wrapper 而非直接 `-e <program> <args>`：
/// Ghostty 在 macOS 上把 `-e`/`--command` 的命令套进 `/usr/bin/login -flp`，
/// 多词命令会让 login 解析失败。让 `-e` 只执行单脚本路径即可。
///
/// PATH 在 **Rust 进程内缓存**（首次 launch 解析 login shell，之后复用），
/// wrapper 只注入已解析好的 PATH，避免每次 zsh -lc 的延迟。
pub(crate) fn write_command_wrapper(
    spec: &CommandSpec,
    cwd: Option<&Path>,
) -> Result<PathBuf, LaunchError> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let mut command = shell_escape(&spec.program);
    for arg in &spec.args {
        command.push(' ');
        command.push_str(&shell_escape(arg));
    }

    let login_path = cached_login_path();
    let cd_clause = match cwd {
        Some(dir) => format!("cd {} && ", shell_escape(&dir.to_string_lossy())),
        None => String::new(),
    };
    let script = format!(
        "#!/bin/sh\n\
         PATH={path}\n\
         if [ -n \"$HOME\" ] && [ -d \"$HOME/.grok/bin\" ]; then\n\
         \x20\x20PATH=\"$HOME/.grok/bin:$PATH\"\n\
         fi\n\
         export PATH\n\
         {cd_clause}exec {command}\n",
        path = shell_escape(login_path),
    );

    let dir = std::env::temp_dir().join("fast-start-ghostty");
    fs::create_dir_all(&dir).map_err(|err| LaunchError::Spawn(err.to_string()))?;

    let wrapper = dir.join(format!("run-{}.sh", uuid::Uuid::new_v4()));
    fs::write(&wrapper, script).map_err(|err| LaunchError::Spawn(err.to_string()))?;

    let mut perms = fs::metadata(&wrapper)
        .map_err(|err| LaunchError::Spawn(err.to_string()))?
        .permissions();
    perms.set_mode(0o700);
    fs::set_permissions(&wrapper, perms).map_err(|err| LaunchError::Spawn(err.to_string()))?;

    Ok(wrapper)
}

/// 进程级缓存：login shell PATH 只解析一次。
static CACHED_LOGIN_PATH: OnceLock<String> = OnceLock::new();

fn cached_login_path() -> &'static str {
    CACHED_LOGIN_PATH.get_or_init(resolve_login_path_once)
}

fn resolve_login_path_once() -> String {
    for shell in ["zsh", "bash"] {
        let Ok(output) = Command::new(shell)
            .args(["-lc", r#"printf %s "$PATH""#])
            .output()
        else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        let raw = String::from_utf8_lossy(&output.stdout);
        // banner/插件常污染 stdout 前部：取最后一行非空。
        let Some(line) = raw.lines().rev().find(|l| !l.trim().is_empty()) else {
            continue;
        };
        let candidate = line.trim();
        if is_plausible_path(candidate) {
            return candidate.to_string();
        }
    }
    fallback_path_string()
}

fn is_plausible_path(path: &str) -> bool {
    !path.is_empty()
        && path.contains('/')
        && !path
            .chars()
            .any(|ch| ch.is_whitespace() || matches!(ch, '\n' | '\r'))
}

/// 当登录 shell 解析失败时的兜底 PATH。
fn fallback_path_string() -> String {
    let mut entries: Vec<String> = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    let candidates = [
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        "/usr/local/bin",
        "/usr/local/sbin",
    ];
    for c in candidates {
        if !entries.iter().any(|e| e == c) {
            entries.push(c.to_string());
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let home = home.to_string_lossy().into_owned();
        for sub in [
            ".local/bin",
            ".grok/bin",
            ".opencode/bin",
            ".bun/bin",
            ".cargo/bin",
            ".nvm/versions/node",
            ".volta/bin",
            ".asdf/shims",
        ] {
            let p = format!("{home}/{sub}");
            if !entries.iter().any(|e| e == &p) {
                entries.push(p);
            }
        }
    }

    entries.join(":")
}

#[cfg(test)]
mod tests {
    use super::{
        is_plausible_path, wrapper_shell_command, write_command_wrapper,
    };
    use crate::models::CommandSpec;
    use crate::security::validate_command_spec;
    use std::path::PathBuf;

    fn codex_spec() -> CommandSpec {
        CommandSpec {
            cwd: PathBuf::from("/tmp/project with space"),
            program: "codex".to_string(),
            args: vec!["resume".to_string(), "abc-123".to_string()],
            cd: true,
        }
    }

    #[test]
    fn plausible_path_rejects_banner_and_spaces() {
        assert!(is_plausible_path("/usr/bin:/bin:/Users/x/.local/bin"));
        assert!(!is_plausible_path("hello world from zsh"));
        assert!(!is_plausible_path(""));
        assert!(!is_plausible_path("nopath"));
    }

    #[test]
    fn applescript_terminals_receive_wrapper_path_only() {
        use super::terminals::{iterm_open_tab_applescript, terminal_applescript};

        let wrapper = PathBuf::from("/tmp/project with space/run.sh");
        let command = wrapper_shell_command(&wrapper);

        assert_eq!(command, "'/tmp/project with space/run.sh'");
        assert!(terminal_applescript(&command)
            .contains("do script \"'/tmp/project with space/run.sh'\""));
        assert!(iterm_open_tab_applescript(&command)
            .contains("write text \"'/tmp/project with space/run.sh'\""));
    }

    #[test]
    fn command_wrapper_cd_then_execs_command() {
        let cwd = PathBuf::from("/tmp/project with space");
        let wrapper = write_command_wrapper(&codex_spec(), Some(&cwd)).unwrap();
        let content = std::fs::read_to_string(&wrapper).unwrap();

        assert!(
            content.contains("cd '/tmp/project with space'"),
            "wrapper should cd to escaped cwd"
        );
        assert!(
            content.contains("exec codex resume abc-123"),
            "wrapper should exec the command with its arguments"
        );
        assert!(
            content.contains("$HOME/.grok/bin"),
            "wrapper should prepend ~/.grok/bin so grok is found even when login PATH omits it"
        );
        // PATH 已在 Rust 侧解析并注入，不再每次在脚本里 zsh -lc。
        assert!(
            !content.contains("zsh -lc"),
            "wrapper must not re-run login shell PATH resolution"
        );
        assert!(
            content.contains("PATH="),
            "wrapper should export a pre-resolved PATH"
        );
        let _ = std::fs::remove_file(&wrapper);
    }

    #[test]
    fn command_wrapper_uses_unique_paths_for_concurrent_launches() {
        let cwd = PathBuf::from("/tmp/project with space");
        let first = write_command_wrapper(&codex_spec(), Some(&cwd)).unwrap();
        let second = write_command_wrapper(&codex_spec(), Some(&cwd)).unwrap();

        assert_ne!(first, second);
        let _ = std::fs::remove_file(&first);
        let _ = std::fs::remove_file(&second);
    }

    #[test]
    fn validate_command_spec_accepts_codex_shape() {
        let temp = tempfile::tempdir().unwrap();
        let spec = CommandSpec {
            cwd: temp.path().to_path_buf(),
            program: "codex".into(),
            args: vec!["resume".into(), "abc-123".into()],
            cd: true,
        };
        assert!(validate_command_spec(&spec).is_ok());
    }
}
