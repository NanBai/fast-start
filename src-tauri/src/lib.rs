mod commands;
mod launcher;
mod models;
mod port_monitor;
mod preferences;
mod scanner;
mod security;
mod session_delete;
mod state;

use commands::{
    delete_session, get_favorite_project_dirs, get_launch_mode, get_port_auto_refresh,
    get_preferred_terminal, get_theme_mode, launch_session, list_available_terminals,
    refresh_ports, refresh_sessions, scan_ports, scan_sessions, set_favorite_project_dirs,
    set_launch_mode, set_port_auto_refresh, set_preferred_terminal, set_theme_mode,
    terminate_port_processes,
};
use state::{
    load_favorite_project_dirs, load_launch_mode, load_port_auto_refresh, load_preferred_terminal,
    load_theme_mode, save_preferred_terminal, AppState,
};
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
            let theme_mode = load_theme_mode(app.handle()).unwrap_or(models::ThemeMode::System);
            let favorite_project_dirs =
                load_favorite_project_dirs(app.handle()).unwrap_or_default();
            let port_auto_refresh = load_port_auto_refresh(app.handle()).unwrap_or(true);
            let state = AppState::new(
                preferred,
                launch_mode,
                theme_mode,
                favorite_project_dirs,
                port_auto_refresh,
            );
            let available = state.list_available_terminals();
            if !available.contains(&preferred) {
                preferred = available
                    .first()
                    .copied()
                    .unwrap_or(models::TerminalType::System);
                save_preferred_terminal(app.handle(), preferred)?;
                state.set_preferred_terminal(preferred)?;
            }
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_sessions,
            refresh_sessions,
            launch_session,
            delete_session,
            scan_ports,
            refresh_ports,
            terminate_port_processes,
            list_available_terminals,
            get_preferred_terminal,
            set_preferred_terminal,
            get_launch_mode,
            set_launch_mode,
            get_theme_mode,
            set_theme_mode,
            get_favorite_project_dirs,
            set_favorite_project_dirs,
            get_port_auto_refresh,
            set_port_auto_refresh,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
