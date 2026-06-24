use crate::models::{CliType, Session, SessionDeleteKind, SessionDeleteTarget};
use crate::scanner::{clean_summary, decode_claude_project_dir, ScanError, SessionScanner};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct ClaudeCodeScanner {
    root: Option<PathBuf>,
}

impl ClaudeCodeScanner {
    #[cfg(test)]
    pub fn with_root(root: PathBuf) -> Self {
        Self { root: Some(root) }
    }

    fn root(&self) -> Result<PathBuf, ScanError> {
        if let Some(root) = &self.root {
            return Ok(root.clone());
        }
        dirs::home_dir()
            .map(|home| home.join(".claude/projects"))
            .ok_or_else(|| ScanError::NotFound("无法定位用户主目录".to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct TimestampLine {
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    #[serde(rename = "type")]
    line_type: Option<String>,
    #[serde(default)]
    #[serde(rename = "aiTitle")]
    ai_title: Option<String>,
    #[serde(default)]
    #[serde(rename = "lastPrompt")]
    last_prompt: Option<String>,
}

impl SessionScanner for ClaudeCodeScanner {
    fn cli_type(&self) -> CliType {
        CliType::ClaudeCode
    }

    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError> {
        let root = self.root()?;

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

                let (last_active_at, cwd, summary) =
                    parse_claude_file(&file_path, fallback_cwd.clone())?;
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
                    summary,
                    delete_target: Some(SessionDeleteTarget {
                        root: root.clone(),
                        path: file_path,
                        kind: SessionDeleteKind::File,
                    }),
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
) -> Result<(DateTime<Utc>, Option<PathBuf>, Option<String>), ScanError> {
    let content = fs::read_to_string(path)?;
    let mut latest = file_mtime(path)?;
    // 优先用 jsonl 文件里记录的真实 cwd（精确路径）；只有文件里完全没有 cwd
    // 字段时才退回目录名 decode 的 fallback（decode 无法无损还原含 `-` 或 `.`
    // 的路径，只是粗略猜测）。
    let mut cwd: Option<PathBuf> = None;
    // 简介：优先 AI 生成的 aiTitle（类似 cursor 的 meta.title），其次 lastPrompt。
    // 文件里同一个 type 可能写多行（标题被改写 / 每轮都记 lastPrompt），
    // 保留最后见到的一份即可。
    let mut summary: Option<String> = None;

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
        match parsed.line_type.as_deref() {
            Some("ai-title") => {
                if let Some(title) = clean_summary(parsed.ai_title.as_deref()) {
                    summary = Some(title);
                }
            }
            Some("last-prompt") if summary.is_none() => {
                if let Some(prompt) = clean_summary(parsed.last_prompt.as_deref()) {
                    summary = Some(prompt);
                }
            }
            _ => {}
        }
    }

    Ok((latest, cwd.or(fallback_cwd), summary))
}

fn file_mtime(path: &Path) -> Result<DateTime<Utc>, ScanError> {
    let metadata = fs::metadata(path)?;
    let modified = metadata
        .modified()
        .map_err(|err| ScanError::Io(std::io::Error::new(err.kind(), err.to_string())))?;
    Ok(DateTime::<Utc>::from(modified))
}

#[cfg(test)]
mod tests {
    use super::{ClaudeCodeScanner, TimestampLine};
    use crate::scanner::SessionScanner;
    use std::fs;

    #[test]
    fn timestamp_line_reads_claude_camel_case_summary_fields() {
        let title: TimestampLine =
            serde_json::from_str(r#"{"type":"ai-title","aiTitle":"实现会话筛选"}"#)
                .expect("ai-title line should parse");
        assert_eq!(title.ai_title.as_deref(), Some("实现会话筛选"));

        let prompt: TimestampLine =
            serde_json::from_str(r#"{"type":"last-prompt","lastPrompt":"修复列表展示"}"#)
                .expect("last-prompt line should parse");
        assert_eq!(prompt.last_prompt.as_deref(), Some("修复列表展示"));
    }

    #[test]
    fn scanner_reads_fixture_sessions_without_home_data() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("-tmp-fast-start");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("claude-fixture.jsonl"),
            [
                r#"{"timestamp":"2026-06-19T01:00:00Z","cwd":"/tmp","type":"last-prompt","lastPrompt":"修复列表展示"}"#,
                r#"{"timestamp":"2026-06-19T01:01:00Z","cwd":"/tmp","type":"ai-title","aiTitle":"优化扫描器"}"#,
            ]
            .join("\n"),
        )
        .unwrap();

        let sessions = ClaudeCodeScanner::with_root(temp.path().to_path_buf())
            .scan_sessions()
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "claude-fixture");
        assert_eq!(sessions[0].project_dir, std::path::PathBuf::from("/tmp"));
        assert_eq!(sessions[0].summary.as_deref(), Some("优化扫描器"));
        let delete_target = sessions[0].delete_target.as_ref().unwrap();
        assert_eq!(delete_target.root, temp.path());
        assert_eq!(delete_target.path, project.join("claude-fixture.jsonl"));
        assert_eq!(delete_target.kind, crate::models::SessionDeleteKind::File);
    }
}
