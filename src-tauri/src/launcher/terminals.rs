//! 各 macOS 终端的 AppleScript / open 实现。

use super::{
    command_output_result, run_osascript, wrapper_shell_command, write_command_wrapper, LaunchError,
    TerminalLauncher,
};
use crate::models::{CommandSpec, LaunchMode, TerminalType};
use crate::security::{applescript_string, validate_command_spec};
use std::path::Path;
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
