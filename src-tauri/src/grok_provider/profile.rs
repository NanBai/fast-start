use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GrokModelDef {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub api_backend: String,
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
    #[serde(default)]
    pub supports_backend_search: bool,
    #[serde(default)]
    pub context_window: i64,
    #[serde(default)]
    pub max_completion_tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GrokProfile {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default = "default_upstream")]
    pub upstream_format: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub available_models: Vec<String>,
    #[serde(default)]
    pub default_model: String,
    #[serde(default)]
    pub web_search_model: String,
    #[serde(default)]
    pub subagents_default_model: String,
    #[serde(default)]
    pub models: Vec<GrokModelDef>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub is_active: bool,
}

fn default_upstream() -> String {
    "openai_chat".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokBackupInfo {
    pub file: String,
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokProviderStatus {
    pub active_profile: Option<GrokProfile>,
    pub config_path: PathBuf,
    pub data_dir: PathBuf,
    pub config_matches_active: bool,
    pub config_exists: bool,
}

impl GrokProfile {
    pub fn normalize(mut self) -> Self {
        if self.upstream_format.is_empty()
            || self.upstream_format == "openai"
            || self.upstream_format == "grok"
        {
            self.upstream_format = "openai_chat".to_string();
        }
        if self.api_key.is_empty() {
            self.api_key = self.effective_api_key();
        }
        if self.models.is_empty() {
            let names = unique_nonempty(&[
                self.default_model.as_str(),
                self.web_search_model.as_str(),
                self.subagents_default_model.as_str(),
            ]);
            for name in names {
                self.models.push(GrokModelDef {
                    name: name.clone(),
                    model: name,
                    ..Default::default()
                });
            }
        }
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let backend = api_backend_for_upstream(&self.upstream_format);
        for m in &mut self.models {
            if m.name.is_empty() {
                m.name = m.model.clone();
            }
            if m.model.is_empty() {
                m.model = m.name.clone();
            }
            if m.base_url.is_empty() {
                m.base_url = base_url.clone();
            }
            if m.api_key.is_empty() {
                m.api_key = api_key.clone();
            }
            if m.api_backend.is_empty() {
                m.api_backend = backend.clone();
            }
        }
        self.available_models = unique_nonempty(
            &self
                .available_models
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn effective_api_key(&self) -> String {
        if !self.api_key.is_empty() {
            return self.api_key.clone();
        }
        self.models
            .iter()
            .find(|m| !m.api_key.is_empty())
            .map(|m| m.api_key.clone())
            .unwrap_or_default()
    }

    pub fn matches_config(&self, other: &GrokProfile) -> bool {
        let a = self.clone().normalize();
        let b = other.clone().normalize();
        if a.base_url != b.base_url
            || a.default_model != b.default_model
            || a.web_search_model != b.web_search_model
            || a.subagents_default_model != b.subagents_default_model
        {
            return false;
        }
        if a.models.is_empty() && b.models.is_empty() {
            return true;
        }
        if a.effective_api_key() != b.effective_api_key() || a.models.len() != b.models.len() {
            return false;
        }
        let by_name: HashMap<_, _> = a
            .models
            .iter()
            .map(|m| (model_key(m), m))
            .collect();
        b.models.iter().all(|m| {
            by_name
                .get(&model_key(m))
                .map(|stored| model_equal(stored, m))
                .unwrap_or(false)
        })
    }
}

impl Default for GrokModelDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            model: String::new(),
            base_url: String::new(),
            api_key: String::new(),
            api_backend: String::new(),
            extra_headers: HashMap::new(),
            supports_backend_search: false,
            context_window: 0,
            max_completion_tokens: 0,
        }
    }
}

fn model_key(m: &GrokModelDef) -> String {
    if !m.name.is_empty() {
        m.name.clone()
    } else {
        m.model.clone()
    }
}

fn model_equal(a: &GrokModelDef, b: &GrokModelDef) -> bool {
    model_key(a) == model_key(b)
        && a.model == b.model
        && a.base_url == b.base_url
        && a.api_key == b.api_key
        && a.api_backend == b.api_backend
        && a.supports_backend_search == b.supports_backend_search
        && a.context_window == b.context_window
        && a.max_completion_tokens == b.max_completion_tokens
        && a.extra_headers == b.extra_headers
}

pub fn api_backend_for_upstream(upstream: &str) -> String {
    match upstream {
        "openai_responses" | "responses" => "responses".into(),
        "anthropic" | "messages" => "messages".into(),
        _ => "chat_completions".into(),
    }
}

fn unique_nonempty(items: &[&str]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for item in items {
        let s = item.trim();
        if s.is_empty() || !seen.insert(s.to_string()) {
            continue;
        }
        out.push(s.to_string());
    }
    out
}
