use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandEffect {
    ResetSession,
    SaveConfig,
    RestartGateway,
    RestartAiGroups,
    StopAiGroups,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    pub title: String,
    pub message: String,
    pub effects: Vec<CommandEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedCommand {
    Help,
    New,
    View(bool),
    AiGroupGlobal(bool),
    AiGroupStart,
    AiGroupStop,
}

pub fn execute(input: &str, config: &mut Config) -> Option<Result<CommandOutcome, String>> {
    let parsed = match parse(input) {
        Some(Ok(command)) => command,
        Some(Err(error)) => return Some(Err(error)),
        None => return None,
    };

    let zh = config.language != "en";
    let outcome = match parsed {
        ParsedCommand::Help => CommandOutcome {
            title: label(zh, "帮助", "Help"),
            message: help_text(zh),
            effects: Vec::new(),
        },
        ParsedCommand::New => CommandOutcome {
            title: label(zh, "新会话", "New Session"),
            message: label(
                zh,
                "已开启全新对话。当前版本会清空命令会话标记，后续 AI 上下文会从这里重新开始。",
                "Started a new conversation. This version resets the command session marker; future AI context starts here.",
            ),
            effects: vec![CommandEffect::ResetSession],
        },
        ParsedCommand::View(enabled) => {
            config.use_vision = enabled;
            CommandOutcome {
                title: label(zh, "识别控制屏", "Vision Control"),
                message: if enabled {
                    label(zh, "已开启识别控制屏。AI 会先观察屏幕，再决定下一步。", "Vision control is on. AI will inspect the screen before choosing the next step.")
                } else {
                    label(zh, "已关闭识别控制屏。AI 将优先使用文字/工具上下文执行。", "Vision control is off. AI will prefer text/tool context.")
                },
                effects: vec![CommandEffect::SaveConfig],
            }
        }
        ParsedCommand::AiGroupGlobal(enabled) => {
            config.ai_groups_enabled = enabled;
            CommandOutcome {
                title: label(zh, "AI 群组", "AI Groups"),
                message: if enabled {
                    label(zh, "AI 群组已全局开启，并会重新加载群组服务。", "AI Groups are enabled globally and will be reloaded.")
                } else {
                    label(zh, "AI 群组已全局关闭，当前群组服务会停止。", "AI Groups are disabled globally and the current group service will stop.")
                },
                effects: if enabled {
                    vec![CommandEffect::SaveConfig, CommandEffect::RestartAiGroups]
                } else {
                    vec![CommandEffect::SaveConfig, CommandEffect::StopAiGroups]
                },
            }
        }
        ParsedCommand::AiGroupStart => {
            config.ai_groups_enabled = true;
            CommandOutcome {
                title: label(zh, "AI 群组", "AI Groups"),
                message: label(zh, "当前 AI 群组会话已启动。", "The current AI group session is started."),
                effects: vec![CommandEffect::SaveConfig, CommandEffect::RestartAiGroups],
            }
        }
        ParsedCommand::AiGroupStop => CommandOutcome {
            title: label(zh, "AI 群组", "AI Groups"),
            message: label(
                zh,
                "当前 AI 群组会话已停止。全局开关保持不变，可用 \\ai group start 再次启动。",
                "The current AI group session is stopped. The global switch is unchanged; use \\ai group start to start it again.",
            ),
            effects: vec![CommandEffect::StopAiGroups],
        },
    };

    Some(Ok(outcome))
}

fn parse(input: &str) -> Option<Result<ParsedCommand, String>> {
    let trimmed = input.trim();
    let body = trimmed.strip_prefix('\\')?.trim();
    if body.is_empty() {
        return Some(Err("缺少命令。输入 \\help 查看可用命令。".into()));
    }

    let normalized = body
        .split_whitespace()
        .map(|part| part.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let parts = normalized.iter().map(String::as_str).collect::<Vec<_>>();

    let command = match parts.as_slice() {
        ["help"] | ["?"] => ParsedCommand::Help,
        ["new"] => ParsedCommand::New,
        ["view", "on"] => ParsedCommand::View(true),
        ["view", "off"] => ParsedCommand::View(false),
        ["ai", "group", "on"] => ParsedCommand::AiGroupGlobal(true),
        ["ai", "group", "off"] => ParsedCommand::AiGroupGlobal(false),
        ["ai", "group", "start"] => ParsedCommand::AiGroupStart,
        ["ai", "group", "stop"] => ParsedCommand::AiGroupStop,
        _ => {
            return Some(Err(format!(
                "未知命令：\\{body}。输入 \\help 查看可用命令。"
            )))
        }
    };

    Some(Ok(command))
}

fn label(zh: bool, zh_text: &str, en_text: &str) -> String {
    if zh {
        zh_text.into()
    } else {
        en_text.into()
    }
}

fn help_text(zh: bool) -> String {
    if zh {
        [
            "可用命令：",
            "\\help：查看帮助",
            "\\new：开启全新对话",
            "\\view on/off：开启或关闭识别控制屏",
            "\\ai group on/off：全局开启或关闭 AI 群组",
            "\\ai group start/stop：启动或停止当前 AI 群组会话",
        ]
        .join("\n")
    } else {
        [
            "Available commands:",
            "\\help: show help",
            "\\new: start a new conversation",
            "\\view on/off: enable or disable vision control",
            "\\ai group on/off: globally enable or disable AI Groups",
            "\\ai group start/stop: start or stop the current AI group session",
        ]
        .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_view_command() {
        let mut config = Config::default();
        let outcome = execute("\\view off", &mut config)
            .expect("command")
            .expect("ok");

        assert!(!config.use_vision);
        assert_eq!(outcome.effects, vec![CommandEffect::SaveConfig]);
    }

    #[test]
    fn ignores_plain_tasks() {
        let mut config = Config::default();
        assert!(execute("用QQ发消息", &mut config).is_none());
    }
}
