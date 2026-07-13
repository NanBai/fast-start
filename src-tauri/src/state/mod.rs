mod ports;
mod scan_cache;
mod session_ops;

use crate::models::{
    CliScanError, CliType, LaunchMode, PortScanResponse, RecentLaunch, ScanResponse, Session,
    TerminalType, ThemeMode,
};
use crate::scanner::scanners;
use scan_cache::{load_scan_cache, save_scan_cache, snapshot_from_sessions};

pub use crate::preferences::{
    load_favorite_project_dirs, load_favorite_session_ids, load_launch_mode, load_port_auto_refresh,
    load_port_ignore_ports, load_port_project_path_prefixes, load_port_protect_ports,
    load_preferred_terminal, load_recent_launches, load_theme_mode, save_favorite_project_dirs,
    save_favorite_session_ids, save_launch_mode, save_port_auto_refresh, save_port_ignore_ports,
    save_port_project_path_prefixes, save_port_protect_ports, save_preferred_terminal,
    save_recent_launches, save_theme_mode,
};

pub const RECENT_LAUNCHES_LIMIT: usize = 20;
pub const BULK_DELETE_LIMIT: usize = 50;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

pub struct AppState {
    pub(crate) inner: Mutex<AppStateInner>,
    /// 全量扫描 single-flight：并发 refresh 串行，后到者可复用刚完成的结果。
    scan_lock: Mutex<()>,
    scan_generation: AtomicU64,
}

pub(crate) struct AppStateInner {
    sessions: Vec<Session>,
    scan_errors: HashMap<CliType, String>,
    port_scan: Option<PortScanResponse>,
    preferred_terminal: TerminalType,
    launch_mode: LaunchMode,
    theme_mode: ThemeMode,
    favorite_project_dirs: Vec<String>,
    favorite_session_ids: Vec<String>,
    port_auto_refresh: bool,
    port_ignore_ports: Vec<u16>,
    port_protect_ports: Vec<u16>,
    port_project_path_prefixes: Vec<String>,
    recent_launches: Vec<RecentLaunch>,
    /// 是否已有可展示的 sessions（磁盘缓存或 full scan）。
    scanned: bool,
    /// full scan 完成：sessions 含 delete_target，可安全删除文件型 CLI。
    ops_ready: bool,
    scan_cache_path: Option<PathBuf>,
    last_scan_duration_ms: Option<u64>,
}

impl AppState {
    pub fn new(
        preferred_terminal: TerminalType,
        launch_mode: LaunchMode,
        theme_mode: ThemeMode,
        favorite_project_dirs: Vec<String>,
        favorite_session_ids: Vec<String>,
        port_auto_refresh: bool,
    ) -> Self {
        Self {
            inner: Mutex::new(AppStateInner {
                sessions: Vec::new(),
                scan_errors: HashMap::new(),
                port_scan: None,
                preferred_terminal,
                launch_mode,
                theme_mode,
                favorite_project_dirs: normalize_project_dirs(favorite_project_dirs),
                favorite_session_ids: normalize_id_list(favorite_session_ids),
                port_auto_refresh,
                port_ignore_ports: Vec::new(),
                port_protect_ports: Vec::new(),
                port_project_path_prefixes: Vec::new(),
                recent_launches: Vec::new(),
                scanned: false,
                ops_ready: false,
                scan_cache_path: None,
                last_scan_duration_ms: None,
            }),
            scan_lock: Mutex::new(()),
            scan_generation: AtomicU64::new(0),
        }
    }

    /// 由 Tauri setup 注入 app_data/scan-cache-v1.json。
    pub fn set_scan_cache_path(&self, path: PathBuf) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.scan_cache_path = Some(path);
        Ok(())
    }

    pub fn scan_all(&self) -> Result<ScanResponse, String> {
        let gen_before = self.scan_generation.load(Ordering::SeqCst);
        let _flight = self
            .scan_lock
            .lock()
            .map_err(|_| "无法获取扫描锁".to_string())?;

        // 等待期间若已有完整扫描完成，直接返回同一次结果（single-flight）。
        let gen_after = self.scan_generation.load(Ordering::SeqCst);
        if gen_after > gen_before {
            let guard = self
                .inner
                .lock()
                .map_err(|_| "无法获取应用状态".to_string())?;
            if guard.ops_ready {
                return Ok(self.response_from_guard(&guard, false));
            }
        }

        let started = Instant::now();
        let scanners = scanners();
        let mut handles = Vec::with_capacity(scanners.len());

        for scanner in scanners {
            handles.push(std::thread::spawn(move || {
                let cli_type = scanner.cli_type();
                let result = scanner.scan_sessions();
                (cli_type, result)
            }));
        }

        let mut sessions = Vec::new();
        let mut scan_errors = Vec::new();

        for handle in handles {
            let (cli_type, result) = handle.join().map_err(|_| "扫描线程异常退出".to_string())?;
            match result {
                Ok(mut found) => sessions.append(&mut found),
                Err(err) => {
                    scan_errors.push(CliScanError {
                        cli_type,
                        message: err.message(),
                    });
                }
            }
        }

        sessions.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));
        let total_ms = started.elapsed().as_millis() as u64;

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.sessions = sessions.clone();
        guard.scan_errors = scan_errors
            .iter()
            .map(|err| (err.cli_type, err.message.clone()))
            .collect();
        guard.scanned = true;
        guard.ops_ready = true;
        guard.last_scan_duration_ms = Some(total_ms);

        if let Some(path) = guard.scan_cache_path.clone() {
            let snapshot = snapshot_from_sessions(&sessions, &scan_errors, total_ms);
            // 写盘失败不阻断扫描结果返回，仅日志化到 stderr。
            if let Err(err) = save_scan_cache(&path, &snapshot) {
                eprintln!("scan-cache write failed: {err}");
            }
        }

        self.scan_generation.fetch_add(1, Ordering::SeqCst);

        Ok(ScanResponse {
            sessions,
            scan_errors,
            from_cache: Some(false),
            scan_duration_ms: Some(total_ms),
        })
    }

    pub fn cached_scan(&self) -> Result<ScanResponse, String> {
        {
            let guard = self
                .inner
                .lock()
                .map_err(|_| "无法获取应用状态".to_string())?;
            if guard.scanned {
                return Ok(self.response_from_guard(&guard, !guard.ops_ready));
            }

            // 冷启动：优先读磁盘 snapshot，秒开列表（delete_target 全无）。
            if let Some(path) = guard.scan_cache_path.clone() {
                if let Some(snapshot) = load_scan_cache(&path) {
                    drop(guard);
                    return self.apply_disk_cache(snapshot);
                }
            }
        }

        self.scan_all()
    }

    fn apply_disk_cache(
        &self,
        snapshot: scan_cache::ScanCacheSnapshot,
    ) -> Result<ScanResponse, String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        // 二次检查：并发 scan 可能已完成。
        if guard.scanned {
            return Ok(self.response_from_guard(&guard, !guard.ops_ready));
        }

        guard.sessions = snapshot.sessions.clone();
        guard.scan_errors = snapshot
            .scan_errors
            .iter()
            .map(|err| (err.cli_type, err.message.clone()))
            .collect();
        guard.scanned = true;
        guard.ops_ready = false;
        guard.last_scan_duration_ms = Some(snapshot.total_ms);

        Ok(ScanResponse {
            sessions: snapshot.sessions,
            scan_errors: snapshot.scan_errors,
            from_cache: Some(true),
            scan_duration_ms: Some(snapshot.total_ms),
        })
    }

    fn response_from_guard(&self, guard: &AppStateInner, from_cache: bool) -> ScanResponse {
        ScanResponse {
            sessions: guard.sessions.clone(),
            scan_errors: guard
                .scan_errors
                .iter()
                .map(|(cli_type, message)| CliScanError {
                    cli_type: *cli_type,
                    message: message.clone(),
                })
                .collect(),
            from_cache: Some(from_cache),
            scan_duration_ms: guard.last_scan_duration_ms,
        }
    }
}

pub(crate) fn normalize_project_dirs(project_dirs: Vec<String>) -> Vec<String> {
    normalize_id_list(project_dirs)
}

pub(crate) fn normalize_id_list(ids: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for id in ids {
        if id.is_empty() || normalized.contains(&id) {
            continue;
        }
        normalized.push(id);
    }
    normalized
}

pub(crate) fn normalize_project_dirs_for_sessions(
    project_dirs: Vec<String>,
    sessions: &[Session],
) -> Vec<String> {
    let allowed_project_dirs: Vec<String> = sessions
        .iter()
        .map(|session| session.project_dir.to_string_lossy().to_string())
        .collect();
    normalize_project_dirs(project_dirs)
        .into_iter()
        .filter(|project_dir| allowed_project_dirs.contains(project_dir))
        .collect()
}

pub(crate) fn normalize_session_ids_for_sessions(session_ids: Vec<String>, sessions: &[Session]) -> Vec<String> {
    let allowed: Vec<String> = sessions.iter().map(|session| session.id.clone()).collect();
    normalize_id_list(session_ids)
        .into_iter()
        .filter(|id| allowed.contains(id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{scan_cache, AppState, AppStateInner};
    use crate::models::{
        CliType, LaunchMode, Session, SessionDeleteKind, SessionDeleteTarget, TerminalType,
        ThemeMode,
    };
    use crate::scanner::{codex::CodexScanner, SessionScanner};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicU64;
    use std::sync::Mutex;

    fn state_with_sessions(sessions: Vec<Session>) -> AppState {
        AppState {
            inner: Mutex::new(AppStateInner {
                sessions,
                scan_errors: HashMap::new(),
                port_scan: None,
                preferred_terminal: TerminalType::System,
                launch_mode: LaunchMode::NewTab,
                theme_mode: ThemeMode::System,
                favorite_project_dirs: Vec::new(),
                favorite_session_ids: Vec::new(),
                port_auto_refresh: true,
                port_ignore_ports: Vec::new(),
                port_protect_ports: Vec::new(),
                port_project_path_prefixes: Vec::new(),
                recent_launches: Vec::new(),
                scanned: true,
                ops_ready: true,
                scan_cache_path: None,
                last_scan_duration_ms: Some(10),
            }),
            scan_lock: Mutex::new(()),
            scan_generation: AtomicU64::new(1),
        }
    }

    fn state_from_cache_window(sessions: Vec<Session>, cache_path: PathBuf) -> AppState {
        AppState {
            inner: Mutex::new(AppStateInner {
                sessions,
                scan_errors: HashMap::new(),
                port_scan: None,
                preferred_terminal: TerminalType::System,
                launch_mode: LaunchMode::NewTab,
                theme_mode: ThemeMode::System,
                favorite_project_dirs: Vec::new(),
                favorite_session_ids: Vec::new(),
                port_auto_refresh: true,
                port_ignore_ports: Vec::new(),
                port_protect_ports: Vec::new(),
                port_project_path_prefixes: Vec::new(),
                recent_launches: Vec::new(),
                scanned: true,
                ops_ready: false,
                scan_cache_path: Some(cache_path),
                last_scan_duration_ms: Some(99),
            }),
            scan_lock: Mutex::new(()),
            scan_generation: AtomicU64::new(0),
        }
    }

    fn test_session(id: &str, target: Option<SessionDeleteTarget>) -> Session {
        test_session_at_project(id, PathBuf::from("/tmp"), target)
    }

    fn test_session_at_project(
        id: &str,
        project_dir: PathBuf,
        target: Option<SessionDeleteTarget>,
    ) -> Session {
        let project_name = project_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("tmp")
            .to_string();
        Session {
            id: id.to_string(),
            cli_type: CliType::Codex,
            session_id: "abc-123".to_string(),
            project_dir,
            project_name,
            last_active_at: Utc::now(),
            summary: None,
            delete_target: target,
        }
    }

    #[test]
    fn favorite_project_dirs_are_normalized_in_state() {
        let state = AppState::new(
            TerminalType::System,
            LaunchMode::NewTab,
            ThemeMode::System,
            vec!["/tmp/a".to_string(), "/tmp/a".to_string(), String::new()],
            Vec::new(),
            true,
        );

        assert_eq!(state.favorite_project_dirs().unwrap(), vec!["/tmp/a"]);

        state
            .set_favorite_project_dirs(vec![
                "/tmp/b".to_string(),
                "/tmp/b".to_string(),
                "/tmp/c".to_string(),
            ])
            .unwrap();

        assert_eq!(
            state.favorite_project_dirs().unwrap(),
            vec!["/tmp/b", "/tmp/c"]
        );
    }

    #[test]
    fn favorite_project_dirs_are_limited_to_scanned_sessions_before_save() {
        let state = state_with_sessions(vec![
            test_session_at_project("a", PathBuf::from("/tmp/a"), None),
            test_session_at_project("b", PathBuf::from("/tmp/b"), None),
        ]);

        let sanitized = state
            .sanitize_favorite_project_dirs(vec![
                "/tmp/b".to_string(),
                "/tmp/missing".to_string(),
                "/tmp/b".to_string(),
                "/tmp/a".to_string(),
                String::new(),
            ])
            .unwrap();

        assert_eq!(sanitized, vec!["/tmp/b", "/tmp/a"]);
    }

    #[test]
    fn delete_session_removes_file_and_cached_session() {
        let temp = tempfile::tempdir().unwrap();
        let session_file = temp.path().join("session.jsonl");
        fs::write(&session_file, "{}").unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: session_file.clone(),
            kind: SessionDeleteKind::File,
        };
        let state = state_with_sessions(vec![test_session("remove-me", Some(target))]);

        let response = state.delete_session("remove-me").unwrap();

        assert!(!session_file.exists());
        assert!(response.sessions.is_empty());
    }

    #[test]
    fn deleted_scanned_session_disappears_after_rescan() {
        let temp = tempfile::tempdir().unwrap();
        let session_file = temp.path().join("session.jsonl");
        fs::write(
            &session_file,
            [
                r#"{"timestamp":"2026-06-20T01:00:00Z","type":"session_meta","payload":{"id":"codex-delete-smoke","cwd":"/tmp"}}"#,
                r#"{"timestamp":"2026-06-20T01:01:00Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"删除 smoke"}]}}"#,
            ]
            .join("\n"),
        )
        .unwrap();
        let scanner = CodexScanner::with_root(temp.path().to_path_buf());
        let sessions = scanner.scan_sessions().unwrap();
        let session_id = sessions[0].id.clone();
        let state = state_with_sessions(sessions);

        let response = state.delete_session(&session_id).unwrap();
        let refreshed = scanner.scan_sessions().unwrap();

        assert!(response.sessions.is_empty());
        assert!(refreshed.is_empty());
        assert!(!session_file.exists());
    }

    #[test]
    fn cached_scan_returns_from_disk_without_delete_target() {
        let temp = tempfile::tempdir().unwrap();
        let cache_path = temp.path().join("scan-cache-v1.json");
        let mut session = test_session(
            "cached-1",
            Some(SessionDeleteTarget {
                root: PathBuf::from("/tmp"),
                path: PathBuf::from("/tmp/x.jsonl"),
                kind: SessionDeleteKind::File,
            }),
        );
        let snapshot = scan_cache::snapshot_from_sessions(std::slice::from_ref(&session), &[], 77);
        scan_cache::save_scan_cache(&cache_path, &snapshot).unwrap();
        // 确认落盘后无 delete_target
        session.delete_target = None;

        let state = AppState::new(
            TerminalType::System,
            LaunchMode::NewTab,
            ThemeMode::System,
            Vec::new(),
            Vec::new(),
            true,
        );
        state.set_scan_cache_path(cache_path).unwrap();

        let response = state.cached_scan().unwrap();
        assert_eq!(response.from_cache, Some(true));
        assert_eq!(response.scan_duration_ms, Some(77));
        assert_eq!(response.sessions.len(), 1);
        assert!(response.sessions[0].delete_target.is_none());
        assert_eq!(response.sessions[0].id, "cached-1");

        // 再次 cached_scan 走内存，仍标记为缓存展示态（ops 未就绪）
        let again = state.cached_scan().unwrap();
        assert_eq!(again.from_cache, Some(true));
    }

    #[test]
    fn cache_window_delete_fails_for_file_cli() {
        let temp = tempfile::tempdir().unwrap();
        let cache_path = temp.path().join("scan-cache-v1.json");
        let session = test_session("need-refresh", None);
        let state = state_from_cache_window(vec![session], cache_path);

        let err = state.delete_session("need-refresh").unwrap_err();
        assert!(
            err.contains("刷新"),
            "expected refresh guidance in error, got: {err}"
        );
    }

    #[test]
    fn favorite_session_ids_are_sanitized_to_scanned_sessions() {
        let state = state_with_sessions(vec![
            test_session_at_project("keep-me", PathBuf::from("/tmp/a"), None),
            test_session_at_project("also", PathBuf::from("/tmp/b"), None),
        ]);
        let sanitized = state
            .sanitize_favorite_session_ids(vec![
                "also".to_string(),
                "gone".to_string(),
                "also".to_string(),
                "keep-me".to_string(),
                String::new(),
            ])
            .unwrap();
        assert_eq!(sanitized, vec!["also", "keep-me"]);
    }

    #[test]
    fn sanitize_recent_launches_drops_missing_and_reports_changed() {
        use crate::models::RecentLaunch;
        let state = state_with_sessions(vec![test_session_at_project(
            "alive",
            PathBuf::from("/tmp/a"),
            None,
        )]);
        state
            .set_recent_launches(vec![
                RecentLaunch {
                    session_list_id: "alive".into(),
                    cli_type: CliType::Codex,
                    project_name: "a".into(),
                    project_dir: "/tmp/a".into(),
                    summary: None,
                    launched_at: Utc::now(),
                },
                RecentLaunch {
                    session_list_id: "gone".into(),
                    cli_type: CliType::Codex,
                    project_name: "b".into(),
                    project_dir: "/tmp/b".into(),
                    summary: None,
                    launched_at: Utc::now(),
                },
            ])
            .unwrap();
        let (launches, changed) = state.sanitize_recent_launches().unwrap();
        assert!(changed);
        assert_eq!(launches.len(), 1);
        assert_eq!(launches[0].session_list_id, "alive");
        let (_, changed_again) = state.sanitize_recent_launches().unwrap();
        assert!(!changed_again);
    }

    #[test]
    fn full_scan_write_cache_and_delete_succeeds() {
        let temp = tempfile::tempdir().unwrap();
        let cache_path = temp.path().join("scan-cache-v1.json");
        let session_file = temp.path().join("session.jsonl");
        fs::write(&session_file, "{}").unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: session_file.clone(),
            kind: SessionDeleteKind::File,
        };
        let state = state_with_sessions(vec![test_session("full-ops", Some(target))]);
        state.set_scan_cache_path(cache_path.clone()).unwrap();

        let response = state.delete_session("full-ops").unwrap();
        assert!(response.sessions.is_empty());
        assert!(!session_file.exists());
        assert!(cache_path.exists());
        let loaded = scan_cache::load_scan_cache(&cache_path).unwrap();
        assert!(loaded.sessions.is_empty());
    }

    #[test]
    fn bulk_delete_rejects_empty_and_over_limit() {
        let state = state_with_sessions(vec![]);
        assert!(state.delete_sessions(&[]).is_err());
        let too_many: Vec<String> = (0..51).map(|i| format!("id-{i}")).collect();
        let err = state.delete_sessions(&too_many).unwrap_err();
        assert!(err.contains("50"));
    }

    #[test]
    fn bulk_delete_dedupes_ids_before_limit_and_work() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("a.jsonl");
        fs::write(&file, "{}").unwrap();
        let state = state_with_sessions(vec![test_session(
            "only-once",
            Some(SessionDeleteTarget {
                root: temp.path().to_path_buf(),
                path: file.clone(),
                kind: SessionDeleteKind::File,
            }),
        )]);
        let result = state
            .delete_sessions(&["only-once".into(), "only-once".into()])
            .unwrap();
        assert_eq!(result.deleted_ids, vec!["only-once"]);
        assert!(result.failures.is_empty());
        assert!(!file.exists());
    }

    #[test]
    fn bulk_delete_partial_success_via_full_path() {
        let temp = tempfile::tempdir().unwrap();
        let file_a = temp.path().join("a.jsonl");
        let file_b = temp.path().join("b.jsonl");
        fs::write(&file_a, "{}").unwrap();
        fs::write(&file_b, "{}").unwrap();
        let state = state_with_sessions(vec![
            test_session(
                "ok-a",
                Some(SessionDeleteTarget {
                    root: temp.path().to_path_buf(),
                    path: file_a.clone(),
                    kind: SessionDeleteKind::File,
                }),
            ),
            test_session(
                "ok-b",
                Some(SessionDeleteTarget {
                    root: temp.path().to_path_buf(),
                    path: file_b.clone(),
                    kind: SessionDeleteKind::File,
                }),
            ),
        ]);

        let result = state
            .delete_sessions(&["ok-a".into(), "missing".into(), "ok-b".into()])
            .unwrap();
        assert_eq!(result.deleted_ids, vec!["ok-a", "ok-b"]);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].session_list_id, "missing");
        assert!(result.sessions.is_empty());
        assert!(!file_a.exists());
        assert!(!file_b.exists());
        // 响应不得含路径字段名以外的 delete 源路径（Session 无 delete_target 序列化）
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("delete_target"));
        assert!(!json.contains(file_a.to_string_lossy().as_ref()));
    }
}
