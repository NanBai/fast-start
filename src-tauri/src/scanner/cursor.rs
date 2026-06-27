use crate::models::{CliType, Session, SessionDeleteKind, SessionDeleteTarget};
use crate::scanner::{ScanError, SessionScanner};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Default)]
pub struct CursorScanner {
    root: Option<PathBuf>,
}

impl CursorScanner {
    #[cfg(test)]
    pub fn with_root(root: PathBuf) -> Self {
        Self { root: Some(root) }
    }

    fn root(&self) -> Result<PathBuf, ScanError> {
        if let Some(root) = &self.root {
            return Ok(root.clone());
        }
        dirs::home_dir()
            .map(|home| home.join(".cursor/chats"))
            .ok_or_else(|| ScanError::NotFound("无法定位用户主目录".to_string()))
    }
}

/// cursor 的 `~/.cursor/chats/<hash>/<uuid>/meta.json` 内容。
#[derive(Debug, Deserialize)]
struct CursorMeta {
    #[serde(default)]
    title: Option<String>,
    #[serde(default, rename = "createdAtMs", alias = "created_at_ms")]
    created_at_ms: Option<u64>,
    #[serde(default, rename = "updatedAtMs", alias = "updated_at_ms")]
    updated_at_ms: Option<u64>,
}

struct CursorStoreInfo {
    cwd: PathBuf,
    first_user_query: Option<String>,
}

impl SessionScanner for CursorScanner {
    fn cli_type(&self) -> CliType {
        CliType::Cursor
    }

    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError> {
        let root = self.root()?;

        if !root.exists() {
            return Err(ScanError::NotFound("cursor chat 目录不存在".to_string()));
        }

        let mut sessions = Vec::new();

        // 目录结构：~/.cursor/chats/<workspace-hash>/<chat-uuid>/{meta.json, store.db}
        for hash_entry in fs::read_dir(&root)? {
            let hash_entry = hash_entry?;
            if !hash_entry.file_type()?.is_dir() {
                continue;
            }
            for chat_entry in fs::read_dir(hash_entry.path())? {
                let chat_entry = chat_entry?;
                let chat_dir = chat_entry.path();
                if !chat_entry.file_type()?.is_dir() {
                    continue;
                }

                let Some(session_id) = chat_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(str::to_string)
                else {
                    continue;
                };

                // cursor resume 必须在正确 workspace（cwd）下才能恢复指定 chat。
                // cwd 来自 chat 自己 store.db 里 cursor 注入的 "Workspace Path: <路径>"。
                // 拿不到的 chat 跳过（resume 会失败）。
                let Some(store_info) = extract_store_info(&chat_dir.join("store.db")) else {
                    continue;
                };

                let meta_path = chat_dir.join("meta.json");
                let Ok(meta_content) = fs::read_to_string(&meta_path) else {
                    continue;
                };
                let Ok(meta) = serde_json::from_str::<CursorMeta>(&meta_content) else {
                    continue;
                };

                let summary = store_info
                    .first_user_query
                    .as_deref()
                    .and_then(|query| crate::scanner::clean_summary(Some(query)))
                    .or_else(|| crate::scanner::clean_summary(meta.title.as_deref()));
                let Some(summary) = summary else {
                    // 既没有真实 user query，也没有 meta title 时无可展示简介，跳过。
                    continue;
                };

                let last_active_at = meta
                    .updated_at_ms
                    .or(meta.created_at_ms)
                    .and_then(ms_to_datetime)
                    .unwrap_or_else(|| DateTime::<Utc>::from(SystemTime::now()));

                sessions.push(Session {
                    id: Session::stable_id(CliType::Cursor, &session_id, &store_info.cwd),
                    cli_type: CliType::Cursor,
                    session_id,
                    project_name: Session::project_name_from_dir(&store_info.cwd),
                    project_dir: store_info.cwd,
                    last_active_at,
                    summary: Some(summary),
                    delete_target: Some(SessionDeleteTarget {
                        root: root.clone(),
                        path: chat_dir,
                        kind: SessionDeleteKind::Directory,
                    }),
                });
            }
        }

        sessions.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));
        Ok(sessions)
    }
}

/// 从 chat 的 store.db 提取 cursor 注入的 "Workspace Path: <真实路径>"。
/// cursor 把 workspace 真实路径写进 system prompt（存在 blobs 表）。
/// 返回 canonicalize 后的路径（验证目录存在），拿不到返回 None。
fn extract_store_info(db_path: &Path) -> Option<CursorStoreInfo> {
    let conn = rusqlite::Connection::open(db_path).ok()?;
    let mut stmt = conn.prepare("SELECT data FROM blobs;").ok()?;
    let rows = stmt.query_map([], |row| row.get::<_, Vec<u8>>(0)).ok()?;
    let mut cwd = None;
    let mut first_user_query = None;
    for row in rows.flatten() {
        let text = String::from_utf8_lossy(&row);
        if cwd.is_none() {
            cwd = extract_workspace_path_from_text(&text);
        }
        if first_user_query.is_none() {
            first_user_query = extract_user_query(&text);
        }
        if cwd.is_some() && first_user_query.is_some() {
            break;
        }
    }
    cwd.map(|cwd| CursorStoreInfo {
        cwd,
        first_user_query,
    })
}

fn extract_workspace_path_from_text(text: &str) -> Option<PathBuf> {
    for line in text.lines() {
        let Some(idx) = line.find("Workspace Path:") else {
            continue;
        };
        let rest = &line[idx + "Workspace Path:".len()..];
        // 路径取到第一个空白 或 字面 "\n"（cursor 内容是 JSON，换行转义成 \n 两字符，
        // 不是真空白），否则 candidate 会带上 "\nIf..." 导致 canonicalize 失败。
        let candidate: String = rest
            .trim_start()
            .chars()
            .take_while(|c| !c.is_whitespace() && !matches!(*c, '\\' | ','))
            .collect();
        if candidate.is_empty() {
            continue;
        }
        if let Ok(cwd) = fs::canonicalize(&candidate) {
            if cwd.is_dir() {
                return Some(cwd);
            }
        }
    }
    None
}

fn extract_user_query(text: &str) -> Option<String> {
    let start = text.find(r#"{"role":"user""#)?;
    let value: serde_json::Value = serde_json::from_str(&text[start..]).ok()?;
    if value.get("role")?.as_str()? != "user" {
        return None;
    }
    let content = value.get("content")?;
    let raw = if let Some(text) = content.as_str() {
        text.to_string()
    } else {
        content
            .as_array()?
            .iter()
            .filter_map(|item| item.get("text").and_then(|value| value.as_str()))
            .collect::<Vec<_>>()
            .join("\n")
    };
    extract_tagged_query(&raw)
}

fn extract_tagged_query(raw: &str) -> Option<String> {
    let start = raw.find("<user_query>")? + "<user_query>".len();
    let end = raw[start..].find("</user_query>")? + start;
    let query = raw[start..end].trim();
    if query.is_empty() {
        None
    } else {
        Some(query.to_string())
    }
}

fn ms_to_datetime(ms: u64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp_millis(ms as i64)
}

#[cfg(test)]
mod tests {
    use super::CursorScanner;
    use crate::scanner::SessionScanner;
    use rusqlite::Connection;
    use std::fs;

    #[test]
    fn scanner_prefers_user_query_over_meta_title() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).unwrap();
        let chat_dir = temp.path().join("hash").join("cursor-fixture");
        fs::create_dir_all(&chat_dir).unwrap();
        fs::write(
            chat_dir.join("meta.json"),
            r#"{"title":"English Auto Title","createdAtMs":1781830800000,"updatedAtMs":1781830860000}"#,
        )
        .unwrap();
        create_cursor_store_db(
            &chat_dir.join("store.db"),
            &[
                format!("Workspace Path: {}\\nIf this path", workspace.display()),
                r#"{"role":"user","content":[{"type":"text","text":"<user_query>\n修复 Cursor 简介显示\n</user_query>"}]}"#.to_string(),
            ],
        );

        let sessions = CursorScanner::with_root(temp.path().to_path_buf())
            .scan_sessions()
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].summary.as_deref(), Some("修复 Cursor 简介显示"));
        let delete_target = sessions[0].delete_target.as_ref().unwrap();
        assert_eq!(delete_target.root, temp.path());
        assert_eq!(delete_target.path, chat_dir);
        assert_eq!(
            delete_target.kind,
            crate::models::SessionDeleteKind::Directory
        );
    }

    #[test]
    fn scanner_falls_back_to_meta_title_without_user_query() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).unwrap();
        let chat_dir = temp.path().join("hash").join("cursor-fixture");
        fs::create_dir_all(&chat_dir).unwrap();
        fs::write(
            chat_dir.join("meta.json"),
            r#"{"title":"优化 Cursor 扫描","createdAtMs":1781830800000,"updatedAtMs":1781830860000}"#,
        )
        .unwrap();
        create_cursor_store_db(
            &chat_dir.join("store.db"),
            &[format!(
                "Workspace Path: {}\\nIf this path",
                workspace.display()
            )],
        );

        let sessions = CursorScanner::with_root(temp.path().to_path_buf())
            .scan_sessions()
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "cursor-fixture");
        assert_eq!(
            sessions[0].project_dir,
            fs::canonicalize(&workspace).unwrap()
        );
        assert_eq!(sessions[0].summary.as_deref(), Some("优化 Cursor 扫描"));
        assert_eq!(sessions[0].last_active_at.timestamp_millis(), 1781830860000);
        let delete_target = sessions[0].delete_target.as_ref().unwrap();
        assert_eq!(delete_target.root, temp.path());
        assert_eq!(delete_target.path, chat_dir);
        assert_eq!(
            delete_target.kind,
            crate::models::SessionDeleteKind::Directory
        );
    }

    fn create_cursor_store_db(path: &std::path::Path, rows: &[String]) {
        let conn = Connection::open(path).expect("fixture db should open");
        conn.execute("CREATE TABLE blobs(data BLOB)", [])
            .expect("fixture table should be created");
        for data in rows {
            conn.execute("INSERT INTO blobs(data) VALUES(?)", [data.as_bytes()])
                .expect("fixture data should be inserted");
        }
    }
}
