use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};

use crate::config::Config;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

static HAPI: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

fn state() -> &'static Mutex<Option<Child>> {
    HAPI.get_or_init(|| Mutex::new(None))
}

pub fn restart(config: &Config) -> Result<String, String> {
    stop();

    let script =
        server_script_path().ok_or_else(|| "hapi-server/server.mjs was not found".to_string())?;
    let workdir = script
        .parent()
        .ok_or_else(|| "Invalid HAPI server path".to_string())?;
    let port = hapi_port(config);
    let mut command = Command::new("node");
    command
        .arg(&script)
        .current_dir(workdir)
        .env("HAPI_HOST", "127.0.0.1")
        .env("HAPI_PORT", port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        command.creation_flags(0x08000000);
    }

    let child = command
        .spawn()
        .map_err(|error| format!("Failed to start embedded HAPI server: {error}"))?;

    *state()
        .lock()
        .map_err(|_| "HAPI server state poisoned".to_string())? = Some(child);

    Ok(format!(
        "Embedded HAPI server running at http://127.0.0.1:{port}"
    ))
}

pub fn stop() {
    let mut guard = match state().lock() {
        Ok(value) => value,
        Err(_) => return,
    };

    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn hapi_port(config: &Config) -> u16 {
    let endpoint = config.ai_group_hapi_endpoint.trim();
    endpoint
        .rsplit_once(':')
        .and_then(|(_, port)| port.trim_end_matches('/').parse::<u16>().ok())
        .unwrap_or(8766)
}

fn server_script_path() -> Option<PathBuf> {
    let relative = PathBuf::from("hapi-server").join("server.mjs");

    if let Ok(current_dir) = std::env::current_dir() {
        let candidate = current_dir.join(&relative);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    let exe = std::env::current_exe().ok()?;
    for ancestor in exe.ancestors() {
        let candidate = ancestor.join(&relative);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}
