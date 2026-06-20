use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CliType {
    Codex,
    ClaudeCode,
    Cursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerminalType {
    System,
    ITerm2,
    Ghostty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Dark,
    Light,
    System,
}

/// 启动时打开新 tab 还是新窗口（用户全局偏好）。
/// 注意：Terminal.app 无法开 tab（AppleScript 硬限制），选 NewTab 时会回退到窗口。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LaunchMode {
    NewTab,
    NewWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub cli_type: CliType,
    pub session_id: String,
    pub project_dir: PathBuf,
    pub project_name: String,
    pub last_active_at: DateTime<Utc>,
    /// 一句话简介，各 CLI 来源不同（cursor 的 meta.title / claude 的 aiTitle /
    /// codex 的首条真实用户消息）。拿不到为 None，前端展示时回退到 project_name。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub cwd: PathBuf,
    pub program: String,
    pub args: Vec<String>,
    /// launch 时是否 cd 到 cwd。三家 CLI 当前都需要在原工作目录下恢复上下文。
    pub cd: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResponse {
    pub sessions: Vec<Session>,
    pub scan_errors: Vec<CliScanError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliScanError {
    pub cli_type: CliType,
    pub message: String,
}

impl Session {
    pub fn project_name_from_dir(path: &std::path::Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}
