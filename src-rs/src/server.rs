use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json as ExtractJson, Query, State,
    },
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use enigo::{Button, Direction, Keyboard, Mouse};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot};

use crate::config::Config;
use crate::{channels, runtime, voice, wechat};

pub const GATEWAY_WS_PATH: &str = "/ws";

struct ServerHandle {
    shutdown: oneshot::Sender<()>,
    thread: JoinHandle<()>,
}

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

static SERVER: OnceLock<Mutex<Option<ServerHandle>>> = OnceLock::new();

fn state() -> &'static Mutex<Option<ServerHandle>> {
    SERVER.get_or_init(|| Mutex::new(None))
}

pub fn restart(config: &Config) -> Result<(), String> {
    stop();

    if !config.ws_enabled {
        return Ok(());
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let config_clone = config.clone();

    let thread = std::thread::spawn(move || {
        let runtime = match Runtime::new() {
            Ok(value) => value,
            Err(error) => {
                eprintln!("Failed to create tokio runtime: {error}");
                return;
            }
        };

        runtime.block_on(async move {
            if let Err(error) = run_server(config_clone, shutdown_rx).await {
                eprintln!("Rust gateway stopped with error: {error}");
            }
        });
    });

    *state()
        .lock()
        .map_err(|_| "server state poisoned".to_string())? = Some(ServerHandle {
        shutdown: shutdown_tx,
        thread,
    });

    Ok(())
}

pub fn stop() {
    let mut guard = match state().lock() {
        Ok(value) => value,
        Err(_) => return,
    };

    if let Some(handle) = guard.take() {
        let _ = handle.shutdown.send(());
        let _ = handle.thread.join();
    }
}

async fn run_server(config: Config, shutdown: oneshot::Receiver<()>) -> Result<(), String> {
    let address = format!("{}:{}", config.ws_host, config.ws_port);
    let listener = TcpListener::bind(address.as_str())
        .await
        .map_err(|error| format!("Failed to bind Rust gateway on {address}: {error}"))?;

    let app = Router::new()
        .route("/", get(index))
        .route("/app.js", get(app_js))
        .route("/styles.css", get(styles_css))
        .route("/health", get(health))
        .route("/api/channels", get(channel_status))
        .route("/api/wechat/qr-login", post(wechat_qr_login_start))
        .route("/api/wechat/qr-status", get(wechat_qr_login_status))
        .route("/api/ai-group", get(ai_group_get).post(ai_group_save))
        .route("/api/ai-group/collaborate", post(ai_group_collaborate))
        .route("/api/codex/status", get(codex_status))
        .route("/api/codex/task", post(codex_task))
        .route("/api/opencode/status", get(opencode_status))
        .route("/api/opencode/task", post(opencode_task))
        .route("/v1/ai-groups/dispatch", post(ai_group_dispatch))
        .route("/ai-groups/dispatch", post(ai_group_dispatch))
        .route("/dispatch", post(ai_group_dispatch))
        .route(
            "/api/channel-config",
            get(channel_config_get).post(channel_config_save),
        )
        .route("/api/voice/devices", get(voice_devices))
        .route("/api/voice/transcribe", post(voice_transcribe))
        .route("/api/screenshot", get(screenshot_handler))
        .route("/api/desktop/click", post(desktop_click_handler))
        .route("/api/desktop/type", post(desktop_type_handler))
        .route("/api/desktop/hotkey", post(desktop_hotkey_handler))
        .route("/api/desktop/scroll", post(desktop_scroll_handler))
        .route("/api/screen-info", get(screen_info_handler))
        .route(GATEWAY_WS_PATH, get(ws_handler))
        .with_state(AppState {
            config: Arc::new(config),
        });

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = shutdown.await;
        })
        .await
        .map_err(|error| format!("Rust gateway server error: {error}"))
}

async fn index() -> Response {
    let mut response = Html(include_str!("../../web-ui/index.html")).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    response
}

async fn app_js() -> Response {
    let mut response = Response::new(Body::from(include_str!("../../web-ui/app.js")));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    response
}

async fn styles_css() -> Response {
    let mut response = Response::new(Body::from(include_str!("../../web-ui/styles.css")));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/css; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    response
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

async fn channel_status(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "channels": channels::collect(&state.config)
    }))
}

#[derive(Debug, Deserialize)]
struct WechatQrStatusQuery {
    key: String,
}

async fn wechat_qr_login_start() -> impl IntoResponse {
    match wechat::begin_qr_login().await {
        Ok(session) => {
            use base64::Engine;

            let image = base64::engine::general_purpose::STANDARD.encode(session.image_bytes);
            (
                StatusCode::OK,
                Json(json!({
                    "key": session.key,
                    "image_data_url": format!("data:image/png;base64,{image}"),
                })),
            )
                .into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error })),
        )
            .into_response(),
    }
}

async fn wechat_qr_login_status(Query(query): Query<WechatQrStatusQuery>) -> impl IntoResponse {
    match wechat::query_qr_status(&query.key).await {
        Ok(status) => {
            if status.status == "confirmed" && !status.token.trim().is_empty() {
                let mut config = Config::load();
                config.wc_token = status.token.clone();
                config.wc_enabled = true;
                config.save();
            }

            (
                StatusCode::OK,
                Json(json!({
                    "status": status.status,
                    "token": status.token,
                    "bot_id": status.bot_id,
                    "user_id": status.user_id,
                })),
            )
                .into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error })),
        )
            .into_response(),
    }
}

async fn channel_config_get() -> impl IntoResponse {
    let config = Config::load();
    (
        StatusCode::OK,
        Json(json!({
            "ws_enabled": config.ws_enabled,
            "ws_host": config.ws_host,
            "ws_port": config.ws_port,
            "ws_token": config.ws_token,
            "wc_enabled": config.wc_enabled,
            "wc_token": config.wc_token,
            "wcom_enabled": config.wcom_enabled,
            "wcom_corp_id": config.wcom_corp_id,
            "wcom_agent_id": config.wcom_agent_id,
            "wcom_secret": config.wcom_secret,
            "wcom_token": config.wcom_token,
            "wcom_aes_key": config.wcom_aes_key,
            "dt_enabled": config.dt_enabled,
            "dt_app_key": config.dt_app_key,
            "dt_app_secret": config.dt_app_secret,
            "dt_webhook": config.dt_webhook,
            "qq_enabled": config.qq_enabled,
            "qq_mode": config.qq_mode,
            "qq_ws_host": config.qq_ws_host,
            "qq_ws_port": config.qq_ws_port,
            "qq_bot_appid": config.qq_bot_appid,
            "qq_bot_token": config.qq_bot_token,
        })),
    )
        .into_response()
}

async fn channel_config_save(
    ExtractJson(payload): ExtractJson<serde_json::Value>,
) -> impl IntoResponse {
    let mut config = Config::load();
    let kind = payload
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    match kind {
        "gateway" => {
            config.ws_enabled = json_bool(&payload, "enabled", config.ws_enabled);
            config.ws_host = json_string(&payload, "host", &config.ws_host);
            config.ws_port = json_u16(&payload, "port", config.ws_port);
            config.ws_token = json_string(&payload, "token", &config.ws_token);
        }
        "wechat" => {
            config.wc_enabled = json_bool(&payload, "enabled", config.wc_enabled);
            config.wc_token = json_string(&payload, "token", &config.wc_token);
        }
        "wecom" => {
            config.wcom_enabled = json_bool(&payload, "enabled", config.wcom_enabled);
            config.wcom_corp_id = json_string(&payload, "corp_id", &config.wcom_corp_id);
            config.wcom_agent_id = json_string(&payload, "agent_id", &config.wcom_agent_id);
            config.wcom_secret = json_string(&payload, "secret", &config.wcom_secret);
            config.wcom_token = json_string(&payload, "token", &config.wcom_token);
            config.wcom_aes_key = json_string(&payload, "aes_key", &config.wcom_aes_key);
        }
        "dingtalk" => {
            config.dt_enabled = json_bool(&payload, "enabled", config.dt_enabled);
            config.dt_app_key = json_string(&payload, "app_key", &config.dt_app_key);
            config.dt_app_secret = json_string(&payload, "app_secret", &config.dt_app_secret);
            config.dt_webhook = json_string(&payload, "webhook", &config.dt_webhook);
        }
        "qq" => {
            config.qq_enabled = json_bool(&payload, "enabled", config.qq_enabled);
            config.qq_mode = json_string(&payload, "mode", &config.qq_mode);
            config.qq_ws_host = json_string(&payload, "ws_host", &config.qq_ws_host);
            config.qq_ws_port = json_u16(&payload, "ws_port", config.qq_ws_port);
            config.qq_bot_appid = json_string(&payload, "bot_appid", &config.qq_bot_appid);
            config.qq_bot_token = json_string(&payload, "bot_token", &config.qq_bot_token);
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "unknown channel kind" })),
            )
                .into_response();
        }
    }

    config.save();
    (StatusCode::OK, Json(json!({ "ok": true }))).into_response()
}

fn json_string(payload: &serde_json::Value, key: &str, fallback: &str) -> String {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or(fallback)
        .trim()
        .to_string()
}

fn json_bool(payload: &serde_json::Value, key: &str, fallback: bool) -> bool {
    payload
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(fallback)
}

fn json_u16(payload: &serde_json::Value, key: &str, fallback: u16) -> u16 {
    payload
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or(fallback)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiGroupMember {
    name: String,
    role: String,
    endpoint: Option<String>,
    kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiGroupPayload {
    enabled: bool,
    name: String,
    people: Vec<AiGroupMember>,
    agents: Vec<AiGroupMember>,
    hapi_endpoint: String,
    strategy: String,
}

async fn ai_group_get() -> impl IntoResponse {
    let config = Config::load();
    (StatusCode::OK, Json(ai_group_from_config(&config))).into_response()
}

async fn ai_group_save(ExtractJson(payload): ExtractJson<AiGroupPayload>) -> impl IntoResponse {
    let mut config = Config::load();
    config.ai_groups_enabled = payload.enabled;
    config.ai_group_name = payload.name.trim().to_string();
    config.ai_group_people = members_to_lines(&payload.people, false);
    config.ai_group_hapi_endpoint = payload.hapi_endpoint.trim().to_string();
    config.ai_group_strategy = payload.strategy.trim().to_string();
    config.ai_group_openclaw_members = agents_to_lines(&payload.agents, "openclaw");
    config.ai_group_astrbot_members = agents_to_lines(&payload.agents, "astrbot");
    config.ai_group_opencode_members = agents_to_lines(&payload.agents, "opencode");
    config.ai_group_codex_members = agents_to_lines(&payload.agents, "codex");
    config.ai_group_claude_code_members = agents_to_lines(&payload.agents, "claude");
    config.ai_group_api_members = agents_to_lines(&payload.agents, "api");
    config.save();

    (StatusCode::OK, Json(ai_group_from_config(&config))).into_response()
}

async fn ai_group_dispatch(
    State(state): State<AppState>,
    ExtractJson(payload): ExtractJson<serde_json::Value>,
) -> impl IntoResponse {
    match crate::ai_groups::dispatch_external_request(&state.config, payload).await {
        Ok(Some(plan)) => (StatusCode::OK, Json(json!({ "action_plan": plan }))).into_response(),
        Ok(None) => (StatusCode::OK, Json(json!({ "action_plan": null }))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct AiGroupCollaborateRequest {
    task: String,
}

async fn ai_group_collaborate(
    ExtractJson(payload): ExtractJson<AiGroupCollaborateRequest>,
) -> impl IntoResponse {
    let config = Config::load();
    match crate::ai_groups::collaborate_task(&config, &payload.task).await {
        Ok(result) => (StatusCode::OK, Json(json!(result))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    }
}

async fn codex_status() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(crate::ai_groups::builtin_agent_status("codex")),
    )
        .into_response()
}

async fn opencode_status() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(crate::ai_groups::builtin_agent_status("opencode")),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
struct BuiltinAgentTaskRequest {
    task: String,
}

async fn codex_task(
    ExtractJson(payload): ExtractJson<BuiltinAgentTaskRequest>,
) -> impl IntoResponse {
    builtin_agent_task("codex", payload).await
}

async fn opencode_task(
    ExtractJson(payload): ExtractJson<BuiltinAgentTaskRequest>,
) -> impl IntoResponse {
    builtin_agent_task("opencode", payload).await
}

async fn builtin_agent_task(kind: &str, payload: BuiltinAgentTaskRequest) -> Response {
    if payload.task.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "task is empty" })),
        )
            .into_response();
    }

    let config = Config::load();
    match crate::ai_groups::dispatch_builtin_agent(kind, &payload.task, config.shell_enabled).await
    {
        Ok(Some(plan)) => (StatusCode::OK, Json(json!({ "action_plan": plan }))).into_response(),
        Ok(None) => (StatusCode::OK, Json(json!({ "action_plan": null }))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    }
}

fn ai_group_from_config(config: &Config) -> AiGroupPayload {
    let mut agents = Vec::new();
    agents.extend(lines_to_members(
        &config.ai_group_openclaw_members,
        "openclaw",
        true,
    ));
    agents.extend(lines_to_members(
        &config.ai_group_astrbot_members,
        "astrbot",
        true,
    ));
    agents.extend(lines_to_members(
        &config.ai_group_opencode_members,
        "opencode",
        true,
    ));
    agents.extend(lines_to_members(
        &config.ai_group_codex_members,
        "codex",
        true,
    ));
    agents.extend(lines_to_members(
        &config.ai_group_claude_code_members,
        "claude",
        true,
    ));
    agents.extend(lines_to_members(&config.ai_group_api_members, "api", true));

    AiGroupPayload {
        enabled: config.ai_groups_enabled,
        name: config.ai_group_name.clone(),
        people: lines_to_members(&config.ai_group_people, "person", false),
        agents,
        hapi_endpoint: config.ai_group_hapi_endpoint.clone(),
        strategy: config.ai_group_strategy.clone(),
    }
}

fn lines_to_members(source: &str, kind: &str, endpoint_is_status: bool) -> Vec<AiGroupMember> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut parts = line.split('|').map(str::trim);
            let name = parts.next().unwrap_or("Member").to_string();
            let role = parts.next().unwrap_or("").to_string();
            let third = parts.next().unwrap_or("").to_string();
            AiGroupMember {
                name,
                role: if role.is_empty() {
                    if endpoint_is_status { "AI" } else { "Member" }.to_string()
                } else {
                    role
                },
                endpoint: (!third.is_empty()).then_some(third),
                kind: Some(kind.to_string()),
            }
        })
        .collect()
}

fn members_to_lines(members: &[AiGroupMember], include_endpoint: bool) -> String {
    members
        .iter()
        .filter(|member| !member.name.trim().is_empty())
        .map(|member| {
            if include_endpoint {
                format!(
                    "{} | {} | {}",
                    member.name.trim(),
                    member.role.trim(),
                    member.endpoint.as_deref().unwrap_or("").trim()
                )
            } else {
                format!(
                    "{} | {} | {}",
                    member.name.trim(),
                    member.role.trim(),
                    member.endpoint.as_deref().unwrap_or("本地备注").trim()
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn agents_to_lines(agents: &[AiGroupMember], kind: &str) -> String {
    let normalized = kind.to_ascii_lowercase();
    let selected = agents
        .iter()
        .filter(|agent| {
            agent
                .kind
                .as_deref()
                .unwrap_or("codex")
                .to_ascii_lowercase()
                .contains(&normalized)
        })
        .cloned()
        .collect::<Vec<_>>();

    members_to_lines(&selected, true)
}

async fn voice_devices() -> impl IntoResponse {
    match voice::list_input_devices() {
        Ok(devices) => (StatusCode::OK, Json(json!({ "devices": devices }))).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct VoiceRequest {
    seconds: Option<u32>,
}

async fn voice_transcribe(
    State(state): State<AppState>,
    ExtractJson(payload): ExtractJson<VoiceRequest>,
) -> impl IntoResponse {
    let seconds = payload.seconds.unwrap_or(4).clamp(1, 15);
    match voice::transcribe_default_input(seconds, (*state.config).clone()).await {
        Ok(result) => (StatusCode::OK, Json(json!({ "result": result }))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut authenticated = false;

    while let Some(frame) = socket.next().await {
        match frame {
            Ok(Message::Text(text)) => {
                let payload: serde_json::Value = match serde_json::from_str(&text) {
                    Ok(value) => value,
                    Err(error) => {
                        let _ = socket
                            .send(Message::Text(
                                json!({
                                    "type": "error",
                                    "message": format!("invalid JSON: {error}")
                                })
                                .to_string(),
                            ))
                            .await;
                        continue;
                    }
                };

                match payload
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                {
                    "auth" => {
                        let token = payload
                            .get("token")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("");
                        let success = token == state.config.ws_token;
                        let _ = socket
                            .send(Message::Text(
                                json!({
                                    "type": "auth_result",
                                    "success": success,
                                    "message": if success { "authenticated" } else { "invalid token" }
                                })
                                .to_string(),
                            ))
                            .await;
                        authenticated = success;
                        if !success {
                            break;
                        }
                    }
                    "task" => {
                        if !authenticated {
                            let _ = socket
                                .send(Message::Text(
                                    json!({
                                        "type": "error",
                                        "message": "authentication required"
                                    })
                                    .to_string(),
                                ))
                                .await;
                            continue;
                        }

                        let task = payload
                            .get("task")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("")
                            .trim()
                            .to_string();

                        if task.is_empty() {
                            let _ = socket
                                .send(Message::Text(
                                    json!({
                                        "type": "error",
                                        "message": "task is empty"
                                    })
                                    .to_string(),
                                ))
                                .await;
                            continue;
                        }

                        let _ = socket
                            .send(Message::Text(
                                json!({
                                    "type": "status",
                                    "message": "task received by Rust gateway"
                                })
                                .to_string(),
                            ))
                            .await;

                        let (event_tx, mut event_rx) =
                            mpsc::unbounded_channel::<runtime::RuntimeEvent>();
                        let task_config = (*state.config).clone();
                        let mut execution = tokio::spawn(async move {
                            let mut reporter = move |event| {
                                let _ = event_tx.send(event);
                            };
                            runtime::execute_task_with_events(task, task_config, &mut reporter)
                                .await
                        });

                        let result = loop {
                            tokio::select! {
                                event = event_rx.recv() => {
                                    if let Some(event) = event {
                                        let _ = socket
                                            .send(Message::Text(
                                                json!({
                                                    "type": "status",
                                                    "kind": format!("{:?}", event.kind).to_ascii_lowercase(),
                                                    "title": event.title,
                                                    "message": event.message
                                                })
                                                .to_string(),
                                            ))
                                            .await;
                                    }
                                }
                                result = &mut execution => {
                                    break match result {
                                        Ok(value) => value,
                                        Err(error) => Err(format!("task worker failed: {error}")),
                                    };
                                }
                            }
                        };

                        let response = match result {
                            Ok(outcome) => json!({
                                "type": "result",
                                "status": outcome.status,
                                "message": outcome.message,
                                "data": merge_data(outcome.data, outcome.transcript)
                            }),
                            Err(error) => json!({
                                "type": "result",
                                "status": "error",
                                "message": error
                            }),
                        };

                        let _ = socket.send(Message::Text(response.to_string())).await;
                    }
                    _ => {
                        let _ = socket
                            .send(Message::Text(
                                json!({
                                    "type": "error",
                                    "message": "unknown type"
                                })
                                .to_string(),
                            ))
                            .await;
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(payload)) => {
                let _ = socket.send(Message::Pong(payload)).await;
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

fn merge_data(data: Option<serde_json::Value>, transcript: Vec<String>) -> serde_json::Value {
    let mut base = match data {
        Some(serde_json::Value::Object(map)) => map,
        Some(other) => {
            let mut map = serde_json::Map::new();
            map.insert("payload".into(), other);
            map
        }
        None => serde_json::Map::new(),
    };

    base.insert("transcript".into(), json!(transcript));
    serde_json::Value::Object(base)
}

// ── 截图与桌面操作 API ──

fn create_enigo() -> Result<enigo::Enigo, String> {
    enigo::Enigo::new(&enigo::Settings::default()).map_err(|e| format!("enigo init failed: {e}"))
}

fn map_key(value: &str) -> Option<enigo::Key> {
    match value.to_lowercase().as_str() {
        "enter" | "return" => Some(enigo::Key::Return),
        "tab" => Some(enigo::Key::Tab),
        "space" => Some(enigo::Key::Space),
        "backspace" => Some(enigo::Key::Backspace),
        "escape" | "esc" => Some(enigo::Key::Escape),
        "up" => Some(enigo::Key::UpArrow),
        "down" => Some(enigo::Key::DownArrow),
        "left" => Some(enigo::Key::LeftArrow),
        "right" => Some(enigo::Key::RightArrow),
        "shift" => Some(enigo::Key::Shift),
        "control" | "ctrl" => Some(enigo::Key::Control),
        "alt" => Some(enigo::Key::Alt),
        "delete" | "del" => Some(enigo::Key::Delete),
        "home" => Some(enigo::Key::Home),
        "end" => Some(enigo::Key::End),
        "pageup" => Some(enigo::Key::PageUp),
        "pagedown" => Some(enigo::Key::PageDown),
        "capslock" => Some(enigo::Key::CapsLock),
        "f1" => Some(enigo::Key::F1),
        "f2" => Some(enigo::Key::F2),
        "f3" => Some(enigo::Key::F3),
        "f4" => Some(enigo::Key::F4),
        "f5" => Some(enigo::Key::F5),
        "f6" => Some(enigo::Key::F6),
        "f7" => Some(enigo::Key::F7),
        "f8" => Some(enigo::Key::F8),
        "f9" => Some(enigo::Key::F9),
        "f10" => Some(enigo::Key::F10),
        "f11" => Some(enigo::Key::F11),
        "f12" => Some(enigo::Key::F12),
        _ if value.len() == 1 => {
            let c = value.chars().next().unwrap();
            Some(enigo::Key::Unicode(c))
        }
        _ => None,
    }
}

#[derive(Serialize)]
struct ScreenshotResponse {
    success: bool,
    data: String,  // base64 PNG
    width: u32,
    height: u32,
    error: Option<String>,
}

async fn screenshot_handler() -> Json<ScreenshotResponse> {
    let monitors = xcap::Monitor::all().map_err(|e| e.to_string());
    match monitors {
        Ok(monitors) => {
            if let Some(monitor) = monitors.first() {
                match monitor.capture_image() {
                    Ok(image) => {
                        let (w, h) = (image.width(), image.height());
                        let mut buf = std::io::Cursor::new(Vec::new());
                        let result = image.write_to(&mut buf, image::ImageFormat::Png);
                        match result {
                            Ok(_) => {
                                let b64 = base64::Engine::encode(
                                    &base64::engine::general_purpose::STANDARD,
                                    buf.get_ref(),
                                );
                                Json(ScreenshotResponse {
                                    success: true,
                                    data: b64,
                                    width: w,
                                    height: h,
                                    error: None,
                                })
                            }
                            Err(e) => Json(ScreenshotResponse {
                                success: false,
                                data: String::new(),
                                width: 0,
                                height: 0,
                                error: Some(format!("encode failed: {e}")),
                            }),
                        }
                    }
                    Err(e) => Json(ScreenshotResponse {
                        success: false,
                        data: String::new(),
                        width: 0,
                        height: 0,
                        error: Some(format!("capture failed: {e}")),
                    }),
                }
            } else {
                Json(ScreenshotResponse {
                    success: false,
                    data: String::new(),
                    width: 0,
                    height: 0,
                    error: Some("no monitor found".into()),
                })
            }
        }
        Err(e) => Json(ScreenshotResponse {
            success: false,
            data: String::new(),
            width: 0,
            height: 0,
            error: Some(e),
        }),
    }
}

#[derive(Deserialize)]
struct ClickRequest {
    x: Option<i32>,
    y: Option<i32>,
    button: Option<String>,
    double: Option<bool>,
}

async fn desktop_click_handler(ExtractJson(payload): ExtractJson<ClickRequest>) -> Json<serde_json::Value> {
    let Ok(mut enigo) = create_enigo() else {
        return Json(json!({"success": false, "error": "enigo init failed"}));
    };
    let _ = (payload.x, payload.y);
    let btn = match payload.button.as_deref() {
        Some("right") => Button::Right,
        _ => Button::Left,
    };
    let _ = enigo.button(btn, Direction::Click);
    if payload.double.unwrap_or(false) {
        let _ = enigo.button(btn, Direction::Click);
    }
    Json(json!({"success": true}))
}

#[derive(Deserialize)]
struct TypeRequest {
    text: String,
}

async fn desktop_type_handler(ExtractJson(payload): ExtractJson<TypeRequest>) -> Json<serde_json::Value> {
    let Ok(mut enigo) = create_enigo() else {
        return Json(json!({"success": false, "error": "enigo init failed"}));
    };
    let _ = enigo.text(&payload.text);
    Json(json!({"success": true}))
}

#[derive(Deserialize)]
struct HotkeyRequest {
    keys: Vec<String>,
}

async fn desktop_hotkey_handler(ExtractJson(payload): ExtractJson<HotkeyRequest>) -> Json<serde_json::Value> {
    let Ok(mut enigo) = create_enigo() else {
        return Json(json!({"success": false, "error": "enigo init failed"}));
    };
    for key_str in &payload.keys {
        let _ = enigo.text(key_str);
    }
    Json(json!({"success": true}))
}

#[derive(Deserialize)]
struct ScrollRequest {
    x: i32,
    y: i32,
    delta_x: Option<i32>,
    delta_y: Option<i32>,
}

async fn desktop_scroll_handler(ExtractJson(payload): ExtractJson<ScrollRequest>) -> Json<serde_json::Value> {
    let Ok(mut enigo) = create_enigo() else {
        return Json(json!({"success": false, "error": "enigo init failed"}));
    };
    let dx = payload.delta_x.unwrap_or(0);
    let dy = payload.delta_y.unwrap_or(-3);
    let _ = enigo.scroll(dy, enigo::Axis::Vertical);
    Json(json!({"success": true}))
}

#[derive(Serialize)]
struct ScreenInfo {
    success: bool,
    width: i32,
    height: i32,
    count: usize,
    error: Option<String>,
}

async fn screen_info_handler() -> Json<ScreenInfo> {
    match xcap::Monitor::all() {
        Ok(monitors) => {
            if let Some(m) = monitors.first() {
                Json(ScreenInfo {
                    success: true,
                    width: m.width().unwrap_or(0) as i32,
                    height: m.height().unwrap_or(0) as i32,
                    count: monitors.len(),
                    error: None,
                })
            } else {
                Json(ScreenInfo {
                    success: false,
                    width: 0,
                    height: 0,
                    count: 0,
                    error: Some("no monitor".into()),
                })
            }
        }
        Err(e) => Json(ScreenInfo {
            success: false,
            width: 0,
            height: 0,
            count: 0,
            error: Some(e.to_string()),
        }),
    }
}
