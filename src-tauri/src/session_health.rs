//! Session 健康探测：只读 cwd/源/flags；不向调用方暴露 delete_target 路径。

use crate::models::Session;
use crate::session_source::{check_session_source, SourceCheck};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const INSPECT_SESSION_HEALTH_LIMIT: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionHealthReport {
    pub items: Vec<SessionHealth>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionHealth {
    pub session_list_id: String,
    pub cwd_exists: bool,
    /// `null` = 缓存窗或无法源探测
    pub source_exists: Option<bool>,
    pub approx_bytes: Option<u64>,
    pub flags: Vec<String>,
}

/// 将单条 session 映射为健康结果（未知 session 由调用方省略）。
pub fn inspect_one(session: &Session, ops_ready: bool) -> SessionHealth {
    let cwd_exists = Path::new(&session.project_dir).is_dir();
    let mut flags = Vec::new();

    if !cwd_exists {
        flags.push("missing_cwd".to_string());
    }

    let empty_summary = session
        .summary
        .as_ref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(true);
    if empty_summary {
        flags.push("empty_summary".to_string());
    }

    let (source_exists, approx_bytes) = match check_session_source(session, ops_ready) {
        SourceCheck::Unverified => {
            flags.push("cache_limited".to_string());
            (None, None)
        }
        SourceCheck::Missing => {
            flags.push("missing_source".to_string());
            (Some(false), None)
        }
        SourceCheck::Present {
            approx_bytes,
            size_capped,
        } => {
            if size_capped {
                flags.push("size_capped".to_string());
            }
            (Some(true), approx_bytes)
        }
    };

    SessionHealth {
        session_list_id: session.id.clone(),
        cwd_exists,
        source_exists,
        approx_bytes,
        flags,
    }
}

#[cfg(test)]
mod tests {
    use super::{inspect_one, INSPECT_SESSION_HEALTH_LIMIT};
    use crate::models::{
        CliType, Session, SessionDeleteKind, SessionDeleteTarget,
    };
    use chrono::Utc;
    use rusqlite::Connection;
    use std::fs;
    use std::path::PathBuf;

    fn make_session(
        cli: CliType,
        session_id: &str,
        project: PathBuf,
        summary: Option<String>,
        target: Option<SessionDeleteTarget>,
    ) -> Session {
        Session {
            id: Session::stable_id(cli, session_id, &project),
            cli_type: cli,
            session_id: session_id.to_string(),
            project_dir: project.clone(),
            project_name: Session::project_name_from_dir(&project),
            last_active_at: Utc::now(),
            summary,
            delete_target: target,
        }
    }

    #[test]
    fn inspect_limit_is_200() {
        assert_eq!(INSPECT_SESSION_HEALTH_LIMIT, 200);
    }

    #[test]
    fn missing_cwd_and_empty_summary_flags() {
        let s = make_session(
            CliType::Codex,
            "abc",
            PathBuf::from("/tmp/no-such-fast-start-health-cwd"),
            Some("   ".into()),
            None,
        );
        let h = inspect_one(&s, false);
        assert!(!h.cwd_exists);
        assert!(h.flags.iter().any(|f| f == "missing_cwd"));
        assert!(h.flags.iter().any(|f| f == "empty_summary"));
        assert!(h.flags.iter().any(|f| f == "cache_limited"));
        assert_eq!(h.source_exists, None);
        // 不得包含路径字段：序列化后检查
        let json = serde_json::to_string(&h).unwrap();
        assert!(!json.contains("delete_target"));
        assert!(!json.contains("no-such-fast-start"));
    }

    #[test]
    fn file_source_present_reports_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("s.jsonl");
        fs::write(&file, b"hello!").unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: file,
            kind: SessionDeleteKind::File,
        };
        let s = make_session(
            CliType::Codex,
            "abc",
            temp.path().to_path_buf(),
            Some("title".into()),
            Some(target),
        );
        let h = inspect_one(&s, true);
        assert!(h.cwd_exists);
        assert_eq!(h.source_exists, Some(true));
        assert_eq!(h.approx_bytes, Some(6));
        assert!(!h.flags.iter().any(|f| f == "missing_source"));
    }

    #[test]
    fn opencode_row_missing_is_missing_source_bytes_null() {
        let temp = tempfile::tempdir().unwrap();
        let cwd = temp.path().join("proj");
        fs::create_dir_all(&cwd).unwrap();
        let db = temp.path().join("opencode.db");
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch(
            "CREATE TABLE session (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                slug TEXT NOT NULL,
                directory TEXT NOT NULL,
                title TEXT NOT NULL,
                version TEXT NOT NULL,
                time_created INTEGER NOT NULL,
                time_updated INTEGER NOT NULL
            );",
        )
        .unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: db,
            kind: SessionDeleteKind::File,
        };
        let s = make_session(
            CliType::OpenCode,
            "ses_gone",
            cwd,
            Some("t".into()),
            Some(target),
        );
        let h = inspect_one(&s, true);
        assert_eq!(h.source_exists, Some(false));
        assert_eq!(h.approx_bytes, None);
        assert!(h.flags.iter().any(|f| f == "missing_source"));
    }
}
