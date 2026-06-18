use crate::models::{CliType, Session};
use crate::scanner::{ScanError, SessionScanner};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct CodexScanner;

#[derive(Debug, Deserialize)]
struct CodexLine {
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(rename = "type")]
    line_type: Option<String>,
    payload: Option<serde_json::Value>,
}

impl SessionScanner for CodexScanner {
    fn cli_type(&self) -> CliType {
        CliType::Codex
    }

    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError> {
        let root = dirs::home_dir()
            .map(|home| home.join(".codex/sessions"))
            .ok_or_else(|| ScanError::NotFound("无法定位用户主目录".to_string()))?;

        if !root.exists() {
            return Err(ScanError::NotFound("codex session 目录不存在".to_string()));
        }

        let mut by_session: HashMap<String, Session> = HashMap::new();

        collect_jsonl_files(&root, &mut by_session)?;

        let mut sessions: Vec<Session> = by_session.into_values().collect();
        sessions.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));
        Ok(sessions)
    }
}

fn collect_jsonl_files(
    dir: &Path,
    by_session: &mut HashMap<String, Session>,
) -> Result<(), ScanError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, by_session)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        if let Some(session) = parse_codex_file(&path)? {
            by_session
                .entry(session.session_id.clone())
                .and_modify(|existing| {
                    if session.last_active_at > existing.last_active_at {
                        *existing = session.clone();
                    }
                })
                .or_insert(session);
        }
    }
    Ok(())
}

fn parse_codex_file(path: &Path) -> Result<Option<Session>, ScanError> {
    let content = fs::read_to_string(path)?;
    let mut session_id = None;
    let mut cwd = None;
    let mut last_active_at = file_mtime(path)?;

    for line in content.lines().take(32) {
        if line.trim().is_empty() {
            continue;
        }
        let parsed: CodexLine = serde_json::from_str(line)
            .map_err(|err| ScanError::Parse(format!("解析 codex jsonl 失败: {err}")))?;

        if let Some(ts) = parsed.timestamp.as_deref() {
            if let Ok(parsed_ts) = DateTime::parse_from_rfc3339(ts) {
                let utc = parsed_ts.with_timezone(&Utc);
                if utc > last_active_at {
                    last_active_at = utc;
                }
            }
        }

        if parsed.line_type.as_deref() != Some("session_meta") {
            continue;
        }

        let Some(payload) = parsed.payload else {
            continue;
        };

        if session_id.is_none() {
            session_id = payload
                .get("id")
                .and_then(|value| value.as_str())
                .map(str::to_string);
        }
        if cwd.is_none() {
            cwd = payload
                .get("cwd")
                .and_then(|value| value.as_str())
                .map(PathBuf::from);
        }
    }

    let (session_id, cwd) = match (session_id, cwd) {
        (Some(id), Some(dir)) => (id, dir),
        _ => return Ok(None),
    };

    Ok(Some(Session {
        id: uuid::Uuid::new_v4().to_string(),
        cli_type: CliType::Codex,
        session_id,
        project_name: Session::project_name_from_dir(&cwd),
        project_dir: cwd,
        last_active_at,
    }))
}

fn file_mtime(path: &Path) -> Result<DateTime<Utc>, ScanError> {
    let metadata = fs::metadata(path)?;
    let modified = metadata
        .modified()
        .map_err(|err| ScanError::Io(std::io::Error::new(err.kind(), err.to_string())))?;
    Ok(DateTime::<Utc>::from(modified))
}
