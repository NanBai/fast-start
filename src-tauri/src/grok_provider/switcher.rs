use super::config::{apply_profile_to_file, current_matches, import_profile};
use super::profile::{GrokBackupInfo, GrokProfile, GrokProviderStatus};
use super::store::ProfileStore;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct GrokSwitcher {
    config_path: PathBuf,
    data_dir: PathBuf,
    backups_dir: PathBuf,
    profiles: ProfileStore,
}

impl GrokSwitcher {
    pub fn open() -> Result<Self, String> {
        let (config_path, data_dir) = resolve_paths()?;
        let backups_dir = data_dir.join("backups");
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&backups_dir).map_err(|e| e.to_string())?;
        let profiles = ProfileStore::new(data_dir.join("profiles.json"));
        let sw = Self {
            config_path,
            data_dir,
            backups_dir,
            profiles,
        };
        let _ = sw.ensure_default_profile();
        Ok(sw)
    }

    #[cfg(test)]
    pub fn open_with(config_path: PathBuf, data_dir: PathBuf) -> Result<Self, String> {
        let backups_dir = data_dir.join("backups");
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&backups_dir).map_err(|e| e.to_string())?;
        let profiles = ProfileStore::new(data_dir.join("profiles.json"));
        Ok(Self {
            config_path,
            data_dir,
            backups_dir,
            profiles,
        })
    }

    pub fn list_profiles(&self) -> Result<Vec<GrokProfile>, String> {
        self.profiles.list()
    }

    pub fn create_profile(&self, profile: GrokProfile) -> Result<GrokProfile, String> {
        self.profiles.create(profile)
    }

    pub fn update_profile(&self, id: &str, profile: GrokProfile) -> Result<GrokProfile, String> {
        self.profiles.update(id, profile)
    }

    pub fn delete_profile(&self, id: &str) -> Result<(), String> {
        self.profiles.delete(id)
    }

    pub fn activate(&self, id: &str) -> Result<GrokProfile, String> {
        let profile = self.profiles.get(id)?;
        self.backup()?;
        apply_profile_to_file(&self.config_path, &profile)?;
        self.profiles.set_active(id)?;
        let mut active = self.profiles.get(id)?;
        active.is_active = true;
        Ok(active)
    }

    pub fn import_current(&self, name: &str, active: bool) -> Result<GrokProfile, String> {
        if !self.config_path.exists() {
            return Err("config.toml 不存在，无法导入".to_string());
        }
        let mut profile = import_profile(&self.config_path, name)?;
        profile.is_active = active;
        let created = self.profiles.create(profile)?;
        if active {
            self.profiles.set_active(&created.id)?;
            let mut out = self.profiles.get(&created.id)?;
            out.is_active = true;
            return Ok(out);
        }
        Ok(created)
    }

    pub fn ensure_default_profile(&self) -> Result<(), String> {
        if !self.profiles.list()?.is_empty() {
            return Ok(());
        }
        if !self.config_path.exists() {
            return Ok(());
        }
        let _ = self.import_current("Default", true)?;
        Ok(())
    }

    pub fn status(&self) -> Result<GrokProviderStatus, String> {
        let profiles = self.profiles.list()?;
        let active = profiles.into_iter().find(|p| p.is_active);
        let config_exists = self.config_path.exists();
        let config_matches_active = match &active {
            Some(p) if config_exists => current_matches(&self.config_path, p).unwrap_or(false),
            _ => false,
        };
        Ok(GrokProviderStatus {
            active_profile: active,
            config_path: self.config_path.clone(),
            data_dir: self.data_dir.clone(),
            config_matches_active,
            config_exists,
        })
    }

    pub fn backup(&self) -> Result<GrokBackupInfo, String> {
        if !self.config_path.exists() {
            return Err("config.toml 不存在，无法备份".to_string());
        }
        fs::create_dir_all(&self.backups_dir).map_err(|e| e.to_string())?;
        let data = fs::read(&self.config_path).map_err(|e| e.to_string())?;
        let stamp = Utc::now().format("%Y%m%d-%H%M%S");
        let file = format!("config-{stamp}.toml");
        let path = self.backups_dir.join(&file);
        fs::write(&path, data).map_err(|e| e.to_string())?;
        self.prune_backups(10)?;
        let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
        Ok(GrokBackupInfo {
            file,
            path,
            created_at: system_time_to_utc(meta.modified().ok()),
            size: meta.len(),
        })
    }

    pub fn list_backups(&self) -> Result<Vec<GrokBackupInfo>, String> {
        if !self.backups_dir.exists() {
            return Ok(Vec::new());
        }
        let mut backups = Vec::new();
        for entry in fs::read_dir(&self.backups_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(true) {
                continue;
            }
            if !name.to_lowercase().ends_with(".toml") {
                continue;
            }
            let meta = entry.metadata().map_err(|e| e.to_string())?;
            backups.push(GrokBackupInfo {
                file: name,
                path: entry.path(),
                created_at: system_time_to_utc(meta.modified().ok()),
                size: meta.len(),
            });
        }
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    pub fn restore_backup(&self, file: &str) -> Result<(), String> {
        let base = Path::new(file)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "非法备份文件名".to_string())?;
        if base != file || !base.to_lowercase().ends_with(".toml") {
            return Err("非法备份文件名".to_string());
        }
        let src = self.backups_dir.join(base);
        let data = fs::read(&src).map_err(|e| e.to_string())?;
        if self.config_path.exists() {
            let _ = self.backup();
        }
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let tmp = self.config_path.with_extension("toml.tmp");
        fs::write(&tmp, data).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &self.config_path).map_err(|e| e.to_string())
    }

    fn prune_backups(&self, keep: usize) -> Result<(), String> {
        let backups = self.list_backups()?;
        for backup in backups.into_iter().skip(keep) {
            let _ = fs::remove_file(backup.path);
        }
        Ok(())
    }
}

fn resolve_paths() -> Result<(PathBuf, PathBuf), String> {
    let home = dirs::home_dir().ok_or_else(|| "无法定位用户主目录".to_string())?;
    let grok_home = std::env::var("GROK_HOME")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".grok"));
    let config_path = std::env::var("GROK_CONFIG")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| grok_home.join("config.toml"));
    let data_dir = home.join(".grok_switch");
    Ok((config_path, data_dir))
}

fn system_time_to_utc(time: Option<SystemTime>) -> DateTime<Utc> {
    time.map(DateTime::<Utc>::from)
        .unwrap_or_else(Utc::now)
}

#[cfg(test)]
mod tests {
    use super::GrokSwitcher;
    use crate::grok_provider::profile::{GrokModelDef, GrokProfile};
    use std::fs;

    #[test]
    fn activate_writes_config_and_marks_active() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(
            &config,
            r#"[cli]
show_tips = false

[models]
default = "old"
"#,
        )
        .unwrap();
        let sw = GrokSwitcher::open_with(config.clone(), data).unwrap();
        let created = sw
            .create_profile(GrokProfile {
                id: String::new(),
                name: "Alpha".into(),
                upstream_format: "openai_chat".into(),
                base_url: "http://127.0.0.1:9/v1".into(),
                api_key: "k".into(),
                available_models: vec![],
                default_model: "m".into(),
                web_search_model: "m".into(),
                subagents_default_model: "m".into(),
                models: vec![GrokModelDef {
                    name: "m".into(),
                    model: "m".into(),
                    api_key: "k".into(),
                    api_backend: "chat_completions".into(),
                    ..Default::default()
                }],
                created_at: None,
                updated_at: None,
                is_active: false,
            })
            .unwrap();
        let active = sw.activate(&created.id).unwrap();
        assert!(active.is_active);
        let text = fs::read_to_string(&config).unwrap();
        assert!(text.contains("models_base_url = \"http://127.0.0.1:9/v1\""));
        assert!(text.contains("default = \"m\""));
        assert!(text.contains("[cli]"));
        assert!(sw.list_backups().unwrap().len() >= 1);
    }
}
