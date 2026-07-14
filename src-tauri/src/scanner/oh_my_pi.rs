use crate::models::{CliType, Session, SessionDeleteKind, SessionDeleteTarget};
use crate::scanner::{clean_summary, ScanError, SessionScanner};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 扫描 Oh My Pi (omp) 本地 session。
///
/// 存储布局（实测）：
/// `~/.omp/agent/sessions/<group>/<ts>_<id>.jsonl`
/// 以及同目录下的 branch 子文件 `.../<ts>_<id>/<BranchName>.jsonl`。
///
/// JSONL 不一定以 session header 开头：常见首行是 `{"type":"title",...}`，
/// 随后才是 `{"type":"session","id","cwd",...}`，再是 message / model_change 等。
///
/// resume：`omp -r <id>`（推荐在 cwd 下执行）
///
/// 根目录覆盖：
/// - `OMP_HOME` → `$OMP_HOME/agent/sessions`
/// - `PI_CODING_AGENT_DIR` → `$PI_CODING_AGENT_DIR/sessions`
#[derive(Default)]
pub struct OhMyPiScanner {
    root: Option<PathBuf>,
}

impl OhMyPiScanner {
    #[cfg(test)]
    pub fn with_root(root: PathBuf) -> Self {
        Self { root: Some(root) }
    }

    fn root(&self) -> Result<PathBuf, ScanError> {
        if let Some(root) = &self.root {
            return Ok(root.clone());
        }
        if let Ok(home) = std::env::var("OMP_HOME") {
            let home = home.trim();
            if !home.is_empty() {
                return Ok(PathBuf::from(home).join("agent").join("sessions"));
            }
        }
        // 备选：部分环境把 agent 目录本身暴露为 PI_CODING_AGENT_DIR
        if let Ok(dir) = std::env::var("PI_CODING_AGENT_DIR") {
            let dir = dir.trim();
            if !dir.is_empty() {
                return Ok(PathBuf::from(dir).join("sessions"));
            }
        }
        dirs::home_dir()
            .map(|home| home.join(".omp/agent/sessions"))
            .ok_or_else(|| ScanError::NotFound("无法定位用户主目录".to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct OmpLooseEntry {
    #[serde(rename = "type")]
    #[serde(default)]
    type_: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    message: Option<OmpNestedMessage>,
}

#[derive(Debug, Deserialize)]
struct OmpNestedMessage {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    content: Option<Value>,
}

impl SessionScanner for OhMyPiScanner {
    fn cli_type(&self) -> CliType {
        CliType::OhMyPi
    }

    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError> {
        let root = self.root()?;

        if !root.exists() {
            return Err(ScanError::NotFound(
                "oh-my-pi session 目录不存在".to_string(),
            ));
        }

        let mut sessions = Vec::new();
        collect_omp_sessions(&root, &root, &mut sessions)?;
        sessions.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));
        Ok(sessions)
    }
}

fn collect_omp_sessions(
    root: &Path,
    dir: &Path,
    out: &mut Vec<Session>,
) -> Result<(), ScanError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_omp_sessions(root, &path, out)?;
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }

        if let Some(session) = parse_omp_jsonl(root, &path)? {
            out.push(session);
        }
    }
    Ok(())
}

fn parse_omp_jsonl(root: &Path, file_path: &Path) -> Result<Option<Session>, ScanError> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut session_id: Option<String> = None;
    let mut project_dir: Option<PathBuf> = None;
    let mut header_title: Option<String> = None;
    let mut title_entry: Option<String> = None;
    let mut header_ts: Option<DateTime<Utc>> = None;
    let mut last_entry_ts: Option<DateTime<Utc>> = None;
    let mut last_user_text: Option<String> = None;
    let mut last_assistant_text: Option<String> = None;

    // 预算：大文件只扫前 N 行找 header/title，后半用 mtime；
    // 但为了 summary 仍需要扫 message。限制最大扫描行数避免极端膨胀。
    const MAX_LINES: usize = 4_000;
    for (idx, line_res) in reader.lines().enumerate() {
        if idx >= MAX_LINES {
            break;
        }
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: OmpLooseEntry = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let ty = entry.type_.as_deref().unwrap_or("");

        if let Some(ts) = entry.timestamp.as_deref().and_then(parse_ts) {
            last_entry_ts = Some(ts);
        }

        match ty {
            "session" => {
                if let Some(id) = entry.id.filter(|s| !s.is_empty()) {
                    session_id.get_or_insert(id);
                }
                if let Some(cwd) = entry.cwd.filter(|s| !s.is_empty()) {
                    project_dir.get_or_insert(PathBuf::from(cwd));
                }
                if header_title.is_none() {
                    if let Some(t) = entry.title.filter(|s| !s.trim().is_empty()) {
                        header_title = Some(t);
                    }
                }
                if header_ts.is_none() {
                    header_ts = entry.timestamp.as_deref().and_then(parse_ts);
                }
            }
            "title" => {
                if title_entry.is_none() {
                    if let Some(t) = entry.title.filter(|s| !s.trim().is_empty()) {
                        title_entry = Some(t);
                    }
                }
            }
            "message" => {
                if let Some(msg) = entry.message {
                    let role = msg.role.as_deref().unwrap_or("");
                    if let Some(text) = extract_message_text(msg.content.as_ref()) {
                        match role {
                            "user" => last_user_text = Some(text),
                            "assistant" => last_assistant_text = Some(text),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let session_id = match session_id {
        Some(id) => id,
        None => return Ok(None),
    };
    let project_dir = match project_dir {
        Some(dir) => dir,
        None => return Ok(None),
    };

    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let last_active_at = last_entry_ts
        .or(header_ts)
        .unwrap_or_else(|| file_mtime_to_utc(file_path));

    let summary = clean_summary(title_entry.as_deref())
        .or_else(|| clean_summary(header_title.as_deref()))
        .or_else(|| clean_summary(last_user_text.as_deref()))
        .or_else(|| clean_summary(last_assistant_text.as_deref()));

    let delete_target = Some(SessionDeleteTarget {
        root: root.to_path_buf(),
        path: file_path.to_path_buf(),
        kind: SessionDeleteKind::File,
    });

    let id = Session::stable_id(CliType::OhMyPi, &session_id, &project_dir);

    Ok(Some(Session {
        id,
        cli_type: CliType::OhMyPi,
        session_id,
        project_dir,
        project_name,
        last_active_at,
        summary,
        delete_target,
    }))
}

fn parse_ts(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

/// 从 omp message.content 提取可展示文本。
/// content 可能是 string，也可能是 `[{type:text,text:...}, {type:thinking,...}]`。
fn extract_message_text(content: Option<&Value>) -> Option<String> {
    let content = content?;
    match content {
        Value::String(s) => {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        }
        Value::Array(parts) => {
            let mut texts = Vec::new();
            for part in parts {
                let ty = part.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if ty == "text" || ty == "input_text" || ty == "output_text" {
                    if let Some(t) = part.get("text").and_then(|v| v.as_str()) {
                        let t = t.trim();
                        if !t.is_empty() {
                            texts.push(t);
                        }
                    }
                }
            }
            if texts.is_empty() {
                None
            } else {
                Some(texts.join(" "))
            }
        }
        _ => None,
    }
}

fn file_mtime_to_utc(path: &Path) -> DateTime<Utc> {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|st| {
            let secs = st
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()?
                .as_secs() as i64;
            Some(DateTime::<Utc>::from_timestamp(secs, 0).unwrap_or_else(Utc::now))
        })
        .unwrap_or_else(Utc::now)
}

#[cfg(test)]
mod tests {
    use super::{extract_message_text, OhMyPiScanner};
    use crate::scanner::SessionScanner;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn omp_scanner_parses_basic_header() {
        let dir = tempdir().unwrap();
        let group = dir.path().join("group1");
        fs::create_dir_all(&group).unwrap();

        let session_file = group.join("2026-01-01_abc123.jsonl");
        let content = r#"{"type":"session","id":"abc123","cwd":"/Users/xb/project","title":"修复登录","timestamp":"2026-07-01T10:00:00Z"}
{"type":"message","message":{"role":"user","content":"hello"}}
"#;
        fs::write(&session_file, content).unwrap();

        let scanner = OhMyPiScanner::with_root(dir.path().to_path_buf());
        let sessions = scanner.scan_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        let s = &sessions[0];
        assert_eq!(s.cli_type, crate::models::CliType::OhMyPi);
        assert_eq!(s.session_id, "abc123");
        assert_eq!(s.project_dir, PathBuf::from("/Users/xb/project"));
        assert_eq!(s.project_name, "project");
        assert_eq!(s.summary.as_deref(), Some("修复登录"));
        assert!(s.delete_target.is_some());
    }

    #[test]
    fn omp_scanner_parses_title_before_session_header() {
        let dir = tempdir().unwrap();
        let group = dir.path().join("group1");
        fs::create_dir_all(&group).unwrap();

        let session_file = group.join("2026-07-13_sid.jsonl");
        let content = r#"{"type":"title","v":1,"title":"Finalize checklist and code review","source":"auto"}
{"type":"session","version":3,"id":"019f5c0d-1f1d-7000-96bc-f5be43f115fc","timestamp":"2026-07-13T15:16:31.647Z","cwd":"/Users/xb/Desktop/codes/fast-start"}
{"type":"message","id":"m1","timestamp":"2026-07-13T15:20:00Z","message":{"role":"user","content":[{"type":"text","text":"继续"}]}}
"#;
        fs::write(&session_file, content).unwrap();

        let scanner = OhMyPiScanner::with_root(dir.path().to_path_buf());
        let sessions = scanner.scan_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        let s = &sessions[0];
        assert_eq!(s.session_id, "019f5c0d-1f1d-7000-96bc-f5be43f115fc");
        assert_eq!(
            s.project_dir,
            PathBuf::from("/Users/xb/Desktop/codes/fast-start")
        );
        assert_eq!(s.summary.as_deref(), Some("Finalize checklist and code review"));
        assert_eq!(
            s.last_active_at.to_rfc3339(),
            "2026-07-13T15:20:00+00:00"
        );
    }

    #[test]
    fn omp_scanner_falls_back_to_user_message_summary() {
        let dir = tempdir().unwrap();
        let session_file = dir.path().join("no-title.jsonl");
        let content = r#"{"type":"session","id":"sid-1","cwd":"/tmp/demo","timestamp":"2026-07-01T10:00:00Z"}
{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"助手回复"}]}}
{"type":"message","message":{"role":"user","content":[{"type":"text","text":"真实用户问题"}]}}
"#;
        fs::write(&session_file, content).unwrap();

        let scanner = OhMyPiScanner::with_root(dir.path().to_path_buf());
        let sessions = scanner.scan_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].summary.as_deref(), Some("真实用户问题"));
    }

    #[test]
    fn omp_scanner_ignores_non_session_files() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("not-a-session.jsonl");
        fs::write(&f, r#"{"type":"message","content":"no header"}"#).unwrap();

        let scanner = OhMyPiScanner::with_root(dir.path().to_path_buf());
        let sessions = scanner.scan_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn extract_message_text_supports_string_and_parts() {
        assert_eq!(
            extract_message_text(Some(&json!("hello"))),
            Some("hello".into())
        );
        assert_eq!(
            extract_message_text(Some(&json!([
                {"type":"thinking","thinking":"x"},
                {"type":"text","text":"可见文本"}
            ]))),
            Some("可见文本".into())
        );
        assert_eq!(extract_message_text(Some(&json!([]))), None);
    }
}
