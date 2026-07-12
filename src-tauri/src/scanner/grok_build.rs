use crate::models::{CliType, Session, SessionDeleteKind, SessionDeleteTarget};
use crate::scanner::{clean_summary, ScanError, SessionScanner};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 扫描 Grok Build 本地 session。
///
/// 布局（见 `~/.grok/docs/user-guide/17-sessions.md`）：
/// ```text
/// ~/.grok/sessions/<url-encoded-cwd>/<session-id>/summary.json
/// ```
/// 过长 cwd 时 group 目录可能是 slug+hash，真实路径写在 group 内 `.cwd`。
/// `GROK_HOME` 可覆盖默认 `~/.grok`。
#[derive(Default)]
pub struct GrokBuildScanner {
    root: Option<PathBuf>,
}

impl GrokBuildScanner {
    #[cfg(test)]
    pub fn with_root(root: PathBuf) -> Self {
        Self { root: Some(root) }
    }

    fn root(&self) -> Result<PathBuf, ScanError> {
        if let Some(root) = &self.root {
            return Ok(root.clone());
        }
        if let Ok(home) = std::env::var("GROK_HOME") {
            let home = home.trim();
            if !home.is_empty() {
                return Ok(PathBuf::from(home).join("sessions"));
            }
        }
        dirs::home_dir()
            .map(|home| home.join(".grok/sessions"))
            .ok_or_else(|| ScanError::NotFound("无法定位用户主目录".to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct GrokSummary {
    info: Option<GrokSummaryInfo>,
    #[serde(default)]
    session_summary: Option<String>,
    #[serde(default)]
    generated_title: Option<String>,
    #[serde(default)]
    last_active_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GrokSummaryInfo {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
}

impl SessionScanner for GrokBuildScanner {
    fn cli_type(&self) -> CliType {
        CliType::GrokBuild
    }

    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError> {
        let root = self.root()?;

        if !root.exists() {
            return Err(ScanError::NotFound(
                "grok-build session 目录不存在".to_string(),
            ));
        }

        let mut sessions = Vec::new();

        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let group_dir = entry.path();
            if !group_dir.is_dir() {
                continue;
            }

            let group_name = group_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            let fallback_cwd = resolve_group_cwd(&group_dir, group_name);

            for session_entry in fs::read_dir(&group_dir)? {
                let session_entry = session_entry?;
                let session_dir = session_entry.path();
                if !session_dir.is_dir() {
                    continue;
                }

                let summary_path = session_dir.join("summary.json");
                if !summary_path.is_file() {
                    continue;
                }

                if let Some(session) =
                    parse_summary(&root, &session_dir, &summary_path, fallback_cwd.as_ref())?
                {
                    sessions.push(session);
                }
            }
        }

        sessions.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));
        Ok(sessions)
    }
}

/// group 目录 cwd：优先 `.cwd`（长路径 slug+hash 布局）；
/// 否则仅在目录名看起来是 URL-encoded 路径时 percent-decode（至少含 `%`）。
fn resolve_group_cwd(group_dir: &Path, group_name: &str) -> Option<PathBuf> {
    let cwd_file = group_dir.join(".cwd");
    if let Ok(content) = fs::read_to_string(&cwd_file) {
        let path = content.trim();
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    if !group_name.contains('%') {
        return None;
    }
    percent_decode(group_name).map(PathBuf::from)
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = from_hex(bytes[i + 1])?;
            let lo = from_hex(bytes[i + 2])?;
            out.push((hi << 4) | lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn parse_summary(
    root: &Path,
    session_dir: &Path,
    summary_path: &Path,
    fallback_cwd: Option<&PathBuf>,
) -> Result<Option<Session>, ScanError> {
    let content = fs::read_to_string(summary_path)?;
    let parsed: GrokSummary = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };

    let dir_name_id = session_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

    let session_id = parsed
        .info
        .as_ref()
        .and_then(|info| info.id.clone())
        .filter(|id| !id.is_empty())
        .unwrap_or(dir_name_id);
    if session_id.is_empty() {
        return Ok(None);
    }

    // cwd 优先 summary.info.cwd（权威），其次 group 目录名 / .cwd 回退。
    let cwd = parsed
        .info
        .as_ref()
        .and_then(|info| info.cwd.as_ref())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| fallback_cwd.cloned());
    let Some(cwd) = cwd else {
        return Ok(None);
    };

    let summary = clean_summary(parsed.generated_title.as_deref())
        .or_else(|| clean_summary(parsed.session_summary.as_deref()));

    let last_active_at = parse_rfc3339(parsed.last_active_at.as_deref())
        .or_else(|| parse_rfc3339(parsed.updated_at.as_deref()))
        .or_else(|| parse_rfc3339(parsed.created_at.as_deref()))
        .unwrap_or_else(|| file_mtime(summary_path));

    Ok(Some(Session {
        id: Session::stable_id(CliType::GrokBuild, &session_id, &cwd),
        cli_type: CliType::GrokBuild,
        session_id,
        project_name: Session::project_name_from_dir(&cwd),
        project_dir: cwd,
        last_active_at,
        summary,
        delete_target: Some(SessionDeleteTarget {
            root: root.to_path_buf(),
            path: session_dir.to_path_buf(),
            kind: SessionDeleteKind::Directory,
        }),
    }))
}

fn parse_rfc3339(value: Option<&str>) -> Option<DateTime<Utc>> {
    let value = value?;
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|ts| ts.with_timezone(&Utc))
}

fn file_mtime(path: &Path) -> DateTime<Utc> {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .map(DateTime::<Utc>::from)
        .unwrap_or_else(|_| DateTime::<Utc>::from(SystemTime::now()))
}

#[cfg(test)]
mod tests {
    use super::{percent_decode, GrokBuildScanner};
    use crate::models::SessionDeleteKind;
    use crate::scanner::SessionScanner;
    use std::fs;

    #[test]
    fn percent_decode_restores_project_path() {
        assert_eq!(
            percent_decode("%2FUsers%2Fxb%2FDesktop%2Fcodes%2Ffast-start").as_deref(),
            Some("/Users/xb/Desktop/codes/fast-start")
        );
    }

    #[test]
    fn scanner_reads_fixture_summary_and_deletes_session_dir() {
        let temp = tempfile::tempdir().unwrap();
        let group = temp
            .path()
            .join("%2Ftmp%2Fgrok-project");
        let session_dir = group.join("019f559d-97a5-7ac0-9e2b-3c340dd33d6b");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(
            session_dir.join("summary.json"),
            r#"{
              "info": {
                "id": "019f559d-97a5-7ac0-9e2b-3c340dd33d6b",
                "cwd": "/tmp/grok-project"
              },
              "session_summary": "wire up grok build sessions",
              "generated_title": "Add Grok Build Support",
              "created_at": "2026-07-12T09:00:00Z",
              "updated_at": "2026-07-12T09:10:00Z",
              "last_active_at": "2026-07-12T09:12:00Z"
            }"#,
        )
        .unwrap();
        // 额外噪声文件：不应被当成 session
        fs::write(temp.path().join("session_search.sqlite"), "x").unwrap();
        fs::write(group.join("prompt_history.jsonl"), "{}\n").unwrap();

        let sessions = GrokBuildScanner::with_root(temp.path().to_path_buf())
            .scan_sessions()
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "019f559d-97a5-7ac0-9e2b-3c340dd33d6b");
        assert_eq!(
            sessions[0].project_dir,
            std::path::PathBuf::from("/tmp/grok-project")
        );
        assert_eq!(sessions[0].summary.as_deref(), Some("Add Grok Build Support"));
        assert_eq!(
            sessions[0].last_active_at.to_rfc3339(),
            "2026-07-12T09:12:00+00:00"
        );
        let delete_target = sessions[0].delete_target.as_ref().unwrap();
        assert_eq!(delete_target.root, temp.path());
        assert_eq!(delete_target.path, session_dir);
        assert_eq!(delete_target.kind, SessionDeleteKind::Directory);
    }

    #[test]
    fn scanner_uses_dot_cwd_when_summary_cwd_missing() {
        let temp = tempfile::tempdir().unwrap();
        let group = temp.path().join("long-slug-hash");
        let session_dir = group.join("session-abc");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(group.join(".cwd"), "/tmp/from-dot-cwd\n").unwrap();
        fs::write(
            session_dir.join("summary.json"),
            r#"{
              "info": { "id": "session-abc" },
              "session_summary": "fallback cwd works"
            }"#,
        )
        .unwrap();

        let sessions = GrokBuildScanner::with_root(temp.path().to_path_buf())
            .scan_sessions()
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(
            sessions[0].project_dir,
            std::path::PathBuf::from("/tmp/from-dot-cwd")
        );
        assert_eq!(sessions[0].summary.as_deref(), Some("fallback cwd works"));
    }

    #[test]
    fn scanner_skips_sessions_without_cwd() {
        let temp = tempfile::tempdir().unwrap();
        let group = temp.path().join("no-cwd-group");
        let session_dir = group.join("orphan");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(
            session_dir.join("summary.json"),
            r#"{"info":{"id":"orphan"},"session_summary":"no cwd"}"#,
        )
        .unwrap();

        let sessions = GrokBuildScanner::with_root(temp.path().to_path_buf())
            .scan_sessions()
            .unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn scanner_uses_percent_decoded_group_name_when_summary_cwd_missing() {
        let temp = tempfile::tempdir().unwrap();
        let group = temp.path().join("%2Ftmp%2Fdecoded-only");
        let session_dir = group.join("session-decoded");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(
            session_dir.join("summary.json"),
            r#"{
              "info": { "id": "session-decoded" },
              "session_summary": "cwd from encoded group name"
            }"#,
        )
        .unwrap();

        let sessions = GrokBuildScanner::with_root(temp.path().to_path_buf())
            .scan_sessions()
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(
            sessions[0].project_dir,
            std::path::PathBuf::from("/tmp/decoded-only")
        );
        assert_eq!(
            sessions[0].summary.as_deref(),
            Some("cwd from encoded group name")
        );
    }
}
