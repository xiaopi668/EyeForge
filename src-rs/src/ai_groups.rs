use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration as StdDuration, SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HapiMember {
    kind: String,
    name: String,
    role: String,
    endpoint: String,
}

#[derive(Debug, Deserialize)]
struct ExternalDispatchRequest {
    task: String,
    #[serde(default)]
    strategy: String,
    #[serde(default)]
    capabilities: Value,
    #[serde(default)]
    member: Option<HapiMember>,
    #[serde(default)]
    members: Vec<HapiMember>,
}

#[derive(Debug, Clone)]
struct AiGroupState {
    hapi_endpoint: String,
    strategy: String,
    shell_enabled: bool,
    members: Vec<HapiMember>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollaborationMessage {
    pub speaker: String,
    pub role: String,
    pub kind: String,
    pub status: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollaborationResult {
    pub strategy: String,
    pub member_count: usize,
    pub messages: Vec<CollaborationMessage>,
    pub summary: String,
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
            "AI groups bridge ready; hapi {}; strategy {}; members: {}",
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
    let mut errors = Vec::new();
    let hapi_members = group
        .members
        .iter()
        .filter(|member| !member.uses_websocket() && !member.uses_openai_compatible_api())
        .collect::<Vec<_>>();

    if !hapi_members.is_empty() && !group.hapi_endpoint.trim().is_empty() {
        let body = json!({
            "task": task,
            "strategy": group.strategy,
            "members": hapi_members,
            "capabilities": {
                "shell_enabled": config.shell_enabled,
            },
            "response_format": "eyeforge_action_plan",
        });

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
    }

    match group.strategy.trim().to_ascii_lowercase().as_str() {
        "broadcast" => dispatch_broadcast(&group, task, &mut errors).await,
        "fallback" => dispatch_fallback(&group, task, &mut errors).await,
        _ => dispatch_primary(&group, task, &mut errors).await,
    }
}

pub async fn collaborate_task(config: &Config, task: &str) -> Result<CollaborationResult, String> {
    let task = task.trim();
    if task.is_empty() {
        return Err("AI group collaboration task is empty".into());
    }
    if !config.ai_groups_enabled {
        return Err("AI groups are disabled".into());
    }

    let group = build_state(config)?;
    let strategy = normalized_strategy(&group.strategy);
    let people_context = local_people_context(&config.ai_group_people);
    let collaboration_task = if people_context.is_empty() {
        format!("Execution mode: collaboration.\nUser request:\n{task}")
    } else {
        format!(
            "Execution mode: collaboration.\nLocal human participants:\n{people_context}\n\nUser request:\n{task}"
        )
    };

    let mut messages = vec![CollaborationMessage {
        speaker: "User".into(),
        role: "requester".into(),
        kind: "person".into(),
        status: "input".into(),
        content: task.to_string(),
    }];
    let mut errors = Vec::new();
    let mut success_count = 0usize;

    match strategy.as_str() {
        "primary" => {
            if let Some(member) = group.members.first() {
                if collaborate_member(
                    member,
                    &collaboration_task,
                    &strategy,
                    group.shell_enabled,
                    &mut messages,
                    &mut errors,
                )
                .await?
                {
                    success_count += 1;
                }
            }
        }
        "fallback" => {
            for member in &group.members {
                if collaborate_member(
                    member,
                    &collaboration_task,
                    &strategy,
                    group.shell_enabled,
                    &mut messages,
                    &mut errors,
                )
                .await?
                {
                    success_count += 1;
                    break;
                }
            }
        }
        _ => {
            for member in &group.members {
                if collaborate_member(
                    member,
                    &collaboration_task,
                    &strategy,
                    group.shell_enabled,
                    &mut messages,
                    &mut errors,
                )
                .await?
                {
                    success_count += 1;
                }
            }
        }
    }

    if success_count == 0 && !errors.is_empty() {
        messages.push(CollaborationMessage {
            speaker: "EyeForge".into(),
            role: "coordinator".into(),
            kind: "system".into(),
            status: "error".into(),
            content: format_dispatch_errors(&errors),
        });
    }

    let summary = if success_count == 0 {
        "AI group collaboration finished with no successful member response".into()
    } else {
        format!(
            "AI group collaboration completed: {success_count}/{} member(s) responded",
            group.members.len()
        )
    };

    messages.push(CollaborationMessage {
        speaker: "EyeForge".into(),
        role: "coordinator".into(),
        kind: "system".into(),
        status: if success_count == 0 {
            "error"
        } else {
            "success"
        }
        .into(),
        content: summary.clone(),
    });

    Ok(CollaborationResult {
        strategy,
        member_count: group.members.len(),
        messages,
        summary,
    })
}

pub async fn dispatch_external_request(
    config: &Config,
    payload: Value,
) -> Result<Option<String>, String> {
    let request: ExternalDispatchRequest = serde_json::from_value(payload)
        .map_err(|error| format!("Invalid HAPI dispatch payload: {error}"))?;
    let task = request.task.trim().to_string();
    if task.is_empty() {
        return Err("HAPI task is empty".into());
    }

    let strategy = request.strategy.trim().to_string();
    let shell_enabled = request
        .capabilities
        .get("shell_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(config.shell_enabled);
    let mut members = request.members;
    if let Some(member) = request.member {
        members.insert(0, member);
    }

    if members.is_empty() {
        return crate::llm::plan_actions(&task, config).await.map(Some);
    }

    let group = AiGroupState {
        hapi_endpoint: String::new(),
        strategy,
        shell_enabled,
        members,
    };
    let mut errors = Vec::new();

    match group.strategy.trim().to_ascii_lowercase().as_str() {
        "broadcast" => dispatch_broadcast(&group, &task, &mut errors).await,
        "fallback" => dispatch_fallback(&group, &task, &mut errors).await,
        _ => dispatch_primary(&group, &task, &mut errors).await,
    }
}

pub async fn dispatch_builtin_agent(
    kind: &str,
    task: &str,
    shell_enabled: bool,
) -> Result<Option<String>, String> {
    let normalized = normalize_agent_kind(kind);
    let endpoint = match normalized.as_str() {
        "codex" => "builtin://codex",
        "opencode" => "builtin://opencode",
        _ => return Err(format!("Unsupported built-in agent: {kind}")),
    };
    let member = HapiMember {
        kind: normalized,
        name: kind.to_string(),
        role: "assistant".into(),
        endpoint: endpoint.into(),
    };

    post_builtin_cli(&member, task, "primary", shell_enabled).await
}

pub fn builtin_agent_status(kind: &str) -> Value {
    let normalized = normalize_agent_kind(kind);
    let executable = match normalized.as_str() {
        "codex" => resolve_codex_executable(),
        "opencode" => resolve_opencode_executable(),
        _ => None,
    };
    let path = executable.as_ref().map(|path| path.display().to_string());
    let available = executable
        .as_ref()
        .map(|path| path.exists() || path.components().count() == 1)
        .unwrap_or(false);

    json!({
        "kind": normalized,
        "available": available,
        "executable": path,
        "endpoint": format!("builtin://{kind}", kind = normalize_agent_kind(kind)),
    })
}

pub fn stop() {
    if let Ok(mut guard) = state().lock() {
        *guard = None;
    }
}

fn build_state(config: &Config) -> Result<AiGroupState, String> {
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
    members.extend(parse_members("api", &config.ai_group_api_members)?);

    if members.is_empty() {
        return Err("AI groups enabled but no members are configured".into());
    }

    Ok(AiGroupState {
        hapi_endpoint: config.ai_group_hapi_endpoint.trim().to_string(),
        strategy: config.ai_group_strategy.clone(),
        shell_enabled: config.shell_enabled,
        members,
    })
}

async fn dispatch_primary(
    group: &AiGroupState,
    task: &str,
    errors: &mut Vec<String>,
) -> Result<Option<String>, String> {
    if let Some(member) = group.members.first() {
        return dispatch_member(member, task, &group.strategy, group.shell_enabled, errors).await;
    }

    Err(format_dispatch_errors(errors))
}

async fn dispatch_fallback(
    group: &AiGroupState,
    task: &str,
    errors: &mut Vec<String>,
) -> Result<Option<String>, String> {
    for member in &group.members {
        if let Some(plan) =
            dispatch_member(member, task, &group.strategy, group.shell_enabled, errors).await?
        {
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
        if let Some(plan) =
            dispatch_member(member, task, &group.strategy, group.shell_enabled, errors).await?
        {
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
    shell_enabled: bool,
    errors: &mut Vec<String>,
) -> Result<Option<String>, String> {
    let result = if member.uses_builtin_cli() {
        post_builtin_cli(member, task, strategy, shell_enabled).await
    } else if member.uses_openai_compatible_api() {
        post_openai_compatible_api(member, task, strategy, shell_enabled).await
    } else {
        let body = json!({
            "task": task,
            "strategy": strategy,
            "member": member,
            "capabilities": {
                "shell_enabled": shell_enabled,
            },
            "response_format": "eyeforge_action_plan",
        });

        if member.uses_websocket() {
            post_ws_json(&member.endpoint, &body).await
        } else {
            post_hapi_json(&member.endpoint, "", &body).await
        }
    };

    match result {
        Ok(plan) => Ok(plan),
        Err(error) => {
            errors.push(error);
            Ok(None)
        }
    }
}

async fn collaborate_member(
    member: &HapiMember,
    task: &str,
    strategy: &str,
    shell_enabled: bool,
    messages: &mut Vec<CollaborationMessage>,
    errors: &mut Vec<String>,
) -> Result<bool, String> {
    match dispatch_member(member, task, strategy, shell_enabled, errors).await? {
        Some(plan) => {
            messages.push(CollaborationMessage {
                speaker: member.name.clone(),
                role: member.role.clone(),
                kind: member.kind.clone(),
                status: "success".into(),
                content: action_plan_to_text(&plan),
            });
            Ok(true)
        }
        None => {
            messages.push(CollaborationMessage {
                speaker: member.name.clone(),
                role: member.role.clone(),
                kind: member.kind.clone(),
                status: "error".into(),
                content: "No usable response from this member".into(),
            });
            Ok(false)
        }
    }
}

async fn post_openai_compatible_api(
    member: &HapiMember,
    task: &str,
    strategy: &str,
    shell_enabled: bool,
) -> Result<Option<String>, String> {
    let spec = OpenAiApiSpec::parse(&member.endpoint)?;
    let prompt = build_builtin_prompt(member, task, strategy, shell_enabled);
    let body = json!({
        "model": spec.model,
        "messages": [
            {
                "role": "system",
                "content": "You are an EyeForge AI group member. Return only valid JSON action plans."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.2
    });

    let mut request = reqwest::Client::new().post(&spec.chat_url).json(&body);
    if let Some(key) = spec.api_key.as_deref().filter(|key| !key.is_empty()) {
        request = request.bearer_auth(key);
    }

    let response = timeout(Duration::from_secs(180), request.send())
        .await
        .map_err(|_| format!("OpenAI-compatible API timed out: {}", spec.chat_url))?
        .map_err(|error| format!("OpenAI-compatible API request failed: {error}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("Failed to read OpenAI-compatible API response: {error}"))?;

    if !status.is_success() {
        return Err(format!(
            "OpenAI-compatible API {} returned {status}: {text}",
            spec.chat_url
        ));
    }

    let value: Value = serde_json::from_str(&text)
        .map_err(|error| format!("OpenAI-compatible API returned invalid JSON: {error}"))?;
    let content = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .or_else(|| value.get("content").and_then(Value::as_str))
        .or_else(|| value.get("text").and_then(Value::as_str))
        .ok_or_else(|| "OpenAI-compatible API returned no message content".to_string())?;

    Ok(extract_action_plan(content))
}

async fn post_builtin_cli(
    member: &HapiMember,
    task: &str,
    strategy: &str,
    shell_enabled: bool,
) -> Result<Option<String>, String> {
    let prompt = build_builtin_prompt(member, task, strategy, shell_enabled);
    match member.kind.as_str() {
        "codex" => run_codex_cli(&prompt).await,
        "opencode" => run_opencode_cli(&prompt).await,
        other => Err(format!("Unsupported built-in CLI member kind: {other}")),
    }
}

async fn post_ws_json(endpoint: &str, body: &Value) -> Result<Option<String>, String> {
    let request = connect_async(endpoint);
    let (mut socket, _) = timeout(Duration::from_secs(20), request)
        .await
        .map_err(|_| format!("WebSocket connection timed out: {endpoint}"))?
        .map_err(|error| format!("WebSocket connection failed at {endpoint}: {error}"))?;

    socket
        .send(Message::Text(body.to_string()))
        .await
        .map_err(|error| format!("WebSocket send failed at {endpoint}: {error}"))?;

    let response = timeout(Duration::from_secs(60), socket.next())
        .await
        .map_err(|_| format!("WebSocket response timed out: {endpoint}"))?
        .ok_or_else(|| format!("WebSocket closed without a response: {endpoint}"))?
        .map_err(|error| format!("WebSocket read failed at {endpoint}: {error}"))?;

    match response {
        Message::Text(text) => Ok(extract_action_plan(&text)),
        Message::Binary(bytes) => String::from_utf8(bytes)
            .map_err(|error| format!("WebSocket returned non-UTF8 binary data: {error}"))
            .map(|text| extract_action_plan(&text)),
        Message::Close(_) => Err(format!("WebSocket closed without content: {endpoint}")),
        _ => Ok(None),
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

fn build_builtin_prompt(
    member: &HapiMember,
    task: &str,
    strategy: &str,
    shell_enabled: bool,
) -> String {
    let shell_instruction = if shell_enabled {
        "Shell/open launcher actions are enabled. You may output {\"type\":\"shell\",\"command\":\"...\"} or {\"type\":\"open\",\"target\":\"...\"} only when it is the safest direct route.\n"
    } else {
        "Shell/open launcher actions are disabled. Do not output any {\"type\":\"shell\",...} or {\"type\":\"open\",...} actions; use GUI click/type/hotkey/scroll/wait/screenshot/complete actions instead.\n"
    };
    format!(
        concat!(
            "You are acting as the {kind} member \"{name}\" for EyeForge AI Groups.\n",
            "Role: {role}\n",
            "Strategy: {strategy}\n",
            "Runtime capability: shell={shell_enabled}, open_launcher={shell_enabled}\n",
            "{shell_instruction}",
            "Return only valid JSON for the final answer.\n",
            "The JSON must match this exact shape:\n",
            "{{\"actions\":[{{\"type\":\"complete\",\"result\":\"...\"}}]}}\n",
            "If the task says execution mode is step-by-step, return exactly one next EyeForge action.\n",
            "If the task says execution mode is adaptive or batch, choose the mode yourself: return multiple safe actions when the next steps are certain, or exactly one action when the next step depends on feedback.\n",
            "Do not batch uncertain GUI actions. In a batch plan, complete may appear only as the final action.\n",
            "Do not wrap the JSON in markdown fences.\n",
            "Task:\n{task}\n"
        ),
        kind = member.kind,
        name = member.name,
        role = member.role,
        strategy = strategy,
        shell_enabled = if shell_enabled { "enabled" } else { "disabled" },
        shell_instruction = shell_instruction,
        task = task
    )
}

async fn run_codex_cli(prompt: &str) -> Result<Option<String>, String> {
    let executable = resolve_codex_executable()
        .ok_or_else(|| "Built-in Codex adapter could not find a codex executable".to_string())?;
    let output_file = temp_output_file("codex");
    let mut command = Command::new(executable);
    command
        .arg("-a")
        .arg("never")
        .arg("-s")
        .arg("read-only")
        .arg("exec")
        .arg("--skip-git-repo-check")
        .arg("--color")
        .arg("never")
        .arg("--output-last-message")
        .arg(&output_file)
        .arg("-");
    apply_hidden_process_flags(&mut command);

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start Codex CLI: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .await
            .map_err(|error| format!("Failed to send prompt to Codex CLI: {error}"))?;
    }

    let output = timeout(Duration::from_secs(180), child.wait_with_output())
        .await
        .map_err(|_| "Codex CLI timed out".to_string())?
        .map_err(|error| format!("Codex CLI execution failed: {error}"))?;

    let text = std::fs::read_to_string(&output_file).unwrap_or_default();
    let _ = std::fs::remove_file(&output_file);

    if text.trim().is_empty() && !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("Codex CLI exited with status {:?}", output.status.code())
        } else {
            format!("Codex CLI failed: {stderr}")
        });
    }

    Ok(extract_action_plan(&text))
}

async fn run_opencode_cli(prompt: &str) -> Result<Option<String>, String> {
    let executable = resolve_opencode_executable().ok_or_else(|| {
        "Built-in OpenCode adapter could not find an opencode executable".to_string()
    })?;
    let mut command = Command::new(executable);
    command
        .arg("-p")
        .arg(prompt)
        .arg("-f")
        .arg("text")
        .arg("-q");
    apply_hidden_process_flags(&mut command);

    let output = timeout(Duration::from_secs(180), command.output())
        .await
        .map_err(|_| "OpenCode CLI timed out".to_string())?
        .map_err(|error| format!("OpenCode CLI execution failed: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() && stdout.is_empty() {
        return Err(if stderr.is_empty() {
            format!("OpenCode CLI exited with status {:?}", output.status.code())
        } else {
            format!("OpenCode CLI failed: {stderr}")
        });
    }

    Ok(extract_action_plan(&stdout))
}

fn action_plan_to_text(plan: &str) -> String {
    let trimmed = plan.trim();
    if trimmed.is_empty() {
        return "Empty response".into();
    }

    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return trimmed.to_string();
    };

    if let Some(result) = value
        .get("result")
        .and_then(Value::as_str)
        .or_else(|| value.get("message").and_then(Value::as_str))
        .or_else(|| value.get("text").and_then(Value::as_str))
    {
        return result.to_string();
    }

    if let Some(actions) = value.get("actions").and_then(Value::as_array) {
        if let Some(result) = actions.iter().rev().find_map(|action| {
            action
                .get("result")
                .and_then(Value::as_str)
                .filter(|text| !text.trim().is_empty())
        }) {
            return result.to_string();
        }

        let summaries = actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let action_type = action
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("action");
                format!("{}. {}", index + 1, action_type)
            })
            .collect::<Vec<_>>();
        if !summaries.is_empty() {
            return format!("Action plan:\n{}", summaries.join("\n"));
        }
    }

    serde_json::to_string_pretty(&value).unwrap_or_else(|_| trimmed.to_string())
}

#[derive(Debug)]
struct OpenAiApiSpec {
    chat_url: String,
    model: String,
    api_key: Option<String>,
}

impl OpenAiApiSpec {
    fn parse(endpoint: &str) -> Result<Self, String> {
        let raw = endpoint
            .trim()
            .strip_prefix("openai-compatible:")
            .or_else(|| endpoint.trim().strip_prefix("openai:"))
            .ok_or_else(|| "OpenAI-compatible endpoint must start with openai:".to_string())?;
        let mut parts = raw.split(';').map(str::trim);
        let base = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "OpenAI-compatible endpoint is missing URL".to_string())?;
        let mut model = "gpt-4o-mini".to_string();
        let mut api_key = None;

        for part in parts {
            let Some((key, value)) = part.split_once('=') else {
                continue;
            };
            match key.trim().to_ascii_lowercase().as_str() {
                "model" => model = value.trim().to_string(),
                "key" | "api_key" | "token" => api_key = Some(value.trim().to_string()),
                _ => {}
            }
        }

        if model.trim().is_empty() {
            return Err("OpenAI-compatible endpoint is missing model".into());
        }

        let trimmed = base.trim_end_matches('/');
        let chat_url = if trimmed.ends_with("/chat/completions") {
            trimmed.to_string()
        } else if trimmed.ends_with("/v1") {
            format!("{trimmed}/chat/completions")
        } else {
            trimmed.to_string()
        };

        Ok(Self {
            chat_url,
            model,
            api_key,
        })
    }
}

fn resolve_codex_executable() -> Option<PathBuf> {
    if let Some(path) = env_path("EYEFORGE_CODEX_PATH") {
        return Some(path);
    }

    let local_app_data = std::env::var_os("LOCALAPPDATA")?;
    let root = PathBuf::from(local_app_data)
        .join("OpenAI")
        .join("Codex")
        .join("bin");
    find_latest_named_file(&root, "codex.exe").or_else(|| Some(PathBuf::from("codex")))
}

fn resolve_opencode_executable() -> Option<PathBuf> {
    if let Some(path) = env_path("EYEFORGE_OPENCODE_PATH") {
        return Some(path);
    }

    let mut candidates = Vec::new();

    if let Some(app_data) = std::env::var_os("APPDATA") {
        candidates.push(
            PathBuf::from(app_data.clone())
                .join("npm")
                .join("opencode.cmd"),
        );
        candidates.push(PathBuf::from(app_data).join("npm").join("opencode"));
    }

    if let Some(home) = std::env::var_os("USERPROFILE") {
        candidates.push(
            PathBuf::from(home.clone())
                .join(".local")
                .join("bin")
                .join("opencode"),
        );
        candidates.push(
            PathBuf::from(home)
                .join(".local")
                .join("bin")
                .join("opencode.cmd"),
        );
    }

    candidates
        .into_iter()
        .find(|path| path.exists())
        .or_else(|| Some(PathBuf::from("opencode")))
}

fn find_latest_named_file(root: &Path, filename: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(root).ok()?;
    let mut latest: Option<(SystemTime, PathBuf)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let candidate = path.join(filename);
            if let Ok(metadata) = std::fs::metadata(&candidate) {
                let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
                match &latest {
                    Some((current, _)) if &modified <= current => {}
                    _ => latest = Some((modified, candidate)),
                }
            }
        }
    }

    latest.map(|(_, path)| path)
}

fn env_path(key: &str) -> Option<PathBuf> {
    let value = std::env::var_os(key)?;
    let path = PathBuf::from(value);
    (!path.as_os_str().is_empty()).then_some(path)
}

fn temp_output_file(prefix: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(StdDuration::from_secs(0))
        .as_millis();
    std::env::temp_dir().join(format!("eyeforge-{prefix}-{timestamp}.txt"))
}

fn apply_hidden_process_flags(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(0x08000000);
    }
}

fn format_dispatch_errors(errors: &[String]) -> String {
    if errors.is_empty() {
        "AI group dispatch returned no usable response".into()
    } else {
        format!("AI group dispatch failed: {}", errors.join(" | "))
    }
}

fn normalized_strategy(strategy: &str) -> String {
    match strategy.trim().to_ascii_lowercase().as_str() {
        "primary" => "primary".into(),
        "fallback" => "fallback".into(),
        _ => "broadcast".into(),
    }
}

fn local_people_context(source: &str) -> String {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n")
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
            "Invalid {kind} member at line {line_number}. Expected: name | role | endpoint"
        ));
    }

    Ok(HapiMember {
        kind: kind.to_string(),
        name: parts[0].to_string(),
        role: parts[1].to_string(),
        endpoint: parts[2].to_string(),
    })
}

fn normalize_agent_kind(kind: &str) -> String {
    let normalized = kind.trim().to_ascii_lowercase();
    if normalized.contains("open") {
        "opencode".into()
    } else if normalized.contains("codex") {
        "codex".into()
    } else {
        normalized
    }
}

impl HapiMember {
    fn uses_websocket(&self) -> bool {
        matches!(self.kind.as_str(), "openclaw" | "astrbot")
            || self.endpoint.starts_with("ws://")
            || self.endpoint.starts_with("wss://")
    }

    fn uses_builtin_cli(&self) -> bool {
        matches!(self.kind.as_str(), "codex" | "opencode")
            && matches!(
                self.endpoint.trim(),
                "" | "builtin://codex"
                    | "builtin://opencode"
                    | "local://codex"
                    | "local://opencode"
                    | "http://127.0.0.1:9101"
                    | "http://localhost:9101"
                    | "http://127.0.0.1:9102"
                    | "http://localhost:9102"
            )
    }

    fn uses_openai_compatible_api(&self) -> bool {
        self.kind == "api"
            && (self.endpoint.starts_with("openai:")
                || self.endpoint.starts_with("openai-compatible:"))
    }
}
