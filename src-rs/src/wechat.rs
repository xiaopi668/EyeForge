use serde::Deserialize;
use serde_json::{json, Value};
use std::io::Cursor;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration, Instant};

use crate::config::Config;
use crate::runtime as task_runtime;

const ILINK_BASE: &str = "https://ilinkai.weixin.qq.com";
const QR_REQUEST_TIMEOUT_SECS: u64 = 20;
const QR_STATUS_TIMEOUT_SECS: u64 = 35;
const ILINK_CHANNEL_VERSION: &str = "2.0.0";
const ILINK_POLL_TIMEOUT_SECS: u64 = 40;
const ILINK_REPLY_CHARS: usize = 1800;

struct WechatHandle {
    shutdown: oneshot::Sender<()>,
    thread: JoinHandle<()>,
}

static WECHAT: OnceLock<Mutex<Option<WechatHandle>>> = OnceLock::new();

fn worker_state() -> &'static Mutex<Option<WechatHandle>> {
    WECHAT.get_or_init(|| Mutex::new(None))
}

#[derive(Debug, Clone)]
pub struct QrLoginSession {
    pub key: String,
    pub image_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct QrLoginResult {
    pub token: String,
    pub bot_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone)]
pub struct QrLoginStatus {
    pub status: String,
    pub token: String,
    pub bot_id: String,
    pub user_id: String,
}

#[derive(Debug)]
struct IlinkMessage {
    from_user_id: String,
    context_token: String,
    text: String,
}

pub fn restart(config: &Config) -> Result<String, String> {
    stop();

    log_ilink(&format!(
        "restart requested; config_path={}; enabled={}; token_present={}",
        Config::path().display(),
        config.wc_enabled,
        !config.wc_token.trim().is_empty()
    ));

    if !config.wc_enabled {
        log_ilink("channel disabled");
        return Ok("WeChat iLink channel disabled".into());
    }

    if config.wc_token.trim().is_empty() {
        log_ilink("channel enabled but token missing");
        return Ok("WeChat iLink enabled but Bot Token is missing".into());
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let config_clone = config.clone();

    let thread = std::thread::spawn(move || {
        let runtime = match Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("Failed to create WeChat iLink runtime: {error}");
                return;
            }
        };

        runtime.block_on(async move {
            if let Err(error) = run_message_channel(config_clone, shutdown_rx).await {
                eprintln!("WeChat iLink channel stopped with error: {error}");
            }
        });
    });

    *worker_state()
        .lock()
        .map_err(|_| "wechat worker state poisoned".to_string())? = Some(WechatHandle {
        shutdown: shutdown_tx,
        thread,
    });

    log_ilink("message receiver started");
    Ok("WeChat iLink message receiver started".into())
}

pub fn stop() {
    let mut guard = match worker_state().lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    if let Some(handle) = guard.take() {
        let _ = handle.shutdown.send(());
        std::thread::spawn(move || {
            let _ = handle.thread.join();
            log_ilink("message receiver stopped");
        });
    }
}

#[derive(Debug, Deserialize)]
struct QrCodeResponse {
    #[serde(default)]
    qrcode: String,
    #[serde(default)]
    qrcode_img_content: String,
}

pub async fn begin_qr_login() -> Result<QrLoginSession, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(QR_REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|error| format!("Failed to create WeChat client: {error}"))?;

    let payload: Value = client
        .get(format!("{ILINK_BASE}/ilink/bot/get_bot_qrcode?bot_type=3"))
        .send()
        .await
        .map_err(|error| format!("Failed to request WeChat QR code: {error}"))?
        .error_for_status()
        .map_err(|error| format!("WeChat QR endpoint returned an error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Failed to decode WeChat QR response: {error}"))?;
    let qr = parse_qr_code_response(&payload)?;

    let image_bytes = load_qr_image(&client, &qr)
        .await
        .map_err(|error| format!("Failed to prepare WeChat QR image: {error}"))?;

    Ok(QrLoginSession {
        key: qr.qrcode,
        image_bytes,
    })
}

async fn load_qr_image(client: &reqwest::Client, qr: &QrCodeResponse) -> Result<Vec<u8>, String> {
    let content = qr.qrcode_img_content.trim();

    if let Some(data) = content.strip_prefix("data:image") {
        if let Some((_, payload)) = data.split_once(',') {
            return decode_base64(payload);
        }
    }

    if content.starts_with("http://") || content.starts_with("https://") {
        if let Ok(bytes) = download_image(client, content).await {
            if is_supported_image(&bytes) {
                return Ok(bytes);
            }
        }

        return render_qr_png(content);
    }

    if !content.is_empty() {
        if let Ok(bytes) = decode_base64(content) {
            if is_supported_image(&bytes) {
                return Ok(bytes);
            }
        }

        return render_qr_png(content);
    }

    render_qr_png(&qr.qrcode)
}

async fn download_image(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("download failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("endpoint returned an error: {error}"))?
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|error| format!("failed to read image bytes: {error}"))
}

fn decode_base64(value: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;

    base64::engine::general_purpose::STANDARD
        .decode(value.trim())
        .map_err(|error| format!("base64 decode failed: {error}"))
}

fn is_supported_image(bytes: &[u8]) -> bool {
    image::guess_format(bytes)
        .map(|format| matches!(format, image::ImageFormat::Png | image::ImageFormat::Jpeg))
        .unwrap_or(false)
}

fn render_qr_png(content: &str) -> Result<Vec<u8>, String> {
    let code = qrcode::QrCode::new(content.as_bytes())
        .map_err(|error| format!("failed to encode QR content: {error}"))?;
    let image = code
        .render::<image::Luma<u8>>()
        .min_dimensions(260, 260)
        .quiet_zone(true)
        .build();

    let mut cursor = Cursor::new(Vec::new());
    image::DynamicImage::ImageLuma8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|error| format!("failed to render QR image: {error}"))?;

    Ok(cursor.into_inner())
}

pub async fn wait_for_qr_confirmation(qr_key: &str) -> Result<QrLoginResult, String> {
    let deadline = Instant::now() + Duration::from_secs(180);

    loop {
        if Instant::now() >= deadline {
            return Err("WeChat QR login timed out after 180 seconds".into());
        }

        let status = query_qr_status(qr_key).await?;

        match status.status.as_str() {
            "confirmed" => {
                if status.token.trim().is_empty() {
                    return Err("WeChat QR login completed without a bot token".into());
                }

                return Ok(QrLoginResult {
                    token: status.token,
                    bot_id: status.bot_id,
                    user_id: status.user_id,
                });
            }
            "expired" => return Err("WeChat QR code expired, please try again".into()),
            _ => sleep(Duration::from_millis(1500)).await,
        }
    }
}

pub async fn query_qr_status(qr_key: &str) -> Result<QrLoginStatus, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(QR_STATUS_TIMEOUT_SECS))
        .build()
        .map_err(|error| format!("Failed to create WeChat client: {error}"))?;
    let response = client
        .get(format!(
            "{ILINK_BASE}/ilink/bot/get_qrcode_status?qrcode={}",
            qr_key
        ))
        .header("iLink-App-ClientVersion", "1")
        .send()
        .await
        .map_err(|error| {
            if error.is_timeout() {
                "timeout".to_string()
            } else {
                format!("Failed to poll WeChat QR status: {error}")
            }
        });
    let response = match response {
        Ok(response) => response,
        Err(error) if error == "timeout" => {
            return Ok(QrLoginStatus {
                status: "wait".into(),
                token: String::new(),
                bot_id: String::new(),
                user_id: String::new(),
            });
        }
        Err(error) => return Err(error),
    };

    let payload: Value = response
        .error_for_status()
        .map_err(|error| format!("WeChat QR status returned an error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Failed to decode WeChat QR status: {error}"))?;

    parse_qr_status_response(&payload)
}

async fn run_message_channel(
    config: Config,
    mut shutdown: oneshot::Receiver<()>,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(ILINK_POLL_TIMEOUT_SECS + 5))
        .build()
        .map_err(|error| format!("Failed to create WeChat iLink client: {error}"))?;
    let token = config.wc_token.trim().to_string();
    let mut buffer = String::new();
    log_ilink("poll loop entered");

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                log_ilink("poll loop shutdown requested");
                return Ok(());
            },
            result = poll_messages(&client, &token, &buffer) => {
                match result {
                    Ok((next_buffer, messages)) => {
                        if !messages.is_empty() {
                            log_ilink(&format!("received {} message(s)", messages.len()));
                        }
                        buffer = next_buffer;
                        for message in messages {
                            handle_incoming_message(&client, &token, &config, message).await;
                        }
                    }
                    Err(error) => {
                        log_ilink(&format!("polling error: {error}"));
                        eprintln!("WeChat iLink polling error: {error}");
                        sleep(Duration::from_secs(3)).await;
                    }
                }
            }
        }
    }
}

async fn poll_messages(
    client: &reqwest::Client,
    token: &str,
    buffer: &str,
) -> Result<(String, Vec<IlinkMessage>), String> {
    let request_body = json!({
        "get_updates_buf": buffer,
        "buf": buffer,
        "base_info": {
            "channel_version": ILINK_CHANNEL_VERSION,
        }
    });
    log_ilink(&format!("poll request; buffer_len={}", buffer.len()));

    let response = client
        .post(format!("{ILINK_BASE}/ilink/bot/getupdates"))
        .headers(ilink_headers(token)?)
        .json(&request_body)
        .send()
        .await
        .map_err(|error| {
            if error.is_timeout() {
                "timeout".to_string()
            } else {
                format!("Failed to poll WeChat messages: {error}")
            }
        });

    let response = match response {
        Ok(response) => response,
        Err(error) if error == "timeout" => {
            log_ilink("poll timeout; continue waiting");
            return Ok((buffer.to_string(), Vec::new()));
        }
        Err(error) => return Err(error),
    };

    let payload: Value = response
        .error_for_status()
        .map_err(|error| format!("WeChat message poll returned an error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Failed to decode WeChat message poll response: {error}"))?;

    let ret = payload.get("ret").and_then(Value::as_i64).unwrap_or(0);
    if ret != 0 {
        return Err(format!("WeChat message poll ret={ret}: {payload}"));
    }

    let body = payload.get("data").unwrap_or(&payload);
    let next_buffer = json_string(body, "get_updates_buf")
        .or_else(|| json_string(body, "buf"))
        .unwrap_or_else(|| buffer.to_string());
    let messages = parse_ilink_messages(body);
    log_ilink(&format!(
        "poll response; ret={ret}; buffer_len={}=>{}; message_count={}; keys={}",
        buffer.len(),
        next_buffer.len(),
        messages.len(),
        json_object_keys(body)
    ));
    if messages.is_empty() {
        log_ilink(&format!("poll response preview: {}", redact_json(body)));
    }
    Ok((next_buffer, messages))
}

async fn handle_incoming_message(
    client: &reqwest::Client,
    token: &str,
    config: &Config,
    message: IlinkMessage,
) {
    log_ilink(&format!(
        "handling message from {}: {}",
        message.from_user_id,
        redact_text(&message.text)
    ));
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<task_runtime::RuntimeEvent>();
    let event_client = client.clone();
    let event_token = token.to_string();
    let event_user = message.from_user_id.clone();
    let event_context = message.context_token.clone();
    let event_forwarder = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let text = format!("【{}】{}", event.title, event.message);
            if let Err(error) = send_message(
                &event_client,
                &event_token,
                &event_user,
                &event_context,
                &text,
            )
            .await
            {
                log_ilink(&format!("event push failed: {error}"));
            }
        }
    });

    let mut reporter = move |event| {
        let _ = event_tx.send(event);
    };
    let result =
        task_runtime::execute_task_with_events(message.text.clone(), config.clone(), &mut reporter)
            .await;
    drop(reporter);
    let _ = event_forwarder.await;
    let reply = match result {
        Ok(outcome) => outcome.message,
        Err(error) => format!("执行失败：{error}"),
    };

    for part in split_reply(&reply) {
        if let Err(error) = send_message(
            client,
            token,
            &message.from_user_id,
            &message.context_token,
            &part,
        )
        .await
        {
            log_ilink(&format!("reply failed: {error}"));
            eprintln!("Failed to reply to WeChat iLink message: {error}");
        } else {
            log_ilink("reply sent");
        }
    }
}

async fn send_message(
    client: &reqwest::Client,
    token: &str,
    to_user_id: &str,
    context_token: &str,
    text: &str,
) -> Result<(), String> {
    let payload = json!({
        "msg": {
            "from_user_id": "",
            "to_user_id": to_user_id,
            "client_id": client_id(),
            "message_type": 2,
            "message_state": 2,
            "context_token": context_token,
            "item_list": [
                {
                    "type": 1,
                    "text_item": {
                        "text": text,
                    },
                    "content": text,
                }
            ],
        },
        "base_info": {
            "channel_version": ILINK_CHANNEL_VERSION,
        }
    });

    let response: Value = client
        .post(format!("{ILINK_BASE}/ilink/bot/sendmessage"))
        .headers(ilink_headers(token)?)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to send WeChat reply: {error}"))?
        .error_for_status()
        .map_err(|error| format!("WeChat reply endpoint returned an error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Failed to decode WeChat reply response: {error}"))?;

    let ret = response.get("ret").and_then(Value::as_i64).unwrap_or(0);
    if ret != 0 {
        return Err(format!("WeChat reply ret={ret}: {response}"));
    }

    Ok(())
}

fn parse_ilink_messages(body: &Value) -> Vec<IlinkMessage> {
    let mut parsed = Vec::new();

    for key in ["msgs", "msg_list", "messages", "updates", "items", "data"] {
        if let Some(messages) = body.get(key).and_then(Value::as_array) {
            parsed.extend(messages.iter().filter_map(parse_ilink_message));
        }
    }

    if let Some(message) = body.get("msg").or_else(|| body.get("message")) {
        if let Some(parsed_message) = parse_ilink_message(message) {
            parsed.push(parsed_message);
        }
    }

    parsed
}

fn parse_ilink_message(message: &Value) -> Option<IlinkMessage> {
    let message = message.get("msg").unwrap_or(message);
    let message_type = json_i64(message, "message_type")
        .or_else(|| json_i64(message, "msg_type"))
        .or_else(|| json_i64(message, "type"))
        .unwrap_or(1);
    let text = extract_message_text(message);
    if text.trim().is_empty() || (message_type != 1 && message_type != 2) {
        return None;
    }

    let from_user_id = first_json_string(
        message,
        &[
            "from_user_id",
            "from_userid",
            "from",
            "sender",
            "sender_id",
            "user_id",
        ],
    )?;
    let context_token = first_json_string(
        message,
        &[
            "context_token",
            "contextToken",
            "chat_context_token",
            "session_token",
        ],
    )
    .unwrap_or_default();

    Some(IlinkMessage {
        from_user_id,
        context_token,
        text: text.trim().to_string(),
    })
}

fn extract_message_text(message: &Value) -> String {
    if let Some(text) = first_json_string(
        message,
        &["content", "text", "text_content", "message", "msg_content"],
    ) {
        return text;
    }

    ["item_list", "items", "content_list"]
        .iter()
        .filter_map(|key| message.get(key).and_then(Value::as_array))
        .flatten()
        .filter_map(|item| {
            if let Some(text) = item
                .get("text_item")
                .and_then(|text_item| first_json_string(text_item, &["text", "content"]))
            {
                return Some(text);
            }

            first_json_string(
                item,
                &["content", "text", "text_content", "value", "message"],
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn ilink_headers(token: &str) -> Result<reqwest::header::HeaderMap, String> {
    use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "AuthorizationType",
        HeaderValue::from_static("ilink_bot_token"),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&authorization_header(token))
            .map_err(|error| format!("Invalid WeChat token: {error}"))?,
    );
    headers.insert(
        "iLink-App-ClientVersion",
        HeaderValue::from_static(ILINK_CHANNEL_VERSION),
    );
    headers.insert(
        "X-WECHAT-UIN",
        HeaderValue::from_str(&wechat_uin())
            .map_err(|error| format!("Invalid generated WeChat UIN: {error}"))?,
    );

    Ok(headers)
}

fn authorization_header(token: &str) -> String {
    let trimmed = token.trim();
    if trimmed.to_ascii_lowercase().starts_with("bearer ") {
        trimmed.to_string()
    } else {
        format!("Bearer {trimmed}")
    }
}

fn parse_qr_code_response(payload: &Value) -> Result<QrCodeResponse, String> {
    let body = payload.get("data").unwrap_or(payload);
    let qr = QrCodeResponse {
        qrcode: json_string(body, "qrcode").unwrap_or_default(),
        qrcode_img_content: json_string(body, "qrcode_img_content")
            .or_else(|| json_string(body, "qrcode_url"))
            .unwrap_or_default(),
    };

    if qr.qrcode.trim().is_empty() {
        return Err(format!(
            "WeChat QR response did not include qrcode: {payload}"
        ));
    }

    Ok(qr)
}

fn parse_qr_status_response(payload: &Value) -> Result<QrLoginStatus, String> {
    let body = payload.get("data").unwrap_or(payload);
    let credentials = body.get("credentials").unwrap_or(body);
    let status = json_string(body, "status").unwrap_or_default();
    let status = if status.trim().is_empty() {
        "wait".into()
    } else {
        status
    };

    Ok(QrLoginStatus {
        status,
        token: json_string(credentials, "bot_token")
            .or_else(|| json_string(body, "bot_token"))
            .unwrap_or_default(),
        bot_id: json_string(credentials, "ilink_bot_id")
            .or_else(|| json_string(body, "ilink_bot_id"))
            .unwrap_or_default(),
        user_id: json_string(credentials, "ilink_user_id")
            .or_else(|| json_string(body, "ilink_user_id"))
            .unwrap_or_default(),
    })
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn first_json_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| json_string(value, key))
}

fn json_i64(value: &Value, key: &str) -> Option<i64> {
    let value = value.get(key)?;
    value
        .as_i64()
        .or_else(|| value.as_str().and_then(|text| text.parse::<i64>().ok()))
}

fn split_reply(text: &str) -> Vec<String> {
    if text.trim().is_empty() {
        return vec!["任务已完成。".into()];
    }

    let mut parts = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if current.chars().count() >= ILINK_REPLY_CHARS {
            parts.push(current);
            current = String::new();
        }
        current.push(ch);
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn client_id() -> String {
    let now = now_millis();
    format!("eyeforge-{now}")
}

fn wechat_uin() -> String {
    use base64::Engine;

    let value = (now_millis() % 10_000_000_000).to_string();
    base64::engine::general_purpose::STANDARD.encode(value.as_bytes())
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn log_ilink(message: &str) {
    let timestamp = now_millis();
    let line = format!("[{timestamp}] {message}\n");
    for path in ilink_log_paths() {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

fn ilink_log_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    if let Some(config_dir) = Config::path().parent() {
        paths.push(config_dir.join("eyeforge-ilink.log"));
    }
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            paths.push(exe_dir.join("eyeforge-ilink.log"));
        }
    }
    if let Ok(current_dir) = std::env::current_dir() {
        paths.push(current_dir.join("eyeforge-ilink.log"));
    }
    paths.dedup();
    paths
}

fn json_object_keys(value: &Value) -> String {
    value
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>().join(","))
        .unwrap_or_else(|| "-".into())
}

fn redact_json(value: &Value) -> String {
    let mut text = value.to_string();
    for key in ["bot_token", "Authorization", "authorization", "wc_token"] {
        text = redact_json_key(&text, key);
    }
    let preview = text.chars().take(600).collect::<String>();
    if text.chars().count() > 600 {
        format!("{preview}...")
    } else {
        preview
    }
}

fn redact_json_key(text: &str, key: &str) -> String {
    let marker = format!("\"{key}\":\"");
    let mut result = String::new();
    let mut rest = text;
    while let Some(index) = rest.find(&marker) {
        result.push_str(&rest[..index + marker.len()]);
        let after = &rest[index + marker.len()..];
        if let Some(end) = after.find('"') {
            result.push_str("***");
            rest = &after[end..];
        } else {
            rest = "";
        }
    }
    result.push_str(rest);
    result
}

fn redact_text(text: &str) -> String {
    let trimmed = text.trim();
    let preview = trimmed.chars().take(80).collect::<String>();
    if trimmed.chars().count() > 80 {
        format!("{preview}...")
    } else {
        preview
    }
}
