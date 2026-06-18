use crate::launcher::{launcher_for, launchers, LaunchError};
use crate::models::{CliScanError, CliType, LaunchMode, ScanResponse, Session, TerminalType};
use crate::scanner::{command_spec_for_session, scanners};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const PREFERRED_TERMINAL_KEY: &str = "preferred_terminal";
const LAUNCH_MODE_KEY: &str = "launch_mode";

pub struct AppState {
    inner: Mutex<AppStateInner>,
}

struct AppStateInner {
    sessions: Vec<Session>,
    scan_errors: HashMap<CliType, String>,
    preferred_terminal: TerminalType,
    launch_mode: LaunchMode,
    scanned: bool,
}

impl AppState {
    pub fn new(preferred_terminal: TerminalType, launch_mode: LaunchMode) -> Self {
        Self {
            inner: Mutex::new(AppStateInner {
                sessions: Vec::new(),
                scan_errors: HashMap::new(),
                preferred_terminal,
                launch_mode,
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
