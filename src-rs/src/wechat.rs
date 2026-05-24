use serde::Deserialize;
use tokio::time::{sleep, Duration, Instant};

const ILINK_BASE: &str = "https://ilinkai.weixin.qq.com";
const QR_RENDER_BASE: &str = "https://api.qrserver.com/v1/create-qr-code/";

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

#[derive(Debug, Deserialize)]
struct QrCodeResponse {
    qrcode: String,
    qrcode_img_content: String,
}

#[derive(Debug, Deserialize)]
struct QrStatusResponse {
    status: String,
    #[serde(default)]
    bot_token: String,
    #[serde(default)]
    ilink_bot_id: String,
    #[serde(default)]
    ilink_user_id: String,
}

pub async fn begin_qr_login() -> Result<QrLoginSession, String> {
    let client = reqwest::Client::new();

    let qr: QrCodeResponse = client
        .get(format!("{ILINK_BASE}/ilink/bot/get_bot_qrcode?bot_type=3"))
        .send()
        .await
        .map_err(|error| format!("Failed to request WeChat QR code: {error}"))?
        .error_for_status()
        .map_err(|error| format!("WeChat QR endpoint returned an error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Failed to decode WeChat QR response: {error}"))?;

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
            return Ok(bytes);
        }
    }

    if !content.is_empty() {
        if let Ok(bytes) = decode_base64(content) {
            return Ok(bytes);
        }
    }

    let url = reqwest::Url::parse_with_params(
        QR_RENDER_BASE,
        &[
            ("size", "260x260"),
            ("margin", "12"),
            ("data", qr.qrcode.as_str()),
        ],
    )
    .map_err(|error| format!("Failed to build QR renderer URL: {error}"))?;

    download_image(client, url.as_str()).await
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

pub async fn wait_for_qr_confirmation(qr_key: &str) -> Result<QrLoginResult, String> {
    let client = reqwest::Client::new();
    let deadline = Instant::now() + Duration::from_secs(180);

    loop {
        if Instant::now() >= deadline {
            return Err("WeChat QR login timed out after 180 seconds".into());
        }

        let status: QrStatusResponse = client
            .get(format!(
                "{ILINK_BASE}/ilink/bot/get_qrcode_status?qrcode={}",
                qr_key
            ))
            .header("iLink-App-ClientVersion", "1")
            .send()
            .await
            .map_err(|error| format!("Failed to poll WeChat QR status: {error}"))?
            .error_for_status()
            .map_err(|error| format!("WeChat QR status returned an error: {error}"))?
            .json()
            .await
            .map_err(|error| format!("Failed to decode WeChat QR status: {error}"))?;

        match status.status.as_str() {
            "confirmed" => {
                if status.bot_token.trim().is_empty() {
                    return Err("WeChat QR login completed without a bot token".into());
                }

                return Ok(QrLoginResult {
                    token: status.bot_token,
                    bot_id: status.ilink_bot_id,
                    user_id: status.ilink_user_id,
                });
            }
            "expired" => return Err("WeChat QR code expired, please try again".into()),
            _ => sleep(Duration::from_millis(1500)).await,
        }
    }
}
