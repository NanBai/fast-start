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
        // 同一 hash 下 chat 共享 workspace：cwd 每个 hash 最多成功解析一次 store.db；
        // 有 meta.title 时不为每条 chat 再开 SQLite（启动扫描主瓶颈）。
        for hash_entry in fs::read_dir(&root)? {
            let hash_entry = hash_entry?;
            if !hash_entry.file_type()?.is_dir() {
                continue;
            }
            let mut workspace_cwd: Option<PathBuf> = None;

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

                let meta_path = chat_dir.join("meta.json");
                let Ok(meta_content) = fs::read_to_string(&meta_path) else {
                    continue;
                };
                let Ok(meta) = serde_json::from_str::<CursorMeta>(&meta_content) else {
                    continue;
                };
                let meta_summary = crate::scanner::clean_summary(meta.title.as_deref());
                let store_path = chat_dir.join("store.db");

                // 解析 cwd：优先 hash 缓存；未命中则打开本 chat 的 store.db。
                let mut store_query: Option<String> = None;
                if workspace_cwd.is_none() {
                    match extract_store_info(&store_path) {
                        Some(info) => {
                            workspace_cwd = Some(info.cwd);
                            store_query = info.first_user_query;
                        }
                        None => continue,
                    }
                }

                let Some(cwd) = workspace_cwd.clone() else {
                    continue;
                };

                // 简介：若本轮已打开 store（解析 cwd），优先 user query；
                // 否则有 meta.title 直接用，避免再开 SQLite；无 title 才补查 store。
                let summary = if let Some(q) = store_query.as_deref() {
                    crate::scanner::clean_summary(Some(q)).or(meta_summary)
                } else if let Some(s) = meta_summary {
                    Some(s)
                } else if let Some(info) = extract_store_info(&store_path) {
                    info.first_user_query
                        .as_deref()
                        .and_then(|query| crate::scanner::clean_summary(Some(query)))
                } else {
                    None
                };
                let Some(summary) = summary else {
                    continue;
                };

                let last_active_at = meta_last_active(&meta, &meta_path, &chat_dir);
                sessions.push(Session {
                    id: Session::stable_id(CliType::Cursor, &session_id, &cwd),
                    cli_type: CliType::Cursor,
                    session_id,
                    project_name: Session::project_name_from_dir(&cwd),
                    project_dir: cwd,
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

fn meta_last_active(meta: &CursorMeta, meta_path: &Path, chat_dir: &Path) -> DateTime<Utc> {
    // 缺时间戳时用 meta / chat 目录 mtime，禁止 now()——否则每次扫描
    // 都变成「刚刚」，绕过最近天数过滤并扰乱排序。
    meta.updated_at_ms
        .or(meta.created_at_ms)
        .and_then(ms_to_datetime)
        .or_else(|| path_mtime(meta_path))
        .or_else(|| path_mtime(chat_dir))
        .unwrap_or_else(|| DateTime::<Utc>::from(SystemTime::UNIX_EPOCH))
}

/// 从 chat 的 store.db 提取 cursor 注入的 "Workspace Path: <真实路径>"。
/// cursor 把 workspace 真实路径写进 system prompt（存在 blobs 表）。
/// 返回 canonicalize 后的路径（验证目录存在），拿不到返回 None。
///
/// 查询收敛：只扫可能含 workspace / user 消息的 blob，并优先较小行，
/// 避免把整库对话全文无差别拉进内存。
fn extract_store_info(db_path: &Path) -> Option<CursorStoreInfo> {
    let conn = rusqlite::Connection::open(db_path).ok()?;
    // 过滤 + 小 blob 优先：降低大对话读放大，同时提高 system prompt 先被扫到的概率。
    let mut stmt = conn
        .prepare(
            "SELECT data FROM blobs \
             WHERE instr(data, 'Workspace Path:') > 0 \
                OR instr(data, '\"role\":\"user\"') > 0 \
             ORDER BY length(data) ASC \
             LIMIT 64",
        )
        .ok()?;
    let rows = stmt.query_map([], |row| row.get::<_, Vec<u8>>(0)).ok()?;
    let mut cwd_candidates: Vec<PathBuf> = Vec::new();
    let mut first_user_query = None;
    // 扫完过滤后的 blob（已 LIMIT），再选最长 cwd——避免提前 break 漏掉更具体路径。
    for row in rows.flatten() {
        let text = String::from_utf8_lossy(&row);
        for path in extract_workspace_paths_from_text(&text) {
            if !cwd_candidates.iter().any(|existing| existing == &path) {
                cwd_candidates.push(path);
            }
        }
        if first_user_query.is_none() {
            first_user_query = extract_user_query(&text);
        }
    }
    // 多命中时选最长路径（更具体）；避免 user 文本里短假路径抢先。
    let cwd = cwd_candidates
        .into_iter()
        .max_by_key(|path| path.as_os_str().len())?;
    Some(CursorStoreInfo {
        cwd,
        first_user_query,
    })
}

/// 从 blob 文本提取所有可 canonicalize 且为目录的 Workspace Path。
/// 路径可含空格；边界是 JSON 字面 `\n` / `"` / 真换行，而不是第一个空白。
fn extract_workspace_paths_from_text(text: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut search_from = 0;
    let marker = "Workspace Path:";
    while let Some(rel) = text[search_from..].find(marker) {
        let abs = search_from + rel;
        let rest = text[abs + marker.len()..].trim_start();
        let end = workspace_path_end(rest);
        let candidate = rest[..end]
            .trim_end()
            .trim_end_matches(',')
            .trim()
            .to_string();
        search_from = abs + marker.len();
        if candidate.is_empty() {
            continue;
        }
        if let Ok(cwd) = fs::canonicalize(&candidate) {
            if cwd.is_dir() && !paths.iter().any(|p| p == &cwd) {
                paths.push(cwd);
            }
        }
    }
    paths
}

/// 路径终点：JSON 转义换行 `\n`/`\r`、双引号、真换行；允许路径内空白。
fn workspace_path_end(rest: &str) -> usize {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' | b'\r' | b'"' => return i,
            b'\\' if i + 1 < bytes.len() && matches!(bytes[i + 1], b'n' | b'r' | b'"') => {
                return i;
            }
            _ => i += 1,
        }
    }
    rest.len()
}

fn path_mtime(path: &Path) -> Option<DateTime<Utc>> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    Some(DateTime::<Utc>::from(modified))
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
    use super::{
        extract_workspace_paths_from_text, workspace_path_end, CursorScanner,
    };
    use crate::scanner::SessionScanner;
    use rusqlite::Connection;
    use std::fs;
    use std::time::{Duration, SystemTime};

    #[test]
    fn workspace_path_end_allows_spaces_until_json_escape() {
        let rest = "/Users/xb/My Project/code\\nIf this path";
        assert_eq!(&rest[..workspace_path_end(rest)], "/Users/xb/My Project/code");
        assert_eq!(
            &"/tmp/plain"[..workspace_path_end("/tmp/plain")],
            "/tmp/plain"
        );
    }

    #[test]
    fn extract_workspace_paths_keeps_spaces_and_prefers_existing_dirs() {
        let temp = tempfile::tempdir().unwrap();
        let spaced = temp.path().join("My Project");
        fs::create_dir_all(&spaced).unwrap();
        let text = format!(
            "Workspace Path: {}\\nIf this path continues",
            spaced.display()
        );
        let paths = extract_workspace_paths_from_text(&text);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], fs::canonicalize(&spaced).unwrap());
    }

    #[test]
    fn scanner_prefers_longest_workspace_path_over_earlier_short_fake() {
        let temp = tempfile::tempdir().unwrap();
        let short = temp.path().join("short");
        let long = temp.path().join("short").join("nested project");
        fs::create_dir_all(&long).unwrap();
        let chat_dir = temp.path().join("hash").join("space-cwd");
        fs::create_dir_all(&chat_dir).unwrap();
        fs::write(
            chat_dir.join("meta.json"),
            r#"{"title":"含空格路径","createdAtMs":1781830800000,"updatedAtMs":1781830860000}"#,
        )
        .unwrap();
        // 先插入短假路径（user 文本），再插入更长真实路径——旧逻辑会抢先用 short。
        create_cursor_store_db(
            &chat_dir.join("store.db"),
            &[
                format!("Workspace Path: {}\\nnoise", short.display()),
                format!("Workspace Path: {}\\nIf this path", long.display()),
                r#"{"role":"user","content":[{"type":"text","text":"<user_query>\n含空格 cwd\n</user_query>"}]}"#.to_string(),
            ],
        );

        let sessions = CursorScanner::with_root(temp.path().to_path_buf())
            .scan_sessions()
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(
            sessions[0].project_dir,
            fs::canonicalize(&long).unwrap()
        );
    }

    #[test]
    fn scanner_uses_meta_mtime_when_timestamps_missing() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).unwrap();
        let chat_dir = temp.path().join("hash").join("no-ts");
        fs::create_dir_all(&chat_dir).unwrap();
        let meta_path = chat_dir.join("meta.json");
        fs::write(&meta_path, r#"{"title":"无时间戳"}"#).unwrap();
        // 把 mtime 固定到过去，避免与 now() 混淆。
        let past = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let _ = filetime_set(&meta_path, past);
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
        let active = sessions[0].last_active_at;
        // 不应接近「现在」；允许 mtime 精度误差，只要不是秒级 now。
        let now = chrono::Utc::now();
        assert!(
            (now - active).num_seconds() > 60,
            "last_active_at should not fall back to now(), got {active}"
        );
    }

    /// 尽量设置 mtime；失败时测试仍可依赖「不是 now」的粗判。
    fn filetime_set(path: &std::path::Path, when: SystemTime) {
        use std::fs::File;
        let file = File::options().write(true).open(path).unwrap();
        let _ = file.set_modified(when);
    }

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
