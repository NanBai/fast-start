use crate::models::{LaunchMode, ScanResponse, TerminalType, ThemeMode};
use crate::state::{
    save_favorite_project_dirs, save_launch_mode, save_preferred_terminal, save_theme_mode,
    AppState,
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

#[tauri::command]
pub fn launch_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.launch_session(&session_id)
}

#[tauri::command]
pub fn delete_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<ScanResponse, String> {
    state.delete_session(&session_id)
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
