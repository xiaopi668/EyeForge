use std::sync::{Mutex, OnceLock};

use serde::Serialize;
use serde_json::{json, Value};
use tokio::time::{timeout, Duration};

use crate::config::Config;

#[derive(Debug, Clone, Serialize)]
struct HapiMember {
    kind: &'static str,
    name: String,
    role: String,
    endpoint: String,
}

#[derive(Debug, Clone)]
struct AiGroupState {
    hapi_endpoint: String,
    strategy: String,
    members: Vec<HapiMember>,
}

impl AiGroupState {
    fn summary(&self) -> String {
        let members = self
            .members
            .iter()
            .map(|member| {
                format!(
                    "{}:{}({})@{}",
                    member.kind, member.name, member.role, member.endpoint
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "AI groups hapi bridge ready at {}; strategy {}; members: {}",
            self.hapi_endpoint, self.strategy, members
        )
    }
}

static AI_GROUPS: OnceLock<Mutex<Option<AiGroupState>>> = OnceLock::new();

fn state() -> &'static Mutex<Option<AiGroupState>> {
    AI_GROUPS.get_or_init(|| Mutex::new(None))
}

pub fn restart(config: &Config) -> Result<String, String> {
    stop();

    if !config.ai_groups_enabled {
        return Ok("AI groups disabled".into());
    }

    let next_state = build_state(config)?;
    let message = next_state.summary();

    *state()
        .lock()
        .map_err(|_| "AI group state poisoned".to_string())? = Some(next_state);

    Ok(message)
}

pub async fn dispatch_task(config: &Config, task: &str) -> Result<Option<String>, String> {
    if !config.ai_groups_enabled {
        return Ok(None);
    }

    let group = build_state(config)?;
    let body = json!({
        "task": task,
        "strategy": group.strategy,
        "members": group.members,
        "response_format": "eyeforge_action_plan",
    });

    let mut errors = Vec::new();
    for path in ["/v1/ai-groups/dispatch", "/ai-groups/dispatch", "/dispatch"] {
        match post_hapi_json(&group.hapi_endpoint, path, &body).await {
            Ok(Some(plan)) => return Ok(Some(plan)),
            Ok(None) => errors.push(format!(
                "{} returned no usable content",
                join_url(&group.hapi_endpoint, path)
            )),
            Err(error) => errors.push(error),
        }
    }

    match group.strategy.trim().to_ascii_lowercase().as_str() {
        "broadcast" => dispatch_broadcast(&group, task, &mut errors).await,
        "fallback" => dispatch_fallback(&group, task, &mut errors).await,
        _ => dispatch_primary(&group, task, &mut errors).await,
    }
}

pub fn stop() {
    if let Ok(mut guard) = state().lock() {
        *guard = None;
    }
}

fn build_state(config: &Config) -> Result<AiGroupState, String> {
    let hapi_endpoint = config.ai_group_hapi_endpoint.trim().to_string();
    if hapi_endpoint.is_empty() {
        return Err("HAPI endpoint is required for AI groups".into());
    }

    let mut members = Vec::new();
    members.extend(parse_members(
        "openclaw",
        &config.ai_group_openclaw_members,
    )?);
    members.extend(parse_members("astrbot", &config.ai_group_astrbot_members)?);
    members.extend(parse_members(
        "opencode",
        &config.ai_group_opencode_members,
    )?);
    members.extend(parse_members("codex", &config.ai_group_codex_members)?);
    members.extend(parse_members(
        "claude_code",
        &config.ai_group_claude_code_members,
    )?);

    if members.is_empty() {
        return Err("AI groups enabled but no members are configured".into());
    }

    Ok(AiGroupState {
        hapi_endpoint,
        strategy: config.ai_group_strategy.clone(),
        members,
    })
}

async fn dispatch_primary(
    group: &AiGroupState,
    task: &str,
    errors: &mut Vec<String>,
) -> Result<Option<String>, String> {
    if let Some(member) = group.members.first() {
        return dispatch_member(member, task, &group.strategy, errors).await;
    }

    Err(format_dispatch_errors(errors))
}

async fn dispatch_fallback(
    group: &AiGroupState,
    task: &str,
    errors: &mut Vec<String>,
) -> Result<Option<String>, String> {
    for member in &group.members {
        if let Some(plan) = dispatch_member(member, task, &group.strategy, errors).await? {
            return Ok(Some(plan));
        }
    }

    Err(format_dispatch_errors(errors))
}

async fn dispatch_broadcast(
    group: &AiGroupState,
    task: &str,
    errors: &mut Vec<String>,
) -> Result<Option<String>, String> {
    let mut replies = Vec::new();
    for member in &group.members {
        if let Some(plan) = dispatch_member(member, task, &group.strategy, errors).await? {
            replies.push(json!({
                "member": member.name,
                "kind": member.kind,
                "role": member.role,
                "reply": plan,
            }));
        }
    }

    if replies.is_empty() {
        return Err(format_dispatch_errors(errors));
    }

    if replies.len() == 1 {
        return Ok(replies[0]["reply"].as_str().map(str::to_string));
    }

    Ok(Some(json!({
        "actions": [{
            "type": "complete",
            "result": serde_json::to_string_pretty(&replies).unwrap_or_else(|_| "AI group broadcast completed".into())
        }]
    }).to_string()))
}

async fn dispatch_member(
    member: &HapiMember,
    task: &str,
    strategy: &str,
    errors: &mut Vec<String>,
) -> Result<Option<String>, String> {
    let body = json!({
        "task": task,
        "strategy": strategy,
        "member": member,
        "response_format": "eyeforge_action_plan",
    });

    match post_hapi_json(&member.endpoint, "", &body).await {
        Ok(plan) => Ok(plan),
        Err(error) => {
            errors.push(error);
            Ok(None)
        }
    }
}

async fn post_hapi_json(
    base_url: &str,
    path: &str,
    body: &Value,
) -> Result<Option<String>, String> {
    let endpoint = join_url(base_url, path);
    let request = reqwest::Client::new().post(&endpoint).json(body).send();
    let response = timeout(Duration::from_secs(60), request)
        .await
        .map_err(|_| format!("HAPI request timed out: {endpoint}"))?
        .map_err(|error| format!("HAPI request failed at {endpoint}: {error}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("Failed to read HAPI response from {endpoint}: {error}"))?;

    if !status.is_success() {
        return Err(format!("HAPI {endpoint} returned {status}: {text}"));
    }

    Ok(extract_action_plan(&text))
}

fn extract_action_plan(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            return extract_action_plan_value(&value).or_else(|| Some(trimmed.to_string()));
        }
    }

    Some(
        json!({
            "actions": [{
                "type": "complete",
                "result": trimmed,
            }]
        })
        .to_string(),
    )
}

fn extract_action_plan_value(value: &Value) -> Option<String> {
    for key in [
        "action_plan",
        "plan",
        "actions",
        "content",
        "message",
        "result",
        "text",
    ] {
        if let Some(candidate) = value.get(key) {
            if key == "actions" {
                return Some(json!({ "actions": candidate }).to_string());
            }
            if let Some(text) = candidate.as_str() {
                return extract_action_plan(text);
            }
            if candidate.is_object() || candidate.is_array() {
                return Some(candidate.to_string());
            }
        }
    }

    None
}

fn join_url(base_url: &str, path: &str) -> String {
    if path.is_empty() {
        base_url.trim_end_matches('/').to_string()
    } else {
        format!("{}{}", base_url.trim_end_matches('/'), path)
    }
}

fn format_dispatch_errors(errors: &[String]) -> String {
    if errors.is_empty() {
        "AI group hapi dispatch returned no usable response".into()
    } else {
        format!("AI group hapi dispatch failed: {}", errors.join(" | "))
    }
}

fn parse_members(kind: &'static str, value: &str) -> Result<Vec<HapiMember>, String> {
    value
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(parse_member(kind, index + 1, trimmed))
            }
        })
        .collect()
}

fn parse_member(kind: &'static str, line_number: usize, line: &str) -> Result<HapiMember, String> {
    let parts = line.split('|').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        return Err(format!(
            "Invalid {kind} member at line {line_number}. Expected: name | role | hapi-endpoint"
        ));
    }

    Ok(HapiMember {
        kind,
        name: parts[0].to_string(),
        role: parts[1].to_string(),
        endpoint: parts[2].to_string(),
    })
}
