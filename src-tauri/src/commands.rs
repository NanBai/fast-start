use crate::models::{ScanResponse, TerminalType};
use crate::state::{save_preferred_terminal, AppState};
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
    state.set_preferred_terminal(terminal)?;
    save_preferred_terminal(&app, terminal)
}
