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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub cli_type: CliType,
    pub session_id: String,
    pub project_dir: PathBuf,
    pub project_name: String,
    pub last_active_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub cwd: PathBuf,
    pub program: String,
    pub args: Vec<String>,
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
