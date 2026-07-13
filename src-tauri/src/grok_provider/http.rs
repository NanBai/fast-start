//! 用户触发的 Grok 上游 HTTP（拉模型 / 连通测试）。
//! 硬约束：仅 http/https、超时 ≤10s、响应体 ≤2MiB、不 log apiKey。

use serde::Deserialize;
use serde_json::Value;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(10);
const MAX_BODY_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokFetchModelsResult {
    pub models: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokTestConnectionResult {
    pub ok: bool,
    pub latency_ms: u64,
    pub message: String,
}

/// 校验 scheme 与 upstream_format；返回规范化 base（去尾 `/`）。
pub fn normalize_base_url(base_url: &str, upstream_format: &str) -> Result<String, String> {
    validate_upstream_format(upstream_format)?;
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        return Err("Base URL 不能为空".to_string());
    }
    let lower = trimmed.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err("Base URL 仅支持 http 或 https".to_string());
    }
    // 粗略拒绝含空白（防注入奇怪 URL）
    if trimmed.chars().any(|c| c.is_whitespace()) {
        return Err("Base URL 不能包含空白字符".to_string());
    }
    Ok(trimmed.trim_end_matches('/').to_string())
}

pub fn models_url(base_url: &str) -> String {
    format!("{base_url}/models")
}

pub fn validate_upstream_format(upstream_format: &str) -> Result<(), String> {
    let f = if upstream_format.is_empty()
        || upstream_format == "openai"
        || upstream_format == "grok"
    {
        "openai_chat"
    } else {
        upstream_format
    };
    match f {
        "openai_chat" | "openai_responses" | "anthropic_messages" => Ok(()),
        _ => Err(format!("不支持的 upstreamFormat: {upstream_format}")),
    }
}

/// 解析 OpenAI 兼容 `/models` JSON：`{ "data": [ { "id": "..." } ] }` 或字符串数组。
pub fn parse_models_json(body: &str) -> Result<Vec<String>, String> {
    let value: Value =
        serde_json::from_str(body).map_err(|e| format!("解析模型列表失败: {e}"))?;
    if let Some(arr) = value.as_array() {
        return Ok(arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect());
    }
    let data = value
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "模型列表格式无法识别（缺少 data 数组）".to_string())?;
    let mut models = Vec::new();
    for item in data {
        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
            if !id.is_empty() {
                models.push(id.to_string());
            }
        } else if let Some(s) = item.as_str() {
            if !s.is_empty() {
                models.push(s.to_string());
            }
        }
    }
    Ok(models)
}

pub fn fetch_models(
    base_url: &str,
    api_key: &str,
    upstream_format: &str,
) -> Result<GrokFetchModelsResult, String> {
    let base = normalize_base_url(base_url, upstream_format)?;
    let url = models_url(&base);
    let body = http_get_limited(&url, api_key)?;
    let models = parse_models_json(&body)?;
    Ok(GrokFetchModelsResult { models })
}

pub fn test_connection(
    base_url: &str,
    api_key: &str,
    upstream_format: &str,
) -> Result<GrokTestConnectionResult, String> {
    let base = normalize_base_url(base_url, upstream_format)?;
    let url = models_url(&base);
    let started = Instant::now();
    match http_get_limited(&url, api_key) {
        Ok(body) => {
            let latency_ms = started.elapsed().as_millis() as u64;
            // 连通成功：能读到响应即可；若可解析模型列表则附带数量
            let extra = parse_models_json(&body)
                .map(|m| format!("，识别到 {} 个模型", m.len()))
                .unwrap_or_default();
            Ok(GrokTestConnectionResult {
                ok: true,
                latency_ms,
                message: format!("连通成功（{latency_ms}ms）{extra}"),
            })
        }
        Err(err) => {
            let latency_ms = started.elapsed().as_millis() as u64;
            Err(format!("连通失败（{latency_ms}ms）：{err}"))
        }
    }
}

fn http_get_limited(url: &str, api_key: &str) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(TIMEOUT)
        .timeout_read(TIMEOUT)
        .timeout_write(TIMEOUT)
        .redirects(3)
        .build();

    let mut req = agent.get(url);
    if !api_key.trim().is_empty() {
        req = req.set("Authorization", &format!("Bearer {}", api_key.trim()));
    }
    req = req.set("Accept", "application/json");

    let response = req.call().map_err(|err| map_ureq_error(err))?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(format!("HTTP {status}"));
    }
    let body = response
        .into_string()
        .map_err(|e| format!("读取响应失败: {e}"))?;
    if body.len() as u64 > MAX_BODY_BYTES {
        return Err(format!(
            "响应体超过上限 {} 字节",
            MAX_BODY_BYTES
        ));
    }
    Ok(body)
}

fn map_ureq_error(err: ureq::Error) -> String {
    // 刻意不包含请求头/密钥
    match err {
        ureq::Error::Status(code, _) => format!("HTTP {code}"),
        ureq::Error::Transport(t) => format!("网络错误: {t}"),
    }
}

// 保留 Deserialize 以免将来扩展 response 结构时编译噪音
#[allow(dead_code)]
#[derive(Deserialize)]
struct OpenAiModelList {
    data: Vec<OpenAiModel>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct OpenAiModel {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::{
        models_url, normalize_base_url, parse_models_json, validate_upstream_format,
    };

    #[test]
    fn rejects_non_http_scheme() {
        assert!(normalize_base_url("ftp://x", "openai_chat").is_err());
        assert!(normalize_base_url("file:///tmp", "openai_chat").is_err());
    }

    #[test]
    fn accepts_http_https_and_strips_slash() {
        assert_eq!(
            normalize_base_url("https://api.example.com/v1/", "openai_chat").unwrap(),
            "https://api.example.com/v1"
        );
        assert_eq!(
            models_url("https://api.example.com/v1"),
            "https://api.example.com/v1/models"
        );
    }

    #[test]
    fn rejects_unknown_upstream() {
        assert!(validate_upstream_format("weird").is_err());
        assert!(validate_upstream_format("openai_chat").is_ok());
        assert!(validate_upstream_format("").is_ok());
    }

    #[test]
    fn parses_openai_models_payload() {
        let body = r#"{"data":[{"id":"a"},{"id":"b"},{"id":""}]}"#;
        let models = parse_models_json(body).unwrap();
        assert_eq!(models, vec!["a", "b"]);
    }

    #[test]
    fn parse_models_rejects_garbage() {
        assert!(parse_models_json("not-json").is_err());
        assert!(parse_models_json(r#"{"foo":1}"#).is_err());
    }
}
