//! Grok CLI 供应商切换（融合 grok-build-switch MVP + v0.2.0 官方/隐私/布局）。
//!
//! - 档案：`~/.grok_switch/profiles.json`
//! - 备份：`~/.grok_switch/backups/`
//! - 生效配置：`~/.grok/config.toml`（可用 `GROK_HOME` / `GROK_CONFIG` 覆盖）
//! - 官方 OAuth：`GROK_HOME/auth.json`（不受 `GROK_CONFIG` 影响）

mod config;
mod profile;
mod store;
mod switcher;

pub use profile::{
    GrokActivateOfficialResult, GrokBackupInfo, GrokPrivacyResult, GrokProfile,
    GrokProviderLayout, GrokProviderStatus,
};
pub use switcher::GrokSwitcher;

use std::sync::Mutex;

pub struct GrokProviderState {
    inner: Mutex<GrokSwitcher>,
}

impl GrokProviderState {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            inner: Mutex::new(GrokSwitcher::open()?),
        })
    }

    fn with<T>(&self, f: impl FnOnce(&mut GrokSwitcher) -> Result<T, String>) -> Result<T, String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取 Grok 供应商状态".to_string())?;
        f(&mut guard)
    }

    pub fn status(&self) -> Result<GrokProviderStatus, String> {
        self.with(|s| s.status())
    }

    pub fn list_profiles(&self) -> Result<Vec<GrokProfile>, String> {
        self.with(|s| s.list_profiles())
    }

    pub fn create_profile(&self, profile: GrokProfile) -> Result<GrokProfile, String> {
        self.with(|s| s.create_profile(profile))
    }

    pub fn update_profile(&self, id: &str, profile: GrokProfile) -> Result<GrokProfile, String> {
        self.with(|s| s.update_profile(id, profile))
    }

    pub fn delete_profile(&self, id: &str) -> Result<(), String> {
        self.with(|s| s.delete_profile(id))
    }

    pub fn activate_profile(&self, id: &str) -> Result<GrokProfile, String> {
        self.with(|s| s.activate(id))
    }

    pub fn activate_official(&self) -> Result<GrokActivateOfficialResult, String> {
        self.with(|s| s.activate_official())
    }

    pub fn apply_privacy_protection(&self) -> Result<GrokPrivacyResult, String> {
        self.with(|s| s.apply_privacy_protection())
    }

    pub fn import_current(&self, name: String, active: bool) -> Result<GrokProfile, String> {
        self.with(|s| s.import_current(&name, active))
    }

    pub fn list_backups(&self) -> Result<Vec<GrokBackupInfo>, String> {
        self.with(|s| s.list_backups())
    }

    pub fn restore_backup(&self, file: &str) -> Result<(), String> {
        self.with(|s| s.restore_backup(file))
    }
}
