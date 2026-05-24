use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};

use crate::config::Config;
use crate::crypto;

pub async fn plan_actions(task: &str, config: &Config) -> Result<String, String> {
    let provider = config.llm_provider.as_str();
    match provider {
        "openai" => {
            chat_openai_compatible(
                "https://api.openai.com/v1",
                &crypto::decrypt(&config.openai_api_key),
                &config.openai_model,
                task,
                config,
            )
            .await
        }
        "custom" => {
            chat_openai_compatible(
                &config.custom_base_url,
                &crypto::decrypt(&config.custom_api_key),
                &config.custom_model,
                task,
                config,
            )
            .await
        }
        "gemini" => {
            chat_openai_compatible(
                "https://generativelanguage.googleapis.com/v1beta/openai/",
                &crypto::decrypt(&config.gemini_api_key),
                &config.gemini_model,
                task,
                config,
            )
            .await
        }
        "anthropic" => {
            chat_anthropic(
                &crypto::decrypt(&config.anthropic_api_key),
                &config.anthropic_model,
                task,
                config,
            )
            .await
        }
        "ollama" => chat_ollama(&config.ollama_base_url, &config.ollama_model, task, config).await,
        other => Err(format!("Unsupported LLM provider: {other}")),
    }
}

fn system_prompt(config: &Config) -> String {
    let lang = config.language.as_str();
    let supported = r#"You can output only a JSON action plan.
Supported actions:
- {"type":"shell","command":"...","timeout_secs":30}
- {"type":"wait","seconds":1.0}
- {"type":"open","target":"..."}
- {"type":"click","x":100,"y":200,"button":"left","clicks":1}
- {"type":"type","text":"..."}
- {"type":"hotkey","keys":["ctrl","c"]}
- {"type":"scroll","clicks":-400,"axis":"vertical"}
- {"type":"screenshot"}
- {"type":"complete","result":"final answer"}

Return either:
1. {"actions":[ ... ]}
2. [ ... ]
3. a single action object

Rules:
- Prefer shell/open when possible.
- Use screenshot if visual state matters.
- End the plan with complete.
- Return JSON only, with no markdown fences."#;

    if lang == "zh" {
        format!(
            "你是 EyeForge 的 Rust 原生任务规划器。请把用户任务转换成 JSON 动作计划。\n{supported}"
        )
    } else {
        format!(
            "You are the Rust-native EyeForge task planner. Convert the user request into a JSON action plan.\n{supported}"
        )
    }
}

async fn chat_openai_compatible(
    base_url: &str,
    api_key: &str,
    model: &str,
    task: &str,
    config: &Config,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("Missing API key for OpenAI-compatible provider".into());
    }

    let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(|e| e.to_string())?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let body = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt(config) },
            { "role": "user", "content": task }
        ],
        "temperature": 0.1,
        "max_tokens": 1200
    });

    let response: Value = reqwest::Client::new()
        .post(endpoint)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    response["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "Invalid OpenAI-compatible response".into())
}

async fn chat_anthropic(
    api_key: &str,
    model: &str,
    task: &str,
    config: &Config,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("Missing API key for Anthropic".into());
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        "x-api-key",
        HeaderValue::from_str(api_key).map_err(|e| e.to_string())?,
    );
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let body = json!({
        "model": model,
        "max_tokens": 1200,
        "system": system_prompt(config),
        "messages": [
            {
                "role": "user",
                "content": task
            }
        ]
    });

    let response: Value = reqwest::Client::new()
        .post("https://api.anthropic.com/v1/messages")
        .headers(headers)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    response["content"][0]["text"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "Invalid Anthropic response".into())
}

async fn chat_ollama(
    base_url: &str,
    model: &str,
    task: &str,
    config: &Config,
) -> Result<String, String> {
    let endpoint = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let body = json!({
        "model": model,
        "stream": false,
        "messages": [
            { "role": "system", "content": system_prompt(config) },
            { "role": "user", "content": task }
        ]
    });

    let response: Value = reqwest::Client::new()
        .post(endpoint)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    response["message"]["content"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "Invalid Ollama response".into())
}
