//! 各 macOS 终端的 AppleScript / open 实现。

use super::{
    command_output_result, run_osascript, wrapper_shell_command, write_command_wrapper, LaunchError,
    TerminalLauncher,
};
use crate::models::{CommandSpec, LaunchMode, TerminalType};
use crate::security::{applescript_string, validate_command_spec};
use std::path::{Path, PathBuf};
use std::process::Command;

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
        let wrapper = write_command_wrapper(spec, cwd.as_deref())?;
        let shell_command = wrapper_shell_command(&wrapper);
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
        let wrapper = write_command_wrapper(spec, cwd.as_deref())?;
        let shell_command = wrapper_shell_command(&wrapper);
        let script = match mode {
            LaunchMode::NewTab => iterm_open_tab_applescript(&shell_command),
            LaunchMode::NewWindow => iterm_open_window_applescript(&shell_command),
        };
        run_osascript(&script)
    }
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
        let wrapper = write_command_wrapper(spec, cwd.as_deref())?;

        // NewTab 且已有窗口 → 在该窗口开新 tab（AppleScript new tab）。
        // 其余情况（NewWindow，或 NewTab 但无窗口可挂）→ 开新窗口。
        if mode == LaunchMode::NewTab && ghostty_has_window() {
            return ghostty_new_tab(&wrapper, cwd.as_deref());
        }

        // `open -na Ghostty.app --args -e <wrapper>` 开新窗口。
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

pub struct WezTermLauncher;

impl TerminalLauncher for WezTermLauncher {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::WezTerm
    }

    fn is_available(&self) -> bool {
        wezterm_available()
    }

    fn supports_tab(&self) -> bool {
        true
    }

    fn launch(&self, spec: &CommandSpec, mode: LaunchMode) -> Result<(), LaunchError> {
        let cwd = validate_command_spec(spec).map_err(LaunchError::Validation)?;
        let wrapper = write_command_wrapper(spec, cwd.as_deref())?;
        let wezterm = resolve_wezterm_bin().ok_or_else(|| {
            LaunchError::Spawn("未找到 wezterm 可执行文件".to_string())
        })?;

        if mode == LaunchMode::NewTab && wezterm_mux_available(&wezterm) {
            if wezterm_cli_spawn(&wezterm, &wrapper, cwd.as_deref()).is_ok() {
                return Ok(());
            }
            // tab 失败回退新窗口
        }

        wezterm_start_window(&wezterm, &wrapper, cwd.as_deref())
    }
}

fn wezterm_available() -> bool {
    resolve_wezterm_bin().is_some()
}

fn resolve_wezterm_bin() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("/Applications/WezTerm.app/Contents/MacOS/wezterm"),
        PathBuf::from("/opt/homebrew/bin/wezterm"),
        PathBuf::from("/usr/local/bin/wezterm"),
    ];
    // 版本化安装名（WezTerm-macos-*.app）：read_dir 前缀匹配，不用假 glob
    if let Ok(entries) = std::fs::read_dir("/Applications") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if name.starts_with("WezTerm") && name.ends_with(".app") {
                candidates.push(entry.path().join("Contents/MacOS/wezterm"));
            }
        }
    }
    for p in &candidates {
        if p.is_file() {
            return Some(p.clone());
        }
    }
    let output = Command::new("which").arg("wezterm").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return None;
    }
    let p = PathBuf::from(path);
    if p.is_file() {
        Some(p)
    } else {
        None
    }
}

fn wezterm_mux_available(wezterm: &Path) -> bool {
    Command::new(wezterm)
        .args(["cli", "list"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn wezterm_cli_spawn(
    wezterm: &Path,
    wrapper: &Path,
    cwd: Option<&Path>,
) -> Result<(), LaunchError> {
    let mut cmd = Command::new(wezterm);
    cmd.args(["cli", "spawn"]);
    if let Some(dir) = cwd {
        cmd.arg("--cwd").arg(dir);
    }
    cmd.arg("--").arg(wrapper);
    let output = cmd
        .output()
        .map_err(|err| LaunchError::Spawn(err.to_string()))?;
    command_output_result(output, "wezterm cli spawn")
}

fn wezterm_start_window(
    wezterm: &Path,
    wrapper: &Path,
    cwd: Option<&Path>,
) -> Result<(), LaunchError> {
    let mut cmd = Command::new(wezterm);
    cmd.arg("start");
    if let Some(dir) = cwd {
        cmd.arg("--cwd").arg(dir);
    }
    cmd.arg("--").arg(wrapper);
    let output = cmd
        .output()
        .map_err(|err| LaunchError::Spawn(err.to_string()))?;
    command_output_result(output, "wezterm start")
}

/// Terminal.app 的 `do script` 无法干净开 tab；冷启动可能多一个空默认窗口（硬限制）。
pub(super) fn terminal_applescript(shell_command: &str) -> String {
    format!(
        "tell application \"Terminal\"\n\
         \x20\x20activate\n\
         \x20\x20do script {}\n\
         end tell",
        applescript_string(shell_command),
    )
}

/// iTerm2：有窗口开新 tab，无窗口（冷启动）等默认窗口出现并复用。
/// app 名必须是 "iTerm"（不是 "iTerm2"）。
pub(super) fn iterm_open_tab_applescript(shell_command: &str) -> String {
    format!(
        "tell application \"iTerm\"\n\
         \x20\x20activate\n\
         \x20\x20if (count of windows) is 0 then\n\
         \x20\x20\x20\x20repeat with i from 1 to 300\n\
         \x20\x20\x20\x20\x20\x20if (count of windows) > 0 then exit repeat\n\
         \x20\x20\x20\x20\x20\x20delay 0.1\n\
         \x20\x20\x20\x20end repeat\n\
         \x20\x20\x20\x20if (count of windows) is 0 then error \"iTerm 启动超时\"\n\
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

fn iterm_open_window_applescript(shell_command: &str) -> String {
    format!(
        "tell application \"iTerm\"\n\
         \x20\x20activate\n\
         \x20\x20if (count of windows) is 0 then\n\
         \x20\x20\x20\x20repeat with i from 1 to 300\n\
         \x20\x20\x20\x20\x20\x20if (count of windows) > 0 then exit repeat\n\
         \x20\x20\x20\x20\x20\x20delay 0.1\n\
         \x20\x20\x20\x20end repeat\n\
         \x20\x20\x20\x20if (count of windows) is 0 then error \"iTerm 启动超时\"\n\
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

fn ghostty_new_tab(wrapper: &Path, cwd: Option<&Path>) -> Result<(), LaunchError> {
    let cfg = match cwd {
        Some(dir) => format!(
            "{{command:{cmd}, initial working directory:{wd}}}",
            cmd = applescript_string(&wrapper.to_string_lossy()),
            wd = applescript_string(&dir.to_string_lossy()),
        ),
        None => format!(
            "{{command:{cmd}}}",
            cmd = applescript_string(&wrapper.to_string_lossy()),
        ),
    };
    let script = format!(
        "tell application \"Ghostty\"\n\
         \x20\x20new tab with configuration {cfg} in front window\n\
         end tell",
    );
    run_osascript(&script)
}
