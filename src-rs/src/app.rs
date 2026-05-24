use iced::alignment::Horizontal;
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, pick_list, row, scrollable, text,
    text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Fill, Length, Task, Theme};

use crate::config::{Config, EditableConfig};
use crate::runtime::{self, NativeOutcome};
use crate::server;

const VERSION: &str = "1.5.0-beta.2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Settings,
}

impl Default for Page {
    fn default() -> Self {
        Self::Home
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Ai,
    Capture,
    General,
    Channels,
}

impl Default for SettingsSection {
    fn default() -> Self {
        Self::Ai
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAi,
    Anthropic,
    Ollama,
    Gemini,
    Custom,
}

impl Provider {
    const ALL: [Provider; 5] = [
        Provider::OpenAi,
        Provider::Anthropic,
        Provider::Ollama,
        Provider::Gemini,
        Provider::Custom,
    ];

    fn as_config_value(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Ollama => "ollama",
            Self::Gemini => "gemini",
            Self::Custom => "custom",
        }
    }

    fn from_config_value(value: &str) -> Self {
        match value {
            "anthropic" => Self::Anthropic,
            "ollama" => Self::Ollama,
            "gemini" => Self::Gemini,
            "custom" => Self::Custom,
            _ => Self::OpenAi,
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::Ollama => "Ollama",
            Self::Gemini => "Gemini",
            Self::Custom => "Custom",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Zh,
    En,
}

impl Language {
    const ALL: [Language; 2] = [Language::Zh, Language::En];

    fn as_config_value(self) -> &'static str {
        match self {
            Self::Zh => "zh",
            Self::En => "en",
        }
    }

    fn from_config_value(value: &str) -> Self {
        if value == "en" {
            Self::En
        } else {
            Self::Zh
        }
    }

    fn is_zh(self) -> bool {
        matches!(self, Self::Zh)
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Zh => "中文 (zh)",
            Self::En => "English (en)",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    const ALL: [ThemeMode; 2] = [ThemeMode::Dark, ThemeMode::Light];

    fn as_config_value(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    fn from_config_value(value: &str) -> Self {
        if value == "light" {
            Self::Light
        } else {
            Self::Dark
        }
    }

    fn to_theme(self) -> Theme {
        match self {
            Self::Dark => Theme::Dark,
            Self::Light => Theme::Light,
        }
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Dark => "dark",
            Self::Light => "light",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QqMode {
    WebSocket,
    Official,
}

impl QqMode {
    const ALL: [QqMode; 2] = [QqMode::WebSocket, QqMode::Official];

    fn as_config_value(self) -> &'static str {
        match self {
            Self::WebSocket => "ws",
            Self::Official => "official",
        }
    }

    fn from_config_value(value: &str) -> Self {
        if value == "official" {
            Self::Official
        } else {
            Self::WebSocket
        }
    }
}

impl std::fmt::Display for QqMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WebSocket => "go-cqhttp WebSocket",
            Self::Official => "QQ Official Bot",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextField {
    Task,
    OpenAiKey,
    OpenAiModel,
    AnthropicKey,
    AnthropicModel,
    OllamaUrl,
    OllamaModel,
    CustomKey,
    CustomUrl,
    CustomModel,
    GeminiKey,
    GeminiModel,
    ScreenshotQuality,
    ActionDelay,
    FontSize,
    HotkeyFloat,
    HotkeyVoice,
    WakewordList,
    PorcupineKey,
    WsHost,
    WsPort,
    WsToken,
    WcToken,
    WcomCorpId,
    WcomAgentId,
    WcomSecret,
    WcomToken,
    WcomAesKey,
    DtAppKey,
    DtAppSecret,
    DtWebhook,
    QqWsHost,
    QqWsPort,
    QqBotAppId,
    QqBotToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolField {
    WakewordEnabled,
    WsEnabled,
    WcEnabled,
    WcomEnabled,
    DtEnabled,
    QqEnabled,
    VisionEnabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogKind {
    Info,
    Status,
    Success,
    Error,
}

#[derive(Debug, Clone)]
struct LogEntry {
    kind: LogKind,
    title: String,
    detail: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    SidebarClick(Page),
    SettingsSectionSelected(SettingsSection),
    ProviderSelected(Provider),
    LanguageSelected(Language),
    ThemeSelected(ThemeMode),
    QqModeSelected(QqMode),
    TextChanged(TextField, String),
    BoolChanged(BoolField, bool),
    SaveSettings,
    StartTask,
    BackendFinished(Result<NativeOutcome, String>),
}

pub struct EyeForge {
    config: Config,
    settings: EditableConfig,
    theme: Theme,
    current_page: Page,
    current_settings_section: SettingsSection,
    task_input: String,
    status_text: String,
    latest_result: Option<String>,
    logs: Vec<LogEntry>,
    task_running: bool,
}

impl Default for EyeForge {
    fn default() -> Self {
        let config = Config::default();
        let settings = config.to_editable();
        let language = Language::from_config_value(&settings.language);

        Self {
            config,
            settings,
            theme: ThemeMode::from_config_value("dark").to_theme(),
            current_page: Page::Home,
            current_settings_section: SettingsSection::Ai,
            task_input: String::new(),
            status_text: if language.is_zh() {
                "就绪".into()
            } else {
                "Ready".into()
            },
            latest_result: None,
            logs: Vec::new(),
            task_running: false,
        }
    }
}

impl EyeForge {
    pub fn new() -> (Self, Task<Message>) {
        let config = Config::load();
        let settings = config.to_editable();
        let language = Language::from_config_value(&settings.language);
        let theme = ThemeMode::from_config_value(&settings.theme).to_theme();
        let ws_host = settings.ws_host.clone();

        let mut app = Self {
            config,
            settings,
            theme,
            current_page: Page::Home,
            current_settings_section: SettingsSection::Ai,
            task_input: String::new(),
            status_text: if language.is_zh() {
                "就绪".into()
            } else {
                "Ready".into()
            },
            latest_result: None,
            logs: Vec::new(),
            task_running: false,
        };

        let gateway_status = server::restart(&app.config);
        app.push_log(
            LogKind::Info,
            if language.is_zh() { "启动" } else { "Boot" },
            match gateway_status {
                Ok(_) => format!(
                    "Rust gateway listening on http://127.0.0.1:{}/ and ws://127.0.0.1:{}/ws",
                    crate::server::GATEWAY_PORT,
                    crate::server::GATEWAY_PORT
                ),
                Err(error) => format!(
                    "Rust gateway failed to start on {}:{}: {error}",
                    ws_host,
                    crate::server::GATEWAY_PORT
                ),
            },
        );

        (app, Task::none())
    }

    pub fn title(&self) -> String {
        if self.language().is_zh() {
            format!("EyeForge v{VERSION} - AI 屏幕操控助手")
        } else {
            format!("EyeForge v{VERSION} - AI Screen Control Assistant")
        }
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SidebarClick(page) => {
                self.current_page = if page == Page::Settings && self.current_page == Page::Settings
                {
                    Page::Home
                } else {
                    page
                };
            }
            Message::SettingsSectionSelected(section) => {
                self.current_settings_section = section;
            }
            Message::ProviderSelected(provider) => {
                self.settings.llm_provider = provider.as_config_value().to_string();
            }
            Message::LanguageSelected(language) => {
                self.settings.language = language.as_config_value().to_string();
                self.refresh_runtime_preferences();
                self.status_text = if language.is_zh() {
                    "语言已切换，保存后写入配置".into()
                } else {
                    "Language updated. Save to persist it.".into()
                };
            }
            Message::ThemeSelected(theme_mode) => {
                self.settings.theme = theme_mode.as_config_value().to_string();
                self.refresh_runtime_preferences();
                self.status_text = if self.language().is_zh() {
                    "主题已切换，保存后写入配置".into()
                } else {
                    "Theme updated. Save to persist it.".into()
                };
            }
            Message::QqModeSelected(mode) => {
                self.settings.qq_mode = mode.as_config_value().to_string();
            }
            Message::TextChanged(field, value) => self.update_text_field(field, value),
            Message::BoolChanged(field, value) => self.update_bool_field(field, value),
            Message::SaveSettings => {
                self.config = self.config.apply_editable(&self.settings);
                self.config.save();
                self.status_text = if self.language().is_zh() {
                    "配置已保存".into()
                } else {
                    "Configuration saved".into()
                };
                self.push_log(
                    LogKind::Success,
                    if self.language().is_zh() {
                        "配置"
                    } else {
                        "Config"
                    },
                    if self.language().is_zh() {
                        String::from("已保存为 Rust 共享配置，并保留未知字段")
                    } else {
                        String::from(
                            "Saved into the shared Rust config shape and preserved unknown fields",
                        )
                    },
                );

                let restart_result = server::restart(&self.config);
                self.push_log(
                    match restart_result {
                        Ok(_) => LogKind::Info,
                        Err(_) => LogKind::Error,
                    },
                    "Gateway",
                    match restart_result {
                        Ok(_) => format!(
                            "Rust gateway ready on http://127.0.0.1:{}/",
                            crate::server::GATEWAY_PORT
                        ),
                        Err(error) => error,
                    },
                );
                self.current_page = Page::Home;
            }
            Message::StartTask => {
                if self.task_running {
                    return Task::none();
                }

                let task = self.task_input.trim().to_string();
                if task.is_empty() {
                    self.status_text = if self.language().is_zh() {
                        "请输入任务描述".into()
                    } else {
                        "Please enter a task".into()
                    };
                    self.push_log(
                        LogKind::Error,
                        if self.language().is_zh() {
                            "任务"
                        } else {
                            "Task"
                        },
                        self.status_text.clone(),
                    );
                    return Task::none();
                }

                self.task_running = true;
                self.latest_result = None;
                self.status_text = if self.language().is_zh() {
                    "Rust 原生后端正在执行任务...".into()
                } else {
                    "Rust native backend is executing the task...".into()
                };
                self.push_log(
                    LogKind::Status,
                    if self.language().is_zh() {
                        "执行"
                    } else {
                        "Run"
                    },
                    task.clone(),
                );

                let effective = self.config.apply_editable(&self.settings);
                return Task::perform(
                    runtime::execute_task(task, effective),
                    Message::BackendFinished,
                );
            }
            Message::BackendFinished(result) => {
                self.task_running = false;

                match result {
                    Ok(outcome) => {
                        let ok = outcome.status == "success";
                        self.latest_result = Some(outcome.message.clone());
                        self.status_text = if ok {
                            if self.language().is_zh() {
                                "Rust 原生任务执行完成".into()
                            } else {
                                "Rust native task completed".into()
                            }
                        } else if self.language().is_zh() {
                            "Rust 原生后端返回错误".into()
                        } else {
                            "Rust native backend returned an error".into()
                        };

                        for line in outcome.transcript {
                            self.push_log(LogKind::Info, "Runtime", line);
                        }
                        self.push_log(
                            if ok { LogKind::Success } else { LogKind::Error },
                            if self.language().is_zh() {
                                "结果"
                            } else {
                                "Result"
                            },
                            outcome.message,
                        );
                    }
                    Err(error) => {
                        self.latest_result = None;
                        self.status_text = if self.language().is_zh() {
                            "Rust 原生任务执行失败".into()
                        } else {
                            "Rust native task failed".into()
                        };
                        self.push_log(
                            LogKind::Error,
                            if self.language().is_zh() {
                                "执行失败"
                            } else {
                                "Execution failure"
                            },
                            error,
                        );
                    }
                }
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        crate::tray::ensure_tray();

        let navigation = column![
            nav_button(
                self.t("Home", "Home"),
                self.current_page == Page::Home,
                Page::Home
            ),
            nav_button(
                self.t("Settings", "Settings"),
                self.current_page == Page::Settings,
                Page::Settings
            ),
        ]
        .spacing(8)
        .width(160);

        let rail = container(
            column![
                text("EyeForge").size(22).color(self.accent_color()),
                text(self.t("桌面控制台", "Desktop Console"))
                    .size(13)
                    .color(self.muted_text_color()),
                horizontal_rule(1),
                navigation,
                iced::widget::vertical_space(),
                text(format!("v{VERSION}"))
                    .size(12)
                    .color(self.muted_text_color()),
                text("Rust Native").size(12).color(self.muted_text_color()),
            ]
            .spacing(18)
            .padding([22, 18])
            .height(Fill),
        )
        .width(200)
        .height(Fill)
        .style(aside_style);

        let content = match self.current_page {
            Page::Home => self.home_page(),
            Page::Settings => self.settings_page(),
        };

        row![rail, content].width(Fill).height(Fill).into()
    }

    fn home_page(&self) -> Element<'_, Message> {
        let hero = container(
            column![
                text(self.t("桌面主控台", "Desktop Command Center"))
                    .size(13)
                    .color(self.accent_color()),
                text(self.title()).size(34).color(self.primary_text_color()),
                text(self.t(
                    "桌面端任务按钮现在直接调用 Rust 原生后端。Web UI 也可以接到 Rust 自己的 WebSocket gateway，不再依赖 Python。",
                    "The desktop task button now calls the Rust-native backend directly. The Web UI can also talk to Rust's own WebSocket gateway, with no Python dependency.",
                ))
                .size(16)
                .color(self.secondary_text_color()),
            ]
            .spacing(10),
        )
        .padding(24)
        .style(hero_style);

        let bridge_target = format!("http://127.0.0.1:{}/", crate::server::GATEWAY_PORT);
        let provider_name = Provider::from_config_value(&self.settings.llm_provider).to_string();

        let stats = row![
            metric_card(
                self.t("执行状态", "Execution State"),
                if self.task_running {
                    self.t("运行中", "Running")
                } else {
                    self.t("空闲", "Idle")
                },
                self.status_text.as_str(),
                self.theme(),
            ),
            metric_card(
                self.t("Gateway", "Gateway"),
                self.t("固定开放", "Always on"),
                bridge_target.as_str(),
                self.theme(),
            ),
            metric_card(
                self.t("当前模型", "Current Model"),
                provider_name.as_str(),
                self.current_model_label(),
                self.theme(),
            ),
        ]
        .spacing(14);

        let preview = panel(
            self.t("视觉预览", "Vision Preview"),
            column![
                container(
                    text(self.preview_placeholder())
                        .size(18)
                        .color(self.primary_text_color())
                )
                .width(Fill)
                .height(320)
                .center_x(Fill)
                .center_y(Fill)
                .style(feature_surface_style),
                row![
                    info_pill(
                        self.t("模式", "Mode"),
                        if self.config.use_vision {
                            self.t("视觉识别开启", "Vision enabled")
                        } else {
                            self.t("命令模式", "Command only")
                        },
                        self.theme()
                    ),
                    info_pill(
                        self.t("入口", "Entry"),
                        self.t("桌面 + Web", "Desktop + Web"),
                        self.theme()
                    ),
                ]
                .spacing(10),
            ]
            .spacing(14)
            .into(),
            self.theme(),
        );

        let task_box = panel(
            self.t("任务输入", "Task Input"),
            column![
                text(self.t(
                    "这里现在直接走 Rust 原生执行链。`shell:` 会执行本地命令，`wait:` 会做延时；其他任务会进入原生后端占位流程。",
                    "This now runs through the Rust-native execution chain. `shell:` executes a local command, `wait:` delays, and other tasks enter the native backend placeholder flow.",
                ))
                .size(14)
                .color(self.secondary_text_color()),
                row![
                    text_input(
                        self.t("让 AI 帮我完成一个桌面任务...", "Ask AI to complete a desktop task..."),
                        &self.task_input,
                    )
                    .on_input(|value| Message::TextChanged(TextField::Task, value))
                    .padding(12)
                    .width(Fill),
                    button(text(if self.task_running {
                        self.t("执行中", "Running")
                    } else {
                        self.t("启动", "Launch")
                    }).size(14))
                    .padding([12, 18])
                    .style(accent_button_style)
                    .on_press_maybe((!self.task_running).then_some(Message::StartTask)),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                checkbox(self.t("启用视觉识别", "Enable vision mode"), self.config.use_vision)
                    .on_toggle(|value| Message::BoolChanged(BoolField::VisionEnabled, value)),
            ]
            .spacing(14)
            .into(),
            self.theme(),
        );

        let result_text = self
            .latest_result
            .as_deref()
            .unwrap_or(self.t("还没有收到结果", "No result yet"));
        let log_panel = panel(
            self.t("执行日志", "Execution Log"),
            column![
                text(result_text).size(18).color(self.primary_text_color()),
                horizontal_rule(1),
                self.log_view(),
            ]
            .spacing(14)
            .into(),
            self.theme(),
        );

        let main = row![
            column![preview, task_box]
                .spacing(14)
                .width(Length::FillPortion(3)),
            container(log_panel).width(Length::FillPortion(2)),
        ]
        .spacing(14);

        container(
            column![hero, stats, main]
                .spacing(18)
                .padding([20, 24])
                .width(Fill),
        )
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn settings_page(&self) -> Element<'_, Message> {
        let sections = column![
            settings_tab_button(
                self.t("AI 模型", "AI Model"),
                self.current_settings_section == SettingsSection::Ai,
                SettingsSection::Ai
            ),
            settings_tab_button(
                self.t("截图设置", "Capture"),
                self.current_settings_section == SettingsSection::Capture,
                SettingsSection::Capture
            ),
            settings_tab_button(
                self.t("常规设置", "General"),
                self.current_settings_section == SettingsSection::General,
                SettingsSection::General
            ),
            settings_tab_button(
                self.t("通道桥接", "Channels"),
                self.current_settings_section == SettingsSection::Channels,
                SettingsSection::Channels
            ),
        ]
        .spacing(8);

        let settings_nav = container(
            column![
                text(self.t("设置面板", "Settings Surface"))
                    .size(18)
                    .color(self.primary_text_color()),
                text(self.t("即时预览，保存持久化", "Live preview, save to persist"))
                    .size(13)
                    .color(self.secondary_text_color()),
                horizontal_rule(1),
                sections,
            ]
            .spacing(14),
        )
        .padding(18)
        .width(220)
        .style(panel_style);

        let body = match self.current_settings_section {
            SettingsSection::Ai => self.ai_section(),
            SettingsSection::Capture => self.capture_section(),
            SettingsSection::General => self.general_section(),
            SettingsSection::Channels => self.channels_section(),
        };

        let actions = row![
            text(self.t(
                "保存时会写回现有 Rust 共享 config.json，并保留未知字段。",
                "Saving writes back into the shared Rust config shape and preserves unknown fields.",
            ))
            .size(13)
            .color(self.secondary_text_color()),
            iced::widget::horizontal_space(),
            button(text(self.t("保存配置", "Save Settings")).size(14))
                .padding([12, 18])
                .style(accent_button_style)
                .on_press(Message::SaveSettings),
        ]
        .align_y(Alignment::Center);

        let content = panel(
            self.t("当前表单", "Current Form"),
            column![body, horizontal_rule(1), actions]
                .spacing(18)
                .into(),
            self.theme(),
        );

        container(
            row![settings_nav, container(content).width(Fill)]
                .spacing(18)
                .padding([20, 24]),
        )
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn ai_section(&self) -> Element<'_, Message> {
        let provider = Provider::from_config_value(&self.settings.llm_provider);

        let mut content = column![
            section_title(self.t("AI 模型设置", "AI Model Settings"), self.theme()),
            field_row(
                self.t("提供商", "Provider"),
                pick_list(Provider::ALL, Some(provider), Message::ProviderSelected)
                    .width(240)
                    .into(),
            ),
        ]
        .spacing(14);

        content = match provider {
            Provider::OpenAi => content.push(provider_form(vec![
                field_row(
                    self.t("API Key", "API Key"),
                    secure_input("", &self.settings.openai_api_key, TextField::OpenAiKey),
                ),
                field_row(
                    self.t("模型", "Model"),
                    plain_input("", &self.settings.openai_model, TextField::OpenAiModel),
                ),
            ])),
            Provider::Anthropic => content.push(provider_form(vec![
                field_row(
                    self.t("API Key", "API Key"),
                    secure_input(
                        "",
                        &self.settings.anthropic_api_key,
                        TextField::AnthropicKey,
                    ),
                ),
                field_row(
                    self.t("模型", "Model"),
                    plain_input(
                        "",
                        &self.settings.anthropic_model,
                        TextField::AnthropicModel,
                    ),
                ),
            ])),
            Provider::Ollama => content.push(provider_form(vec![
                field_row(
                    self.t("服务地址", "Base URL"),
                    plain_input("", &self.settings.ollama_base_url, TextField::OllamaUrl),
                ),
                field_row(
                    self.t("模型", "Model"),
                    plain_input("", &self.settings.ollama_model, TextField::OllamaModel),
                ),
            ])),
            Provider::Gemini => content.push(provider_form(vec![
                field_row(
                    self.t("API Key", "API Key"),
                    secure_input("", &self.settings.gemini_api_key, TextField::GeminiKey),
                ),
                field_row(
                    self.t("模型", "Model"),
                    plain_input("", &self.settings.gemini_model, TextField::GeminiModel),
                ),
            ])),
            Provider::Custom => content.push(provider_form(vec![
                field_row(
                    self.t("API Key", "API Key"),
                    secure_input("", &self.settings.custom_api_key, TextField::CustomKey),
                ),
                field_row(
                    self.t("基础地址", "Base URL"),
                    plain_input("", &self.settings.custom_base_url, TextField::CustomUrl),
                ),
                field_row(
                    self.t("模型", "Model"),
                    plain_input("", &self.settings.custom_model, TextField::CustomModel),
                ),
            ])),
        };

        content.into()
    }

    fn capture_section(&self) -> Element<'_, Message> {
        column![
            section_title(self.t("截图与执行", "Capture and Execution"), self.theme()),
            provider_form(vec![
                field_row(
                    self.t("截图质量", "Screenshot Quality"),
                    number_input(
                        "95",
                        &self.settings.screenshot_quality,
                        TextField::ScreenshotQuality
                    ),
                ),
                field_row(
                    self.t("动作延迟（秒）", "Action Delay (sec)"),
                    number_input("0.5", &self.settings.action_delay, TextField::ActionDelay),
                ),
            ]),
        ]
        .spacing(14)
        .into()
    }

    fn general_section(&self) -> Element<'_, Message> {
        let language = Language::from_config_value(&self.settings.language);
        let theme = ThemeMode::from_config_value(&self.settings.theme);

        column![
            section_title(self.t("常规设置", "General Settings"), self.theme()),
            provider_form(vec![
                field_row(
                    self.t("语言", "Language"),
                    pick_list(Language::ALL, Some(language), Message::LanguageSelected)
                        .width(240)
                        .into(),
                ),
                field_row(
                    self.t("主题", "Theme"),
                    pick_list(ThemeMode::ALL, Some(theme), Message::ThemeSelected)
                        .width(240)
                        .into(),
                ),
                field_row(
                    self.t("字体大小", "Font Size"),
                    number_input("9", &self.settings.font_size, TextField::FontSize),
                ),
                field_row(
                    self.t("快捷悬浮窗", "Quick Input Hotkey"),
                    plain_input("", &self.settings.hotkey_float, TextField::HotkeyFloat),
                ),
                field_row(
                    self.t("语音输入热键", "Voice Input Hotkey"),
                    plain_input("", &self.settings.hotkey_voice, TextField::HotkeyVoice),
                ),
                checkbox(
                    self.t("启用唤醒词", "Enable Wake Word"),
                    self.settings.wakeword_enabled
                )
                .on_toggle(|value| Message::BoolChanged(BoolField::WakewordEnabled, value))
                .into(),
                field_row(
                    self.t("唤醒词列表", "Wake Words"),
                    plain_input("", &self.settings.wakeword_list, TextField::WakewordList),
                ),
                field_row(
                    self.t("Picovoice AccessKey", "Picovoice AccessKey"),
                    secure_input(
                        "",
                        &self.settings.porcupine_access_key,
                        TextField::PorcupineKey
                    ),
                ),
            ]),
        ]
        .spacing(14)
        .into()
    }

    fn channels_section(&self) -> Element<'_, Message> {
        let qq_mode = QqMode::from_config_value(&self.settings.qq_mode);

        column![
            section_title(
                self.t("通道桥接设置", "Channel Bridge Settings"),
                self.theme()
            ),
            channel_block(
                self.t("WebSocket Gateway", "WebSocket Gateway"),
                vec![
                    checkbox(self.t("启用", "Enabled"), self.settings.ws_enabled)
                        .on_toggle(|value| Message::BoolChanged(BoolField::WsEnabled, value))
                        .into(),
                    field_row(
                        self.t("地址", "Host"),
                        plain_input("", &self.settings.ws_host, TextField::WsHost),
                    ),
                    field_row(
                        self.t("端口", "Port"),
                        number_input("8765", &self.settings.ws_port, TextField::WsPort),
                    ),
                    field_row(
                        self.t("令牌", "Token"),
                        secure_input("", &self.settings.ws_token, TextField::WsToken),
                    ),
                ]
            ),
            channel_block(
                self.t("微信 iLink", "WeChat iLink"),
                vec![
                    checkbox(self.t("启用", "Enabled"), self.settings.wc_enabled)
                        .on_toggle(|value| Message::BoolChanged(BoolField::WcEnabled, value))
                        .into(),
                    field_row(
                        self.t("Bot Token", "Bot Token"),
                        secure_input("", &self.settings.wc_token, TextField::WcToken),
                    ),
                ]
            ),
            channel_block(
                self.t("企业微信", "WeCom"),
                vec![
                    checkbox(self.t("启用", "Enabled"), self.settings.wcom_enabled)
                        .on_toggle(|value| Message::BoolChanged(BoolField::WcomEnabled, value))
                        .into(),
                    field_row(
                        self.t("Corp ID", "Corp ID"),
                        plain_input("", &self.settings.wcom_corp_id, TextField::WcomCorpId),
                    ),
                    field_row(
                        self.t("Agent ID", "Agent ID"),
                        plain_input("", &self.settings.wcom_agent_id, TextField::WcomAgentId),
                    ),
                    field_row(
                        self.t("Secret", "Secret"),
                        secure_input("", &self.settings.wcom_secret, TextField::WcomSecret),
                    ),
                    field_row(
                        self.t("Token", "Token"),
                        secure_input("", &self.settings.wcom_token, TextField::WcomToken),
                    ),
                    field_row(
                        self.t("AES Key", "AES Key"),
                        secure_input("", &self.settings.wcom_aes_key, TextField::WcomAesKey),
                    ),
                ]
            ),
            channel_block(
                self.t("钉钉", "DingTalk"),
                vec![
                    checkbox(self.t("启用", "Enabled"), self.settings.dt_enabled)
                        .on_toggle(|value| Message::BoolChanged(BoolField::DtEnabled, value))
                        .into(),
                    field_row(
                        self.t("App Key", "App Key"),
                        plain_input("", &self.settings.dt_app_key, TextField::DtAppKey),
                    ),
                    field_row(
                        self.t("App Secret", "App Secret"),
                        secure_input("", &self.settings.dt_app_secret, TextField::DtAppSecret),
                    ),
                    field_row(
                        self.t("Webhook", "Webhook"),
                        plain_input("", &self.settings.dt_webhook, TextField::DtWebhook),
                    ),
                ]
            ),
            channel_block(
                self.t("QQ", "QQ"),
                vec![
                    checkbox(self.t("启用", "Enabled"), self.settings.qq_enabled)
                        .on_toggle(|value| Message::BoolChanged(BoolField::QqEnabled, value))
                        .into(),
                    field_row(
                        self.t("模式", "Mode"),
                        pick_list(QqMode::ALL, Some(qq_mode), Message::QqModeSelected)
                            .width(240)
                            .into(),
                    ),
                    field_row(
                        self.t("WebSocket 地址", "WebSocket Host"),
                        plain_input("", &self.settings.qq_ws_host, TextField::QqWsHost),
                    ),
                    field_row(
                        self.t("WebSocket 端口", "WebSocket Port"),
                        number_input("6700", &self.settings.qq_ws_port, TextField::QqWsPort),
                    ),
                    field_row(
                        self.t("Bot AppID", "Bot AppID"),
                        plain_input("", &self.settings.qq_bot_appid, TextField::QqBotAppId),
                    ),
                    field_row(
                        self.t("Bot Token", "Bot Token"),
                        secure_input("", &self.settings.qq_bot_token, TextField::QqBotToken),
                    ),
                ]
            ),
        ]
        .spacing(14)
        .into()
    }

    fn preview_placeholder(&self) -> &'static str {
        if self.config.use_vision {
            self.t("等待截图...", "Waiting for screenshot...")
        } else {
            self.t("屏幕识别已关闭", "Screen recognition disabled")
        }
    }

    fn current_model_label(&self) -> &str {
        match Provider::from_config_value(&self.settings.llm_provider) {
            Provider::OpenAi => self.settings.openai_model.as_str(),
            Provider::Anthropic => self.settings.anthropic_model.as_str(),
            Provider::Ollama => self.settings.ollama_model.as_str(),
            Provider::Gemini => self.settings.gemini_model.as_str(),
            Provider::Custom => self.settings.custom_model.as_str(),
        }
    }

    fn language(&self) -> Language {
        Language::from_config_value(&self.settings.language)
    }

    fn t<'a>(&self, zh: &'a str, en: &'a str) -> &'a str {
        let _ = zh;
        if self.language().is_zh() {
            zh_text(en)
        } else {
            en
        }
    }

    fn refresh_runtime_preferences(&mut self) {
        self.theme = ThemeMode::from_config_value(&self.settings.theme).to_theme();
    }

    fn accent_color(&self) -> Color {
        self.theme.extended_palette().primary.strong.color
    }

    fn primary_text_color(&self) -> Color {
        self.theme.extended_palette().background.base.text
    }

    fn secondary_text_color(&self) -> Color {
        let base = self.theme.extended_palette().background.base.text;
        Color { a: 0.94, ..base }
    }

    fn muted_text_color(&self) -> Color {
        let c = self.theme.extended_palette().background.base.text;
        Color { a: 0.82, ..c }
    }

    fn push_log(&mut self, kind: LogKind, title: impl Into<String>, detail: impl Into<String>) {
        self.logs.push(LogEntry {
            kind,
            title: title.into(),
            detail: detail.into(),
        });

        if self.logs.len() > 64 {
            let overflow = self.logs.len() - 64;
            self.logs.drain(0..overflow);
        }
    }

    fn log_view(&self) -> Element<'_, Message> {
        if self.logs.is_empty() {
            return text(self.t("还没有日志", "No logs yet"))
                .size(14)
                .color(self.secondary_text_color())
                .into();
        }

        let items: Vec<Element<'_, Message>> = self
            .logs
            .iter()
            .rev()
            .take(8)
            .map(|entry| log_card(entry, self.theme()))
            .collect();

        scrollable(column(items).spacing(10)).height(280).into()
    }

    fn update_text_field(&mut self, field: TextField, value: String) {
        match field {
            TextField::Task => self.task_input = value,
            TextField::OpenAiKey => self.settings.openai_api_key = value,
            TextField::OpenAiModel => self.settings.openai_model = value,
            TextField::AnthropicKey => self.settings.anthropic_api_key = value,
            TextField::AnthropicModel => self.settings.anthropic_model = value,
            TextField::OllamaUrl => self.settings.ollama_base_url = value,
            TextField::OllamaModel => self.settings.ollama_model = value,
            TextField::CustomKey => self.settings.custom_api_key = value,
            TextField::CustomUrl => self.settings.custom_base_url = value,
            TextField::CustomModel => self.settings.custom_model = value,
            TextField::GeminiKey => self.settings.gemini_api_key = value,
            TextField::GeminiModel => self.settings.gemini_model = value,
            TextField::ScreenshotQuality => self.settings.screenshot_quality = value,
            TextField::ActionDelay => self.settings.action_delay = value,
            TextField::FontSize => self.settings.font_size = value,
            TextField::HotkeyFloat => self.settings.hotkey_float = value,
            TextField::HotkeyVoice => self.settings.hotkey_voice = value,
            TextField::WakewordList => self.settings.wakeword_list = value,
            TextField::PorcupineKey => self.settings.porcupine_access_key = value,
            TextField::WsHost => self.settings.ws_host = value,
            TextField::WsPort => self.settings.ws_port = value,
            TextField::WsToken => self.settings.ws_token = value,
            TextField::WcToken => self.settings.wc_token = value,
            TextField::WcomCorpId => self.settings.wcom_corp_id = value,
            TextField::WcomAgentId => self.settings.wcom_agent_id = value,
            TextField::WcomSecret => self.settings.wcom_secret = value,
            TextField::WcomToken => self.settings.wcom_token = value,
            TextField::WcomAesKey => self.settings.wcom_aes_key = value,
            TextField::DtAppKey => self.settings.dt_app_key = value,
            TextField::DtAppSecret => self.settings.dt_app_secret = value,
            TextField::DtWebhook => self.settings.dt_webhook = value,
            TextField::QqWsHost => self.settings.qq_ws_host = value,
            TextField::QqWsPort => self.settings.qq_ws_port = value,
            TextField::QqBotAppId => self.settings.qq_bot_appid = value,
            TextField::QqBotToken => self.settings.qq_bot_token = value,
        }
    }

    fn update_bool_field(&mut self, field: BoolField, value: bool) {
        match field {
            BoolField::WakewordEnabled => self.settings.wakeword_enabled = value,
            BoolField::WsEnabled => self.settings.ws_enabled = value,
            BoolField::WcEnabled => self.settings.wc_enabled = value,
            BoolField::WcomEnabled => self.settings.wcom_enabled = value,
            BoolField::DtEnabled => self.settings.dt_enabled = value,
            BoolField::QqEnabled => self.settings.qq_enabled = value,
            BoolField::VisionEnabled => self.config.use_vision = value,
        }
    }
}

fn panel<'a>(title: &'a str, body: Element<'a, Message>, theme: Theme) -> Element<'a, Message> {
    container(
        column![
            text(title)
                .size(18)
                .color(theme.extended_palette().primary.strong.color),
            body,
        ]
        .spacing(14),
    )
    .padding(18)
    .style(panel_style)
    .into()
}

fn provider_form<'a>(fields: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    container(column(fields).spacing(14))
        .padding(18)
        .style(feature_surface_style)
        .width(Fill)
        .into()
}

fn channel_block<'a>(title: &'a str, items: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    container(column![text(title).size(16), column(items).spacing(12)].spacing(14))
        .padding(18)
        .style(feature_surface_style)
        .into()
}

fn metric_card<'a>(
    label: &'a str,
    value: impl Into<String>,
    detail: impl Into<String>,
    theme: Theme,
) -> Element<'a, Message> {
    let value = value.into();
    let detail = detail.into();

    container(
        column![
            text(label)
                .size(14)
                .color(theme.extended_palette().primary.strong.color),
            text(value)
                .size(26)
                .color(theme.extended_palette().background.base.text),
            text(detail)
                .size(14)
                .color(theme.extended_palette().background.base.text),
        ]
        .spacing(8),
    )
    .padding(18)
    .width(Length::FillPortion(1))
    .style(panel_style)
    .into()
}

fn info_pill<'a>(label: &'a str, value: &'a str, theme: Theme) -> Element<'a, Message> {
    container(
        row![
            text(label)
                .size(13)
                .color(theme.extended_palette().primary.strong.color),
            text(value)
                .size(13)
                .color(theme.extended_palette().background.base.text),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .style(feature_surface_style)
    .into()
}

fn log_card<'a>(entry: &'a LogEntry, theme: Theme) -> Element<'a, Message> {
    let accent = match entry.kind {
        LogKind::Info => theme.extended_palette().secondary.strong.color,
        LogKind::Status => theme.extended_palette().primary.strong.color,
        LogKind::Success => theme.extended_palette().success.strong.color,
        LogKind::Error => theme.extended_palette().danger.strong.color,
    };

    container(
        column![
            text(entry.title.as_str()).size(14).color(accent),
            text(entry.detail.as_str())
                .size(15)
                .color(theme.extended_palette().background.base.text),
        ]
        .spacing(6),
    )
    .padding(14)
    .style(feature_surface_style)
    .into()
}

fn section_title<'a>(label: &'a str, theme: Theme) -> Element<'a, Message> {
    text(label)
        .size(22)
        .color(theme.extended_palette().background.base.text)
        .into()
}

fn plain_input<'a>(placeholder: &'a str, value: &'a str, field: TextField) -> Element<'a, Message> {
    text_input(placeholder, value)
        .on_input(move |next| Message::TextChanged(field, next))
        .padding(10)
        .width(Fill)
        .into()
}

fn secure_input<'a>(
    placeholder: &'a str,
    value: &'a str,
    field: TextField,
) -> Element<'a, Message> {
    text_input(placeholder, value)
        .secure(true)
        .on_input(move |next| Message::TextChanged(field, next))
        .padding(10)
        .width(Fill)
        .into()
}

fn number_input<'a>(
    placeholder: &'a str,
    value: &'a str,
    field: TextField,
) -> Element<'a, Message> {
    text_input(placeholder, value)
        .on_input(move |next| Message::TextChanged(field, next))
        .padding(10)
        .width(180)
        .into()
}

fn field_row<'a>(label: &'a str, input: Element<'a, Message>) -> Element<'a, Message> {
    row![
        text(label).width(180).size(14).align_x(Horizontal::Left),
        input,
    ]
    .spacing(14)
    .align_y(Alignment::Center)
    .into()
}

fn nav_button<'a>(label: &'a str, selected: bool, page: Page) -> Element<'a, Message> {
    button(text(label).size(15))
        .padding([12, 14])
        .style(move |theme: &Theme, status| {
            if selected {
                accent_button_style(theme, status)
            } else {
                subtle_button_style(theme, status)
            }
        })
        .width(Fill)
        .on_press(Message::SidebarClick(page))
        .into()
}

fn settings_tab_button<'a>(
    label: &'a str,
    selected: bool,
    section: SettingsSection,
) -> Element<'a, Message> {
    button(text(label).size(14))
        .padding([10, 14])
        .style(move |theme: &Theme, status| {
            if selected {
                accent_button_style(theme, status)
            } else {
                subtle_button_style(theme, status)
            }
        })
        .width(Fill)
        .on_press(Message::SettingsSectionSelected(section))
        .into()
}

fn panel_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let bg = Color {
        r: palette.background.weak.color.r * 0.88 + palette.background.base.color.r * 0.12,
        g: palette.background.weak.color.g * 0.88 + palette.background.base.color.g * 0.12,
        b: palette.background.weak.color.b * 0.88 + palette.background.base.color.b * 0.12,
        a: 0.99,
    };
    container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            width: 1.0,
            radius: 16.0.into(),
            color: palette.primary.base.color,
        },
        text_color: Some(palette.background.base.text),
        ..container::Style::default()
    }
}

fn feature_surface_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let bg = Color {
        a: 0.98,
        ..palette.background.base.color
    };
    container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            width: 1.0,
            radius: 14.0.into(),
            color: palette.primary.weak.color,
        },
        text_color: Some(palette.background.base.text),
        ..container::Style::default()
    }
}

fn hero_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let bg = Color {
        r: palette.primary.strong.color.r * 0.35 + palette.background.base.color.r * 0.65,
        g: palette.primary.strong.color.g * 0.35 + palette.background.base.color.g * 0.65,
        b: palette.primary.strong.color.b * 0.35 + palette.background.base.color.b * 0.65,
        a: 1.0,
    };

    container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            width: 1.0,
            radius: 22.0.into(),
            color: palette.primary.base.color,
        },
        text_color: Some(palette.background.base.text),
        ..container::Style::default()
    }
}

fn aside_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let bg = Color {
        r: palette.background.base.color.r * 0.92,
        g: palette.background.base.color.g * 0.96,
        b: palette.background.base.color.b * 1.04,
        a: 1.0,
    };

    container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            width: 0.0,
            radius: 0.0.into(),
            color: palette.background.strong.color,
        },
        text_color: Some(palette.background.base.text),
        ..container::Style::default()
    }
}

fn accent_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let color = match status {
        button::Status::Hovered => palette.primary.base.color,
        button::Status::Pressed => palette.primary.strong.color,
        button::Status::Disabled => palette.primary.weak.color,
        _ => palette.primary.strong.color,
    };

    button::Style {
        background: Some(Background::Color(color)),
        text_color: palette.background.base.color,
        border: Border {
            width: 0.0,
            radius: 12.0.into(),
            color,
        },
        ..button::Style::default()
    }
}

fn subtle_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let background = match status {
        button::Status::Hovered => palette.background.weak.color,
        button::Status::Pressed => palette.background.strong.color,
        _ => Color {
            a: 0.0,
            ..palette.background.base.color
        },
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.background.base.text,
        border: Border {
            width: 1.0,
            radius: 12.0.into(),
            color: palette.background.strong.color,
        },
        ..button::Style::default()
    }
}

fn zh_text(english: &str) -> &str {
    match english {
        "Home" => "首页",
        "Settings" => "设置",
        "Desktop Console" => "桌面控制台",
        "Desktop Command Center" => "桌面主控台",
        "Execution State" => "执行状态",
        "Running" => "运行中",
        "Idle" => "空闲",
        "Gateway" => "网关",
        "Always on" => "固定开放",
        "Current Model" => "当前模型",
        "Vision Preview" => "视觉预览",
        "Mode" => "模式",
        "Vision enabled" => "视觉识别开启",
        "Command only" => "命令模式",
        "Entry" => "入口",
        "Desktop + Web" => "桌面 + Web",
        "Task Input" => "任务输入",
        "This now runs through the Rust-native execution chain. `shell:` executes a local command, `wait:` delays, and other tasks enter the native backend placeholder flow." =>
            "这里现在直接走 Rust 原生执行链。`shell:` 会执行本地命令，`wait:` 会做延时；其他任务会进入原生后端占位流程。",
        "Ask AI to complete a desktop task..." => "让 AI 帮我完成一个桌面任务...",
        "Launch" => "启动",
        "Enable vision mode" => "启用视觉识别",
        "No result yet" => "还没有收到结果",
        "Execution Log" => "执行日志",
        "AI Model" => "AI 模型",
        "Capture" => "截图设置",
        "General" => "常规设置",
        "Channels" => "通道桥接",
        "Settings Surface" => "设置面板",
        "Live preview, save to persist" => "即时预览，保存持久化",
        "Saving writes back into the shared Rust config shape and preserves unknown fields." =>
            "保存时会写回现有 Rust 共享 config.json，并保留未知字段。",
        "Save Settings" => "保存配置",
        "Current Form" => "当前表单",
        "AI Model Settings" => "AI 模型设置",
        "Provider" => "提供商",
        "Model" => "模型",
        "Base URL" => "基础地址",
        "Capture and Execution" => "截图与执行",
        "Screenshot Quality" => "截图质量",
        "Action Delay (sec)" => "动作延迟（秒）",
        "General Settings" => "常规设置",
        "Language" => "语言",
        "Theme" => "主题",
        "Font Size" => "字体大小",
        "Quick Input Hotkey" => "快捷悬浮窗",
        "Voice Input Hotkey" => "语音输入热键",
        "Enable Wake Word" => "启用唤醒词",
        "Wake Words" => "唤醒词列表",
        "Channel Bridge Settings" => "通道桥接设置",
        "WebSocket Gateway" => "WebSocket 网关",
        "Enabled" => "启用",
        "Host" => "地址",
        "Port" => "端口",
        "Token" => "令牌",
        "WeChat iLink" => "微信 iLink",
        "WeCom" => "企业微信",
        "DingTalk" => "钉钉",
        "QQ" => "QQ",
        "WebSocket Host" => "WebSocket 地址",
        "WebSocket Port" => "WebSocket 端口",
        "Waiting for screenshot..." => "等待截图...",
        "Screen recognition disabled" => "屏幕识别已关闭",
        "No logs yet" => "还没有日志",
        _ => english,
    }
}
