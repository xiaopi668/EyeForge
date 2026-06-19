use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::{Mutex, OnceLock};

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

use crate::ai_groups;
use crate::commands::{self, CommandEffect};
use crate::config::Config;
use crate::llm;

const MAX_AI_STEPS: usize = 20;
const MAX_RUNTIME_EVENTS: usize = 200;

#[derive(Debug, Clone)]
pub struct RuntimeEvent {
    pub kind: RuntimeEventKind,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeEventKind {
    Running,
    Step,
    Success,
    Error,
}

static RUNTIME_EVENTS: OnceLock<Mutex<VecDeque<RuntimeEvent>>> = OnceLock::new();

fn event_queue() -> &'static Mutex<VecDeque<RuntimeEvent>> {
    RUNTIME_EVENTS.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn drain_events() -> Vec<RuntimeEvent> {
    let Ok(mut queue) = event_queue().lock() else {
        return Vec::new();
    };
    queue.drain(..).collect()
}

fn queue_event(event: RuntimeEvent) {
    if let Ok(mut queue) = event_queue().lock() {
        queue.push_back(event);
        while queue.len() > MAX_RUNTIME_EVENTS {
            queue.pop_front();
        }
    }
}

fn emit_event<F>(reporter: &mut F, kind: RuntimeEventKind, title: &str, message: impl Into<String>)
where
    F: FnMut(RuntimeEvent) + Send,
{
    let event = RuntimeEvent {
        kind,
        title: title.into(),
        message: message.into(),
    };
    queue_event(event.clone());
    reporter(event);
}

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
    let mut reporter = |_| {};
    execute_task_with_events(task, config, &mut reporter).await
}

pub async fn execute_task_with_events<F>(
    task: String,
    config: Config,
    reporter: &mut F,
) -> Result<NativeOutcome, String>
where
    F: FnMut(RuntimeEvent) + Send,
{
    emit_event(
        reporter,
        RuntimeEventKind::Running,
        "执行",
        "任务已开始，正在执行中...",
    );
    let result = execute_task_inner(task, config, reporter).await;
    match &result {
        Ok(outcome) => emit_event(
            reporter,
            RuntimeEventKind::Success,
            "完成",
            outcome.message.clone(),
        ),
        Err(error) => emit_event(
            reporter,
            RuntimeEventKind::Error,
            "失败",
            format!("执行失败：{error}"),
        ),
    }
    result
}

async fn execute_task_inner<F>(
    task: String,
    mut config: Config,
    reporter: &mut F,
) -> Result<NativeOutcome, String>
where
    F: FnMut(RuntimeEvent) + Send,
{
    let trimmed = task.trim().to_string();
    let mut transcript = vec![format!("Task received: {trimmed}")];

    if let Some(command_result) = commands::execute(&trimmed, &mut config) {
        let outcome = command_result?;
        apply_command_effects(&config, &outcome.effects);
        transcript.push(format!("Command executed: {}", outcome.title));
        return Ok(NativeOutcome {
            status: "success".into(),
            message: outcome.message,
            transcript,
            data: None,
        });
    }

    if looks_like_json(&trimmed) {
        let actions = parse_action_specs(&trimmed)?;
        transcript.push(format!(
            "Parsed structured action plan with {} step(s)",
            actions.len()
        ));
        return execute_action_plan(actions, &config, transcript, reporter).await;
    }

    if let Some(command) = trimmed.strip_prefix("shell:") {
        emit_event(
            reporter,
            RuntimeEventKind::Step,
            "动作",
            format!("执行 Shell：{command}"),
        );
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
        emit_event(
            reporter,
            RuntimeEventKind::Step,
            "动作",
            format!("等待 {seconds} 秒"),
        );
        let message = execute_wait(seconds, &config, &mut transcript).await?;
        return Ok(NativeOutcome {
            status: "success".into(),
            message,
            transcript,
            data: None,
        });
    }

    if let Some(target) = trimmed.strip_prefix("open:") {
        emit_event(
            reporter,
            RuntimeEventKind::Step,
            "动作",
            format!("打开：{}", target.trim()),
        );
        let message = execute_open(target.trim(), &config, &mut transcript).await?;
        return Ok(NativeOutcome {
            status: "success".into(),
            message,
            transcript,
            data: None,
        });
    }

    execute_ai_task_stepwise(&trimmed, &config, transcript, reporter).await
}

async fn execute_action_plan<F>(
    actions: Vec<ActionSpec>,
    config: &Config,
    mut transcript: Vec<String>,
    reporter: &mut F,
) -> Result<NativeOutcome, String>
where
    F: FnMut(RuntimeEvent) + Send,
{
    let mut last_message = if config.language == "zh" {
        "动作计划已执行".to_string()
    } else {
        "Action plan executed".to_string()
    };
    let mut last_data: Option<Value> = None;

    let (actions, completion_result) = split_executable_actions(actions);
    if actions.is_empty() {
        let message = completion_result.unwrap_or(last_message);
        emit_event(reporter, RuntimeEventKind::Success, "完成", message.clone());
        return Ok(NativeOutcome {
            status: "success".into(),
            message,
            transcript,
            data: last_data,
        });
    }

    for (index, action) in actions.into_iter().enumerate() {
        emit_event(
            reporter,
            RuntimeEventKind::Step,
            "动作",
            format!("步骤 {}：{}", index + 1, action_summary(&action)),
        );
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
                emit_event(
                    reporter,
                    RuntimeEventKind::Success,
                    "完成",
                    result.clone().unwrap_or_else(|| last_message.clone()),
                );
                return Ok(NativeOutcome {
                    status: "success".into(),
                    message: result.unwrap_or(last_message),
                    transcript,
                    data: last_data,
                });
            }
        }
        emit_event(
            reporter,
            RuntimeEventKind::Running,
            "结果",
            last_message.clone(),
        );
    }

    Ok(NativeOutcome {
        status: "success".into(),
        message: completion_result.unwrap_or(last_message),
        transcript,
        data: last_data,
    })
}

async fn execute_ai_task_stepwise<F>(
    task: &str,
    config: &Config,
    mut transcript: Vec<String>,
    reporter: &mut F,
) -> Result<NativeOutcome, String>
where
    F: FnMut(RuntimeEvent) + Send,
{
    let mut history = Vec::new();
    let mut last_message = if config.language == "zh" {
        "AI 自适应执行已开始".to_string()
    } else {
        "AI adaptive execution started".to_string()
    };
    let mut last_data: Option<Value> = None;

    transcript.push(format!(
        "AI task will run adaptively; max feedback steps: {MAX_AI_STEPS}"
    ));

    for step in 1..=MAX_AI_STEPS {
        let step_task = build_adaptive_task_prompt(task, &history, step);
        transcript.push(format!("Step {step}: planning adaptive next action(s)"));
        emit_event(
            reporter,
            RuntimeEventKind::Running,
            "规划",
            format!("第 {step} 步：正在分析下一步动作"),
        );

        let plan = plan_next_ai_step(&step_task, config, &mut transcript).await?;
        transcript.push(format!("Step {step} plan received:\n{plan}"));
        emit_event(
            reporter,
            RuntimeEventKind::Running,
            "规划完成",
            format!("第 {step} 步：已收到动作计划"),
        );

        let mut actions = parse_action_specs(&plan)?;
        if actions.is_empty() {
            return Err("AI returned an empty action plan".into());
        }
        let executable_count = actions
            .iter()
            .filter(|action| is_executable_action(action))
            .count();
        if executable_count > 1 {
            transcript.push(format!(
                "Step {step}: AI chose batch mode with {} executable actions",
                executable_count
            ));
            emit_event(
                reporter,
                RuntimeEventKind::Running,
                "执行模式",
                format!(
                    "AI 认为后续步骤明确，选择一气呵成执行 {} 个动作",
                    executable_count
                ),
            );
            return execute_action_plan(actions, config, transcript, reporter).await;
        }
        if executable_count == 1 && actions.len() > 1 {
            transcript.push(format!(
                "Step {step}: AI returned one executable action plus completion metadata; continuing feedback mode"
            ));
            actions.retain(is_executable_action);
        }

        let action = actions.remove(0);
        let action_label = format!("{action:?}");

        if let ActionSpec::Complete { result } = action {
            emit_event(
                reporter,
                RuntimeEventKind::Success,
                "完成",
                result.clone().unwrap_or_else(|| last_message.clone()),
            );
            return Ok(NativeOutcome {
                status: "success".into(),
                message: result.unwrap_or(last_message),
                transcript,
                data: last_data,
            });
        }

        emit_event(
            reporter,
            RuntimeEventKind::Step,
            "动作",
            format!("第 {step} 步：{}", action_summary(&action)),
        );
        let (message, data) = execute_single_action(action, step, config, &mut transcript).await?;
        history.push(format!("Step {step}: {action_label} => {message}"));
        last_message = message;
        emit_event(
            reporter,
            RuntimeEventKind::Running,
            "步骤结果",
            format!("第 {step} 步完成：{last_message}"),
        );
        if data.is_some() {
            last_data = data;
        }
    }

    Ok(NativeOutcome {
        status: "success".into(),
        message: if config.language == "zh" {
            format!("已达到 {MAX_AI_STEPS} 步上限，已暂停。最后结果：{last_message}")
        } else {
            format!("Reached the {MAX_AI_STEPS}-step limit and paused. Last result: {last_message}")
        },
        transcript,
        data: last_data,
    })
}

fn apply_command_effects(config: &Config, effects: &[CommandEffect]) {
    if effects.contains(&CommandEffect::SaveConfig) {
        config.save();
    }
    if effects.contains(&CommandEffect::RestartGateway) {
        let _ = crate::server::restart(config);
    }
    if effects.contains(&CommandEffect::RestartAiGroups) {
        let _ = crate::ai_groups::restart(config);
    }
    if effects.contains(&CommandEffect::StopAiGroups) {
        crate::ai_groups::stop();
    }
}

async fn plan_next_ai_step(
    step_task: &str,
    config: &Config,
    transcript: &mut Vec<String>,
) -> Result<String, String> {
    if config.use_vision {
        transcript.push("Capturing screenshot for this step".into());
        let (_, screenshot_data) = execute_screenshot(None, None, None, None, transcript).await?;

        let screenshot_plan = screenshot_data
            .get("screenshot_base64")
            .and_then(Value::as_str)
            .zip(screenshot_data.get("width").and_then(Value::as_u64))
            .zip(screenshot_data.get("height").and_then(Value::as_u64))
            .map(|((base64_png, width), height)| {
                llm::plan_actions_with_screenshot(
                    step_task,
                    config,
                    llm::ScreenshotContext {
                        base64_png,
                        width: width as u32,
                        height: height as u32,
                    },
                )
            });

        if let Some(plan) = screenshot_plan {
            match plan.await {
                Ok(plan) => return Ok(plan),
                Err(error) => {
                    transcript.push(format!(
                        "Vision step planning failed; falling back to text-only planning: {error}"
                    ));
                }
            }
        } else {
            transcript.push(
                "Screenshot payload was incomplete; falling back to text-only planning".into(),
            );
        }
    } else {
        match ai_groups::dispatch_task(config, step_task).await {
            Ok(Some(plan)) => return Ok(plan),
            Ok(None) => {
                transcript
                    .push("AI groups disabled or returned no step; falling back to LLM".into());
            }
            Err(error) => {
                transcript.push(format!(
                    "AI group step dispatch failed; falling back to LLM: {error}"
                ));
            }
        }
    }

    transcript.push(format!(
        "Planning one step with Rust-native LLM provider: {}",
        config.llm_provider
    ));
    llm::plan_actions(step_task, config).await
}

async fn execute_single_action(
    action: ActionSpec,
    step: usize,
    config: &Config,
    transcript: &mut Vec<String>,
) -> Result<(String, Option<Value>), String> {
    transcript.push(format!("Step {step} action => {:?}", action));

    match action {
        ActionSpec::Shell {
            command,
            timeout_secs,
        } => execute_shell(&command, timeout_secs.unwrap_or(30), config, transcript)
            .await
            .map(|message| (message, None)),
        ActionSpec::Wait { seconds } => execute_wait(seconds, config, transcript)
            .await
            .map(|message| (message, None)),
        ActionSpec::Open { target } => execute_open(&target, config, transcript)
            .await
            .map(|message| (message, None)),
        ActionSpec::Click {
            x,
            y,
            button,
            clicks,
        } => execute_click(x, y, button.as_deref(), clicks.unwrap_or(1), transcript)
            .await
            .map(|message| (message, None)),
        ActionSpec::TypeText { text } => execute_type_text(&text, transcript)
            .await
            .map(|message| (message, None)),
        ActionSpec::Hotkey { keys } => execute_hotkey(&keys, transcript)
            .await
            .map(|message| (message, None)),
        ActionSpec::Scroll { clicks, axis } => execute_scroll(clicks, axis.as_deref(), transcript)
            .await
            .map(|message| (message, None)),
        ActionSpec::Screenshot {
            x,
            y,
            width,
            height,
        } => {
            let (message, data) = execute_screenshot(x, y, width, height, transcript).await?;
            Ok((message, Some(data)))
        }
        ActionSpec::Complete { result } => Ok((
            result.unwrap_or_else(|| {
                if config.language == "zh" {
                    "任务完成".into()
                } else {
                    "Task complete".into()
                }
            }),
            None,
        )),
    }
}

fn build_adaptive_task_prompt(task: &str, history: &[String], step: usize) -> String {
    let history_text = if history.is_empty() {
        "No actions have been executed yet.".to_string()
    } else {
        history.join("\n")
    };

    format!(
        "Original task:\n{task}\n\nExecution mode: adaptive.\nCurrent feedback round: {step}.\n\nExecuted history:\n{history_text}\n\nChoose the execution style yourself:\n- If the next steps are already certain and do not depend on observing intermediate results, return a batch JSON plan with multiple safe actions.\n- If the next step is uncertain or depends on screen feedback after an action, return exactly one next action.\n- If the task is fully finished, return only {{\"type\":\"complete\",\"result\":\"...\"}} or put complete as the final action in a batch plan.\n\nReturn JSON only."
    )
}

fn parse_action_specs(input: &str) -> Result<Vec<ActionSpec>, String> {
    serde_json::from_str::<ActionEnvelope>(input)
        .map(|envelope| envelope.actions)
        .or_else(|_| serde_json::from_str::<Vec<ActionSpec>>(input))
        .or_else(|_| serde_json::from_str::<ActionSpec>(input).map(|action| vec![action]))
        .map_err(|error| format!("Invalid action JSON: {error}"))
}

fn action_summary(action: &ActionSpec) -> String {
    match action {
        ActionSpec::Shell { command, .. } => format!("执行 Shell：{command}"),
        ActionSpec::Wait { seconds } => format!("等待 {seconds} 秒"),
        ActionSpec::Open { target } => format!("打开 {target}"),
        ActionSpec::Click {
            x,
            y,
            button,
            clicks,
        } => format!(
            "点击 ({x}, {y})，按钮 {}，{} 次",
            button.as_deref().unwrap_or("left"),
            clicks.unwrap_or(1)
        ),
        ActionSpec::TypeText { text } => format!("输入文本（{} 个字符）", text.chars().count()),
        ActionSpec::Hotkey { keys } => format!("发送快捷键 {}", keys.join("+")),
        ActionSpec::Scroll { clicks, axis } => {
            format!(
                "滚动 {clicks}，方向 {}",
                axis.as_deref().unwrap_or("vertical")
            )
        }
        ActionSpec::Screenshot {
            x,
            y,
            width,
            height,
        } => match (x, y, width, height) {
            (Some(x), Some(y), Some(width), Some(height)) => {
                format!("截图区域 ({x}, {y}) {width}x{height}")
            }
            _ => "查看当前屏幕".into(),
        },
        ActionSpec::Complete { result } => result
            .as_deref()
            .map(|result| format!("完成：{result}"))
            .unwrap_or_else(|| "完成任务".into()),
    }
}

fn is_executable_action(action: &ActionSpec) -> bool {
    !matches!(action, ActionSpec::Complete { .. })
}

fn split_executable_actions(actions: Vec<ActionSpec>) -> (Vec<ActionSpec>, Option<String>) {
    let mut executable = Vec::new();
    let mut completion = None;

    for action in actions {
        match action {
            ActionSpec::Complete { result } => {
                if result.is_some() {
                    completion = result;
                }
            }
            action => executable.push(action),
        }
    }

    (executable, completion)
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
    if !config.shell_enabled {
        return Err(if config.language == "zh" {
            "Shell 命令开关未开启".into()
        } else {
            "Shell commands are disabled".into()
        });
    }

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
    if !config.shell_enabled {
        return Err(if config.language == "zh" {
            "Shell 关闭时已禁用 open 启动器；请使用鼠标和键盘动作。".into()
        } else {
            "Open launcher actions are disabled while shell is off; use mouse and keyboard actions instead."
                .into()
        });
    }

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
fn hide_child_window(command: &mut Command) {
    command.creation_flags(0x08000000);
}

#[cfg(target_os = "windows")]
async fn run_system_shell(script: &str) -> std::io::Result<std::process::Output> {
    let mut command = Command::new("powershell");
    hide_child_window(&mut command);
    command
        .arg("-NoProfile")
        .arg("-WindowStyle")
        .arg("Hidden")
        .arg("-Command")
        .arg(script)
        .output()
        .await
}

#[cfg(not(target_os = "windows"))]
async fn run_system_shell(command: &str) -> std::io::Result<std::process::Output> {
    Command::new("sh").arg("-lc").arg(command).output().await
}

#[cfg(target_os = "windows")]
async fn open_target(target: &str) -> std::io::Result<()> {
    let mut command = Command::new("cmd");
    hide_child_window(&mut command);
    command
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
    fn completion_marker_does_not_count_as_batch_action() {
        let parsed = parse_action_specs(
            r#"{"actions":[{"type":"click","x":1,"y":2},{"type":"complete","result":"ok"}]}"#,
        )
        .unwrap();
        let executable_count = parsed
            .iter()
            .filter(|action| is_executable_action(action))
            .count();
        assert_eq!(executable_count, 1);
    }

    #[test]
    fn completion_marker_cannot_truncate_batch_actions() {
        let parsed = parse_action_specs(
            r#"{"actions":[{"type":"click","x":1,"y":2},{"type":"complete","result":"ok"},{"type":"type","text":"hello"},{"type":"hotkey","keys":["enter"]}]}"#,
        )
        .unwrap();
        let (executable, completion) = split_executable_actions(parsed);
        assert_eq!(executable.len(), 3);
        assert_eq!(completion.as_deref(), Some("ok"));
    }

    #[test]
    fn maps_common_keys() {
        assert!(map_keys(&["ctrl".into(), "shift".into(), "a".into()]).is_ok());
    }
}
