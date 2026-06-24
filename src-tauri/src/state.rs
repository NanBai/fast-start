use crate::launcher::{launcher_for, launchers, LaunchError};
use crate::models::{
    CliScanError, CliType, LaunchMode, ScanResponse, Session, TerminalType, ThemeMode,
};
use crate::scanner::{command_spec_for_session, scanners};
use crate::session_delete::delete_session_target;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const PREFERRED_TERMINAL_KEY: &str = "preferred_terminal";
const LAUNCH_MODE_KEY: &str = "launch_mode";
const THEME_MODE_KEY: &str = "theme_mode";
const FAVORITE_PROJECT_DIRS_KEY: &str = "favorite_project_dirs";

pub struct AppState {
    inner: Mutex<AppStateInner>,
}

struct AppStateInner {
    sessions: Vec<Session>,
    scan_errors: HashMap<CliType, String>,
    preferred_terminal: TerminalType,
    launch_mode: LaunchMode,
    theme_mode: ThemeMode,
    favorite_project_dirs: Vec<String>,
    scanned: bool,
}

impl AppState {
    pub fn new(
        preferred_terminal: TerminalType,
        launch_mode: LaunchMode,
        theme_mode: ThemeMode,
        favorite_project_dirs: Vec<String>,
    ) -> Self {
        Self {
            inner: Mutex::new(AppStateInner {
                sessions: Vec::new(),
                scan_errors: HashMap::new(),
                preferred_terminal,
                launch_mode,
                theme_mode,
                favorite_project_dirs: normalize_project_dirs(favorite_project_dirs),
                scanned: false,
            }),
        }
    }

    pub fn scan_all(&self) -> Result<ScanResponse, String> {
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

        Ok(ScanResponse {
            sessions,
            scan_errors,
        })
    }

    pub fn cached_scan(&self) -> Result<ScanResponse, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        if !guard.scanned {
            drop(guard);
            return self.scan_all();
        }

        Ok(ScanResponse {
            sessions: guard.sessions.clone(),
            scan_errors: guard
                .scan_errors
                .iter()
                .map(|(cli_type, message)| CliScanError {
                    cli_type: *cli_type,
                    message: message.clone(),
                })
                .collect(),
        })
    }

    pub fn find_session(&self, session_id: &str) -> Result<Session, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .cloned()
            .ok_or_else(|| "未找到对应 session".to_string())
    }

    pub fn preferred_terminal(&self) -> Result<TerminalType, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.preferred_terminal)
    }

    pub fn set_preferred_terminal(&self, terminal: TerminalType) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.preferred_terminal = terminal;
        Ok(())
    }

    pub fn launch_mode(&self) -> Result<LaunchMode, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.launch_mode)
    }

    pub fn set_launch_mode(&self, mode: LaunchMode) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.launch_mode = mode;
        Ok(())
    }

    pub fn theme_mode(&self) -> Result<ThemeMode, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.theme_mode)
    }

    pub fn set_theme_mode(&self, mode: ThemeMode) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.theme_mode = mode;
        Ok(())
    }

    pub fn favorite_project_dirs(&self) -> Result<Vec<String>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.favorite_project_dirs.clone())
    }

    pub fn sanitize_favorite_project_dirs(
        &self,
        project_dirs: Vec<String>,
    ) -> Result<Vec<String>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(normalize_project_dirs_for_sessions(
            project_dirs,
            &guard.sessions,
        ))
    }

    pub fn set_favorite_project_dirs(&self, project_dirs: Vec<String>) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.favorite_project_dirs = normalize_project_dirs(project_dirs);
        Ok(())
    }

    pub fn launch_session(&self, session_id: &str) -> Result<(), String> {
        let session = self.find_session(session_id)?;
        let preferred = self.preferred_terminal()?;
        let mode = self.launch_mode()?;
        let launcher = launcher_for(preferred).ok_or_else(|| "终端类型不受支持".to_string())?;

        if !launcher.is_available() {
            return Err("所选终端不可用".to_string());
        }

        // Terminal.app 不支持开 tab：选了 NewTab 时回退到 NewWindow 并提示。
        if mode == LaunchMode::NewTab && !launcher.supports_tab() {
            return launcher
                .launch(&command_spec_for_session(&session)?, LaunchMode::NewWindow)
                .map_err(|err: LaunchError| err.message());
        }

        let spec = command_spec_for_session(&session)?;
        launcher
            .launch(&spec, mode)
            .map_err(|err: LaunchError| err.message())
    }

    pub fn delete_session(&self, session_id: &str) -> Result<ScanResponse, String> {
        let session = self.find_session(session_id)?;
        delete_session_target(session.delete_target.as_ref()).map_err(|err| err.message())?;

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.sessions.retain(|item| item.id != session_id);

        Ok(ScanResponse {
            sessions: guard.sessions.clone(),
            scan_errors: guard
                .scan_errors
                .iter()
                .map(|(cli_type, message)| CliScanError {
                    cli_type: *cli_type,
                    message: message.clone(),
                })
                .collect(),
        })
    }

    pub fn list_available_terminals(&self) -> Vec<TerminalType> {
        launchers()
            .iter()
            .filter(|launcher| launcher.is_available())
            .map(|launcher| launcher.terminal_type())
            .collect()
    }
}

pub fn load_preferred_terminal(app: &AppHandle) -> Result<TerminalType, String> {
    let store = app
        .store("preferences.json")
        .map_err(|err| err.to_string())?;
    let value = store.get(PREFERRED_TERMINAL_KEY);
    if let Some(raw) = value {
        serde_json::from_value(raw).map_err(|err| err.to_string())
    } else {
        Ok(TerminalType::System)
    }
}

pub fn save_preferred_terminal(app: &AppHandle, terminal: TerminalType) -> Result<(), String> {
    let store = app
        .store("preferences.json")
        .map_err(|err| err.to_string())?;
    store.set(PREFERRED_TERMINAL_KEY, json!(terminal));
    store.save().map_err(|err| err.to_string())
}

pub fn load_launch_mode(app: &AppHandle) -> Result<LaunchMode, String> {
    let store = app
        .store("preferences.json")
        .map_err(|err| err.to_string())?;
    let value = store.get(LAUNCH_MODE_KEY);
    if let Some(raw) = value {
        serde_json::from_value(raw).map_err(|err| err.to_string())
    } else {
        Ok(LaunchMode::NewTab)
    }
}

pub fn save_launch_mode(app: &AppHandle, mode: LaunchMode) -> Result<(), String> {
    let store = app
        .store("preferences.json")
        .map_err(|err| err.to_string())?;
    store.set(LAUNCH_MODE_KEY, json!(mode));
    store.save().map_err(|err| err.to_string())
}

pub fn load_theme_mode(app: &AppHandle) -> Result<ThemeMode, String> {
    let store = app
        .store("preferences.json")
        .map_err(|err| err.to_string())?;
    let value = store.get(THEME_MODE_KEY);
    if let Some(raw) = value {
        serde_json::from_value(raw).map_err(|err| err.to_string())
    } else {
        Ok(ThemeMode::System)
    }
}

pub fn save_theme_mode(app: &AppHandle, mode: ThemeMode) -> Result<(), String> {
    let store = app
        .store("preferences.json")
        .map_err(|err| err.to_string())?;
    store.set(THEME_MODE_KEY, json!(mode));
    store.save().map_err(|err| err.to_string())
}

pub fn load_favorite_project_dirs(app: &AppHandle) -> Result<Vec<String>, String> {
    let store = app
        .store("preferences.json")
        .map_err(|err| err.to_string())?;
    let value = store.get(FAVORITE_PROJECT_DIRS_KEY);
    if let Some(raw) = value {
        serde_json::from_value(raw)
            .map(normalize_project_dirs)
            .map_err(|err| err.to_string())
    } else {
        Ok(Vec::new())
    }
}

pub fn save_favorite_project_dirs(
    app: &AppHandle,
    project_dirs: Vec<String>,
) -> Result<(), String> {
    let store = app
        .store("preferences.json")
        .map_err(|err| err.to_string())?;
    store.set(
        FAVORITE_PROJECT_DIRS_KEY,
        json!(normalize_project_dirs(project_dirs)),
    );
    store.save().map_err(|err| err.to_string())
}

fn normalize_project_dirs(project_dirs: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for project_dir in project_dirs {
        if project_dir.is_empty() || normalized.contains(&project_dir) {
            continue;
        }
        normalized.push(project_dir);
    }
    normalized
}

fn normalize_project_dirs_for_sessions(
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

#[cfg(test)]
mod tests {
    use super::{AppState, AppStateInner};
    use crate::models::{
        CliType, LaunchMode, Session, SessionDeleteKind, SessionDeleteTarget, TerminalType,
        ThemeMode,
    };
    use crate::scanner::{codex::CodexScanner, SessionScanner};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;

    fn state_with_sessions(sessions: Vec<Session>) -> AppState {
        AppState {
            inner: Mutex::new(AppStateInner {
                sessions,
                scan_errors: HashMap::new(),
                preferred_terminal: TerminalType::System,
                launch_mode: LaunchMode::NewTab,
                theme_mode: ThemeMode::System,
                favorite_project_dirs: Vec::new(),
                scanned: true,
            }),
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
}
