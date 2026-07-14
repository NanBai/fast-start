//! Oh My Pi (omp) 供应商/角色模型的窄支持。
//!
//! 只做：
//! - 读取 `models.yml` 的 providers（消毒，不回传 apiKey）
//! - 读取 `config.yml` 的 modelRoles + 文件健康
//! - 受控写入 `modelRoles.<role>`（先备份）
//!
//! 不复制 grok_provider 的 http/profile UI。

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmpProviderInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    pub models: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_for: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmpConfigHealth {
    pub models_yml_exists: bool,
    pub config_yml_exists: bool,
    pub current_roles: HashMap<String, String>,
    pub issues: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct OmpConfigFile {
    #[serde(default, rename = "modelRoles")]
    model_roles: HashMap<String, String>,
    /// 保留其余键，避免写回时丢掉 theme 等字段。
    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>,
}

pub fn default_agent_dir() -> Result<PathBuf, String> {
    if let Ok(home) = std::env::var("OMP_HOME") {
        let home = home.trim();
        if !home.is_empty() {
            return Ok(PathBuf::from(home).join("agent"));
        }
    }
    if let Ok(dir) = std::env::var("PI_CODING_AGENT_DIR") {
        let dir = dir.trim();
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }
    dirs::home_dir()
        .map(|h| h.join(".omp/agent"))
        .ok_or_else(|| "无法定位用户主目录".to_string())
}

pub fn list_providers(agent_dir: &Path) -> Value {
    let models_path = agent_dir.join("models.yml");
    let mut issues: Vec<String> = Vec::new();
    let providers = if models_path.exists() {
        match fs::read_to_string(&models_path) {
            Ok(content) => match parse_models_yml(&content) {
                Ok(list) => list,
                Err(e) => {
                    issues.push(format!("解析 models.yml 失败: {e}"));
                    Vec::new()
                }
            },
            Err(e) => {
                issues.push(format!("读取 models.yml 失败: {e}"));
                Vec::new()
            }
        }
    } else {
        issues.push("models.yml 不存在".to_string());
        Vec::new()
    };

    json!({
        "providers": providers,
        "modelsYmlExists": models_path.exists(),
        "issues": issues,
    })
}

pub fn get_config_health(agent_dir: &Path) -> OmpConfigHealth {
    let models_yml = agent_dir.join("models.yml");
    let config_yml = agent_dir.join("config.yml");
    let mut current_roles = HashMap::new();
    let mut issues: Vec<String> = Vec::new();

    if config_yml.exists() {
        match fs::read_to_string(&config_yml) {
            Ok(content) => match serde_yaml::from_str::<OmpConfigFile>(&content) {
                Ok(cfg) => current_roles = cfg.model_roles,
                Err(e) => issues.push(format!("解析 config.yml 失败: {e}")),
            },
            Err(e) => issues.push(format!("读取 config.yml 失败: {e}")),
        }
    } else {
        issues.push("config.yml 不存在（将使用 omp 默认）".to_string());
    }

    if !models_yml.exists() {
        issues.push("models.yml 不存在".to_string());
    }

    OmpConfigHealth {
        models_yml_exists: models_yml.exists(),
        config_yml_exists: config_yml.exists(),
        current_roles,
        issues,
    }
}

pub fn set_role_model(agent_dir: &Path, role: &str, model: &str) -> Result<Value, String> {
    let role = role.trim();
    let model = model.trim();
    validate_role(role)?;
    validate_model_ref(model)?;

    let config_yml = agent_dir.join("config.yml");
    if let Some(parent) = config_yml.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
    }

    let mut config: OmpConfigFile = if config_yml.exists() {
        let content =
            fs::read_to_string(&config_yml).map_err(|e| format!("读取 config.yml 失败: {e}"))?;
        serde_yaml::from_str(&content).map_err(|e| format!("解析 config.yml 失败: {e}"))?
    } else {
        OmpConfigFile::default()
    };

    let backup_path = if config_yml.exists() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // 用独立文件名，避免 with_extension 把 .yml 吃掉后变成奇怪后缀。
        let bak = agent_dir.join(format!("config.yml.bak-{ts}"));
        fs::copy(&config_yml, &bak).map_err(|e| format!("备份失败: {e}"))?;
        Some(bak.display().to_string())
    } else {
        None
    };

    let old = config
        .model_roles
        .insert(role.to_string(), model.to_string());

    let new_content =
        serde_yaml::to_string(&config).map_err(|e| format!("序列化失败: {e}"))?;
    fs::write(&config_yml, new_content).map_err(|e| format!("写入 config.yml 失败: {e}"))?;

    Ok(json!({
        "ok": true,
        "role": role,
        "model": model,
        "previous": old,
        "backup": backup_path,
        "message": "已更新 modelRoles 并创建备份（若原文件存在）。"
    }))
}

/// 解析 models.yml。
/// 支持：
/// ```yaml
/// providers:
///   litellm:
///     baseUrl: http://localhost:4000/v1
///     apiKey: secret
///     api: openai-completions
///     models:
///       - a
/// ```
pub fn parse_models_yml(content: &str) -> Result<Vec<OmpProviderInfo>, String> {
    let root: serde_yaml::Value =
        serde_yaml::from_str(content).map_err(|e| format!("yaml 无效: {e}"))?;
    let providers_node = root
        .get("providers")
        .cloned()
        .unwrap_or(serde_yaml::Value::Null);

    let mut out = Vec::new();
    match providers_node {
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                let name = match k.as_str() {
                    Some(s) if !s.is_empty() => s.to_string(),
                    _ => continue,
                };
                out.push(provider_from_node(name, v));
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                if let Some(name) = item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                {
                    out.push(provider_from_node(name.to_string(), item));
                }
            }
        }
        serde_yaml::Value::Null => {}
        _ => return Err("providers 字段类型不受支持".into()),
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn provider_from_node(name: String, node: serde_yaml::Value) -> OmpProviderInfo {
    let base_url = node
        .get("baseUrl")
        .or_else(|| node.get("base_url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let api = node
        .get("api")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut models = Vec::new();
    if let Some(models_node) = node.get("models") {
        match models_node {
            serde_yaml::Value::Sequence(seq) => {
                for m in seq {
                    if let Some(s) = m.as_str() {
                        if !s.is_empty() {
                            models.push(s.to_string());
                        }
                    } else if let Some(id) = m.get("id").and_then(|v| v.as_str()) {
                        if !id.is_empty() {
                            models.push(id.to_string());
                        }
                    }
                }
            }
            serde_yaml::Value::Mapping(map) => {
                for (k, _) in map {
                    if let Some(s) = k.as_str() {
                        if !s.is_empty() {
                            models.push(s.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut default_for = Vec::new();
    if let Some(df) = node.get("defaultFor").or_else(|| node.get("default_for")) {
        if let serde_yaml::Value::Sequence(seq) = df {
            for item in seq {
                if let Some(s) = item.as_str() {
                    default_for.push(s.to_string());
                }
            }
        }
    }

    OmpProviderInfo {
        name,
        base_url,
        api,
        models,
        default_for,
    }
}

fn validate_role(role: &str) -> Result<(), String> {
    if role.is_empty() {
        return Err("role 不能为空".into());
    }
    if role.len() > 64 {
        return Err("role 过长".into());
    }
    if !role
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err("role 仅允许字母数字、.-_".into());
    }
    Ok(())
}

fn validate_model_ref(model: &str) -> Result<(), String> {
    if model.is_empty() {
        return Err("model 不能为空".into());
    }
    if model.len() > 256 {
        return Err("model 过长".into());
    }
    if model.chars().any(|c| c.is_control() || c == '\n' || c == '\r') {
        return Err("model 含非法字符".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parse_models_yml_mapping_strips_api_key() {
        let yml = r#"
providers:
  litellm:
    baseUrl: http://localhost:4000/v1
    apiKey: secret-should-not-leak
    api: openai-completions
    models:
      - gpt-4o
  openai:
    baseUrl: https://api.openai.com/v1
"#;
        let list = parse_models_yml(yml).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "litellm");
        assert_eq!(list[0].base_url.as_deref(), Some("http://localhost:4000/v1"));
        assert_eq!(list[0].models, vec!["gpt-4o".to_string()]);
        let encoded = serde_json::to_string(&list).unwrap();
        assert!(!encoded.contains("secret-should-not-leak"));
        assert!(!encoded.contains("apiKey"));
    }

    #[test]
    fn set_role_model_backs_up_and_updates() {
        let dir = tempdir().unwrap();
        let agent = dir.path();
        fs::write(
            agent.join("config.yml"),
            "theme:\n  dark: titanium\nmodelRoles:\n  default: old/model\n",
        )
        .unwrap();

        let result = set_role_model(agent, "default", "anthropic/claude-3-5-sonnet").unwrap();
        assert_eq!(result["ok"], true);
        assert!(result["backup"].as_str().is_some());

        let health = get_config_health(agent);
        assert_eq!(
            health.current_roles.get("default").map(String::as_str),
            Some("anthropic/claude-3-5-sonnet")
        );
        // theme 应仍保留
        let content = fs::read_to_string(agent.join("config.yml")).unwrap();
        assert!(content.contains("titanium") || content.contains("theme"));
    }

    #[test]
    fn set_role_model_rejects_bad_role() {
        let dir = tempdir().unwrap();
        let err = set_role_model(dir.path(), "bad role", "m").unwrap_err();
        assert!(err.contains("role"));
    }

    #[test]
    fn list_providers_missing_file_reports_issue() {
        let dir = tempdir().unwrap();
        let v = list_providers(dir.path());
        assert_eq!(v["modelsYmlExists"], false);
        assert!(v["issues"].as_array().unwrap().len() >= 1);
    }
}
