use serde::Serialize;

use crate::config::Config;

#[derive(Debug, Clone, Serialize)]
pub struct ChannelStatus {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub configured: bool,
    pub status: String,
    pub detail: String,
}

pub fn collect(config: &Config) -> Vec<ChannelStatus> {
    vec![
        ws_gateway(config),
        wechat(config),
        wecom(config),
        dingtalk(config),
        qq(config),
    ]
}

fn ws_gateway(_config: &Config) -> ChannelStatus {
    ChannelStatus {
        id: "gateway".into(),
        label: "Rust Gateway".into(),
        enabled: true,
        configured: true,
        status: "online".into(),
        detail: format!("http://127.0.0.1:9178/ and ws://127.0.0.1:9178/ws"),
    }
}

fn wechat(config: &Config) -> ChannelStatus {
    let configured = !config.wc_token.trim().is_empty();
    ChannelStatus {
        id: "wechat".into(),
        label: "WeChat iLink".into(),
        enabled: config.wc_enabled,
        configured,
        status: resolve_status(config.wc_enabled, configured),
        detail: if configured {
            "Token configured. Rust transport pending.".into()
        } else {
            "Missing bot token.".into()
        },
    }
}

fn wecom(config: &Config) -> ChannelStatus {
    let configured = !config.wcom_corp_id.trim().is_empty()
        && !config.wcom_agent_id.trim().is_empty()
        && !config.wcom_secret.trim().is_empty();
    ChannelStatus {
        id: "wecom".into(),
        label: "WeCom".into(),
        enabled: config.wcom_enabled,
        configured,
        status: resolve_status(config.wcom_enabled, configured),
        detail: if configured {
            "Credentials configured. Rust transport pending.".into()
        } else {
            "Missing corp ID / agent ID / secret.".into()
        },
    }
}

fn dingtalk(config: &Config) -> ChannelStatus {
    let configured = !config.dt_app_key.trim().is_empty()
        && !config.dt_app_secret.trim().is_empty()
        && !config.dt_webhook.trim().is_empty();
    ChannelStatus {
        id: "dingtalk".into(),
        label: "DingTalk".into(),
        enabled: config.dt_enabled,
        configured,
        status: resolve_status(config.dt_enabled, configured),
        detail: if configured {
            "Webhook and app credentials configured. Rust transport pending.".into()
        } else {
            "Missing app key / app secret / webhook.".into()
        },
    }
}

fn qq(config: &Config) -> ChannelStatus {
    let configured = if config.qq_mode == "official" {
        !config.qq_bot_appid.trim().is_empty() && !config.qq_bot_token.trim().is_empty()
    } else {
        !config.qq_ws_host.trim().is_empty() && config.qq_ws_port > 0
    };

    ChannelStatus {
        id: "qq".into(),
        label: "QQ".into(),
        enabled: config.qq_enabled,
        configured,
        status: resolve_status(config.qq_enabled, configured),
        detail: if config.qq_mode == "official" {
            if configured {
                "Official Bot credentials configured. Rust transport pending.".into()
            } else {
                "Missing QQ official bot credentials.".into()
            }
        } else if configured {
            format!(
                "Reverse WebSocket configured at {}:{}",
                config.qq_ws_host, config.qq_ws_port
            )
        } else {
            "Missing QQ reverse WebSocket host/port.".into()
        },
    }
}

fn resolve_status(enabled: bool, configured: bool) -> String {
    if !enabled {
        "disabled".into()
    } else if configured {
        "configured".into()
    } else {
        "needs_config".into()
    }
}
