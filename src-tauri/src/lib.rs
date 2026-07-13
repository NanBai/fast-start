mod commands;
mod grok_provider;
mod launcher;
mod models;
mod port_monitor;
mod preferences;
mod scanner;
mod security;
mod session_delete;
mod state;

use commands::{
    delete_session, get_favorite_project_dirs, get_grok_provider_layout, get_launch_mode,
    get_port_auto_refresh, get_preferred_terminal, get_theme_mode, grok_activate_official,
    grok_activate_profile, grok_apply_privacy_protection, grok_create_profile, grok_delete_profile,
    grok_import_current, grok_list_backups, grok_list_profiles, grok_provider_status,
    grok_restore_backup, grok_update_profile, launch_session, list_available_terminals,
    refresh_ports, refresh_sessions, scan_ports, scan_sessions, set_favorite_project_dirs,
    set_grok_provider_layout, set_launch_mode, set_port_auto_refresh, set_preferred_terminal,
    set_theme_mode, terminate_port_processes,
};
use grok_provider::GrokProviderState;
use state::{
    load_favorite_project_dirs, load_launch_mode, load_port_auto_refresh, load_preferred_terminal,
    load_theme_mode, save_preferred_terminal, AppState,
};
use tauri::{Manager, path::BaseDirectory};

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
            // 扫描磁盘缓存：{app_data}/scan-cache-v1.json（不含 delete_target）
            if let Ok(app_data) = app.path().resolve("", BaseDirectory::AppData) {
                let cache_path = app_data.join("scan-cache-v1.json");
                if let Err(err) = state.set_scan_cache_path(cache_path) {
                    eprintln!("scan-cache path setup failed: {err}");
                }
            }
            app.manage(state);
            let grok = GrokProviderState::new().unwrap_or_else(|err| {
                eprintln!("grok provider init failed: {err}");
                // Fallback empty state is not available; re-panic with message
                panic!("failed to init grok provider: {err}");
            });
            app.manage(grok);
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
            grok_provider_status,
            grok_list_profiles,
            grok_create_profile,
            grok_update_profile,
            grok_delete_profile,
            grok_activate_profile,
            grok_activate_official,
            grok_apply_privacy_protection,
            grok_import_current,
            grok_list_backups,
            grok_restore_backup,
            get_grok_provider_layout,
            set_grok_provider_layout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
