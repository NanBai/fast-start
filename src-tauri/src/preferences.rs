//! 本机 preferences.json 读写（tauri-plugin-store）。
use crate::models::{LaunchMode, TerminalType, ThemeMode};
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const PREFERRED_TERMINAL_KEY: &str = "preferred_terminal";
const LAUNCH_MODE_KEY: &str = "launch_mode";
const THEME_MODE_KEY: &str = "theme_mode";
const FAVORITE_PROJECT_DIRS_KEY: &str = "favorite_project_dirs";
const PORT_AUTO_REFRESH_KEY: &str = "port_auto_refresh";

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

pub fn load_port_auto_refresh(app: &AppHandle) -> Result<bool, String> {
    let store = app
        .store("preferences.json")
        .map_err(|err| err.to_string())?;
    let value = store.get(PORT_AUTO_REFRESH_KEY);
    if let Some(raw) = value {
        serde_json::from_value(raw).map_err(|err| err.to_string())
    } else {
        Ok(true)
    }
}

pub fn save_port_auto_refresh(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let store = app
        .store("preferences.json")
        .map_err(|err| err.to_string())?;
    store.set(PORT_AUTO_REFRESH_KEY, json!(enabled));
    store.save().map_err(|err| err.to_string())
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


