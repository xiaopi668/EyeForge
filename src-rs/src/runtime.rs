use std::io::Cursor;

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use enigo::{
    Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings as EnigoSettings,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;
use tokio::task::spawn_blocking;
use tokio::time::{sleep, timeout, Duration};
use xcap::{image::DynamicImage, image::ImageFormat, Monitor};

use crate::config::Config;
use crate::llm;

#[derive(Debug, Clone)]
pub struct NativeOutcome {
    pub status: String,
    pub message: String,
    pub transcript: Vec<String>,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ActionEnvelope {
    actions: Vec<ActionSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ActionSpec {
    Shell {
        command: String,
        timeout_secs: Option<u64>,
    },
    Wait {
        seconds: f64,
    },
    Open {
        target: String,
    },
    Click {
        x: i32,
        y: i32,
        button: Option<String>,
        clicks: Option<u32>,
    },
    #[serde(rename = "type")]
    TypeText {
        text: String,
    },
    Hotkey {
        keys: Vec<String>,
    },
    Scroll {
        clicks: i32,
        axis: Option<String>,
    },
    Screenshot {
        x: Option<u32>,
        y: Option<u32>,
        width: Option<u32>,
        height: Option<u32>,
    },
    Complete {
        result: Option<String>,
    },
}

pub async fn execute_task(task: String, config: Config) -> Result<NativeOutcome, String> {
    let trimmed = task.trim().to_string();
    let mut transcript = vec![format!("Task received: {trimmed}")];

    if looks_like_json(&trimmed) {
        let actions = parse_action_specs(&trimmed)?;
        transcript.push(format!(
            "Parsed structured action plan with {} step(s)",
            actions.len()
        ));
        return execute_action_plan(actions, &config, transcript).await;
    }

    if let Some(command) = trimmed.strip_prefix("shell:") {
        let message = execute_shell(command.trim(), 30, &config, &mut transcript).await?;
        return Ok(NativeOutcome {
            status: "success".into(),
            message,
            transcript,
            data: None,
        });
    }

    if let Some(duration) = trimmed.strip_prefix("wait:") {
        let seconds = parse_wait_seconds(duration.trim(), &config)?;
        let message = execute_wait(seconds, &config, &mut transcript).await?;
        return Ok(NativeOutcome {
            status: "success".into(),
            message,
            transcript,
            data: None,
        });
    }

    if let Some(target) = trimmed.strip_prefix("open:") {
        let message = execute_open(target.trim(), &config, &mut transcript).await?;
        return Ok(NativeOutcome {
            status: "success".into(),
            message,
            transcript,
            data: None,
        });
    }

    transcript.push(format!(
        "Planning with Rust-native LLM provider: {}",
        config.llm_provider
    ));

    let llm_plan = llm::plan_actions(&trimmed, &config).await?;
    transcript.push(format!("LLM plan received:\n{llm_plan}"));
    let actions = parse_action_specs(&llm_plan)?;
    transcript.push(format!(
        "Compiled LLM plan into {} executable action(s)",
        actions.len()
    ));
    return execute_action_plan(actions, &config, transcript).await;
}

async fn execute_action_plan(
    actions: Vec<ActionSpec>,
    config: &Config,
    mut transcript: Vec<String>,
) -> Result<NativeOutcome, String> {
    let mut last_message = if config.language == "zh" {
        "动作计划已执行".to_string()
    } else {
        "Action plan executed".to_string()
    };
    let mut last_data: Option<Value> = None;

    for (index, action) in actions.into_iter().enumerate() {
        transcript.push(format!("Action {} => {:?}", index + 1, action));

        match action {
            ActionSpec::Shell {
                command,
                timeout_secs,
            } => {
                last_message = execute_shell(
                    &command,
                    timeout_secs.unwrap_or(30),
                    config,
                    &mut transcript,
                )
                .await?;
            }
            ActionSpec::Wait { seconds } => {
                last_message = execute_wait(seconds, config, &mut transcript).await?;
            }
            ActionSpec::Open { target } => {
                last_message = execute_open(&target, config, &mut transcript).await?;
            }
            ActionSpec::Click {
                x,
                y,
                button,
                clicks,
            } => {
                last_message = execute_click(
                    x,
                    y,
                    button.as_deref(),
                    clicks.unwrap_or(1),
                    &mut transcript,
                )
                .await?;
            }
            ActionSpec::TypeText { text } => {
                last_message = execute_type_text(&text, &mut transcript).await?;
            }
            ActionSpec::Hotkey { keys } => {
                last_message = execute_hotkey(&keys, &mut transcript).await?;
            }
            ActionSpec::Scroll { clicks, axis } => {
                last_message = execute_scroll(clicks, axis.as_deref(), &mut transcript).await?;
            }
            ActionSpec::Screenshot {
                x,
                y,
                width,
                height,
            } => {
                let screenshot = execute_screenshot(x, y, width, height, &mut transcript).await?;
                last_message = screenshot.0;
                last_data = Some(screenshot.1);
            }
            ActionSpec::Complete { result } => {
                return Ok(NativeOutcome {
                    status: "success".into(),
                    message: result.unwrap_or(last_message),
                    transcript,
                    data: last_data,
                });
            }
        }
    }

    Ok(NativeOutcome {
        status: "success".into(),
        message: last_message,
        transcript,
        data: last_data,
    })
}

fn parse_action_specs(input: &str) -> Result<Vec<ActionSpec>, String> {
    serde_json::from_str::<ActionEnvelope>(input)
        .map(|envelope| envelope.actions)
        .or_else(|_| serde_json::from_str::<Vec<ActionSpec>>(input))
        .or_else(|_| serde_json::from_str::<ActionSpec>(input).map(|action| vec![action]))
        .map_err(|error| format!("Invalid action JSON: {error}"))
}

fn looks_like_json(input: &str) -> bool {
    input.starts_with('{') || input.starts_with('[')
}

async fn execute_shell(
    command: &str,
    timeout_secs: u64,
    config: &Config,
    transcript: &mut Vec<String>,
) -> Result<String, String> {
    if command.is_empty() {
        return Err(if config.language == "zh" {
            "shell: 后面缺少命令".into()
        } else {
            "Missing command after shell:".into()
        });
    }

    transcript.push(format!("Executing shell command: {command}"));

    let output = timeout(Duration::from_secs(timeout_secs), run_system_shell(command))
        .await
        .map_err(|_| {
            if config.language == "zh" {
                "Shell 命令执行超时".to_string()
            } else {
                "Shell command timed out".to_string()
            }
        })?
        .map_err(|error| error.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !stdout.is_empty() {
        transcript.push(format!("stdout:\n{stdout}"));
    }
    if !stderr.is_empty() {
        transcript.push(format!("stderr:\n{stderr}"));
    }

    if output.status.success() {
        Ok(if stdout.is_empty() {
            "Shell command completed".into()
        } else {
            stdout
        })
    } else {
        Err(if stderr.is_empty() {
            format!(
                "Shell command failed with exit code {:?}",
                output.status.code()
            )
        } else {
            stderr
        })
    }
}

fn parse_wait_seconds(duration: &str, config: &Config) -> Result<f64, String> {
    duration.parse().map_err(|_| {
        if config.language == "zh" {
            "wait: 需要数字秒数，例如 wait:1.5".to_string()
        } else {
            "wait: expects a numeric duration like wait:1.5".to_string()
        }
    })
}

async fn execute_wait(
    seconds: f64,
    _config: &Config,
    transcript: &mut Vec<String>,
) -> Result<String, String> {
    transcript.push(format!("Waiting for {seconds} seconds"));
    sleep(Duration::from_secs_f64(seconds.max(0.0))).await;
    Ok(format!("Waited {seconds} seconds"))
}

async fn execute_open(
    target: &str,
    config: &Config,
    transcript: &mut Vec<String>,
) -> Result<String, String> {
    if target.is_empty() {
        return Err(if config.language == "zh" {
            "open: 后面缺少目标".into()
        } else {
            "Missing target after open:".into()
        });
    }

    transcript.push(format!("Opening target: {target}"));
    open_target(target)
        .await
        .map_err(|error| format!("Failed to open target: {error}"))?;
    Ok(format!("Opened: {target}"))
}

async fn execute_click(
    x: i32,
    y: i32,
    button: Option<&str>,
    clicks: u32,
    transcript: &mut Vec<String>,
) -> Result<String, String> {
    transcript.push(format!("Clicking at ({x}, {y}) x{clicks}"));
    let button = parse_button(button);
    spawn_blocking(move || {
        let mut enigo = create_enigo()?;
        enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|error| error.to_string())?;
        for _ in 0..clicks.max(1) {
            enigo
                .button(button, Direction::Click)
                .map_err(|error| error.to_string())?;
        }
        Ok::<(), String>(())
    })
    .await
    .map_err(|error| error.to_string())??;

    Ok(format!("Clicked at ({x}, {y})"))
}

async fn execute_type_text(text: &str, transcript: &mut Vec<String>) -> Result<String, String> {
    transcript.push(format!("Typing {} character(s)", text.chars().count()));
    let text = text.to_string();
    let summary = text.clone();
    spawn_blocking(move || {
        let mut enigo = create_enigo()?;
        enigo.text(&text).map_err(|error| error.to_string())?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|error| error.to_string())??;

    Ok(if summary.is_empty() {
        "Typed empty text".into()
    } else {
        format!("Typed text: {summary}")
    })
}

async fn execute_hotkey(keys: &[String], transcript: &mut Vec<String>) -> Result<String, String> {
    if keys.is_empty() {
        return Err("hotkey requires at least one key".into());
    }

    transcript.push(format!("Sending hotkey: {:?}", keys));
    let keys_owned = keys.to_vec();
    spawn_blocking(move || {
        let mut enigo = create_enigo()?;
        let mapped = map_keys(&keys_owned)?;
        for key in mapped.iter().take(mapped.len().saturating_sub(1)) {
            enigo
                .key(*key, Direction::Press)
                .map_err(|error| error.to_string())?;
        }
        if let Some(last) = mapped.last() {
            enigo
                .key(*last, Direction::Click)
                .map_err(|error| error.to_string())?;
        }
        for key in mapped.iter().take(mapped.len().saturating_sub(1)).rev() {
            enigo
                .key(*key, Direction::Release)
                .map_err(|error| error.to_string())?;
        }
        Ok::<(), String>(())
    })
    .await
    .map_err(|error| error.to_string())??;

    Ok(format!("Hotkey sent: {}", keys.join("+")))
}

async fn execute_scroll(
    clicks: i32,
    axis: Option<&str>,
    transcript: &mut Vec<String>,
) -> Result<String, String> {
    transcript.push(format!("Scrolling {clicks} tick(s)"));
    let axis = parse_axis(axis);
    spawn_blocking(move || {
        let mut enigo = create_enigo()?;
        enigo
            .scroll(clicks, axis)
            .map_err(|error| error.to_string())?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|error| error.to_string())??;

    Ok(format!("Scrolled {clicks} tick(s)"))
}

async fn execute_screenshot(
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    transcript: &mut Vec<String>,
) -> Result<(String, Value), String> {
    transcript.push("Capturing screenshot".into());
    let capture = spawn_blocking(move || capture_primary_monitor(x, y, width, height))
        .await
        .map_err(|error| error.to_string())??;

    transcript.push(format!(
        "Screenshot captured: {}x{}",
        capture.width, capture.height
    ));

    let data = json!({
        "screenshot_base64": capture.base64_png,
        "width": capture.width,
        "height": capture.height,
    });

    Ok((
        format!("Captured screenshot {}x{}", capture.width, capture.height),
        data,
    ))
}

#[derive(Debug)]
struct ScreenshotPayload {
    base64_png: String,
    width: u32,
    height: u32,
}

fn capture_primary_monitor(
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<ScreenshotPayload, String> {
    let monitors = Monitor::all().map_err(|error| error.to_string())?;
    let monitor = monitors
        .into_iter()
        .next()
        .ok_or_else(|| "No monitor found".to_string())?;

    let image = match (x, y, width, height) {
        (Some(x), Some(y), Some(width), Some(height)) => monitor
            .capture_region(x, y, width, height)
            .map_err(|error| error.to_string())?,
        _ => monitor.capture_image().map_err(|error| error.to_string())?,
    };

    let width = image.width();
    let height = image.height();
    let mut bytes = Vec::new();
    let dynamic = DynamicImage::ImageRgba8(image);
    dynamic
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|error| error.to_string())?;

    Ok(ScreenshotPayload {
        base64_png: STANDARD.encode(bytes),
        width,
        height,
    })
}

fn create_enigo() -> Result<Enigo, String> {
    Enigo::new(&EnigoSettings::default()).map_err(|error| error.to_string())
}

fn parse_button(button: Option<&str>) -> Button {
    match button.unwrap_or("left").to_ascii_lowercase().as_str() {
        "right" => Button::Right,
        "middle" => Button::Middle,
        "back" => Button::Back,
        "forward" => Button::Forward,
        _ => Button::Left,
    }
}

fn parse_axis(axis: Option<&str>) -> Axis {
    match axis.unwrap_or("vertical").to_ascii_lowercase().as_str() {
        "horizontal" => Axis::Horizontal,
        _ => Axis::Vertical,
    }
}

fn map_keys(keys: &[String]) -> Result<Vec<Key>, String> {
    keys.iter().map(|value| map_key(value)).collect()
}

fn map_key(value: &str) -> Result<Key, String> {
    let lower = value.trim().to_ascii_lowercase();
    let key = match lower.as_str() {
        "ctrl" | "control" => Key::Control,
        "shift" => Key::Shift,
        "alt" => Key::Alt,
        "meta" | "win" | "super" | "command" => Key::Meta,
        "enter" | "return" => Key::Return,
        "tab" => Key::Tab,
        "esc" | "escape" => Key::Escape,
        "space" => Key::Space,
        "backspace" => Key::Backspace,
        "delete" => Key::Delete,
        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        _ if lower.chars().count() == 1 => Key::Unicode(lower.chars().next().unwrap()),
        _ => return Err(format!("Unsupported key: {value}")),
    };

    Ok(key)
}

#[cfg(target_os = "windows")]
async fn run_system_shell(command: &str) -> std::io::Result<std::process::Output> {
    Command::new("powershell")
        .arg("-Command")
        .arg(command)
        .output()
        .await
}

#[cfg(not(target_os = "windows"))]
async fn run_system_shell(command: &str) -> std::io::Result<std::process::Output> {
    Command::new("sh").arg("-lc").arg(command).output().await
}

#[cfg(target_os = "windows")]
async fn open_target(target: &str) -> std::io::Result<()> {
    Command::new("cmd")
        .args(["/C", "start", "", target])
        .spawn()?
        .wait()
        .await
        .map(|_| ())
}

#[cfg(target_os = "macos")]
async fn open_target(target: &str) -> std::io::Result<()> {
    Command::new("open")
        .arg(target)
        .spawn()?
        .wait()
        .await
        .map(|_| ())
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
async fn open_target(target: &str) -> std::io::Result<()> {
    Command::new("xdg-open")
        .arg(target)
        .spawn()?
        .wait()
        .await
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_action() {
        let parsed = parse_action_specs(r#"{"type":"wait","seconds":1.5}"#).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn parses_action_array() {
        let parsed = parse_action_specs(
            r#"[{"type":"wait","seconds":1},{"type":"complete","result":"ok"}]"#,
        )
        .unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn parses_action_envelope() {
        let parsed = parse_action_specs(
            r#"{"actions":[{"type":"shell","command":"echo hi"},{"type":"complete"}]}"#,
        )
        .unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn parses_click_and_type_variants() {
        let parsed = parse_action_specs(
            r#"{"actions":[{"type":"click","x":1,"y":2},{"type":"type","text":"hello"}]}"#,
        )
        .unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn maps_common_keys() {
        assert!(map_keys(&["ctrl".into(), "shift".into(), "a".into()]).is_ok());
    }
}
