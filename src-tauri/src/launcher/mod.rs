//! 外部终端拉起：校验 CommandSpec → 写临时 wrapper → 各终端实现开窗/tab。
//!
//! 终端实现见 `terminals`；PATH 在进程内缓存，避免每次 launch 同步跑 login shell。

mod terminals;

use crate::models::{CommandSpec, LaunchMode, TerminalType};
use crate::security::shell_escape;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use terminals::{GhosttyLauncher, ITerm2Launcher, SystemTerminalLauncher, WezTermLauncher};

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
        Box::new(WezTermLauncher),
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
         # 防御：若此时仍无 node（常见于 nvm/fnm 仅在 rc 里生效的情况），\n\
         # 尝试把已存在的 node 目录并入 PATH。codex 等是 env node shebang。\n\
         if ! command -v node >/dev/null 2>&1; then\n\
         \x20\x20for d in \"$HOME/.nvm/versions/node\"/*/bin /opt/homebrew/bin /usr/local/bin \"$HOME/.local/bin\" \"$HOME/.asdf/shims\"; do\n\
         \x20\x20\x20\x20if [ -x \"$d/node\" ]; then\n\
         \x20\x20\x20\x20\x20\x20PATH=\"$d:$PATH\"\n\
         \x20\x20\x20\x20\x20\x20break\n\
         \x20\x20\x20\x20fi\n\
         \x20\x20done\n\
         \x20\x20export PATH\n\
         fi\n\
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

/// 与 Ghostty wrapper 注入语义一致的 PATH：优先 `~/.grok/bin`，再 login shell PATH。
/// 供 launch preflight 的 `ResolveProgram` 生产实现复用，避免与 wrapper 分叉。
pub fn launch_path_string() -> String {
    let mut path = String::new();
    if let Some(home) = std::env::var_os("HOME") {
        let grok_bin = PathBuf::from(home).join(".grok/bin");
        if grok_bin.is_dir() {
            path.push_str(&grok_bin.to_string_lossy());
            path.push(':');
        }
    }
    path.push_str(cached_login_path());
    path
}

/// 在 launch PATH 上解析 program（普通文件且具可执行位）。
pub fn resolve_program_on_launch_path(program: &str) -> Option<PathBuf> {
    if program.is_empty() || program.contains('/') {
        // 仅允许白名单短名；带路径的 program 不走 PATH 搜索
        let p = PathBuf::from(program);
        return if is_executable_file(&p) { Some(p) } else { None };
    }
    for dir in launch_path_string().split(':').filter(|d| !d.is_empty()) {
        let candidate = Path::new(dir).join(program);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    meta.permissions().mode() & 0o111 != 0
}

fn resolve_login_path_once() -> String {
    // 优先尝试非交互 login（快），再尝试交互 login（能 source ~/.zshrc 等，捕获 nvm/fnm）。
    // 很多 node 版本管理器只在 rc 文件里改 PATH，纯 -lc 拿不到。
    for shell in ["zsh", "bash"] {
        for login_args in [vec!["-lc"], vec!["-ilc"]] {
            let mut cmd_args: Vec<&str> = login_args;
            cmd_args.push(r#"printf %s "$PATH""#);
            let Ok(output) = Command::new(shell).args(&cmd_args).output() else {
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
                // 即便 login PATH 成功，也合并关键目录 + 动态 nvm 目录，
                // 确保 codex/claude 等 `#!/usr/bin/env node` 能找到 node。
                return merge_critical_path_dirs(candidate);
            }
        }
    }
    fallback_path_string()
}

/// 判断是否像 PATH 列表，而非 shell banner。
///
/// 允许条目内空格（如 `Application Support`）；拒绝无冒号却含空格的整句 banner。
fn is_plausible_path(path: &str) -> bool {
    if path.is_empty() || !path.contains('/') {
        return false;
    }
    if path.chars().any(|ch| matches!(ch, '\n' | '\r' | '\0')) {
        return false;
    }
    // "hello world from zsh" 这类无 PATH 分隔符的句子
    if path.contains(' ') && !path.contains(':') {
        return false;
    }
    true
}

/// 把常见 CLI/node 安装目录并入 PATH（已存在则不重复）。
fn merge_critical_path_dirs(base: &str) -> String {
    let mut entries: Vec<String> = base
        .split(':')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    for dir in critical_path_dirs() {
        if !entries.iter().any(|e| e == &dir) {
            entries.push(dir);
        }
    }
    entries.join(":")
}

fn critical_path_dirs() -> Vec<String> {
    let mut dirs = vec![
        "/opt/homebrew/bin".to_string(),
        "/opt/homebrew/sbin".to_string(),
        "/usr/local/bin".to_string(),
        "/usr/local/sbin".to_string(),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        let home = home.to_string_lossy();
        for sub in [
            ".local/bin",
            ".grok/bin",
            ".opencode/bin",
            ".bun/bin",
            ".cargo/bin",
            ".volta/bin",
            ".asdf/shims",
        ] {
            dirs.push(format!("{home}/{sub}"));
        }

        // nvm 常见位置：~/.nvm/versions/node/vXX.Y.Z/bin
        // 许多 codex / node 系 CLI 用 nvm 管理，纯 login shell 常拿不到这些目录。
        let nvm_base = PathBuf::from(home.as_ref()).join(".nvm/versions/node");
        if let Ok(read_dir) = std::fs::read_dir(&nvm_base) {
            for entry in read_dir.flatten() {
                let bin_dir = entry.path().join("bin");
                if bin_dir.join("node").is_file() {
                    dirs.push(bin_dir.to_string_lossy().to_string());
                }
            }
        }
    }
    dirs
}

/// 当登录 shell 解析失败时的兜底 PATH。
fn fallback_path_string() -> String {
    let process_path = std::env::var("PATH").unwrap_or_default();
    merge_critical_path_dirs(&process_path)
}

#[cfg(test)]
mod tests {
    use super::{
        is_plausible_path, launcher_for, wrapper_shell_command, write_command_wrapper,
    };
    use crate::models::{CommandSpec, TerminalType};
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
    fn plausible_path_allows_application_support_rejects_banner() {
        assert!(is_plausible_path("/usr/bin:/bin:/Users/x/.local/bin"));
        assert!(is_plausible_path(
            "/opt/homebrew/bin:/Users/x/Library/Application Support/JetBrains/Toolbox/scripts:/usr/bin"
        ));
        assert!(!is_plausible_path("hello world from zsh"));
        assert!(!is_plausible_path(""));
        assert!(!is_plausible_path("nopath"));
    }

    #[test]
    fn merge_critical_path_dirs_appends_homebrew_when_missing() {
        use super::merge_critical_path_dirs;
        let merged = merge_critical_path_dirs("/usr/bin:/bin:/Users/x/.local/bin");
        assert!(
            merged.contains("/opt/homebrew/bin"),
            "must append homebrew so env node works for codex shebang"
        );
        assert!(merged.starts_with("/usr/bin:/bin:/Users/x/.local/bin"));
        // 不重复
        let again = merge_critical_path_dirs(&merged);
        assert_eq!(
            again.matches("/opt/homebrew/bin").count(),
            1,
            "must not duplicate homebrew"
        );
    }

    #[test]
    fn wezterm_launcher_is_registered_and_reports_availability_safely() {
        let launcher = launcher_for(TerminalType::WezTerm).expect("WezTerm launcher registered");
        assert_eq!(launcher.terminal_type(), TerminalType::WezTerm);
        assert!(launcher.supports_tab());
        // 本机未装时 false，不 panic
        let _ = launcher.is_available();
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
        // 防御 node 缺失的逻辑必须存在（针对 nvm 等 rc-only node 场景）
        assert!(
            content.contains("command -v node"),
            "wrapper should defensively ensure node is in PATH for env-shebang CLIs like codex"
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
