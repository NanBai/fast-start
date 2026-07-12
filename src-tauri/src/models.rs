use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CliType {
    Codex,
    ClaudeCode,
    Cursor,
    GrokBuild,
    OpenCode,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PortProtocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionDeleteKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDeleteTarget {
    pub root: PathBuf,
    pub path: PathBuf,
    pub kind: SessionDeleteKind,
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
    #[serde(skip)]
    pub delete_target: Option<SessionDeleteTarget>,
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub cwd: PathBuf,
    pub program: String,
    pub args: Vec<String>,
    /// launch 时是否 cd 到 cwd。当前各 CLI 都需要在原工作目录下恢复上下文。
    pub cd: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResponse {
    pub sessions: Vec<Session>,
    pub scan_errors: Vec<CliScanError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortUsage {
    pub id: String,
    pub command: String,
    pub pid: i32,
    pub user: String,
    pub protocol: PortProtocol,
    pub address: String,
    pub port: u16,
    pub state: String,
    pub executable_path: String,
    pub working_directory: String,
    pub parent_command: String,
    pub is_project_service: bool,
    pub user_owned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortScanResponse {
    pub ports: Vec<PortUsage>,
    pub raw_line_count: usize,
    pub command_description: String,
    pub scanned_at: DateTime<Utc>,
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

    /// 列表行 id：由 cli + 源 session id + 工作目录确定性生成，刷新扫描后保持不变。
    pub fn stable_id(cli_type: CliType, session_id: &str, project_dir: &std::path::Path) -> String {
        let key = format!(
            "{}:{}:{}",
            cli_type_stable_key(cli_type),
            session_id,
            project_dir.to_string_lossy()
        );
        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, key.as_bytes()).to_string()
    }
}

fn cli_type_stable_key(cli_type: CliType) -> &'static str {
    match cli_type {
        CliType::Codex => "codex",
        CliType::ClaudeCode => "claude-code",
        CliType::Cursor => "cursor",
        CliType::GrokBuild => "grok-build",
        CliType::OpenCode => "opencode",
    }
}

#[cfg(test)]
mod tests {
    use super::{CliType, Session};
    use std::path::PathBuf;

    #[test]
    fn stable_id_is_deterministic_for_same_session() {
        let first =
            Session::stable_id(CliType::Codex, "abc-123", PathBuf::from("/tmp/a").as_path());
        let second =
            Session::stable_id(CliType::Codex, "abc-123", PathBuf::from("/tmp/a").as_path());
        assert_eq!(first, second);
    }

    #[test]
    fn stable_id_differs_across_cli_or_project() {
        let codex =
            Session::stable_id(CliType::Codex, "abc-123", PathBuf::from("/tmp/a").as_path());
        let claude = Session::stable_id(
            CliType::ClaudeCode,
            "abc-123",
            PathBuf::from("/tmp/a").as_path(),
        );
        let other_dir =
            Session::stable_id(CliType::Codex, "abc-123", PathBuf::from("/tmp/b").as_path());
        assert_ne!(codex, claude);
        assert_ne!(codex, other_dir);
    }
}
