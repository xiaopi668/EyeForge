use serde::Deserialize;
use tokio::time::{sleep, Duration, Instant};

const ILINK_BASE: &str = "https://ilinkai.weixin.qq.com";

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

    let image_bytes = client
        .get(&qr.qrcode_img_content)
        .send()
        .await
        .map_err(|error| format!("Failed to download WeChat QR image: {error}"))?
        .error_for_status()
        .map_err(|error| format!("WeChat QR image returned an error: {error}"))?
        .bytes()
        .await
        .map_err(|error| format!("Failed to read WeChat QR image: {error}"))?
        .to_vec();

    Ok(QrLoginSession {
        key: qr.qrcode,
        image_bytes,
    })
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
