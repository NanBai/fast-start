use crate::grok_provider::{
    GrokActivateOfficialResult, GrokBackupInfo, GrokPrivacyResult, GrokProfile,
    GrokProviderLayout, GrokProviderState, GrokProviderStatus,
};
use crate::preferences::{load_grok_provider_layout, save_grok_provider_layout};
use crate::models::{LaunchMode, PortScanResponse, ScanResponse, TerminalType, ThemeMode};
use crate::state::{
    save_favorite_project_dirs, save_launch_mode, save_port_auto_refresh, save_preferred_terminal,
    save_theme_mode, AppState,
};
use tauri::State;

#[tauri::command]
pub fn scan_sessions(state: State<'_, AppState>) -> Result<ScanResponse, String> {
    state.cached_scan()
}

#[tauri::command]
pub fn refresh_sessions(state: State<'_, AppState>) -> Result<ScanResponse, String> {
    state.scan_all()
}

/// `session_list_id` 是列表稳定 `Session.id`，**不是** CLI 原始 `session_id`。
#[tauri::command]
pub fn launch_session(
    session_list_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.launch_session(&session_list_id)
}

/// `session_list_id` 是列表稳定 `Session.id`，**不是** CLI 原始 `session_id`。
#[tauri::command]
pub fn delete_session(
    session_list_id: String,
    state: State<'_, AppState>,
) -> Result<ScanResponse, String> {
    state.delete_session(&session_list_id)
}

#[tauri::command]
pub fn scan_ports(state: State<'_, AppState>) -> Result<PortScanResponse, String> {
    state.scan_ports()
}

#[tauri::command]
pub fn refresh_ports(state: State<'_, AppState>) -> Result<PortScanResponse, String> {
    state.refresh_ports()
}

#[tauri::command]
pub fn terminate_port_processes(
    port_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<PortScanResponse, String> {
    state.terminate_port_processes(port_ids)
}

#[tauri::command]
pub fn list_available_terminals(state: State<'_, AppState>) -> Result<Vec<TerminalType>, String> {
    Ok(state.list_available_terminals())
}

#[tauri::command]
pub fn get_preferred_terminal(state: State<'_, AppState>) -> Result<TerminalType, String> {
    state.preferred_terminal()
}

#[tauri::command]
pub fn set_preferred_terminal(
    terminal: TerminalType,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    save_preferred_terminal(&app, terminal)?;
    state.set_preferred_terminal(terminal)
}

#[tauri::command]
pub fn get_launch_mode(state: State<'_, AppState>) -> Result<LaunchMode, String> {
    state.launch_mode()
}

#[tauri::command]
pub fn set_launch_mode(
    mode: LaunchMode,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    save_launch_mode(&app, mode)?;
    state.set_launch_mode(mode)
}

#[tauri::command]
pub fn get_theme_mode(state: State<'_, AppState>) -> Result<ThemeMode, String> {
    state.theme_mode()
}

#[tauri::command]
pub fn set_theme_mode(
    mode: ThemeMode,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    save_theme_mode(&app, mode)?;
    state.set_theme_mode(mode)
}

#[tauri::command]
pub fn get_favorite_project_dirs(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.favorite_project_dirs()
}

#[tauri::command]
pub fn set_favorite_project_dirs(
    project_dirs: Vec<String>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project_dirs = state.sanitize_favorite_project_dirs(project_dirs)?;
    save_favorite_project_dirs(&app, project_dirs.clone())?;
    state.set_favorite_project_dirs(project_dirs)
}

#[tauri::command]
pub fn get_port_auto_refresh(state: State<'_, AppState>) -> Result<bool, String> {
    state.port_auto_refresh()
}

#[tauri::command]
pub fn set_port_auto_refresh(
    enabled: bool,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    save_port_auto_refresh(&app, enabled)?;
    state.set_port_auto_refresh(enabled)
}

// --- Grok provider (suppliers) ---

#[tauri::command]
pub fn grok_provider_status(
    state: State<'_, GrokProviderState>,
) -> Result<GrokProviderStatus, String> {
    state.status()
}

#[tauri::command]
pub fn grok_list_profiles(state: State<'_, GrokProviderState>) -> Result<Vec<GrokProfile>, String> {
    state.list_profiles()
}

#[tauri::command]
pub fn grok_create_profile(
    profile: GrokProfile,
    state: State<'_, GrokProviderState>,
) -> Result<GrokProfile, String> {
    state.create_profile(profile)
}

#[tauri::command]
pub fn grok_update_profile(
    id: String,
    profile: GrokProfile,
    state: State<'_, GrokProviderState>,
) -> Result<GrokProfile, String> {
    state.update_profile(&id, profile)
}

#[tauri::command]
pub fn grok_delete_profile(id: String, state: State<'_, GrokProviderState>) -> Result<(), String> {
    state.delete_profile(&id)
}

#[tauri::command]
pub fn grok_activate_profile(
    id: String,
    state: State<'_, GrokProviderState>,
) -> Result<GrokProfile, String> {
    state.activate_profile(&id)
}

#[tauri::command]
pub fn grok_import_current(
    name: Option<String>,
    active: Option<bool>,
    state: State<'_, GrokProviderState>,
) -> Result<GrokProfile, String> {
    state.import_current(name.unwrap_or_else(|| "Default".into()), active.unwrap_or(true))
}

#[tauri::command]
pub fn grok_list_backups(
    state: State<'_, GrokProviderState>,
) -> Result<Vec<GrokBackupInfo>, String> {
    state.list_backups()
}

#[tauri::command]
pub fn grok_restore_backup(
    file: String,
    state: State<'_, GrokProviderState>,
) -> Result<(), String> {
    state.restore_backup(&file)
}

#[tauri::command]
pub fn grok_activate_official(
    state: State<'_, GrokProviderState>,
) -> Result<GrokActivateOfficialResult, String> {
    state.activate_official()
}

#[tauri::command]
pub fn grok_apply_privacy_protection(
    state: State<'_, GrokProviderState>,
) -> Result<GrokPrivacyResult, String> {
    state.apply_privacy_protection()
}

#[tauri::command]
pub fn get_grok_provider_layout(app: tauri::AppHandle) -> Result<GrokProviderLayout, String> {
    load_grok_provider_layout(&app)
}

#[tauri::command]
pub fn set_grok_provider_layout(
    layout: GrokProviderLayout,
    app: tauri::AppHandle,
) -> Result<GrokProviderLayout, String> {
    save_grok_provider_layout(&app, layout)
}
