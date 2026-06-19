use crate::models::{CliType, Session};
use crate::scanner::{ScanError, SessionScanner};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[derive(Default)]
pub struct CursorScanner;

/// cursor 的 `~/.cursor/chats/<hash>/<uuid>/meta.json` 内容。
#[derive(Debug, Deserialize)]
struct CursorMeta {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    created_at_ms: Option<u64>,
    #[serde(default)]
    updated_at_ms: Option<u64>,
}

impl SessionScanner for CursorScanner {
    fn cli_type(&self) -> CliType {
        CliType::Cursor
    }

    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError> {
        let root = dirs::home_dir()
            .map(|home| home.join(".cursor/chats"))
            .ok_or_else(|| ScanError::NotFound("无法定位用户主目录".to_string()))?;

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
                let Some(cwd) = extract_workspace_path(&chat_dir.join("store.db")) else {
                    continue;
                };

                let meta_path = chat_dir.join("meta.json");
                let Ok(meta_content) = fs::read_to_string(&meta_path) else {
                    continue;
                };
                let Ok(meta) = serde_json::from_str::<CursorMeta>(&meta_content) else {
                    continue;
                };

                let Some(title) = meta.title.filter(|t| !t.trim().is_empty()) else {
                    // cursor 没有 codex/claude 那样的首条用户消息兜底，
                    // 无 title 的 chat 既无法 resume 出有意义的现场、也没简介可展示，跳过。
                    continue;
                };

                let last_active_at = meta
                    .updated_at_ms
                    .or(meta.created_at_ms)
                    .and_then(ms_to_datetime)
                    .unwrap_or_else(|| DateTime::<Utc>::from(SystemTime::now()));

                sessions.push(Session {
                    id: uuid::Uuid::new_v4().to_string(),
                    cli_type: CliType::Cursor,
                    session_id,
                    project_name: Session::project_name_from_dir(&cwd),
                    project_dir: cwd,
                    last_active_at,
                    // title 即 cursor 自带的会话简介（类似 claude 的 aiTitle），
                    // 已确认非空，规整后放进 summary。
                    summary: crate::scanner::clean_summary(Some(&title)),
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
fn extract_workspace_path(db_path: &Path) -> Option<PathBuf> {
    let output = Command::new("sqlite3")
        .arg(db_path)
        .arg("SELECT data FROM blobs;")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(idx) = line.find("Workspace Path:") {
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
    }
    None
}

fn ms_to_datetime(ms: u64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp_millis(ms as i64)
}

#[cfg(test)]
mod tests {
    use super::extract_workspace_path;
    use std::path::PathBuf;

    #[test]
    fn extract_workspace_path_strips_json_escape_suffix() {
        // cursor 的 Workspace Path 后面跟 JSON 转义的 \n（字面两字符），
        // 提取时必须在该处截断，否则 candidate 带上 "\nIf..." 导致 canonicalize 失败。
        let db = dirs::home_dir().unwrap().join(
            ".cursor/chats/7d7e83bb3d75edbb5cb09e3d28f03c8c/\
             378d71e3-412c-406f-a98a-ddbce1dab74a/store.db",
        );
        if !db.exists() {
            return; // 测试机没装 cursor / 该 chat 已清理时跳过
        }
        let cwd = extract_workspace_path(&db).expect("应提取到 Workspace Path");
        assert!(
            cwd.to_string_lossy().ends_with("fast-start"),
            "cwd 应是 fast-start 项目，实际: {}",
            cwd.display()
        );
    }
}
