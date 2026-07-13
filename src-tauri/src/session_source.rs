//! 共享 session 源探测：preflight 与 health inspect 必须走同一函数，禁止两套 IO。
//!
//! OpenCode 源语义 = SQLite **行**是否存在，不是 `opencode.db` 文件是否存在。

use crate::models::{CliType, Session, SessionDeleteKind, SessionDeleteTarget};
use rusqlite::Connection;
use std::fs;
use std::path::Path;

/// 目录有界 du（与 health design / roadmap §4.2 对齐）。
const DIR_MAX_DEPTH: u32 = 3;
const DIR_MAX_FILES: u32 = 2000;
const DIR_BUDGET_MS: u128 = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceCheck {
    /// 缓存窗或缺少 delete_target 等元数据，无法判断。
    Unverified,
    /// 源载体缺失（文件/目录不存在，或 OpenCode 行不在 db）。
    Missing,
    Present {
        approx_bytes: Option<u64>,
        size_capped: bool,
    },
}

/// preflight / inspect 共用入口。
pub fn check_session_source(session: &Session, ops_ready: bool) -> SourceCheck {
    if !ops_ready {
        return SourceCheck::Unverified;
    }

    // OpenCode：强制按行探测，禁止用 db 文件存在性代替。
    if session.cli_type == CliType::OpenCode {
        return check_opencode_row(session);
    }

    let Some(target) = session.delete_target.as_ref() else {
        return SourceCheck::Unverified;
    };
    check_path_target(target)
}

fn check_path_target(target: &SessionDeleteTarget) -> SourceCheck {
    let path = &target.path;
    match target.kind {
        SessionDeleteKind::File => {
            if !path.is_file() {
                return SourceCheck::Missing;
            }
            let approx_bytes = fs::metadata(path).ok().map(|m| m.len());
            SourceCheck::Present {
                approx_bytes,
                size_capped: false,
            }
        }
        SessionDeleteKind::Directory => {
            if !path.is_dir() {
                return SourceCheck::Missing;
            }
            let (approx_bytes, size_capped) = approx_dir_size(path);
            SourceCheck::Present {
                approx_bytes,
                size_capped,
            }
        }
    }
}

fn check_opencode_row(session: &Session) -> SourceCheck {
    let Some(target) = session.delete_target.as_ref() else {
        return SourceCheck::Unverified;
    };
    let db_path = &target.path;
    if !db_path.is_file() {
        return SourceCheck::Missing;
    }
    match opencode_session_row_exists(db_path, &session.session_id) {
        Ok(true) => {
            // 行存在：不把整库体积当 session 体积
            SourceCheck::Present {
                approx_bytes: None,
                size_capped: false,
            }
        }
        Ok(false) => SourceCheck::Missing,
        // db 损坏/锁失败：对调用方视为缺失源（block），避免假阳性 Present
        Err(_) => SourceCheck::Missing,
    }
}

fn opencode_session_row_exists(db_path: &Path, session_id: &str) -> Result<bool, String> {
    let conn =
        Connection::open(db_path).map_err(|err| format!("打开 opencode.db 失败: {err}"))?;
    let mut stmt = conn
        .prepare("SELECT 1 FROM session WHERE id = ?1 LIMIT 1")
        .map_err(|err| format!("查询 opencode session 失败: {err}"))?;
    let mut rows = stmt
        .query([session_id])
        .map_err(|err| format!("查询 opencode session 失败: {err}"))?;
    Ok(rows.next().map_err(|err| err.to_string())?.is_some())
}

/// 返回 `(approx_bytes, size_capped)`；超限时 `approx_bytes=None` 且 `size_capped=true`。
fn approx_dir_size(root: &Path) -> (Option<u64>, bool) {
    use std::time::Instant;
    let started = Instant::now();
    let mut total = 0u64;
    let mut files = 0u32;
    // (path, depth from root children = 1)
    let mut stack = vec![(root.to_path_buf(), 0u32)];
    while let Some((dir, depth)) = stack.pop() {
        if started.elapsed().as_millis() > DIR_BUDGET_MS {
            return (None, true);
        }
        if depth > DIR_MAX_DEPTH {
            return (None, true);
        }
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            if started.elapsed().as_millis() > DIR_BUDGET_MS {
                return (None, true);
            }
            let path = entry.path();
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_dir() {
                let next_depth = depth + 1;
                if next_depth > DIR_MAX_DEPTH {
                    return (None, true);
                }
                stack.push((path, next_depth));
            } else if meta.is_file() {
                files = files.saturating_add(1);
                if files > DIR_MAX_FILES {
                    return (None, true);
                }
                total = total.saturating_add(meta.len());
            }
        }
    }
    (Some(total), false)
}

#[cfg(test)]
mod tests {
    use super::{check_session_source, SourceCheck};
    use crate::models::{
        CliType, Session, SessionDeleteKind, SessionDeleteTarget,
    };
    use chrono::Utc;
    use rusqlite::Connection;
    use std::fs;
    use std::path::PathBuf;

    fn session_with(
        cli: CliType,
        session_id: &str,
        project: PathBuf,
        target: Option<SessionDeleteTarget>,
    ) -> Session {
        Session {
            id: Session::stable_id(cli, session_id, &project),
            cli_type: cli,
            session_id: session_id.to_string(),
            project_dir: project.clone(),
            project_name: Session::project_name_from_dir(&project),
            last_active_at: Utc::now(),
            summary: None,
            delete_target: target,
        }
    }

    fn fixture_opencode_db(path: &std::path::Path, session_id: &str) {
        let conn = Connection::open(path).unwrap();
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
        conn.execute(
            "INSERT INTO session (id, project_id, slug, directory, title, version, time_created, time_updated)
             VALUES (?1, 'p', 's', '/tmp/p', 't', '1', 1, 2)",
            [session_id],
        )
        .unwrap();
    }

    #[test]
    fn ops_not_ready_is_unverified() {
        let s = session_with(CliType::Codex, "abc", PathBuf::from("/tmp/p"), None);
        assert_eq!(
            check_session_source(&s, false),
            SourceCheck::Unverified
        );
    }

    #[test]
    fn file_target_present_and_missing() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("s.jsonl");
        fs::write(&file, b"hello").unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: file.clone(),
            kind: SessionDeleteKind::File,
        };
        let s = session_with(
            CliType::Codex,
            "abc",
            PathBuf::from("/tmp/p"),
            Some(target.clone()),
        );
        match check_session_source(&s, true) {
            SourceCheck::Present {
                approx_bytes: Some(5),
                size_capped: false,
            } => {}
            other => panic!("expected present file, got {other:?}"),
        }

        fs::remove_file(&file).unwrap();
        assert_eq!(check_session_source(&s, true), SourceCheck::Missing);
    }

    #[test]
    fn directory_target_kind_mismatch_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("not-a-dir");
        fs::write(&file, b"x").unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: file,
            kind: SessionDeleteKind::Directory,
        };
        let s = session_with(
            CliType::Cursor,
            "c1",
            PathBuf::from("/tmp/p"),
            Some(target),
        );
        assert_eq!(check_session_source(&s, true), SourceCheck::Missing);
    }

    #[test]
    fn opencode_uses_row_not_db_file() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("opencode.db");
        fixture_opencode_db(&db, "ses_alive");
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: db.clone(),
            kind: SessionDeleteKind::File,
        };

        let alive = session_with(
            CliType::OpenCode,
            "ses_alive",
            PathBuf::from("/tmp/p"),
            Some(target.clone()),
        );
        assert!(matches!(
            check_session_source(&alive, true),
            SourceCheck::Present { .. }
        ));

        // 行已删但 db 文件仍在 → Missing（禁止把 db 存在当源存在）
        Connection::open(&db)
            .unwrap()
            .execute("DELETE FROM session WHERE id = ?1", ["ses_alive"])
            .unwrap();
        assert_eq!(check_session_source(&alive, true), SourceCheck::Missing);
        assert!(db.is_file());
    }

    #[test]
    fn opencode_without_delete_target_is_unverified_even_when_ops_ready() {
        let s = session_with(
            CliType::OpenCode,
            "ses_x",
            PathBuf::from("/tmp/p"),
            None,
        );
        assert_eq!(check_session_source(&s, true), SourceCheck::Unverified);
    }
}
