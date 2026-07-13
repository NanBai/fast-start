//! Grok 配置只读健康诊断：白名单字段 + issues[]，禁止 secret / 绝对路径。

use super::profile::{GrokBackupInfo, GrokProfile, GrokProviderStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokHealthBackup {
    pub name: String,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokHealthIssue {
    pub code: String,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokHealthReport {
    pub config_present: bool,
    pub auth_present: bool,
    pub profiles_count: usize,
    /// official | profile | unknown
    pub active_mode: String,
    pub active_profile_id: Option<String>,
    pub config_matches_active: bool,
    pub backups: Vec<GrokHealthBackup>,
    pub issues: Vec<GrokHealthIssue>,
}

/// 从已有 status / profiles / backups 映射消毒报告（不读盘二次）。
pub fn build_health_report(
    status: &GrokProviderStatus,
    profiles: &[GrokProfile],
    backups: &[GrokBackupInfo],
    backup_dir_readable: bool,
) -> GrokHealthReport {
    let active_profile_id = status
        .active_profile
        .as_ref()
        .map(|p| p.id.clone())
        .filter(|id| !id.is_empty());

    let active_mode = if status.official_active {
        "official".to_string()
    } else if active_profile_id.is_some() {
        "profile".to_string()
    } else {
        "unknown".to_string()
    };

    let backups_safe: Vec<GrokHealthBackup> = backups
        .iter()
        .map(|b| GrokHealthBackup {
            name: file_name_only(&b.file),
            modified_at: b.created_at,
        })
        .collect();

    let mut issues = Vec::new();

    if !status.config_exists {
        issues.push(issue(
            "config_missing",
            "error",
            "未找到 Grok config.toml，供应商切换可能无法生效",
        ));
    }

    if status.official_active && !status.official_logged_in {
        issues.push(issue(
            "auth_missing_official",
            "warn",
            "当前为官方模式但未检测到 auth 登录态，可能需要重新登录",
        ));
    }

    if !status.official_active {
        if active_profile_id.is_none() {
            issues.push(issue(
                "active_profile_missing",
                "error",
                "非官方模式但没有有效的 active profile",
            ));
        } else if status.config_exists && !status.config_matches_active {
            issues.push(issue(
                "config_mismatch_active",
                "warn",
                "config 与当前 active profile 不一致，可能尚未切换或被外部修改",
            ));
        }
    }

    if profiles.is_empty() {
        issues.push(issue(
            "profiles_empty",
            "info",
            "尚无自定义供应商档案（可导入或新建）",
        ));
    }

    if !backup_dir_readable {
        issues.push(issue(
            "backup_dir_unreadable",
            "warn",
            "备份目录不可读，无法列出历史备份",
        ));
    }

    GrokHealthReport {
        config_present: status.config_exists,
        auth_present: status.official_logged_in,
        profiles_count: profiles.len(),
        active_mode,
        active_profile_id,
        config_matches_active: status.config_matches_active,
        backups: backups_safe,
        issues,
    }
}

fn file_name_only(name: &str) -> String {
    std::path::Path::new(name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(name)
        .to_string()
}

fn issue(code: &str, severity: &str, message: &str) -> GrokHealthIssue {
    GrokHealthIssue {
        code: code.to_string(),
        severity: severity.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grok_provider::profile::GrokProfile;
    use std::path::PathBuf;

    fn empty_status() -> GrokProviderStatus {
        GrokProviderStatus {
            active_profile: None,
            config_path: PathBuf::from("/secret/home/.grok/config.toml"),
            data_dir: PathBuf::from("/secret/home/.grok_switch"),
            config_matches_active: false,
            config_exists: false,
            official_active: true,
            official_logged_in: false,
        }
    }

    #[test]
    fn official_without_auth_emits_auth_missing() {
        let report = build_health_report(&empty_status(), &[], &[], true);
        assert!(report.issues.iter().any(|i| i.code == "auth_missing_official"));
        assert!(report.issues.iter().any(|i| i.code == "config_missing"));
        assert!(report.issues.iter().any(|i| i.code == "profiles_empty"));
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("apiKey"));
        assert!(!json.contains("api_key"));
        assert!(!json.contains("/secret/home"));
        assert!(!json.contains("config_path"));
        assert!(!json.contains("data_dir"));
    }

    #[test]
    fn profile_mismatch_and_backup_name_only() {
        let mut status = empty_status();
        status.config_exists = true;
        status.official_active = false;
        status.official_logged_in = true;
        status.config_matches_active = false;
        status.active_profile = Some(GrokProfile {
            id: "p1".into(),
            name: "Proxy".into(),
            upstream_format: "openai_chat".into(),
            base_url: "https://example.com".into(),
            api_key: "sk-SECRET-SHOULD-NOT-LEAK".into(),
            available_models: vec![],
            default_model: String::new(),
            web_search_model: String::new(),
            subagents_default_model: String::new(),
            models: vec![],
            created_at: None,
            updated_at: None,
            is_active: true,
        });
        let backups = vec![GrokBackupInfo {
            file: "config-20260101.toml".into(),
            path: PathBuf::from("/secret/home/.grok_switch/backups/config-20260101.toml"),
            created_at: Utc::now(),
            size: 99,
        }];
        let profiles = [status.active_profile.clone().unwrap()];
        let report = build_health_report(&status, &profiles, &backups, true);
        assert!(report
            .issues
            .iter()
            .any(|i| i.code == "config_mismatch_active"));
        assert_eq!(report.backups[0].name, "config-20260101.toml");
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("sk-SECRET"));
        assert!(!json.contains("/secret/home"));
        assert!(!json.contains("apiKey"));
    }
}
