use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};

use crate::config::Config;
use crate::crypto;

pub struct ScreenshotContext<'a> {
    pub base64_png: &'a str,
    pub width: u32,
    pub height: u32,
}

pub async fn plan_actions(task: &str, config: &Config) -> Result<String, String> {
    plan_actions_internal(task, config, None).await
}

pub async fn plan_actions_with_screenshot(
    task: &str,
    config: &Config,
    screenshot: ScreenshotContext<'_>,
) -> Result<String, String> {
    plan_actions_internal(task, config, Some(&screenshot)).await
}

async fn plan_actions_internal(
    task: &str,
    config: &Config,
    screenshot: Option<&ScreenshotContext<'_>>,
) -> Result<String, String> {
    let provider = config.llm_provider.as_str();
    match provider {
        "openai" => {
            chat_openai_compatible(
                "https://api.openai.com/v1",
                &crypto::decrypt(&config.openai_api_key),
                &config.openai_model,
                task,
                config,
                screenshot,
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
                screenshot,
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
                screenshot,
            )
            .await
        }
        "anthropic" => {
            chat_anthropic(
                &crypto::decrypt(&config.anthropic_api_key),
                &config.anthropic_model,
                task,
                config,
                screenshot,
            )
            .await
        }
        "ollama" => {
            chat_ollama(
                &config.ollama_base_url,
                &config.ollama_model,
                task,
                config,
                screenshot,
            )
            .await
        }
        other => Err(format!("Unsupported LLM provider: {other}")),
    }
}

fn system_prompt(config: &Config) -> String {
    let lang = config.language.as_str();
    let shell_action = if config.shell_enabled {
        r#"- {"type":"shell","command":"...","timeout_secs":30}"#
    } else {
        "- Shell command execution is disabled. Do not output shell actions."
    };
    let open_action = if config.shell_enabled {
        r#"- {"type":"open","target":"..."}"#
    } else {
        "- OS launcher/open is disabled while shell is off. Do not output open actions."
    };
    let shell_rule = if config.shell_enabled {
        "- Shell/open launcher actions are enabled; prefer them only when they are the safest direct route."
    } else {
        "- Shell/open launcher actions are disabled; operate through the GUI using click/type/hotkey/scroll/screenshot/wait/complete."
    };
    let shell_state = if config.shell_enabled {
        "enabled"
    } else {
        "disabled"
    };
    let supported = format!(
        r#"You can output only a JSON action plan.
Runtime capability state:
- shell: {shell_state}
- open_launcher: {shell_state}

Supported actions:
{shell_action}
- {{"type":"wait","seconds":1.0}}
{open_action}
- {{"type":"click","x":100,"y":200,"button":"left","clicks":1}}
- {{"type":"type","text":"..."}}
- {{"type":"hotkey","keys":["ctrl","c"]}}
- {{"type":"scroll","clicks":-400,"axis":"vertical"}}
- {{"type":"screenshot"}}
- {{"type":"complete","result":"final answer"}}

Return either:
1. {{"actions":[ ... ]}}
2. [ ... ]
3. a single action object

Rules:
{shell_rule}
- If the user prompt says execution mode is step-by-step, return exactly one next action.
- If the user prompt says execution mode is adaptive or batch, choose the mode yourself:
  - Return multiple actions when the next steps are already certain and do not depend on observing intermediate results.
  - Return exactly one action when the next step depends on what happens after the current action.
- Do not batch uncertain GUI actions.
- In a batch plan, complete may appear only as the final action.
- If a screenshot is provided, use its visible screen coordinates to plan GUI actions.
- Do not answer that you need to view the screen when a screenshot has already been provided.
- Use screenshot if visual state matters and no screenshot was provided.
- When shell/open are disabled and the task requires app interaction, output click/type/hotkey/scroll actions.
- Return complete only when the whole task is finished.
- Return JSON only, with no markdown fences."#
    );

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

fn openai_user_content(task: &str, screenshot: Option<&ScreenshotContext<'_>>) -> Value {
    if let Some(screenshot) = screenshot {
        json!([
            {
                "type": "text",
                "text": vision_task_text(task, screenshot)
            },
            {
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/png;base64,{}", screenshot.base64_png),
                    "detail": "high"
                }
            }
        ])
    } else {
        json!(task)
    }
}

fn anthropic_user_content(task: &str, screenshot: Option<&ScreenshotContext<'_>>) -> Value {
    if let Some(screenshot) = screenshot {
        json!([
            {
                "type": "text",
                "text": vision_task_text(task, screenshot)
            },
            {
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": "image/png",
                    "data": screenshot.base64_png
                }
            }
        ])
    } else {
        json!(task)
    }
}

fn vision_task_text(task: &str, screenshot: &ScreenshotContext<'_>) -> String {
    format!(
        "Task: {task}\n\nCurrent screenshot is attached. Screenshot size: {}x{} pixels. Use absolute screen coordinates from this screenshot for click actions. If the task involves typing Chinese text, output a type action with the exact Chinese text.",
        screenshot.width, screenshot.height
    )
}

async fn chat_openai_compatible(
    base_url: &str,
    api_key: &str,
    model: &str,
    task: &str,
    config: &Config,
    screenshot: Option<&ScreenshotContext<'_>>,
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
            { "role": "user", "content": openai_user_content(task, screenshot) }
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
    screenshot: Option<&ScreenshotContext<'_>>,
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
                "content": anthropic_user_content(task, screenshot)
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
    screenshot: Option<&ScreenshotContext<'_>>,
) -> Result<String, String> {
    let endpoint = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let user_message = if let Some(screenshot) = screenshot {
        json!({
            "role": "user",
            "content": vision_task_text(task, screenshot),
            "images": [screenshot.base64_png]
        })
    } else {
        json!({ "role": "user", "content": task })
    };
    let body = json!({
        "model": model,
        "stream": false,
        "messages": [
            { "role": "system", "content": system_prompt(config) },
            user_message
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
