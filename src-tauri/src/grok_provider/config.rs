//! 读写 `~/.grok/config.toml`：导入 profile、按段重写启用配置（尽量保留其它段落）。

use super::profile::{api_backend_for_upstream, GrokModelDef, GrokProfile};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn import_profile(path: &Path, name: &str) -> Result<GrokProfile, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("读取 config.toml 失败: {e}"))?;
    let doc: toml::Value =
        toml::from_str(&data).map_err(|e| format!("解析 config.toml 失败: {e}"))?;

    let endpoints = doc.get("endpoints").and_then(|v| v.as_table());
    let models = doc.get("models").and_then(|v| v.as_table());
    let subagents = doc.get("subagents").and_then(|v| v.as_table());

    let profile = GrokProfile {
        id: String::new(),
        name: if name.is_empty() {
            "Default".into()
        } else {
            name.into()
        },
        upstream_format: "openai_chat".into(),
        base_url: table_str(endpoints, "models_base_url"),
        api_key: String::new(),
        available_models: Vec::new(),
        default_model: table_str(models, "default"),
        web_search_model: table_str(models, "web_search"),
        subagents_default_model: table_str(subagents, "default_model"),
        models: read_models(&doc),
        created_at: None,
        updated_at: None,
        is_active: false,
    };
    Ok(profile.normalize())
}

pub fn apply_profile_to_file(path: &Path, profile: &GrokProfile) -> Result<(), String> {
    let data = if path.exists() {
        fs::read(path).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    let next = apply_profile_text(&data, profile)?;
    atomic_write(path, &next)
}

/// 清除供应商拥有的 API 覆盖，使 Grok 回退官方 OAuth（auth.json）。
pub fn use_official_auth_to_file(path: &Path) -> Result<(), String> {
    let data = if path.exists() {
        fs::read(path).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    let next = use_official_auth_text(&data);
    atomic_write(path, &next)
}

pub fn use_official_auth_text(data: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(data)
        .trim_start_matches('\u{feff}')
        .to_string();
    let lines = split_lines(&text);
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let header = parse_header(&lines[i]);
        if header.is_empty() {
            out.push(lines[i].clone());
            i += 1;
            continue;
        }
        if header == "model" || header.starts_with("model.") {
            i = skip_section(&lines, i + 1);
            continue;
        }
        let end = skip_section(&lines, i + 1);
        match header.as_str() {
            "endpoints" => {
                out.extend(remove_assignments(&lines[i..end], &["models_base_url"]));
            }
            "models" => {
                out.extend(remove_assignments(&lines[i..end], &["default", "web_search"]));
            }
            "subagents" => {
                out.extend(remove_assignments(&lines[i..end], &["default_model"]));
            }
            _ => {
                out.extend(lines[i..end].iter().cloned());
            }
        }
        i = end;
    }
    let mut result = out.join("\n");
    result = result.trim_end_matches('\n').to_string();
    if result.is_empty() {
        return Vec::new();
    }
    result.push('\n');
    result.into_bytes()
}

/// 合并本地隐私保护键，其它段尽量保留。
pub fn apply_privacy_protection_to_file(path: &Path) -> Result<(), String> {
    let data = if path.exists() {
        fs::read(path).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    let next = apply_privacy_protection_text(&data);
    atomic_write(path, &next)
}

pub fn apply_privacy_protection_text(data: &[u8]) -> Vec<u8> {
    // 固定隐私键清单（与 design / grok-build-switch v0.2.0 对齐）
    let settings: &[(&str, &[(&str, &str)])] = &[
        ("features", &[("telemetry", "false")]),
        (
            "telemetry",
            &[("trace_upload", "false"), ("mixpanel_enabled", "false")],
        ),
        ("harness", &[("disable_codebase_upload", "true")]),
    ];
    let text = String::from_utf8_lossy(data)
        .trim_start_matches('\u{feff}')
        .to_string();
    let lines = split_lines(&text);
    let mut out: Vec<String> = Vec::new();
    let mut seen = HashMap::new();
    let mut i = 0;
    while i < lines.len() {
        let header = parse_header(&lines[i]);
        if header.is_empty() {
            out.push(lines[i].clone());
            i += 1;
            continue;
        }
        let end = skip_section(&lines, i + 1);
        if let Some((_, values)) = settings.iter().find(|(section, _)| *section == header) {
            let map: HashMap<&str, &str> = values.iter().copied().collect();
            out.extend(rewrite_values(&lines[i..end], &map));
            seen.insert(header.clone(), true);
        } else {
            out.extend(lines[i..end].iter().cloned());
        }
        i = end;
    }
    for (section, values) in settings {
        if seen.contains_key(*section) {
            continue;
        }
        if out.last().map(|s| !s.trim().is_empty()).unwrap_or(false) {
            out.push(String::new());
        }
        let map: HashMap<&str, &str> = values.iter().copied().collect();
        out.extend(rewrite_values(&[format!("[{section}]")], &map));
    }
    let mut result = out.join("\n");
    result = result.trim_end_matches('\n').to_string();
    result.push('\n');
    result.into_bytes()
}

fn remove_assignments(lines: &[String], keys: &[&str]) -> Vec<String> {
    let removed: HashMap<&str, bool> = keys.iter().map(|k| (*k, true)).collect();
    lines
        .iter()
        .filter(|line| !removed.contains_key(assignment_key(line).as_str()))
        .cloned()
        .collect()
}

fn rewrite_values(lines: &[String], values: &HashMap<&str, &str>) -> Vec<String> {
    let mut seen = HashMap::new();
    let mut out = Vec::new();
    if lines.is_empty() {
        return out;
    }
    out.push(lines[0].clone());
    for line in lines.iter().skip(1) {
        let key = assignment_key(line);
        if let Some(val) = values.get(key.as_str()) {
            out.push(format!("{key} = {val}"));
            seen.insert(key, true);
            continue;
        }
        out.push(line.clone());
    }
    let mut missing: Vec<_> = values
        .keys()
        .copied()
        .filter(|k| !seen.contains_key(*k))
        .collect();
    missing.sort();
    for key in missing {
        out.push(format!("{key} = {}", values[key]));
    }
    out
}

pub fn current_matches(path: &Path, profile: &GrokProfile) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }
    let current = import_profile(path, &profile.name)?;
    Ok(profile.matches_config(&current))
}

fn read_models(doc: &toml::Value) -> Vec<GrokModelDef> {
    let Some(table) = doc.get("model").and_then(|v| v.as_table()) else {
        return Vec::new();
    };
    let mut keys: Vec<_> = table.keys().cloned().collect();
    keys.sort();
    keys.into_iter()
        .filter_map(|key| {
            let entry = table.get(&key)?.as_table()?;
            Some(GrokModelDef {
                name: key,
                model: table_str(Some(entry), "model"),
                base_url: table_str(Some(entry), "base_url"),
                api_key: table_str(Some(entry), "api_key"),
                api_backend: table_str(Some(entry), "api_backend"),
                extra_headers: table_string_map(entry, "extra_headers"),
                supports_backend_search: table_bool(entry, "supports_backend_search"),
                context_window: table_i64(entry, "context_window"),
                max_completion_tokens: table_i64(entry, "max_completion_tokens"),
            })
        })
        .collect()
}

fn table_str(table: Option<&toml::map::Map<String, toml::Value>>, key: &str) -> String {
    table
        .and_then(|t| t.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn table_bool(table: &toml::map::Map<String, toml::Value>, key: &str) -> bool {
    table.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn table_i64(table: &toml::map::Map<String, toml::Value>, key: &str) -> i64 {
    table
        .get(key)
        .and_then(|v| v.as_integer())
        .unwrap_or(0)
}

fn table_string_map(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Some(map) = table.get(key).and_then(|v| v.as_table()) else {
        return out;
    };
    for (k, v) in map {
        if let Some(s) = v.as_str() {
            out.insert(k.clone(), s.to_string());
        }
    }
    out
}

/// 行级重写：替换 endpoints/models/subagents 与 model.* 段，其它段原样保留。
pub fn apply_profile_text(data: &[u8], profile: &GrokProfile) -> Result<Vec<u8>, String> {
    let profile = profile.clone().normalize();
    let text = String::from_utf8_lossy(data).trim_start_matches('\u{feff}').to_string();
    let lines = split_lines(&text);
    let mut out: Vec<String> = Vec::new();
    let mut seen = HashMap::new();
    let mut i = 0;
    while i < lines.len() {
        let header = parse_header(&lines[i]);
        if header.is_empty() {
            out.push(lines[i].clone());
            i += 1;
            continue;
        }
        if header == "model" || header.starts_with("model.") {
            i = skip_section(&lines, i + 1);
            continue;
        }
        if matches!(header.as_str(), "endpoints" | "models" | "subagents") {
            let end = skip_section(&lines, i + 1);
            out.extend(rewrite_section(&lines[i..end], &header, &profile));
            seen.insert(header.clone(), true);
            i = end;
            continue;
        }
        let end = skip_section(&lines, i + 1);
        out.extend(lines[i..end].iter().cloned());
        i = end;
    }
    for section in ["endpoints", "models", "subagents"] {
        if !seen.contains_key(section) {
            if out.last().map(|s| !s.trim().is_empty()).unwrap_or(false) {
                out.push(String::new());
            }
            out.extend(rewrite_section(&[format!("[{section}]")], section, &profile));
        }
    }
    if out.last().map(|s| !s.trim().is_empty()).unwrap_or(false) {
        out.push(String::new());
    }
    out.push(marshal_model_section(&profile)?);
    let mut result = out.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }
    Ok(result.into_bytes())
}

fn marshal_model_section(profile: &GrokProfile) -> Result<String, String> {
    let mut parts = Vec::new();
    let effective_key = profile.effective_api_key();
    for model in &profile.models {
        let key = if model.name.is_empty() {
            model.model.as_str()
        } else {
            model.name.as_str()
        };
        if key.is_empty() {
            continue;
        }
        let api_key = if model.api_key.is_empty() {
            effective_key.as_str()
        } else {
            model.api_key.as_str()
        };
        let backend = if model.api_backend.is_empty() {
            api_backend_for_upstream(&profile.upstream_format)
        } else {
            model.api_backend.clone()
        };
        let mut body = vec![
            format!("[model.{}]", quote_key(key)),
            format!("model = {}", toml_quote(&model.model)),
            format!("api_key = {}", toml_quote(api_key)),
            format!("api_backend = {}", toml_quote(&backend)),
            format!(
                "supports_backend_search = {}",
                model.supports_backend_search
            ),
        ];
        if model.context_window > 0 {
            body.push(format!("context_window = {}", model.context_window));
        }
        if model.max_completion_tokens > 0 {
            body.push(format!(
                "max_completion_tokens = {}",
                model.max_completion_tokens
            ));
        }
        if !model.base_url.is_empty() {
            body.push(format!("base_url = {}", toml_quote(&model.base_url)));
        }
        parts.push(body.join("\n"));
    }
    Ok(parts.join("\n\n"))
}

fn rewrite_section(lines: &[String], section: &str, profile: &GrokProfile) -> Vec<String> {
    let mut values = HashMap::new();
    match section {
        "endpoints" => {
            values.insert("models_base_url", toml_quote(&profile.base_url));
        }
        "models" => {
            values.insert("default", toml_quote(&profile.default_model));
            values.insert("web_search", toml_quote(&profile.web_search_model));
        }
        "subagents" => {
            values.insert(
                "default_model",
                toml_quote(&profile.subagents_default_model),
            );
        }
        _ => {}
    }
    let mut seen = HashMap::new();
    let mut out = Vec::new();
    if lines.is_empty() {
        out.push(format!("[{section}]"));
    } else {
        out.push(lines[0].clone());
    }
    for line in lines.iter().skip(1) {
        let key = assignment_key(line);
        if let Some(val) = values.get(key.as_str()) {
            out.push(format!("{key} = {val}"));
            seen.insert(key, true);
            continue;
        }
        out.push(line.clone());
    }
    let mut missing: Vec<_> = values
        .keys()
        .copied()
        .filter(|k| !seen.contains_key(*k))
        .collect();
    missing.sort();
    for key in missing {
        out.push(format!("{key} = {}", values[key]));
    }
    out
}

fn split_lines(text: &str) -> Vec<String> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let text = text.trim_end_matches('\n');
    if text.is_empty() {
        return Vec::new();
    }
    text.split('\n').map(str::to_string).collect()
}

fn parse_header(line: &str) -> String {
    let trimmed = line.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return String::new();
    }
    trimmed
        .trim_matches(|c| c == '[' || c == ']')
        .trim()
        .to_string()
}

fn skip_section(lines: &[String], mut start: usize) -> usize {
    while start < lines.len() {
        if !parse_header(&lines[start]).is_empty() {
            return start;
        }
        start += 1;
    }
    start
}

fn assignment_key(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return String::new();
    }
    let Some(idx) = trimmed.find('=') else {
        return String::new();
    };
    trimmed[..idx].trim().to_string()
}

fn toml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn quote_key(key: &str) -> String {
    if key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        key.to_string()
    } else {
        format!("\"{}\"", key.replace('"', "\\\""))
    }
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, data).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        apply_privacy_protection_text, apply_profile_text, import_profile, use_official_auth_text,
    };
    use crate::grok_provider::profile::{GrokModelDef, GrokProfile};
    use std::fs;

    #[test]
    fn use_official_auth_removes_provider_overrides_keeps_other() {
        let input = r#"[cli]
show_tips = false

[endpoints]
models_base_url = "http://127.0.0.1:9/v1"
other_url = "keep"

[models]
default = "m"
web_search = "m"
default_reasoning_effort = "high"

[subagents]
default_model = "m"

[model.m]
model = "m"
api_key = "k"
"#;
        let out = String::from_utf8(use_official_auth_text(input.as_bytes())).unwrap();
        assert!(!out.contains("models_base_url"));
        assert!(!out.contains("default = \"m\""));
        assert!(!out.contains("web_search"));
        assert!(!out.contains("default_model"));
        assert!(!out.contains("[model.m]"));
        assert!(out.contains("[cli]"));
        assert!(out.contains("show_tips = false"));
        assert!(out.contains("other_url = \"keep\""));
        assert!(out.contains("default_reasoning_effort = \"high\""));
    }

    #[test]
    fn use_official_auth_empty_input_ok() {
        let out = use_official_auth_text(b"");
        assert!(out.is_empty() || out == b"\n" || String::from_utf8_lossy(&out).trim().is_empty());
    }

    #[test]
    fn privacy_merges_keys_preserves_other() {
        let input = r#"[cli]
show_tips = true

[features]
telemetry = true
other = 1
"#;
        let out = String::from_utf8(apply_privacy_protection_text(input.as_bytes())).unwrap();
        assert!(out.contains("[cli]"));
        assert!(out.contains("show_tips = true"));
        assert!(out.contains("telemetry = false"));
        assert!(out.contains("other = 1"));
        assert!(out.contains("trace_upload = false"));
        assert!(out.contains("mixpanel_enabled = false"));
        assert!(out.contains("disable_codebase_upload = true"));
    }

    #[test]
    fn privacy_on_empty_creates_sections() {
        let out = String::from_utf8(apply_privacy_protection_text(b"")).unwrap();
        assert!(out.contains("[features]"));
        assert!(out.contains("telemetry = false"));
        assert!(out.contains("[telemetry]"));
        assert!(out.contains("[harness]"));
    }

    #[test]
    fn apply_rewrites_endpoints_and_models_preserves_other() {
        let input = r#"[cli]
show_tips = false

[models]
default = "old"
default_reasoning_effort = "high"

[ui]
yolo = false
"#;
        let profile = GrokProfile {
            id: "p1".into(),
            name: "Test".into(),
            upstream_format: "openai_chat".into(),
            base_url: "http://127.0.0.1:8317/v1".into(),
            api_key: "secret".into(),
            available_models: vec![],
            default_model: "grok-4.5".into(),
            web_search_model: "grok-4.5".into(),
            subagents_default_model: "grok-4.5".into(),
            models: vec![GrokModelDef {
                name: "cpa".into(),
                model: "grok-4.5".into(),
                base_url: "http://127.0.0.1:8317/v1".into(),
                api_key: "secret".into(),
                api_backend: "responses".into(),
                ..Default::default()
            }],
            created_at: None,
            updated_at: None,
            is_active: false,
        };
        let out = String::from_utf8(apply_profile_text(input.as_bytes(), &profile).unwrap()).unwrap();
        assert!(out.contains("models_base_url = \"http://127.0.0.1:8317/v1\""));
        assert!(out.contains("default = \"grok-4.5\""));
        assert!(out.contains("[ui]"));
        assert!(out.contains("yolo = false"));
        assert!(out.contains("[model.cpa]"));
        assert!(out.contains("api_key = \"secret\""));
        assert!(out.contains("default_reasoning_effort = \"high\""));
    }

    #[test]
    fn import_reads_fixture() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.toml");
        fs::write(
            &path,
            r#"[endpoints]
models_base_url = "http://example.com/v1"

[models]
default = "m1"
web_search = "m2"

[subagents]
default_model = "m3"

[model.m1]
model = "m1"
api_key = "k"
api_backend = "chat_completions"
"#,
        )
        .unwrap();
        let p = import_profile(&path, "Imported").unwrap();
        assert_eq!(p.base_url, "http://example.com/v1");
        assert_eq!(p.default_model, "m1");
        assert_eq!(p.models.len(), 1);
        assert_eq!(p.models[0].api_key, "k");
    }
}
