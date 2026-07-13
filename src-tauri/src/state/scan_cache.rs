//! 扫描结果磁盘快照（`scan-cache-v1.json`）。
//!
//! 仅序列化可展示字段；`Session.delete_target` 带 `#[serde(skip)]`，
//! 缓存窗内内存 sessions 的 delete_target 恒为 None，直至 full scan。

use crate::models::{CliScanError, Session};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const SCAN_CACHE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanCacheSnapshot {
    pub version: u32,
    pub saved_at: DateTime<Utc>,
    pub sessions: Vec<Session>,
    pub scan_errors: Vec<CliScanError>,
    pub total_ms: u64,
}

/// 读取合法 snapshot；版本不匹配、损坏或缺文件时返回 None。
pub fn load_scan_cache(path: &Path) -> Option<ScanCacheSnapshot> {
    let raw = fs::read_to_string(path).ok()?;
    let snapshot: ScanCacheSnapshot = serde_json::from_str(&raw).ok()?;
    if snapshot.version != SCAN_CACHE_VERSION {
        return None;
    }
    // 防御：确保缓存路径字段不会带回任何 delete 信息（serde skip 已保证）。
    let sessions = snapshot
        .sessions
        .into_iter()
        .map(|mut session| {
            session.delete_target = None;
            session
        })
        .collect();
    Some(ScanCacheSnapshot {
        version: snapshot.version,
        saved_at: snapshot.saved_at,
        sessions,
        scan_errors: snapshot.scan_errors,
        total_ms: snapshot.total_ms,
    })
}

/// 原子写入 snapshot（temp + rename）。
pub fn save_scan_cache(path: &Path, snapshot: &ScanCacheSnapshot) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建 scan-cache 目录失败: {err}"))?;
    }

    let tmp_path = temp_path_for(path);
    {
        let mut file = fs::File::create(&tmp_path)
            .map_err(|err| format!("创建 scan-cache 临时文件失败: {err}"))?;
        let json = serde_json::to_vec_pretty(snapshot)
            .map_err(|err| format!("序列化 scan-cache 失败: {err}"))?;
        file.write_all(&json)
            .map_err(|err| format!("写入 scan-cache 失败: {err}"))?;
        file.sync_all()
            .map_err(|err| format!("同步 scan-cache 失败: {err}"))?;
    }
    fs::rename(&tmp_path, path).map_err(|err| {
        let _ = fs::remove_file(&tmp_path);
        format!("提交 scan-cache 失败: {err}")
    })?;
    Ok(())
}

/// 从当前内存 sessions 构建可落盘 snapshot（强制剥离 delete_target）。
pub fn snapshot_from_sessions(
    sessions: &[Session],
    scan_errors: &[CliScanError],
    total_ms: u64,
) -> ScanCacheSnapshot {
    let sessions: Vec<Session> = sessions
        .iter()
        .map(|session| {
            let mut cloned = session.clone();
            cloned.delete_target = None;
            cloned
        })
        .collect();
    ScanCacheSnapshot {
        version: SCAN_CACHE_VERSION,
        saved_at: Utc::now(),
        sessions,
        scan_errors: scan_errors.to_vec(),
        total_ms,
    }
}

fn temp_path_for(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("scan-cache-v1.json");
    tmp.set_file_name(format!("{file_name}.tmp"));
    tmp
}

#[cfg(test)]
mod tests {
    use super::{load_scan_cache, save_scan_cache, snapshot_from_sessions, SCAN_CACHE_VERSION};
    use crate::models::{
        CliScanError, CliType, Session, SessionDeleteKind, SessionDeleteTarget,
    };
    use chrono::Utc;
    use std::fs;
    use std::path::PathBuf;

    fn sample_session() -> Session {
        Session {
            id: "id-1".to_string(),
            cli_type: CliType::Codex,
            session_id: "sess-1".to_string(),
            project_dir: PathBuf::from("/tmp/demo"),
            project_name: "demo".to_string(),
            last_active_at: Utc::now(),
            summary: Some("hello".to_string()),
            delete_target: Some(SessionDeleteTarget {
                root: PathBuf::from("/tmp"),
                path: PathBuf::from("/tmp/demo/session.jsonl"),
                kind: SessionDeleteKind::File,
            }),
        }
    }

    #[test]
    fn roundtrip_strips_delete_target() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("scan-cache-v1.json");
        let session = sample_session();
        let snapshot = snapshot_from_sessions(
            &[session],
            &[CliScanError {
                cli_type: CliType::Cursor,
                message: "boom".to_string(),
            }],
            42,
        );
        assert!(snapshot.sessions[0].delete_target.is_none());
        assert_eq!(snapshot.version, SCAN_CACHE_VERSION);

        save_scan_cache(&path, &snapshot).unwrap();
        let loaded = load_scan_cache(&path).expect("cache should load");
        assert_eq!(loaded.sessions.len(), 1);
        assert!(loaded.sessions[0].delete_target.is_none());
        assert_eq!(loaded.sessions[0].session_id, "sess-1");
        assert_eq!(loaded.total_ms, 42);
        assert_eq!(loaded.scan_errors.len(), 1);
    }

    #[test]
    fn rejects_wrong_version() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("scan-cache-v1.json");
        fs::write(
            &path,
            r#"{"version":99,"savedAt":"2026-07-13T00:00:00Z","sessions":[],"scanErrors":[],"totalMs":1}"#,
        )
        .unwrap();
        assert!(load_scan_cache(&path).is_none());
    }

    #[test]
    fn missing_file_returns_none() {
        let temp = tempfile::tempdir().unwrap();
        assert!(load_scan_cache(&temp.path().join("missing.json")).is_none());
    }
}
