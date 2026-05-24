use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::crypto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub llm_provider: String,
    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default)]
    pub openai_model: String,
    #[serde(default)]
    pub anthropic_api_key: String,
    #[serde(default)]
    pub anthropic_model: String,
    #[serde(default)]
    pub ollama_base_url: String,
    #[serde(default)]
    pub ollama_model: String,
    #[serde(default)]
    pub custom_api_key: String,
    #[serde(default)]
    pub custom_base_url: String,
    #[serde(default)]
    pub custom_model: String,
    #[serde(default)]
    pub gemini_api_key: String,
    #[serde(default)]
    pub gemini_model: String,
    #[serde(default = "default_screenshot_quality")]
    pub screenshot_quality: u32,
    #[serde(default = "default_action_delay")]
    pub action_delay: f64,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default)]
    pub wizard_done: bool,
    #[serde(default)]
    pub hotkey_float: String,
    #[serde(default)]
    pub hotkey_voice: String,
    #[serde(default)]
    pub wakeword_enabled: bool,
    #[serde(default)]
    pub wakeword_list: String,
    #[serde(default)]
    pub porcupine_access_key: String,
    #[serde(default)]
    pub ws_enabled: bool,
    #[serde(default)]
    pub ws_host: String,
    #[serde(default = "default_ws_port")]
    pub ws_port: u16,
    #[serde(default)]
    pub ws_token: String,
    #[serde(default)]
    pub wc_enabled: bool,
    #[serde(default)]
    pub wc_token: String,
    #[serde(default)]
    pub wcom_enabled: bool,
    #[serde(default)]
    pub wcom_corp_id: String,
    #[serde(default)]
    pub wcom_agent_id: String,
    #[serde(default)]
    pub wcom_secret: String,
    #[serde(default)]
    pub wcom_token: String,
    #[serde(default)]
    pub wcom_aes_key: String,
    #[serde(default)]
    pub dt_enabled: bool,
    #[serde(default)]
    pub dt_app_key: String,
    #[serde(default)]
    pub dt_app_secret: String,
    #[serde(default)]
    pub dt_webhook: String,
    #[serde(default)]
    pub qq_enabled: bool,
    #[serde(default = "default_qq_mode")]
    pub qq_mode: String,
    #[serde(default = "default_qq_ws_host")]
    pub qq_ws_host: String,
    #[serde(default = "default_qq_ws_port")]
    pub qq_ws_port: u16,
    #[serde(default)]
    pub qq_bot_appid: String,
    #[serde(default)]
    pub qq_bot_token: String,
    #[serde(default = "default_use_vision")]
    pub use_vision: bool,
    #[serde(default)]
    pub skills_enabled: Vec<String>,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct EditableConfig {
    pub llm_provider: String,
    pub openai_api_key: String,
    pub openai_model: String,
    pub anthropic_api_key: String,
    pub anthropic_model: String,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub custom_api_key: String,
    pub custom_base_url: String,
    pub custom_model: String,
    pub gemini_api_key: String,
    pub gemini_model: String,
    pub screenshot_quality: String,
    pub action_delay: String,
    pub language: String,
    pub theme: String,
    pub font_size: String,
    pub hotkey_float: String,
    pub hotkey_voice: String,
    pub wakeword_enabled: bool,
    pub wakeword_list: String,
    pub porcupine_access_key: String,
    pub ws_enabled: bool,
    pub ws_host: String,
    pub ws_port: String,
    pub ws_token: String,
    pub wc_enabled: bool,
    pub wc_token: String,
    pub wcom_enabled: bool,
    pub wcom_corp_id: String,
    pub wcom_agent_id: String,
    pub wcom_secret: String,
    pub wcom_token: String,
    pub wcom_aes_key: String,
    pub dt_enabled: bool,
    pub dt_app_key: String,
    pub dt_app_secret: String,
    pub dt_webhook: String,
    pub qq_enabled: bool,
    pub qq_mode: String,
    pub qq_ws_host: String,
    pub qq_ws_port: String,
    pub qq_bot_appid: String,
    pub qq_bot_token: String,
}

fn default_screenshot_quality() -> u32 {
    95
}
fn default_action_delay() -> f64 {
    0.5
}
fn default_language() -> String {
    "zh".into()
}
fn default_theme() -> String {
    "dark".into()
}
fn default_font_size() -> u32 {
    9
}
fn default_ws_port() -> u16 {
    8765
}
fn default_qq_mode() -> String {
    "ws".into()
}
fn default_qq_ws_host() -> String {
    "127.0.0.1".into()
}
fn default_qq_ws_port() -> u16 {
    6700
}
fn default_use_vision() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm_provider: "openai".into(),
            openai_api_key: String::new(),
            openai_model: "gpt-4o".into(),
            anthropic_api_key: String::new(),
            anthropic_model: "claude-3-5-sonnet-20241022".into(),
            ollama_base_url: "http://localhost:11434".into(),
            ollama_model: "llava".into(),
            custom_api_key: String::new(),
            custom_base_url: "https://api.openai.com/v1".into(),
            custom_model: "gpt-4o".into(),
            gemini_api_key: String::new(),
            gemini_model: "gemini-2.5-flash".into(),
            screenshot_quality: 95,
            action_delay: 0.5,
            language: "zh".into(),
            theme: "dark".into(),
            font_size: 9,
            wizard_done: true,
            hotkey_float: "ctrl+shift+e".into(),
            hotkey_voice: "ctrl+shift+v".into(),
            wakeword_enabled: false,
            wakeword_list: "computer".into(),
            porcupine_access_key: String::new(),
            ws_enabled: false,
            ws_host: "0.0.0.0".into(),
            ws_port: 8765,
            ws_token: String::new(),
            wc_enabled: false,
            wc_token: String::new(),
            wcom_enabled: false,
            wcom_corp_id: String::new(),
            wcom_agent_id: String::new(),
            wcom_secret: String::new(),
            wcom_token: String::new(),
            wcom_aes_key: String::new(),
            dt_enabled: false,
            dt_app_key: String::new(),
            dt_app_secret: String::new(),
            dt_webhook: String::new(),
            qq_enabled: false,
            qq_mode: default_qq_mode(),
            qq_ws_host: default_qq_ws_host(),
            qq_ws_port: default_qq_ws_port(),
            qq_bot_appid: String::new(),
            qq_bot_token: String::new(),
            use_vision: true,
            skills_enabled: vec![],
            extra: BTreeMap::new(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Config::default()
        }
    }

    pub fn save(&self) {
        let path = config_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, data);
        }
    }

    pub fn to_editable(&self) -> EditableConfig {
        EditableConfig {
            llm_provider: self.llm_provider.clone(),
            openai_api_key: crypto::decrypt(&self.openai_api_key),
            openai_model: self.openai_model.clone(),
            anthropic_api_key: crypto::decrypt(&self.anthropic_api_key),
            anthropic_model: self.anthropic_model.clone(),
            ollama_base_url: self.ollama_base_url.clone(),
            ollama_model: self.ollama_model.clone(),
            custom_api_key: crypto::decrypt(&self.custom_api_key),
            custom_base_url: self.custom_base_url.clone(),
            custom_model: self.custom_model.clone(),
            gemini_api_key: crypto::decrypt(&self.gemini_api_key),
            gemini_model: self.gemini_model.clone(),
            screenshot_quality: self.screenshot_quality.to_string(),
            action_delay: self.action_delay.to_string(),
            language: self.language.clone(),
            theme: self.theme.clone(),
            font_size: self.font_size.to_string(),
            hotkey_float: self.hotkey_float.clone(),
            hotkey_voice: self.hotkey_voice.clone(),
            wakeword_enabled: self.wakeword_enabled,
            wakeword_list: self.wakeword_list.clone(),
            porcupine_access_key: self.porcupine_access_key.clone(),
            ws_enabled: self.ws_enabled,
            ws_host: self.ws_host.clone(),
            ws_port: self.ws_port.to_string(),
            ws_token: self.ws_token.clone(),
            wc_enabled: self.wc_enabled,
            wc_token: self.wc_token.clone(),
            wcom_enabled: self.wcom_enabled,
            wcom_corp_id: self.wcom_corp_id.clone(),
            wcom_agent_id: self.wcom_agent_id.clone(),
            wcom_secret: self.wcom_secret.clone(),
            wcom_token: self.wcom_token.clone(),
            wcom_aes_key: self.wcom_aes_key.clone(),
            dt_enabled: self.dt_enabled,
            dt_app_key: self.dt_app_key.clone(),
            dt_app_secret: self.dt_app_secret.clone(),
            dt_webhook: self.dt_webhook.clone(),
            qq_enabled: self.qq_enabled,
            qq_mode: self.qq_mode.clone(),
            qq_ws_host: self.qq_ws_host.clone(),
            qq_ws_port: self.qq_ws_port.to_string(),
            qq_bot_appid: self.qq_bot_appid.clone(),
            qq_bot_token: self.qq_bot_token.clone(),
        }
    }

    pub fn apply_editable(&self, editable: &EditableConfig) -> Self {
        let mut next = self.clone();
        next.llm_provider = editable.llm_provider.clone();
        next.openai_api_key = crypto::encrypt(&editable.openai_api_key);
        next.openai_model = editable.openai_model.clone();
        next.anthropic_api_key = crypto::encrypt(&editable.anthropic_api_key);
        next.anthropic_model = editable.anthropic_model.clone();
        next.ollama_base_url = editable.ollama_base_url.clone();
        next.ollama_model = editable.ollama_model.clone();
        next.custom_api_key = crypto::encrypt(&editable.custom_api_key);
        next.custom_base_url = editable.custom_base_url.clone();
        next.custom_model = editable.custom_model.clone();
        next.gemini_api_key = crypto::encrypt(&editable.gemini_api_key);
        next.gemini_model = editable.gemini_model.clone();
        next.screenshot_quality = parse_u32(&editable.screenshot_quality, self.screenshot_quality);
        next.action_delay = parse_f64(&editable.action_delay, self.action_delay);
        next.language = editable.language.clone();
        next.theme = editable.theme.clone();
        next.font_size = parse_u32(&editable.font_size, self.font_size);
        next.hotkey_float = editable.hotkey_float.trim().to_string();
        next.hotkey_voice = editable.hotkey_voice.trim().to_string();
        next.wakeword_enabled = editable.wakeword_enabled;
        next.wakeword_list = editable.wakeword_list.trim().to_string();
        next.porcupine_access_key = editable.porcupine_access_key.trim().to_string();
        next.ws_enabled = editable.ws_enabled;
        next.ws_host = editable.ws_host.trim().to_string();
        next.ws_port = parse_u16(&editable.ws_port, self.ws_port);
        next.ws_token = editable.ws_token.trim().to_string();
        next.wc_enabled = editable.wc_enabled;
        next.wc_token = editable.wc_token.trim().to_string();
        next.wcom_enabled = editable.wcom_enabled;
        next.wcom_corp_id = editable.wcom_corp_id.trim().to_string();
        next.wcom_agent_id = editable.wcom_agent_id.trim().to_string();
        next.wcom_secret = editable.wcom_secret.trim().to_string();
        next.wcom_token = editable.wcom_token.trim().to_string();
        next.wcom_aes_key = editable.wcom_aes_key.trim().to_string();
        next.dt_enabled = editable.dt_enabled;
        next.dt_app_key = editable.dt_app_key.trim().to_string();
        next.dt_app_secret = editable.dt_app_secret.trim().to_string();
        next.dt_webhook = editable.dt_webhook.trim().to_string();
        next.qq_enabled = editable.qq_enabled;
        next.qq_mode = editable.qq_mode.clone();
        next.qq_ws_host = editable.qq_ws_host.trim().to_string();
        next.qq_ws_port = parse_u16(&editable.qq_ws_port, self.qq_ws_port);
        next.qq_bot_appid = editable.qq_bot_appid.trim().to_string();
        next.qq_bot_token = editable.qq_bot_token.trim().to_string();
        next
    }
}

fn config_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("config.json")
}

fn parse_u32(value: &str, fallback: u32) -> u32 {
    value.trim().parse().unwrap_or(fallback)
}

fn parse_u16(value: &str, fallback: u16) -> u16 {
    value.trim().parse().unwrap_or(fallback)
}

fn parse_f64(value: &str, fallback: f64) -> f64 {
    value.trim().parse().unwrap_or(fallback)
}
