use crate::models::{CliType, Session};
use crate::scanner::{decode_claude_project_dir, ScanError, SessionScanner};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct ClaudeCodeScanner;

#[derive(Debug, Deserialize)]
struct TimestampLine {
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
}

impl SessionScanner for ClaudeCodeScanner {
    fn cli_type(&self) -> CliType {
        CliType::ClaudeCode
    }

    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError> {
        let root = dirs::home_dir()
            .map(|home| home.join(".claude/projects"))
            .ok_or_else(|| ScanError::NotFound("无法定位用户主目录".to_string()))?;

        if !root.exists() {
            return Err(ScanError::NotFound(
                "claude-code session 目录不存在".to_string(),
            ));
        }

        let mut sessions = Vec::new();

        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let project_dir = entry.path();
            if !project_dir.is_dir() {
                continue;
            }

            let encoded = project_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            let fallback_cwd = decode_claude_project_dir(encoded);

            for file_entry in fs::read_dir(&project_dir)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();
                if !file_path.is_file() {
                    continue;
                }
                if file_path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                    continue;
                }

                let session_id = file_path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or_default()
                    .to_string();
                if session_id.is_empty() {
                    continue;
                }

                let (last_active_at, cwd) = parse_claude_file(&file_path, fallback_cwd.clone())?;
                let Some(cwd) = cwd else {
                    continue;
                };

                sessions.push(Session {
                    id: uuid::Uuid::new_v4().to_string(),
                    cli_type: CliType::ClaudeCode,
                    session_id,
                    project_name: Session::project_name_from_dir(&cwd),
                    project_dir: cwd,
                    last_active_at,
                });
            }
        }

        sessions.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));
        Ok(sessions)
    }
}

fn parse_claude_file(
    path: &Path,
    fallback_cwd: Option<PathBuf>,
) -> Result<(DateTime<Utc>, Option<PathBuf>), ScanError> {
    let content = fs::read_to_string(path)?;
    let mut latest = file_mtime(path)?;
    // 优先用 jsonl 文件里记录的真实 cwd（精确路径）；只有文件里完全没有 cwd
    // 字段时才退回目录名 decode 的 fallback（decode 无法无损还原含 `-` 或 `.`
    // 的路径，只是粗略猜测）。
    let mut cwd: Option<PathBuf> = None;

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parsed: TimestampLine = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if let Some(ts) = parsed.timestamp.as_deref() {
            if let Ok(parsed_ts) = DateTime::parse_from_rfc3339(ts) {
                let utc = parsed_ts.with_timezone(&Utc);
                if utc > latest {
                    latest = utc;
                }
            }
        }
        if cwd.is_none() {
            cwd = parsed.cwd.map(PathBuf::from);
        }
    }

    Ok((latest, cwd.or(fallback_cwd)))
}

fn file_mtime(path: &Path) -> Result<DateTime<Utc>, ScanError> {
    let metadata = fs::metadata(path)?;
    let modified = metadata
        .modified()
        .map_err(|err| ScanError::Io(std::io::Error::new(err.kind(), err.to_string())))?;
    Ok(DateTime::<Utc>::from(modified))
}
