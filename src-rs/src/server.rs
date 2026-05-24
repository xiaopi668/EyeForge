use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json as ExtractJson, State,
    },
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

use crate::config::Config;
use crate::{channels, runtime, voice};

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
        .route("/api/voice/devices", get(voice_devices))
        .route(
            "/api/voice/transcribe",
            axum::routing::post(voice_transcribe),
        )
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

async fn index() -> Html<&'static str> {
    Html(include_str!("../../web-ui/index.html"))
}

async fn app_js() -> Response {
    let mut response = Response::new(Body::from(include_str!("../../web-ui/app.js")));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    response
}

async fn styles_css() -> Response {
    let mut response = Response::new(Body::from(include_str!("../../web-ui/styles.css")));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/css; charset=utf-8"),
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

                        let result = runtime::execute_task(task, (*state.config).clone()).await;
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
