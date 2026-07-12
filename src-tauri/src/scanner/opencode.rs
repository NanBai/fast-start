use crate::models::{CliType, Session, SessionDeleteKind, SessionDeleteTarget};
use crate::scanner::{clean_summary, ScanError, SessionScanner};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 扫描 OpenCode 本地 session（SQLite）。
///
/// 存储：`$XDG_DATA_HOME/opencode/opencode.db` 或 `~/.local/share/opencode/opencode.db`
/// 表 `session`：id / directory / title / time_updated / time_created
/// resume：`opencode --session <id>`（需 cd 到 directory）
#[derive(Default)]
pub struct OpenCodeScanner {
    /// 测试时可注入 db 文件路径；生产读默认 data dir。
    db_path: Option<PathBuf>,
}

impl OpenCodeScanner {
    #[cfg(test)]
    pub fn with_db(db_path: PathBuf) -> Self {
        Self {
            db_path: Some(db_path),
        }
    }

    fn db_path(&self) -> Result<PathBuf, ScanError> {
        if let Some(path) = &self.db_path {
            return Ok(path.clone());
        }
        Ok(default_data_dir()?.join("opencode.db"))
    }

    fn data_root(&self) -> Result<PathBuf, ScanError> {
        if let Some(path) = &self.db_path {
            return Ok(path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| path.clone()));
        }
        default_data_dir()
    }
}

fn default_data_dir() -> Result<PathBuf, ScanError> {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        let xdg = xdg.trim();
        if !xdg.is_empty() {
            return Ok(PathBuf::from(xdg).join("opencode"));
        }
    }
    dirs::home_dir()
        .map(|home| home.join(".local/share/opencode"))
        .ok_or_else(|| ScanError::NotFound("无法定位用户主目录".to_string()))
}

impl SessionScanner for OpenCodeScanner {
    fn cli_type(&self) -> CliType {
        CliType::OpenCode
    }

    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError> {
        let db_path = self.db_path()?;
        if !db_path.exists() {
            return Err(ScanError::NotFound(
                "opencode session 数据库不存在".to_string(),
            ));
        }

        let data_root = self.data_root()?;
        let conn = Connection::open(&db_path)
            .map_err(|err| ScanError::Parse(format!("打开 opencode.db 失败: {err}")))?;

        // 归档 session 仍列出（用户可能想恢复）；删除由 delete_session_by_id 处理。
        let mut stmt = conn
            .prepare(
                "SELECT id, directory, title, time_updated, time_created \
                 FROM session \
                 ORDER BY time_updated DESC",
            )
            .map_err(|err| ScanError::Parse(format!("查询 session 失败: {err}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|err| ScanError::Parse(format!("读取 session 行失败: {err}")))?;

        let mut sessions = Vec::new();
        for row in rows.flatten() {
            let (session_id, directory, title, time_updated, time_created) = row;
            if session_id.is_empty() || directory.is_empty() {
                continue;
            }
            let project_dir = PathBuf::from(directory);
            let last_active_at = ms_to_datetime(time_updated)
                .or_else(|| ms_to_datetime(time_created))
                .unwrap_or_else(|| DateTime::<Utc>::from(SystemTime::UNIX_EPOCH));
            let summary = clean_summary(Some(&title));

            sessions.push(Session {
                id: Session::stable_id(CliType::OpenCode, &session_id, &project_dir),
                cli_type: CliType::OpenCode,
                session_id: session_id.clone(),
                project_name: Session::project_name_from_dir(&project_dir),
                project_dir,
                last_active_at,
                summary,
                // OpenCode 按行删 SQLite；delete_target 记录 db 位置供安全边界校验。
                delete_target: Some(SessionDeleteTarget {
                    root: data_root.clone(),
                    path: db_path.clone(),
                    kind: SessionDeleteKind::File,
                }),
            });
        }

        Ok(sessions)
    }
}

fn ms_to_datetime(ms: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp_millis(ms)
}

/// 从 opencode.db 删除指定 session 行（级联清理由 FK 负责）。
/// 仅允许在默认 data dir 或显式 fixture db 上操作；校验 session id 字符集。
pub fn delete_session_by_id(session_id: &str) -> Result<(), String> {
    crate::security::validate_session_id(session_id)?;
    let db_path = default_data_dir()
        .map_err(|e| e.message())?
        .join("opencode.db");
    delete_session_in_db(&db_path, session_id)
}

pub fn delete_session_in_db(db_path: &Path, session_id: &str) -> Result<(), String> {
    crate::security::validate_session_id(session_id)?;
    if !db_path.exists() {
        return Err("opencode session 数据库不存在".to_string());
    }
    // 安全：db 必须落在 …/opencode/opencode.db 命名模式，且 canonicalize 后仍在 root 内。
    let root = db_path
        .parent()
        .ok_or_else(|| "opencode 数据目录无效".to_string())?;
    let root_can = std::fs::canonicalize(root).map_err(|_| "opencode 数据目录不可用".to_string())?;
    let db_can = std::fs::canonicalize(db_path).map_err(|_| "opencode.db 不可用".to_string())?;
    if !db_can.starts_with(&root_can) || db_can == root_can {
        return Err("opencode.db 不在允许目录内".to_string());
    }
    if db_can.file_name().and_then(|n| n.to_str()) != Some("opencode.db") {
        return Err("拒绝操作非 opencode.db 文件".to_string());
    }

    let conn = Connection::open(&db_can).map_err(|err| format!("打开 opencode.db 失败: {err}"))?;
    let changed = conn
        .execute("DELETE FROM session WHERE id = ?1", [session_id])
        .map_err(|err| format!("删除 opencode session 失败: {err}"))?;
    if changed == 0 {
        return Err("opencode session 不存在".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{delete_session_in_db, OpenCodeScanner};
    use crate::scanner::SessionScanner;
    use rusqlite::Connection;
    use std::fs;

    fn create_fixture_db(path: &std::path::Path) {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE session (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                workspace_id TEXT,
                parent_id TEXT,
                slug TEXT NOT NULL,
                directory TEXT NOT NULL,
                path TEXT,
                title TEXT NOT NULL,
                version TEXT NOT NULL,
                time_created INTEGER NOT NULL,
                time_updated INTEGER NOT NULL,
                time_archived INTEGER
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO session (id, project_id, slug, directory, title, version, time_created, time_updated)
             VALUES (?1, 'p1', 'slug', ?2, ?3, '1.0', ?4, ?5)",
            rusqlite::params![
                "ses_abc123",
                "/tmp/opencode-project",
                "Implement OpenCode support",
                1_700_000_000_000i64,
                1_700_000_100_000i64,
            ],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO session (id, project_id, slug, directory, title, version, time_created, time_updated)
             VALUES (?1, 'p1', 'slug2', ?2, ?3, '1.0', ?4, ?5)",
            rusqlite::params![
                "ses_def456",
                "/tmp/other",
                "Other session",
                1_700_000_000_000i64,
                1_700_000_200_000i64,
            ],
        )
        .unwrap();
    }

    #[test]
    fn scanner_reads_fixture_db_sessions() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("opencode.db");
        create_fixture_db(&db);

        let sessions = OpenCodeScanner::with_db(db).scan_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].session_id, "ses_def456"); // newer updated first
        assert_eq!(
            sessions[0].project_dir,
            std::path::PathBuf::from("/tmp/other")
        );
        assert_eq!(sessions[0].summary.as_deref(), Some("Other session"));
        assert_eq!(sessions[0].last_active_at.timestamp_millis(), 1_700_000_200_000);
        assert!(sessions[0].delete_target.is_some());
    }

    #[test]
    fn delete_session_removes_row_only() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("opencode.db");
        create_fixture_db(&db);

        delete_session_in_db(&db, "ses_abc123").unwrap();
        let sessions = OpenCodeScanner::with_db(db.clone()).scan_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "ses_def456");
        assert!(db.exists());
        assert!(fs::metadata(&db).unwrap().len() > 0);
    }

    #[test]
    fn delete_rejects_bad_session_id() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("opencode.db");
        create_fixture_db(&db);
        assert!(delete_session_in_db(&db, "bad;id").is_err());
    }
}
