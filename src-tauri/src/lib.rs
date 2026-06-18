mod commands;
mod launcher;
mod models;
mod scanner;
mod security;
mod state;

use commands::{
    get_launch_mode, get_preferred_terminal, launch_session, list_available_terminals,
    refresh_sessions, scan_sessions, set_launch_mode, set_preferred_terminal,
};
use state::{load_launch_mode, load_preferred_terminal, save_preferred_terminal, AppState};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let mut preferred =
                load_preferred_terminal(app.handle()).unwrap_or(models::TerminalType::System);
            let launch_mode = load_launch_mode(app.handle()).unwrap_or(models::LaunchMode::NewTab);
            let state = AppState::new(preferred, launch_mode);
            let available = state.list_available_terminals();
            if !available.contains(&preferred) {
                preferred = available
                    .first()
                    .copied()
                    .unwrap_or(models::TerminalType::System);
                state.set_preferred_terminal(preferred)?;
                save_preferred_terminal(app.handle(), preferred)?;
            }
            state.scan_all()?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_sessions,
            refresh_sessions,
            launch_session,
            list_available_terminals,
            get_preferred_terminal,
            set_preferred_terminal,
            get_launch_mode,
            set_launch_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
