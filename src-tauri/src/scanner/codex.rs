use crate::models::{CliType, Session};
use crate::scanner::{clean_summary, ScanError, SessionScanner};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct CodexScanner {
    root: Option<PathBuf>,
}

impl CodexScanner {
    #[cfg(test)]
    pub fn with_root(root: PathBuf) -> Self {
        Self { root: Some(root) }
    }

    fn root(&self) -> Result<PathBuf, ScanError> {
        if let Some(root) = &self.root {
            return Ok(root.clone());
        }
        dirs::home_dir()
            .map(|home| home.join(".codex/sessions"))
            .ok_or_else(|| ScanError::NotFound("无法定位用户主目录".to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct CodexLine {
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(rename = "type")]
    line_type: Option<String>,
    #[serde(default)]
    payload: Option<serde_json::Value>,
}

impl SessionScanner for CodexScanner {
    fn cli_type(&self) -> CliType {
        CliType::Codex
    }

    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError> {
        let root = self.root()?;

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
    let mut summary = None;

    for line in content.lines() {
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

        let Some(payload) = &parsed.payload else {
            continue;
        };

        // session_meta 行拿 id / cwd（幂等，重复行无妨）。
        if parsed.line_type.as_deref() == Some("session_meta") {
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
            continue;
        }

        // 简介取第一条「真实」用户消息——codex 没有 AI 标题，且首条 user
        // message 常常是注入的上下文块（AGENTS.md / <environment_context> 等），
        // 必须跳过这些，否则简介全是指令而非用户意图。拿到第一条就停。
        if summary.is_none() && parsed.line_type.as_deref() == Some("response_item") {
            if let Some(text) = first_real_user_message(payload) {
                summary = clean_summary(Some(&text));
            }
        }

        if session_id.is_some() && cwd.is_some() && summary.is_some() {
            break;
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
        summary,
    }))
}

/// 从一条 response_item payload 里取第一条「真实」用户消息文本。
/// codex 的 user message 形如 `{type:"message", role:"user", content:[{type:"input_text", text}]}`，
/// 但 content 里可能混着指令 / 环境上下文注入，要逐段过滤。返回 None 表示这条没有可用文本。
fn first_real_user_message(payload: &serde_json::Value) -> Option<String> {
    let payload = payload.as_object()?;
    if payload.get("type").and_then(|v| v.as_str()) != Some("message") {
        return None;
    }
    if payload.get("role").and_then(|v| v.as_str()) != Some("user") {
        return None;
    }
    let content = payload.get("content")?.as_array()?;
    for part in content {
        let part = part.as_object()?;
        if part.get("type").and_then(|v| v.as_str()) != Some("input_text") {
            continue;
        }
        if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
            if is_injected_context(text) {
                continue;
            }
            return Some(text.to_string());
        }
    }
    None
}

/// codex 把指令 / 环境信息作为「用户」消息注入到对话开头，这些不是用户真正说的话，
/// 不能当简介。判据基于实测：注入块总是以特定标记开头或包在尖括号标签里。
fn is_injected_context(text: &str) -> bool {
    let stripped = text.trim_start();
    if stripped.starts_with('#') {
        // # AGENTS.md instructions ... / # Codex ...
        return true;
    }
    if stripped.starts_with('<') {
        // <environment_context> / <app-context> / <permissions ...> 等
        return true;
    }
    if stripped.starts_with("Caveat") {
        return true;
    }
    text.contains("<environment_context")
        || text.contains("<app-context")
        || text.contains("<permissions")
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
    use super::{first_real_user_message, is_injected_context, CodexScanner};
    use crate::scanner::SessionScanner;
    use serde_json::json;
    use std::fs;

    #[test]
    fn is_injected_context_flags_codex_context_blocks() {
        assert!(is_injected_context("# AGENTS.md instructions for /x"));
        assert!(is_injected_context("<environment_context>\n  <cwd>/x"));
        assert!(is_injected_context("Caveat: The ..."));
        assert!(!is_injected_context("帮我看看这个接口为什么 500"));
        assert!(!is_injected_context("url: jdbc:mysql://1.2.3.4:3306/db"));
    }

    #[test]
    fn first_real_user_message_skips_injected_parts() {
        // content 里前两段是注入，第三段是真实用户输入。
        let payload = json!({
            "type": "message",
            "role": "user",
            "content": [
                {"type": "input_text", "text": "# AGENTS.md instructions for /x"},
                {"type": "input_text", "text": "<environment_context>\n  <cwd>/x"},
                {"type": "input_text", "text": "sa-token设置的token过期时间是多久"}
            ]
        });
        assert_eq!(
            first_real_user_message(&payload),
            Some("sa-token设置的token过期时间是多久".to_string())
        );
    }

    #[test]
    fn first_real_user_message_returns_none_for_assistant_or_non_message() {
        assert_eq!(
            first_real_user_message(&json!({"type":"message","role":"assistant","content":[]})),
            None
        );
        assert_eq!(
            first_real_user_message(&json!({"type":"function_call","name":"exec_command"})),
            None
        );
    }

    #[test]
    fn scanner_reads_fixture_sessions_without_home_data() {
        let temp = tempfile::tempdir().unwrap();
        let session_file = temp.path().join("session.jsonl");
        fs::write(
            &session_file,
            [
                r#"{"timestamp":"2026-06-19T01:00:00Z","type":"session_meta","payload":{"id":"codex-fixture","cwd":"/tmp"}}"#,
                r##"{"timestamp":"2026-06-19T01:01:00Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"# AGENTS.md instructions"},{"type":"input_text","text":"修复扫描性能"}]}}"##,
            ]
            .join("\n"),
        )
        .unwrap();

        let sessions = CodexScanner::with_root(temp.path().to_path_buf())
            .scan_sessions()
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "codex-fixture");
        assert_eq!(sessions[0].summary.as_deref(), Some("修复扫描性能"));
    }

    #[test]
    fn scanner_reads_real_user_message_after_sixty_four_lines() {
        let temp = tempfile::tempdir().unwrap();
        let session_file = temp.path().join("late-summary.jsonl");
        let mut lines = vec![r#"{"timestamp":"2026-06-19T01:00:00Z","type":"session_meta","payload":{"id":"codex-late","cwd":"/tmp"}}"#.to_string()];
        for _ in 0..64 {
            lines.push(r##"{"timestamp":"2026-06-19T01:00:01Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"# AGENTS.md instructions"}]}}"##.to_string());
        }
        lines.push(r#"{"timestamp":"2026-06-19T01:01:00Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"第 65 行之后的真实需求"}]}}"#.to_string());
        fs::write(&session_file, lines.join("\n")).unwrap();

        let sessions = CodexScanner::with_root(temp.path().to_path_buf())
            .scan_sessions()
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "codex-late");
        assert_eq!(
            sessions[0].summary.as_deref(),
            Some("第 65 行之后的真实需求")
        );
    }
}
