use super::config::{
    apply_privacy_protection_to_file, apply_profile_text, apply_profile_to_file, current_matches,
    import_profile, use_official_auth_to_file,
};
use super::profile::{
    GrokActivateOfficialResult, GrokBackupInfo, GrokPrivacyResult, GrokProfile, GrokProviderStatus,
};
use super::store::ProfileStore;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct GrokSwitcher {
    config_path: PathBuf,
    auth_path: PathBuf,
    data_dir: PathBuf,
    backups_dir: PathBuf,
    profiles: ProfileStore,
}

impl GrokSwitcher {
    pub fn open() -> Result<Self, String> {
        let (config_path, auth_path, data_dir) = resolve_paths()?;
        let backups_dir = data_dir.join("backups");
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&backups_dir).map_err(|e| e.to_string())?;
        let profiles = ProfileStore::new(data_dir.join("profiles.json"));
        let sw = Self {
            config_path,
            auth_path,
            data_dir,
            backups_dir,
            profiles,
        };
        let _ = sw.ensure_default_profile();
        Ok(sw)
    }

    #[cfg(test)]
    pub fn open_with(config_path: PathBuf, data_dir: PathBuf) -> Result<Self, String> {
        let auth_path = config_path
            .parent()
            .map(|p| p.join("auth.json"))
            .unwrap_or_else(|| PathBuf::from("auth.json"));
        let backups_dir = data_dir.join("backups");
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&backups_dir).map_err(|e| e.to_string())?;
        let profiles = ProfileStore::new(data_dir.join("profiles.json"));
        Ok(Self {
            config_path,
            auth_path,
            data_dir,
            backups_dir,
            profiles,
        })
    }

    #[cfg(test)]
    pub fn set_auth_path_for_test(&mut self, path: PathBuf) {
        self.auth_path = path;
    }

    #[cfg(test)]
    pub fn set_backups_dir_for_test(&mut self, path: PathBuf) {
        self.backups_dir = path;
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

    pub fn activate_official(&self) -> Result<GrokActivateOfficialResult, String> {
        if self.config_path.exists() {
            self.backup()?;
        }
        use_official_auth_to_file(&self.config_path)?;
        self.profiles.clear_active().map_err(|e| {
            format!("已清理 config.toml，但清除供应商启用状态失败，请重试「启用官方」：{e}")
        })?;

        let login_required = !self.auth_path.exists();
        let mut message = "已切换到官方账号，新开 Grok 会话生效".to_string();
        if login_required {
            match start_grok_login() {
                Ok(()) => {
                    message =
                        "已切换到官方配置，请在浏览器完成 grok login".to_string();
                }
                Err(_) => {
                    message =
                        "已切换到官方配置，请在终端执行 grok login".to_string();
                }
            }
        }
        Ok(GrokActivateOfficialResult {
            login_required,
            message,
        })
    }

    pub fn apply_privacy_protection(&self) -> Result<GrokPrivacyResult, String> {
        if self.config_path.exists() {
            self.backup()?;
        }
        apply_privacy_protection_to_file(&self.config_path)?;
        Ok(GrokPrivacyResult {
            path: self.config_path.clone(),
            message: "隐私保护配置已写入 config.toml".to_string(),
        })
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
        // 有 models_base_url（API 上游）才 active；纯 OAuth 配置保持官方模式。
        let imported = import_profile(&self.config_path, "Default")?;
        let active = !imported.base_url.trim().is_empty();
        let _ = self.import_current("Default", active)?;
        Ok(())
    }

    /// 预览 apply 结果文本；读取现有 config 但不写盘。
    pub fn preview_apply(&self, profile: &GrokProfile) -> Result<String, String> {
        let data = if self.config_path.exists() {
            fs::read(&self.config_path).map_err(|e| e.to_string())?
        } else {
            Vec::new()
        };
        let next = apply_profile_text(&data, profile)?;
        String::from_utf8(next).map_err(|e| format!("预览编码失败: {e}"))
    }

    pub fn status(&self) -> Result<GrokProviderStatus, String> {
        let profiles = self.profiles.list()?;
        let active = profiles.into_iter().find(|p| p.is_active);
        let official_active = active.is_none();
        let official_logged_in = self.auth_path.exists();
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
            official_active,
            official_logged_in,
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

#[cfg(test)]
fn start_grok_login() -> Result<(), String> {
    // 单测不得拉起真实 OAuth 设备流
    Err("skipped in tests".to_string())
}

#[cfg(not(test))]
fn start_grok_login() -> Result<(), String> {
    use std::process::Command;
    let mut cmd = Command::new("grok");
    cmd.arg("login");
    // best-effort：常见安装路径
    if let Some(home) = dirs::home_dir() {
        let bin = home.join(".grok").join("bin");
        if bin.is_dir() {
            let path = std::env::var_os("PATH").unwrap_or_default();
            let mut paths = std::env::split_paths(&path).collect::<Vec<_>>();
            paths.insert(0, bin);
            if let Ok(joined) = std::env::join_paths(paths) {
                cmd.env("PATH", joined);
            }
        }
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// config 来自 GROK_CONFIG 或 GROK_HOME/config.toml；
/// auth 始终来自 GROK_HOME/auth.json（不受 GROK_CONFIG 影响）。
fn resolve_paths() -> Result<(PathBuf, PathBuf, PathBuf), String> {
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
    let auth_path = grok_home.join("auth.json");
    let data_dir = home.join(".grok_switch");
    Ok((config_path, auth_path, data_dir))
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
    use std::os::unix::fs::PermissionsExt;

    fn sample_profile(name: &str) -> GrokProfile {
        GrokProfile {
            id: String::new(),
            name: name.into(),
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
        }
    }

    #[test]
    fn ensure_default_oauth_only_imports_inactive() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(
            &config,
            r#"[cli]
show_tips = false

[models]
default = "grok-3"
"#,
        )
        .unwrap();
        // open_with 不自动 ensure；手动调用
        let sw = GrokSwitcher::open_with(config, data).unwrap();
        sw.ensure_default_profile().unwrap();
        let profiles = sw.list_profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert!(!profiles[0].is_active);
        assert!(sw.status().unwrap().official_active);
    }

    #[test]
    fn ensure_default_with_api_upstream_imports_active() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(
            &config,
            r#"[endpoints]
models_base_url = "https://api.example.com/v1"

[models]
default = "m"
"#,
        )
        .unwrap();
        let sw = GrokSwitcher::open_with(config, data).unwrap();
        sw.ensure_default_profile().unwrap();
        let profiles = sw.list_profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert!(profiles[0].is_active);
        assert!(!sw.status().unwrap().official_active);
    }

    #[test]
    fn ensure_default_skips_when_profiles_exist() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(
            &config,
            r#"[endpoints]
models_base_url = "https://api.example.com/v1"
"#,
        )
        .unwrap();
        let sw = GrokSwitcher::open_with(config, data).unwrap();
        sw.create_profile(sample_profile("Existing")).unwrap();
        sw.ensure_default_profile().unwrap();
        assert_eq!(sw.list_profiles().unwrap().len(), 1);
        assert_eq!(sw.list_profiles().unwrap()[0].name, "Existing");
    }

    #[test]
    fn preview_apply_contains_keys_without_writing() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(&config, "[cli]\nshow_tips = false\n").unwrap();
        let before = fs::read_to_string(&config).unwrap();
        let sw = GrokSwitcher::open_with(config.clone(), data).unwrap();
        let text = sw.preview_apply(&sample_profile("P")).unwrap();
        assert!(text.contains("models_base_url"));
        assert!(text.contains("default = \"m\""));
        assert_eq!(fs::read_to_string(&config).unwrap(), before);
    }

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
        let created = sw.create_profile(sample_profile("Alpha")).unwrap();
        let active = sw.activate(&created.id).unwrap();
        assert!(active.is_active);
        let text = fs::read_to_string(&config).unwrap();
        assert!(text.contains("models_base_url = \"http://127.0.0.1:9/v1\""));
        assert!(text.contains("default = \"m\""));
        assert!(text.contains("[cli]"));
        assert!(sw.list_backups().unwrap().len() >= 1);
    }

    #[test]
    fn activate_official_clears_provider_and_active() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(
            &config,
            r#"[cli]
show_tips = false

[endpoints]
models_base_url = "http://old"

[models]
default = "old"
"#,
        )
        .unwrap();
        let mut sw = GrokSwitcher::open_with(config.clone(), data).unwrap();
        let created = sw.create_profile(sample_profile("Alpha")).unwrap();
        sw.activate(&created.id).unwrap();
        // no auth.json
        sw.set_auth_path_for_test(temp.path().join("missing-auth.json"));
        let result = sw.activate_official().unwrap();
        assert!(result.login_required);
        let status = sw.status().unwrap();
        assert!(status.official_active);
        assert!(!status.official_logged_in);
        assert!(status.active_profile.is_none());
        let text = fs::read_to_string(&config).unwrap();
        assert!(!text.contains("models_base_url"));
        assert!(text.contains("[cli]"));
        assert!(sw.list_backups().unwrap().len() >= 1);
    }

    #[test]
    fn activate_official_with_auth_not_login_required() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        let auth = temp.path().join("auth.json");
        fs::write(&config, "[cli]\nshow_tips = false\n").unwrap();
        fs::write(&auth, "{}").unwrap();
        let mut sw = GrokSwitcher::open_with(config, data).unwrap();
        sw.set_auth_path_for_test(auth);
        let result = sw.activate_official().unwrap();
        assert!(!result.login_required);
        assert!(sw.status().unwrap().official_logged_in);
    }

    #[test]
    fn activate_official_without_config_skips_backup() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        let mut sw = GrokSwitcher::open_with(config.clone(), data).unwrap();
        sw.set_auth_path_for_test(temp.path().join("missing-auth.json"));
        let result = sw.activate_official().unwrap();
        assert!(result.login_required);
        assert!(config.exists());
        assert!(sw.list_backups().unwrap().is_empty());
        assert!(sw.status().unwrap().official_active);
    }

    #[test]
    fn activate_official_clear_active_failure_returns_err() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(&config, "[cli]\nx = 1\n").unwrap();
        let mut sw = GrokSwitcher::open_with(config.clone(), data.clone()).unwrap();
        let created = sw.create_profile(sample_profile("Locked")).unwrap();
        sw.activate(&created.id).unwrap();
        // 备份写到 data 外，避免锁 data 后 backup 先失败
        let alt_backups = temp.path().join("alt-backups");
        fs::create_dir_all(&alt_backups).unwrap();
        sw.set_backups_dir_for_test(alt_backups);
        let mut perms = fs::metadata(&data).unwrap().permissions();
        perms.set_mode(0o555);
        fs::set_permissions(&data, perms).unwrap();
        let result = sw.activate_official();
        let mut restore = fs::metadata(&data).unwrap().permissions();
        restore.set_mode(0o755);
        fs::set_permissions(&data, restore).unwrap();
        let err = result.unwrap_err();
        assert!(
            err.contains("清除供应商") || err.contains("重试"),
            "unexpected err: {err}"
        );
        // config 可能已清理，但不返回 Ok
        let text = fs::read_to_string(&config).unwrap();
        assert!(!text.contains("models_base_url"));
    }

    #[test]
    fn activate_then_official_then_api_again() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(&config, "[cli]\nx = 1\n").unwrap();
        let sw = GrokSwitcher::open_with(config.clone(), data).unwrap();
        let created = sw.create_profile(sample_profile("Beta")).unwrap();
        sw.activate(&created.id).unwrap();
        sw.activate_official().unwrap();
        assert!(sw.status().unwrap().official_active);
        sw.activate(&created.id).unwrap();
        let status = sw.status().unwrap();
        assert!(!status.official_active);
        assert_eq!(status.active_profile.as_ref().unwrap().id, created.id);
        let text = fs::read_to_string(&config).unwrap();
        assert!(text.contains("models_base_url"));
    }

    #[test]
    fn privacy_without_config_creates_file() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        let sw = GrokSwitcher::open_with(config.clone(), data).unwrap();
        let result = sw.apply_privacy_protection().unwrap();
        assert_eq!(result.path, config);
        let text = fs::read_to_string(&config).unwrap();
        assert!(text.contains("telemetry = false"));
        assert!(text.contains("disable_codebase_upload = true"));
        assert!(sw.list_backups().unwrap().is_empty());
    }

    #[test]
    fn privacy_with_config_backs_up() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(&config, "[cli]\nshow_tips = true\n").unwrap();
        let sw = GrokSwitcher::open_with(config.clone(), data).unwrap();
        sw.apply_privacy_protection().unwrap();
        let text = fs::read_to_string(&config).unwrap();
        assert!(text.contains("show_tips = true"));
        assert!(text.contains("telemetry = false"));
        assert!(!sw.list_backups().unwrap().is_empty());
    }

    #[test]
    fn activate_official_backup_failure_returns_err() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(&config, "[cli]\nx = 1\n").unwrap();
        let mut sw = GrokSwitcher::open_with(config, data.clone()).unwrap();
        // 用文件占用 backups 路径，使 create_dir_all / 写入失败
        let bad_backups = data.join("backups-as-file");
        fs::write(&bad_backups, b"not-a-dir").unwrap();
        sw.set_backups_dir_for_test(bad_backups);
        let err = sw.activate_official().unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn activate_official_backup_unwritable_dir_returns_err() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let data = temp.path().join("data");
        fs::write(&config, "[cli]\nx = 1\n").unwrap();
        let mut sw = GrokSwitcher::open_with(config, data).unwrap();
        let locked = temp.path().join("locked-backups");
        fs::create_dir_all(&locked).unwrap();
        let mut perms = fs::metadata(&locked).unwrap().permissions();
        perms.set_mode(0o555);
        fs::set_permissions(&locked, perms).unwrap();
        sw.set_backups_dir_for_test(locked.clone());
        let result = sw.activate_official();
        // 恢复权限便于 temp 清理
        let mut perms = fs::metadata(&locked).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&locked, perms).unwrap();
        assert!(result.is_err());
    }
}
