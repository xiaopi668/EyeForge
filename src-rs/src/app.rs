use iced::alignment::Horizontal;
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, image, pick_list, row, scrollable, text,
    text_input,
};
use iced::{time, window, Alignment, Background, Border, Color, ContentFit, Element, Fill, Length, Padding};
use iced::{Subscription, Task, Theme};
use std::time::Duration;

use crate::config::{Config, EditableConfig};
use crate::runtime::{self, NativeOutcome};
use crate::server;
use crate::wechat::{QrLoginResult, QrLoginSession};

const VERSION: &str = "1.5.0-beta.2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Settings,
    Skills,
    AiGroups,
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
    SkillsEnabled,
    AiGroupName,
    AiGroupPeople,
    AiGroupStrategy,
    AiGroupOpenclawMembers,
    AiGroupAstrbotMembers,
    AiGroupHapiEndpoint,
    AiGroupOpencodeMembers,
    AiGroupCodexMembers,
    AiGroupClaudeCodeMembers,
    SkillImportPath,
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
    AiGroupsEnabled,
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
    WechatQrLogin,
    WechatQrReady(Result<QrLoginSession, String>),
    WechatQrLoginFinished(Result<QrLoginResult, String>),
    TextChanged(TextField, String),
    BoolChanged(BoolField, bool),
    ImportSkill,
    SkillImported(Result<crate::skills::ImportedSkill, String>),
    CreateAiGroup,
    AddGroupMember,
    AddGroupAi,
    SaveSettings,
    StartTask,
    BackendFinished(Result<NativeOutcome, String>),
    WindowDiscovered(Option<window::Id>),
    WindowCloseRequested(window::Id),
    WindowMinimizePressed,
    WindowMaximizePressed,
    WindowClosePressed,
    TrayTick,
    ShowWindowById(Option<window::Id>),
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
    skill_import_path: String,
    logs: Vec<LogEntry>,
    task_running: bool,
    wechat_login_running: bool,
    wechat_qr_image: Option<image::Handle>,
    wechat_qr_status: Option<String>,
    main_window_id: Option<window::Id>,
    window_maximized: bool,
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
            skill_import_path: String::new(),
            logs: Vec::new(),
            task_running: false,
            wechat_login_running: false,
            wechat_qr_image: None,
            wechat_qr_status: None,
            main_window_id: None,
            window_maximized: false,
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
            skill_import_path: String::new(),
            logs: Vec::new(),
            task_running: false,
            wechat_login_running: false,
            wechat_qr_image: None,
            wechat_qr_status: None,
            main_window_id: None,
            window_maximized: false,
        };

        let gateway_status = server::restart(&app.config);
        app.push_log(
            LogKind::Info,
            if language.is_zh() { "启动" } else { "Boot" },
            match gateway_status {
                Ok(_) if app.config.ws_enabled => format!(
                    "Rust gateway starting on http://{}:{}/ and ws://{}:{}/ws",
                    ws_host, app.config.ws_port, ws_host, app.config.ws_port
                ),
                Ok(_) => app
                    .t(
                        "设置中已禁用 Rust 网关",
                        "Rust gateway is disabled in settings",
                    )
                    .to_string(),
                Err(error) => format!(
                    "Rust gateway failed to start on {}:{}: {error}",
                    ws_host, app.config.ws_port
                ),
            },
        );

        let wakeword_status = crate::wakeword::restart(&app.config);
        app.push_log(
            match wakeword_status {
                Ok(_) => LogKind::Info,
                Err(_) => LogKind::Error,
            },
            if language.is_zh() {
                "语音唤醒"
            } else {
                "Wake Word"
            },
            match wakeword_status {
                Ok(message) => message,
                Err(error) => error,
            },
        );

        let ai_groups_status = crate::ai_groups::restart(&app.config);
        app.push_log(
            match ai_groups_status {
                Ok(_) => LogKind::Info,
                Err(_) => LogKind::Error,
            },
            if language.is_zh() {
                "AI 群组"
            } else {
                "AI Groups"
            },
            match ai_groups_status {
                Ok(message) => message,
                Err(error) => error,
            },
        );

        (app, window::get_latest().map(Message::WindowDiscovered))
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

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::close_requests().map(Message::WindowCloseRequested),
            time::every(Duration::from_millis(250)).map(|_| Message::TrayTick),
        ])
    }

    fn restore_window(id: window::Id) -> Task<Message> {
        Task::batch([
            window::change_mode(id, window::Mode::Windowed),
            window::minimize(id, false),
            window::gain_focus(id),
        ])
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowDiscovered(id) => {
                self.main_window_id = id;
            }
            Message::WindowCloseRequested(id) => {
                self.main_window_id = Some(id);
                self.status_text = if self.language().is_zh() {
                    "已隐藏到系统托盘，右键托盘图标可显示或退出".into()
                } else {
                    "Hidden to the system tray. Right-click the tray icon to show or exit.".into()
                };
                return window::change_mode(id, window::Mode::Hidden);
            }
            Message::WindowMinimizePressed => {
                if let Some(id) = self.main_window_id {
                    return window::minimize(id, true);
                }
            }
            Message::WindowMaximizePressed => {
                if let Some(id) = self.main_window_id {
                    self.window_maximized = !self.window_maximized;
                    return window::maximize(id, self.window_maximized);
                }
            }
            Message::WindowClosePressed => {
                if let Some(id) = self.main_window_id {
                    self.status_text = if self.language().is_zh() {
                        "已隐藏到系统托盘，右键托盘图标可显示或退出".into()
                    } else {
                        "Hidden to system tray. Right-click to show or exit.".into()
                    };
                    return window::change_mode(id, window::Mode::Hidden);
                }
            }
            Message::TrayTick => match crate::tray::next_command() {
                Some(crate::tray::TrayCommand::Show) => {
                    if let Some(id) = self.main_window_id {
                        return Self::restore_window(id);
                    }

                    return window::get_latest().map(Message::ShowWindowById);
                }
                Some(crate::tray::TrayCommand::Exit) => {
                    return iced::exit();
                }
                None => {}
            },
            Message::ShowWindowById(id) => {
                self.main_window_id = id;
                if let Some(id) = id {
                    return Self::restore_window(id);
                }
            }
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
            Message::WechatQrLogin => {
                if self.wechat_login_running {
                    return Task::none();
                }

                self.wechat_login_running = true;
                self.wechat_qr_image = None;
                self.wechat_qr_status = Some(
                    self.t("正在获取二维码，请稍候...", "Requesting the QR code...")
                        .to_string(),
                );
                self.status_text = if self.language().is_zh() {
                    "正在获取微信二维码...".into()
                } else {
                    "Requesting the WeChat QR code...".into()
                };
                self.push_log(
                    LogKind::Info,
                    self.t("微信 iLink", "WeChat iLink"),
                    self.t(
                        "二维码会直接显示在设置页中，请使用微信扫码并在手机上确认。",
                        "The QR code will be shown in the settings page. Scan it with WeChat and confirm on your phone.",
                    ),
                );
                return Task::perform(crate::wechat::begin_qr_login(), Message::WechatQrReady);
            }
            Message::WechatQrReady(result) => match result {
                Ok(session) => {
                    let key = session.key.clone();
                    self.wechat_qr_image = Some(image::Handle::from_bytes(session.image_bytes));
                    self.wechat_qr_status = Some(
                        self.t(
                            "请使用微信扫码，然后在手机上确认登录。",
                            "Scan the QR code with WeChat and confirm the login on your phone.",
                        )
                        .to_string(),
                    );
                    self.status_text = self
                        .t(
                            "二维码已就绪，请扫码确认。",
                            "The QR code is ready. Scan it to continue.",
                        )
                        .to_string();
                    return Task::perform(
                        async move { crate::wechat::wait_for_qr_confirmation(&key).await },
                        Message::WechatQrLoginFinished,
                    );
                }
                Err(error) => {
                    self.wechat_login_running = false;
                    self.wechat_qr_status = Some(error.clone());
                    self.status_text = if self.language().is_zh() {
                        format!("获取微信二维码失败: {error}")
                    } else {
                        format!("Failed to get the WeChat QR code: {error}")
                    };
                    self.push_log(LogKind::Error, self.t("微信 iLink", "WeChat iLink"), error);
                }
            },
            Message::WechatQrLoginFinished(result) => {
                self.wechat_login_running = false;

                match result {
                    Ok(login) => {
                        self.settings.wc_token = login.token;
                        self.wechat_qr_status = Some(
                            self.t(
                                "登录成功，Token 已自动填入。",
                                "Login succeeded and the token was filled in automatically.",
                            )
                            .to_string(),
                        );
                        self.status_text = if self.language().is_zh() {
                            "微信扫码登录成功，Token 已自动填入".into()
                        } else {
                            "WeChat QR login succeeded and the token was filled in automatically"
                                .into()
                        };
                        self.push_log(
                            LogKind::Success,
                            self.t("微信 iLink", "WeChat iLink"),
                            format!(
                                "{} Bot ID: {} | User ID: {}",
                                self.t("登录成功。", "Login succeeded."),
                                if login.bot_id.is_empty() {
                                    self.t("(空)", "(empty)")
                                } else {
                                    login.bot_id.as_str()
                                },
                                if login.user_id.is_empty() {
                                    self.t("(空)", "(empty)")
                                } else {
                                    login.user_id.as_str()
                                }
                            ),
                        );
                    }
                    Err(error) => {
                        self.wechat_qr_status = Some(error.clone());
                        self.status_text = if self.language().is_zh() {
                            format!("微信扫码登录失败: {error}")
                        } else {
                            format!("WeChat QR login failed: {error}")
                        };
                        self.push_log(LogKind::Error, self.t("微信 iLink", "WeChat iLink"), error);
                    }
                }
            }
            Message::TextChanged(field, value) => self.update_text_field(field, value),
            Message::BoolChanged(field, value) => self.update_bool_field(field, value),
            Message::ImportSkill => {
                let source = self.skill_import_path.clone();
                self.status_text = if self.language().is_zh() {
                    "正在导入 Skill...".into()
                } else {
                    "Importing Skill...".into()
                };
                return Task::perform(
                    async move { crate::skills::import_skill(&source) },
                    Message::SkillImported,
                );
            }
            Message::SkillImported(result) => match result {
                Ok(skill) => {
                    let mut enabled = self
                        .settings
                        .skills_enabled
                        .split(',')
                        .map(str::trim)
                        .filter(|item| !item.is_empty())
                        .map(str::to_string)
                        .collect::<Vec<_>>();
                    if !enabled.iter().any(|item| item == &skill.name) {
                        enabled.push(skill.name.clone());
                    }
                    self.settings.skills_enabled = enabled.join(", ");
                    self.status_text = if self.language().is_zh() {
                        format!("Skill 已导入: {}", skill.name)
                    } else {
                        format!("Skill imported: {}", skill.name)
                    };
                    self.push_log(
                        LogKind::Success,
                        self.t("Skill 导入", "Skill Import"),
                        format!("{} -> {}", skill.name, skill.path.display()),
                    );
                }
                Err(error) => {
                    self.status_text = error.clone();
                    self.push_log(LogKind::Error, self.t("Skill 导入", "Skill Import"), error);
                }
            },
            Message::CreateAiGroup => {
                if self.settings.ai_group_name.trim().is_empty() {
                    self.settings.ai_group_name = if self.language().is_zh() {
                        "我的 AI 群组".into()
                    } else {
                        "My AI Group".into()
                    };
                }
                self.settings.ai_groups_enabled = true;
                self.status_text = if self.language().is_zh() {
                    format!("已创建群聊：{}", self.settings.ai_group_name)
                } else {
                    format!("Group created: {}", self.settings.ai_group_name)
                };
            }
            Message::AddGroupMember => {
                let member_line = if self.language().is_zh() {
                    "成员 | 负责人 | 可聊天"
                } else {
                    "Member | Owner | Available"
                };
                append_unique_line(&mut self.settings.ai_group_people, member_line);
                self.status_text = self
                    .t(
                        "已添加成员，请修改名称后保存",
                        "Member added. Rename it and save.",
                    )
                    .to_string();
            }
            Message::AddGroupAi => {
                append_unique_line(
                    &mut self.settings.ai_group_codex_members,
                    "Codex | implementer | http://127.0.0.1:9102",
                );
                self.status_text = self
                    .t(
                        "已添加 AI 成员，请修改连接地址后保存",
                        "AI member added. Update its endpoint and save.",
                    )
                    .to_string();
            }
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
                        Ok(_) if self.config.ws_enabled => format!(
                            "Rust gateway configured for http://{}:{}/",
                            self.config.ws_host, self.config.ws_port
                        ),
                        Ok(_) => self
                            .t("Rust 网关已禁用", "Rust gateway disabled")
                            .to_string(),
                        Err(error) => error,
                    },
                );

                let wakeword_result = crate::wakeword::restart(&self.config);
                self.push_log(
                    match wakeword_result {
                        Ok(_) => LogKind::Info,
                        Err(_) => LogKind::Error,
                    },
                    self.t("语音唤醒", "Wake Word"),
                    match wakeword_result {
                        Ok(message) => message,
                        Err(error) => error,
                    },
                );

                let ai_groups_result = crate::ai_groups::restart(&self.config);
                self.push_log(
                    match ai_groups_result {
                        Ok(_) => LogKind::Info,
                        Err(_) => LogKind::Error,
                    },
                    self.t("AI 群组", "AI Groups"),
                    match ai_groups_result {
                        Ok(message) => message,
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
                            self.push_log(LogKind::Info, self.t("运行时", "Runtime"), line);
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

        // ── 顶部导航栏 ──
        let title_text = row![
            text("EyeForge").size(16).color(Color::WHITE),
            text(format!(" v{VERSION}"))
                .size(11)
                .color(Color { a: 0.7, ..Color::WHITE }),
        ]
        .spacing(2)
        .align_y(Alignment::Center);

        let nav_buttons = row![
            top_nav_button(self.t("首页", "Home"), self.current_page == Page::Home, Page::Home),
            top_nav_button(self.t("设置", "Settings"), self.current_page == Page::Settings, Page::Settings),
            top_nav_button(self.t("Skill", "Skills"), self.current_page == Page::Skills, Page::Skills),
            top_nav_button(self.t("AI 群组", "AI Groups"), self.current_page == Page::AiGroups, Page::AiGroups),
        ]
        .spacing(2);

        let window_buttons = row![
            window_ctrl_button("─", Message::WindowMinimizePressed)
                .width(46)
                .height(32),
            window_ctrl_button(
                if self.window_maximized { "❐" } else { "□" },
                Message::WindowMaximizePressed
            )
            .width(46)
            .height(32),
            window_ctrl_button("✕", Message::WindowClosePressed)
                .width(46)
                .height(32)
                .style(close_button_style),
        ]
        .spacing(0);

        let top_bar = container(
            row![
                title_text,
                iced::widget::horizontal_space(),
                nav_buttons,
                iced::widget::horizontal_space().width(60),
                window_buttons,
            ]
            .align_y(Alignment::Center)
            .padding(Padding::new(0.0).right(16.0)),
        )
        .width(Fill)
        .height(40)
        .style(top_bar_style);

        let content = match self.current_page {
            Page::Home => self.home_page(),
            Page::Settings => self.settings_page(),
            Page::Skills => self.skills_page(),
            Page::AiGroups => self.ai_groups_page(),
        };

        column![top_bar, content]
            .width(Fill)
            .height(Fill)
            .into()
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

        let bridge_target = if self.config.ws_enabled {
            format!("http://{}:{}/", self.config.ws_host, self.config.ws_port)
        } else {
            self.t("网关已禁用", "Gateway disabled").to_string()
        };
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
                if self.config.ws_enabled {
                    self.t("已启用", "Enabled")
                } else {
                    self.t("已禁用", "Disabled")
                },
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
            row![
                settings_nav,
                scrollable(container(content).width(Fill)).height(Fill)
            ]
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
                    self.t("Rustpotter 模型文件", "Rustpotter Model Files"),
                    plain_input("C:\\wakewords\\eyeforge.rpw", &self.settings.wakeword_list, TextField::WakewordList),
                ),
                helper_text(
                    self.t(
                        "使用逗号分隔多个 Rustpotter .rpw 模型/参考文件路径；不再需要 Picovoice AccessKey 或 Porcupine 动态库。",
                        "Use comma-separated Rustpotter .rpw model/reference file paths. Picovoice AccessKey and Porcupine libraries are no longer required."
                    ),
                    self.theme()
                )
                .into(),
            ]),
        ]
        .spacing(14)
        .into()
    }

    fn channels_section(&self) -> Element<'_, Message> {
        let qq_mode = QqMode::from_config_value(&self.settings.qq_mode);
        let wechat_qr_panel: Element<'_, Message> =
            if let Some(handle) = self.wechat_qr_image.clone() {
                container(
                    column![
                        text(self.t("微信登录二维码", "WeChat Login QR Code"))
                            .size(14)
                            .color(self.accent_color()),
                        image(handle)
                            .width(220)
                            .height(220)
                            .content_fit(ContentFit::Contain),
                        text(
                            self.wechat_qr_status
                                .as_deref()
                                .unwrap_or(self.t("请扫码", "Please scan the QR code")),
                        )
                        .size(13)
                        .color(self.secondary_text_color()),
                    ]
                    .spacing(12)
                    .align_x(Alignment::Center),
                )
                .padding(16)
                .style(feature_surface_style)
                .width(Fill)
                .into()
            } else {
                text(self.wechat_qr_status.as_deref().unwrap_or(self.t(
                    "点击“扫码登录”后，这里会显示二维码。",
                    "The QR code will appear here after you click QR Login.",
                )))
                .size(13)
                .color(self.secondary_text_color())
                .into()
            };

        column![
            section_title(
                self.t("通道桥接设置", "Channel Bridge Settings"),
                self.theme()
            ),
            channel_block(
                self.t("WebSocket Gateway", "WebSocket Gateway"),
                vec![
                    checkbox(self.t("启用 Web UI / WebSocket 网关", "Enable Web UI / WebSocket Gateway"), self.settings.ws_enabled)
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
                    helper_text(self.t(
                        "原来的扫码登录流程还没迁移完，当前先保留按钮和 Token 配置入口。",
                        "The original QR login flow is not fully migrated yet. For now, the UI keeps both the button and the token field.",
                    ), self.theme()),
                    row![
                        button(text(self.t("扫码登录", "QR Login")).size(14))
                            .padding([10, 16])
                            .style(subtle_button_style)
                            .on_press_maybe((!self.wechat_login_running).then_some(Message::WechatQrLogin)),
                        text(self.t(
                            "如果你之前依赖扫码登录，这里会继续保留入口。",
                            "If you previously relied on QR login, the entry point stays here.",
                        ))
                        .size(13)
                        .color(self.secondary_text_color()),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .into(),
                    wechat_qr_panel,
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
                    helper_text(self.t(
                        "企业微信需要完整的回调配置，不只是开关。下面这些字段都会保存进共享配置。",
                        "WeCom needs a full callback configuration, not just the toggle. All fields below are saved into the shared config.",
                    ), self.theme()),
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
                    helper_text(self.t(
                        "钉钉至少需要 App Key、App Secret 和 Webhook。",
                        "DingTalk needs at least the App Key, App Secret, and Webhook.",
                    ), self.theme()),
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
                    helper_text(self.t(
                        "QQ 支持 go-cqhttp 反向 WebSocket 和 QQ Official Bot 两种模式。",
                        "QQ supports both go-cqhttp reverse WebSocket and QQ Official Bot mode.",
                    ), self.theme()),
                    field_row(
                        self.t("模式", "Mode"),
                        pick_list(QqMode::ALL, Some(qq_mode), Message::QqModeSelected)
                            .width(240)
                            .into(),
                    ),
                    if qq_mode == QqMode::WebSocket {
                        field_row(
                            self.t("WebSocket 地址", "WebSocket Host"),
                            plain_input("", &self.settings.qq_ws_host, TextField::QqWsHost),
                        )
                    } else {
                        field_row(
                            self.t("Bot AppID", "Bot AppID"),
                            plain_input("", &self.settings.qq_bot_appid, TextField::QqBotAppId),
                        )
                    },
                    if qq_mode == QqMode::WebSocket {
                        field_row(
                            self.t("WebSocket 端口", "WebSocket Port"),
                            number_input("6700", &self.settings.qq_ws_port, TextField::QqWsPort),
                        )
                    } else {
                        field_row(
                            self.t("Bot Token", "Bot Token"),
                            secure_input("", &self.settings.qq_bot_token, TextField::QqBotToken),
                        )
                    },
                ]
            ),
        ]
        .spacing(14)
        .into()
    }

    fn skills_page(&self) -> Element<'_, Message> {
        container(scrollable(container(self.skills_section()).width(Fill)).height(Fill))
            .width(Fill)
            .height(Fill)
            .padding([20, 24])
            .into()
    }

    fn skills_section(&self) -> Element<'_, Message> {
        column![
            section_title(self.t("Skill 设置", "Skill Settings"), self.theme()),
            provider_form(vec![
                helper_text(self.t(
                    "这里填写启用的 Skill 名称，多个用逗号分隔。示例: browser, shell, planner",
                    "Enter enabled skill names here, separated by commas. Example: browser, shell, planner",
                ), self.theme()),
                field_row(
                    self.t("启用列表", "Enabled Skills"),
                    plain_input("browser, shell", &self.settings.skills_enabled, TextField::SkillsEnabled),
                ),
                helper_text(self.t(
                    "支持导入 OpenClaw / Codex 兼容 Skill 目录或 ZIP，导入后会复制到项目根目录 skills/ 并自动加入启用列表。",
                    "Import an OpenClaw / Codex compatible Skill directory or ZIP. It will be copied into skills/ and enabled automatically.",
                ), self.theme()),
                channel_block(
                    self.t("导入 Skill", "Import Skill"),
                    vec![
                        field_row(
                            self.t("路径", "Path"),
                            plain_input(
                                "C:\\path\\to\\skill-or-skill.zip",
                                &self.skill_import_path,
                                TextField::SkillImportPath,
                            ),
                        ),
                        row![
                            button(text(self.t("导入", "Import")).size(14))
                                .padding([10, 16])
                                .style(accent_button_style)
                                .on_press(Message::ImportSkill),
                            helper_text(
                                self.t(
                                    "目录内需要包含 SKILL.md；ZIP 会自动解包并查找 SKILL.md。",
                                    "The directory must contain SKILL.md. ZIP files are unpacked and scanned for SKILL.md.",
                                ),
                                self.theme(),
                            ),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center)
                        .into(),
                    ],
                ),
            ]),
        ]
        .spacing(14)
        .into()
    }

    fn ai_groups_page(&self) -> Element<'_, Message> {
        container(container(self.ai_group_console()).width(Fill).height(Fill))
            .width(Fill)
            .height(Fill)
            .padding([20, 24])
            .into()
    }

    #[allow(unreachable_code)]
    fn ai_group_console(&self) -> Element<'_, Message> {
        let member_count = self.ai_group_member_count();
        let group_name = self.settings.ai_group_name.trim();

        if group_name.is_empty() {
            return container(
                column![
                    text(self.t("还没有群聊", "No group yet"))
                        .size(28)
                        .color(self.primary_text_color()),
                    text(self.t(
                        "AI 群组不会预置“龙虾群”。先创建一个自己的群聊，再添加成员和 AI。",
                        "EyeForge does not create a default group. Create your own group first, then add members and AI agents.",
                    ))
                    .size(15)
                    .color(self.secondary_text_color()),
                    field_row(
                        self.t("群聊名称", "Group Name"),
                        plain_input(
                            self.t("例如：项目协作群", "Example: Project Room"),
                            &self.settings.ai_group_name,
                            TextField::AiGroupName,
                        ),
                    ),
                    row![
                        button(text(self.t("创建群聊", "Create Group")).size(15))
                            .padding([12, 18])
                            .style(accent_button_style)
                            .on_press(Message::CreateAiGroup),
                        button(text(self.t("添加 AI 成员", "Add AI")).size(15))
                            .padding([12, 18])
                            .style(subtle_button_style)
                            .on_press(Message::AddGroupAi),
                    ]
                    .spacing(10),
                ]
                .spacing(18),
            )
            .padding(28)
            .style(panel_style)
            .width(Fill)
            .into();
        }

        let mut people_list = column![].spacing(10);
        for (name, role, status) in parse_member_lines(&self.settings.ai_group_people, "Member") {
            people_list = people_list.push(member_card(name, role, status, self.theme()));
        }
        if self.settings.ai_group_people.trim().is_empty() {
            people_list = people_list.push(empty_member_card(
                self.t("还没有成员", "No members"),
                self.t(
                    "点击添加成员后再改名保存",
                    "Click Add Member, then rename and save",
                ),
                self.theme(),
            ));
        }

        let agents = self.ai_group_agent_members();
        let mut ai_list = column![].spacing(10);
        for (name, role, status) in agents.iter().cloned() {
            ai_list = ai_list.push(member_card(name, role, status, self.theme()));
        }
        if agents.is_empty() {
            ai_list = ai_list.push(empty_member_card(
                self.t("还没有 AI", "No AI agents"),
                self.t(
                    "点击添加 AI 后配置连接地址",
                    "Click Add AI and configure its endpoint",
                ),
                self.theme(),
            ));
        }

        let chat_panel = container(
            column![
                row![
                    column![
                        text(group_name.to_string())
                            .size(28)
                            .color(self.primary_text_color()),
                        text(self.t(
                            "像工作群一样把任务交给不同成员协作",
                            "Coordinate work across members like a focused team chat",
                        ))
                        .size(15)
                        .color(self.secondary_text_color()),
                    ]
                    .spacing(6)
                    .width(Fill),
                    info_pill(self.t("成员", "Members"), member_count.to_string(), self.theme()),
                    info_pill(
                        self.t("状态", "Status"),
                        if self.settings.ai_groups_enabled {
                            self.t("已启用", "Enabled")
                        } else {
                            self.t("未启用", "Disabled")
                        },
                        self.theme(),
                    ),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                horizontal_rule(1),
                scrollable(
                    column![
                        ai_chat_message(
                            "System",
                            self.t("群组助手", "Group Assistant"),
                            self.t(
                                "群聊已创建。添加成员或 AI 后，可以在这里按角色分配任务。",
                                "The group is ready. Add members or AI agents, then route tasks by role here.",
                            ),
                            self.theme(),
                        ),
                        ai_chat_message(
                            group_name,
                            self.t("群聊", "Group"),
                            self.t(
                                "当前桌面端先保存群组结构和连接信息；实际调度会使用配置的 hapi 入口。",
                                "The desktop UI now saves the group structure and endpoints; dispatch uses the configured hapi endpoint.",
                            ),
                            self.theme(),
                        ),
                    ]
                    .spacing(16),
                )
                .height(Fill),
                container(
                    row![
                        text("+").size(24).color(self.accent_color()),
                        text(self.t(
                            "@成员 或输入任务目标，保存配置后即可协作",
                            "Mention a member or type a goal. Save settings before collaboration.",
                        ))
                        .size(15)
                        .color(self.secondary_text_color())
                        .width(Fill),
                        text(self.t("通过 hapi 调度", "hapi dispatch"))
                            .size(13)
                            .color(self.muted_text_color()),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .padding([14, 16])
                .style(feature_surface_style),
            ]
            .spacing(18)
            .height(Fill),
        )
        .padding(22)
        .style(panel_style)
        .height(Fill)
        .width(Length::FillPortion(2));

        let settings_panel = container(
            scrollable(
                column![
                    text(self.t("群聊设置", "Group Settings"))
                        .size(22)
                        .color(self.primary_text_color()),
                    provider_form(vec![
                        field_row(
                            self.t("群聊名称", "Group Name"),
                            plain_input(
                                self.t("项目协作群", "Project Room"),
                                &self.settings.ai_group_name,
                                TextField::AiGroupName,
                            ),
                        ),
                        checkbox(self.t("启用 AI 群组", "Enable AI Groups"), self.settings.ai_groups_enabled)
                            .on_toggle(|value| Message::BoolChanged(BoolField::AiGroupsEnabled, value))
                            .into(),
                    ]),
                    row![
                        button(text(self.t("添加成员", "Add Member")).size(14))
                            .padding([12, 14])
                            .style(subtle_button_style)
                            .on_press(Message::AddGroupMember),
                        button(text(self.t("添加 AI", "Add AI")).size(14))
                            .padding([12, 14])
                            .style(accent_button_style)
                            .on_press(Message::AddGroupAi),
                    ]
                    .spacing(10),
                    text(self.t("成员", "People")).size(16).color(self.primary_text_color()),
                    people_list,
                    text(self.t("AI", "AI")).size(16).color(self.primary_text_color()),
                    ai_list,
                    channel_block(
                        self.t("成员列表", "People List"),
                        vec![multiline_input(
                            "name | role | status",
                            &self.settings.ai_group_people,
                            TextField::AiGroupPeople,
                            120,
                        )],
                    ),
                    provider_form(vec![
                        field_row(
                            self.t("HAPI 入口", "HAPI Endpoint"),
                            plain_input(
                                "http://127.0.0.1:8766",
                                &self.settings.ai_group_hapi_endpoint,
                                TextField::AiGroupHapiEndpoint,
                            ),
                        ),
                        field_row(
                            self.t("调度策略", "Dispatch Strategy"),
                            plain_input(
                                "broadcast / primary / fallback",
                                &self.settings.ai_group_strategy,
                                TextField::AiGroupStrategy,
                            ),
                        ),
                    ]),
                    helper_text(
                        self.t(
                            "AI 成员格式：名称 | 角色 | hapi-endpoint。未填写的平台不会显示在列表里。",
                            "AI format: name | role | hapi-endpoint. Empty platforms are hidden.",
                        ),
                        self.theme(),
                    ),
                    channel_block(
                        "OpenClaw",
                        vec![multiline_input(
                            "claw-1 | planner | ws://127.0.0.1:3000",
                            &self.settings.ai_group_openclaw_members,
                            TextField::AiGroupOpenclawMembers,
                            120,
                        )],
                    ),
                    channel_block(
                        "Codex",
                        vec![multiline_input(
                            "Codex | implementer | http://127.0.0.1:9102",
                            &self.settings.ai_group_codex_members,
                            TextField::AiGroupCodexMembers,
                            120,
                        )],
                    ),
                    channel_block(
                        "Claude Code",
                        vec![multiline_input(
                            "claude-1 | reviewer | http://127.0.0.1:9103",
                            &self.settings.ai_group_claude_code_members,
                            TextField::AiGroupClaudeCodeMembers,
                            120,
                        )],
                    ),
                    channel_block(
                        "AstrBot / OpenCode",
                        vec![
                            multiline_input(
                                "astr-1 | reviewer | ws://127.0.0.1:6185",
                                &self.settings.ai_group_astrbot_members,
                                TextField::AiGroupAstrbotMembers,
                                100,
                            ),
                            multiline_input(
                                "opencode-1 | coder | http://127.0.0.1:9101",
                                &self.settings.ai_group_opencode_members,
                                TextField::AiGroupOpencodeMembers,
                                100,
                            ),
                        ],
                    ),
                ]
                .spacing(16),
            )
            .height(Fill),
        )
        .padding(22)
        .style(panel_style)
        .height(Fill)
        .width(Length::Fixed(390.0));

        return row![chat_panel, settings_panel]
            .spacing(16)
            .height(Fill)
            .into();

        let configured_summary = if member_count == 0 {
            self.t(
                "还没有配置 AI 群组成员。配置成员前，不会显示 OpenClaw、Codex 或其他代理。",
                "No AI group members are configured yet. OpenClaw, Codex, and other agents will not appear until configured.",
            )
            .to_string()
        } else {
            format!(
                "{}: {}",
                self.t("已配置成员", "Configured members"),
                member_count
            )
        };

        let chat_panel = panel(
            self.t("AI 群组", "AI Groups"),
            column![
                row![
                    info_pill(
                        self.t("状态", "Status"),
                        if self.settings.ai_groups_enabled {
                            self.t("已启用", "Enabled")
                        } else {
                            self.t("未启用", "Disabled")
                        },
                        self.theme(),
                    ),
                    info_pill(self.t("成员", "People"), member_count.to_string(), self.theme()),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text(configured_summary)
                    .size(15)
                    .color(self.secondary_text_color()),
                container(
                    column![
                        text(self.t("系统", "System"))
                            .size(15)
                            .color(self.accent_color()),
                        text(self.t(
                            "群组启用后，可以把任务交给已配置的成员进行协作；未配置的成员不会出现在这里。",
                            "After the group is enabled, tasks can be dispatched to configured members. Unconfigured members are hidden.",
                        ))
                        .size(15)
                        .color(self.primary_text_color()),
                    ]
                    .spacing(8),
                )
                .padding(16)
                .style(feature_surface_style),
                container(
                    row![
                        text("+").size(22).color(self.accent_color()),
                        text(self.t(
                            "@成员 或输入任务目标，保存配置后即可协作",
                            "Mention a member or type a goal. Save settings before collaboration.",
                        ))
                        .size(15)
                        .color(self.secondary_text_color()),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .padding([12, 14])
                .style(feature_surface_style),
            ]
            .spacing(14)
            .into(),
            self.theme(),
        );

        let settings_panel = panel(
            self.t("群聊设置", "Group Settings"),
            column![
                provider_form(vec![
                    checkbox(self.t("启用 AI 群组", "Enable AI Groups"), self.settings.ai_groups_enabled)
                        .on_toggle(|value| Message::BoolChanged(BoolField::AiGroupsEnabled, value))
                        .into(),
                    field_row(
                        self.t("HAPI 入口", "HAPI Endpoint"),
                        plain_input(
                            "http://127.0.0.1:8766",
                            &self.settings.ai_group_hapi_endpoint,
                            TextField::AiGroupHapiEndpoint,
                        ),
                    ),
                    field_row(
                        self.t("调度策略", "Dispatch Strategy"),
                        plain_input(
                            "broadcast / primary / fallback",
                            &self.settings.ai_group_strategy,
                            TextField::AiGroupStrategy,
                        ),
                    ),
                ]),
                text(self.t(
                    "每行一个成员，格式：名称 | 角色 | hapi-endpoint。未填写的平台不会显示在群组成员列表里。",
                    "One member per line: name | role | hapi-endpoint. Empty platforms are hidden from the group list.",
                ))
                .size(14)
                .color(self.secondary_text_color()),
                channel_block(
                    self.t("OpenClaw 成员", "OpenClaw Members"),
                    vec![multiline_input(
                        "claw-1 | planner | ws://127.0.0.1:3000",
                        &self.settings.ai_group_openclaw_members,
                        TextField::AiGroupOpenclawMembers,
                        120,
                    )],
                ),
                channel_block(
                    self.t("AstrBot 成员", "AstrBot Members"),
                    vec![multiline_input(
                        "astr-1 | reviewer | ws://127.0.0.1:6185",
                        &self.settings.ai_group_astrbot_members,
                        TextField::AiGroupAstrbotMembers,
                        120,
                    )],
                ),
                channel_block(
                    self.t("OpenCode 成员", "OpenCode Members"),
                    vec![multiline_input(
                        "opencode-1 | coder | http://127.0.0.1:9101",
                        &self.settings.ai_group_opencode_members,
                        TextField::AiGroupOpencodeMembers,
                        120,
                    )],
                ),
                channel_block(
                    self.t("Codex 成员", "Codex Members"),
                    vec![multiline_input(
                        "codex-1 | implementer | http://127.0.0.1:9102",
                        &self.settings.ai_group_codex_members,
                        TextField::AiGroupCodexMembers,
                        120,
                    )],
                ),
                channel_block(
                    self.t("Claude Code 成员", "Claude Code Members"),
                    vec![multiline_input(
                        "claude-1 | reviewer | http://127.0.0.1:9103",
                        &self.settings.ai_group_claude_code_members,
                        TextField::AiGroupClaudeCodeMembers,
                        120,
                    )],
                ),
            ]
            .spacing(14)
            .into(),
            self.theme(),
        );

        return scrollable(column![chat_panel, settings_panel].spacing(16))
            .height(Fill)
            .into();

        let group_name = self.t("龙虾群", "Dragon Group");

        let summary = row![
            column![
                text(group_name).size(28).color(self.primary_text_color()),
                text(self.t(
                    "协助负责人完成日常任务",
                    "Coordinate daily work across specialized agents",
                ))
                .size(15)
                .color(self.secondary_text_color()),
            ]
            .spacing(6)
            .width(Fill),
            info_pill(
                self.t("成员", "People"),
                format!("{}", member_count.max(5)),
                self.theme(),
            ),
            info_pill(
                self.t("状态", "Status"),
                if self.settings.ai_groups_enabled {
                    self.t("在线", "online")
                } else {
                    self.t("未启用", "disabled")
                },
                self.theme(),
            ),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let chat_feed = column![
            ai_chat_message(
                "Kimi",
                self.t("协调者", "Coordinator"),
                self.t(
                    "收到，资料归档、信息搜集、研究报告这几块我记下了。角色定位存好，随时待命。",
                    "I have the archive, collection, and research-report duties noted. Role memory is ready.",
                ),
                self.theme(),
            ),
            ai_chat_message(
                "Claw-Scripte",
                self.t("脚本专家", "Script Specialist"),
                self.t(
                    "欢迎来到 AI 群组。你可以先熟悉沟通规则，后续任务会按角色进入对应成员。",
                    "Welcome to the AI group. Tasks can be routed into the right member by role.",
                ),
                self.theme(),
            ),
            ai_chat_message(
                "Moonwalker1188",
                self.t("群主", "Owner"),
                self.t(
                    "这里专门协助日常任务。可以上传文件、丢一个目标、或者直接 @ 某个成员开始协作。",
                    "This room coordinates daily work. Drop a file, describe a goal, or mention a member to collaborate.",
                ),
                self.theme(),
            ),
            ai_chat_message(
                "Codex",
                self.t("代码执行", "Code Implementer"),
                self.t(
                    "我负责实现、调试和验证。需要改项目时可以直接分配给我。",
                    "I handle implementation, debugging, and verification when the project needs changes.",
                ),
                self.theme(),
            ),
        ]
        .spacing(16);

        let composer = container(
            row![
                text("+").size(24).color(self.accent_color()),
                text(self.t(
                    "@多个 Claw，马上开始协作",
                    "Mention multiple Claws to start collaboration",
                ))
                .size(15)
                .color(self.secondary_text_color())
                .width(Fill),
                text(self.t("通过 hapi 调度", "hapi dispatch"))
                    .size(13)
                    .color(self.muted_text_color()),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding([14, 16])
        .style(feature_surface_style);

        let chat_panel = container(
            column![
                summary,
                horizontal_rule(1),
                scrollable(chat_feed).height(Fill),
                composer
            ]
            .spacing(18)
            .height(Fill),
        )
        .padding(22)
        .style(panel_style)
        .height(Length::Shrink)
        .width(Fill);

        let people = column![
            member_card(
                "Moonwalker1455",
                self.t("群主", "Owner"),
                self.t("真人", "Member"),
                self.theme(),
            ),
            member_card(
                "Moonwalker1456",
                self.t("成员", "Member"),
                self.t("可聊天", "Available"),
                self.theme(),
            ),
            member_card(
                "Moonwalker1187",
                self.t("成员", "Member"),
                self.t("可聊天", "Available"),
                self.theme(),
            ),
        ]
        .spacing(10);

        let claws = column![
            member_card("Kimi", self.t("协调者", "Coordinator"), "47", self.theme()),
            member_card(
                "Claw-Scripte",
                self.t("脚本专家", "Script Specialist"),
                "47",
                self.theme(),
            ),
            member_card(
                "Claw-Archiv",
                self.t("资料归档", "Research Archivist"),
                "47",
                self.theme(),
            ),
            member_card(
                "Codex",
                self.t("代码执行", "Code Implementer"),
                "47",
                self.theme(),
            ),
        ]
        .spacing(10);

        let settings_panel = container(
            scrollable(
                column![
                    text(self.t("群聊设置", "Group Settings"))
                        .size(22)
                        .color(self.primary_text_color()),
                    text(self.t(
                        "AI 群组像工作群一样协调多个专长代理，把任务分发给规划、编码、审查和归档角色。",
                        "The AI group coordinates specialized agents like a work chat. Route tasks to planner, coder, reviewer, and archive roles without leaving EyeForge.",
                    ))
                    .size(14)
                    .color(self.secondary_text_color()),
                    row![
                        action_tile(self.t("邀请成员", "Invite Member"), "people", self.theme()),
                        action_tile(self.t("添加 Claw", "Add Claw"), "+", self.theme()),
                        action_tile(self.t("编辑群信息", "Edit Group"), "edit", self.theme()),
                    ]
                    .spacing(10),
                    text(self.t("成员", "People")).size(16).color(self.primary_text_color()),
                    people,
                    text(self.t("Claw", "Claw")).size(16).color(self.primary_text_color()),
                    claws,
                    provider_form(vec![
                        checkbox(self.t("启用 AI 群组", "Enable AI Groups"), self.settings.ai_groups_enabled)
                            .on_toggle(|value| Message::BoolChanged(BoolField::AiGroupsEnabled, value))
                            .into(),
                        field_row(
                            self.t("HAPI 入口", "HAPI Endpoint"),
                            plain_input(
                                "http://127.0.0.1:8766",
                                &self.settings.ai_group_hapi_endpoint,
                                TextField::AiGroupHapiEndpoint,
                            ),
                        ),
                        field_row(
                            self.t("调度策略", "Dispatch Strategy"),
                            plain_input(
                                "broadcast / primary / fallback",
                                &self.settings.ai_group_strategy,
                                TextField::AiGroupStrategy,
                            ),
                        ),
                    ]),
                    helper_text(
                        self.t(
                            "每行一个成员，格式：名称 | 角色 | hapi-endpoint\n示例：opencode-1 | coder | http://127.0.0.1:9101",
                            "One member per line, format: name | role | hapi-endpoint\nExample: opencode-1 | coder | http://127.0.0.1:9101",
                        ),
                        self.theme(),
                    ),
                    channel_block(
                        "OpenClaw",
                        vec![field_row(
                            self.t("成员列表", "Members"),
                            multiline_input(
                                "claw-1 | planner | ws://127.0.0.1:3000",
                                &self.settings.ai_group_openclaw_members,
                                TextField::AiGroupOpenclawMembers,
                                120,
                            ),
                        )],
                    ),
                    channel_block(
                        "AstrBot",
                        vec![field_row(
                            self.t("成员列表", "Members"),
                            multiline_input(
                                "astr-1 | reviewer | ws://127.0.0.1:6185",
                                &self.settings.ai_group_astrbot_members,
                                TextField::AiGroupAstrbotMembers,
                                120,
                            ),
                        )],
                    ),
                    channel_block(
                        "OpenCode",
                        vec![field_row(
                            self.t("成员列表", "Members"),
                            multiline_input(
                                "opencode-1 | coder | http://127.0.0.1:9101",
                                &self.settings.ai_group_opencode_members,
                                TextField::AiGroupOpencodeMembers,
                                120,
                            ),
                        )],
                    ),
                    channel_block(
                        "Codex",
                        vec![field_row(
                            self.t("成员列表", "Members"),
                            multiline_input(
                                "codex-1 | implementer | http://127.0.0.1:9102",
                                &self.settings.ai_group_codex_members,
                                TextField::AiGroupCodexMembers,
                                120,
                            ),
                        )],
                    ),
                    channel_block(
                        "Claude Code",
                        vec![field_row(
                            self.t("成员列表", "Members"),
                            multiline_input(
                                "claude-1 | reviewer | http://127.0.0.1:9103",
                                &self.settings.ai_group_claude_code_members,
                                TextField::AiGroupClaudeCodeMembers,
                                120,
                            ),
                        )],
                    ),
                ]
                .spacing(16),
            )
            .height(Fill),
        )
        .padding(22)
        .style(panel_style)
        .height(Length::Shrink)
        .width(Fill);

        scrollable(column![chat_panel, settings_panel].spacing(16))
            .height(Fill)
            .into()
    }

    #[allow(dead_code)]
    fn ai_groups_section(&self) -> Element<'_, Message> {
        column![
            section_title(self.t("AI 群组", "AI Groups"), self.theme()),
            provider_form(vec![
                checkbox(self.t("启用 AI 群组", "Enable AI Groups"), self.settings.ai_groups_enabled)
                    .on_toggle(|value| Message::BoolChanged(BoolField::AiGroupsEnabled, value))
                    .into(),
                helper_text(self.t(
                    "AI 群组通过 hapi 连接 OpenClaw、AstrBot、OpenCode、Codex 和 Claude Code 成员。",
                    "AI Groups connect OpenClaw, AstrBot, OpenCode, Codex, and Claude Code members through hapi.",
                ), self.theme()),
                field_row(
                    self.t("HAPI 入口", "HAPI Endpoint"),
                    plain_input("http://127.0.0.1:8766", &self.settings.ai_group_hapi_endpoint, TextField::AiGroupHapiEndpoint),
                ),
                field_row(
                    self.t("调度策略", "Dispatch Strategy"),
                    plain_input("broadcast / primary / fallback", &self.settings.ai_group_strategy, TextField::AiGroupStrategy),
                ),
                helper_text(self.t(
                    "每行一个成员，格式：名称 | 角色 | hapi-endpoint\n例如：opencode-1 | coder | http://127.0.0.1:9101",
                    "One member per line, format: name | role | hapi-endpoint\nExample: opencode-1 | coder | http://127.0.0.1:9101",
                ), self.theme()),
            ]),
            channel_block(
                self.t("OpenClaw", "OpenClaw"),
                vec![
                    field_row(
                        self.t("成员列表", "Members"),
                        multiline_input(
                            "claw-1 | planner | ws://127.0.0.1:3000",
                            &self.settings.ai_group_openclaw_members,
                            TextField::AiGroupOpenclawMembers,
                            140,
                        ),
                    ),
                ],
            ),
            channel_block(
                self.t("AstrBot", "AstrBot"),
                vec![
                    field_row(
                        self.t("成员列表", "Members"),
                        multiline_input(
                            "astr-1 | reviewer | ws://127.0.0.1:6185",
                            &self.settings.ai_group_astrbot_members,
                            TextField::AiGroupAstrbotMembers,
                            140,
                        ),
                    ),
                ],
            ),
            channel_block(
                self.t("OpenCode", "OpenCode"),
                vec![
                    field_row(
                        self.t("成员列表", "Members"),
                        multiline_input(
                            "opencode-1 | coder | http://127.0.0.1:9101",
                            &self.settings.ai_group_opencode_members,
                            TextField::AiGroupOpencodeMembers,
                            140,
                        ),
                    ),
                ],
            ),
            channel_block(
                self.t("Codex", "Codex"),
                vec![
                    field_row(
                        self.t("成员列表", "Members"),
                        multiline_input(
                            "codex-1 | implementer | http://127.0.0.1:9102",
                            &self.settings.ai_group_codex_members,
                            TextField::AiGroupCodexMembers,
                            140,
                        ),
                    ),
                ],
            ),
            channel_block(
                self.t("Claude Code", "Claude Code"),
                vec![
                    field_row(
                        self.t("成员列表", "Members"),
                        multiline_input(
                            "claude-1 | reviewer | http://127.0.0.1:9103",
                            &self.settings.ai_group_claude_code_members,
                            TextField::AiGroupClaudeCodeMembers,
                            140,
                        ),
                    ),
                ],
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

    fn ai_group_member_count(&self) -> usize {
        [
            self.settings.ai_group_people.as_str(),
            self.settings.ai_group_openclaw_members.as_str(),
            self.settings.ai_group_astrbot_members.as_str(),
            self.settings.ai_group_opencode_members.as_str(),
            self.settings.ai_group_codex_members.as_str(),
            self.settings.ai_group_claude_code_members.as_str(),
        ]
        .iter()
        .flat_map(|members| members.lines())
        .filter(|line| !line.trim().is_empty())
        .count()
    }

    fn ai_group_agent_members(&self) -> Vec<(String, String, String)> {
        [
            self.settings.ai_group_openclaw_members.as_str(),
            self.settings.ai_group_astrbot_members.as_str(),
            self.settings.ai_group_opencode_members.as_str(),
            self.settings.ai_group_codex_members.as_str(),
            self.settings.ai_group_claude_code_members.as_str(),
        ]
        .iter()
        .flat_map(|members| parse_member_lines(members, "AI"))
        .collect()
    }

    fn t<'a>(&self, zh: &'a str, en: &'a str) -> &'a str {
        if self.language().is_zh() {
            if looks_mojibake(zh) {
                zh_text(en)
            } else {
                zh
            }
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
        Color { a: 0.98, ..base }
    }

    fn muted_text_color(&self) -> Color {
        let c = self.theme.extended_palette().background.base.text;
        Color { a: 0.92, ..c }
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
            TextField::SkillsEnabled => self.settings.skills_enabled = value,
            TextField::AiGroupName => self.settings.ai_group_name = value,
            TextField::AiGroupPeople => self.settings.ai_group_people = value,
            TextField::AiGroupStrategy => self.settings.ai_group_strategy = value,
            TextField::AiGroupOpenclawMembers => self.settings.ai_group_openclaw_members = value,
            TextField::AiGroupAstrbotMembers => self.settings.ai_group_astrbot_members = value,
            TextField::AiGroupHapiEndpoint => self.settings.ai_group_hapi_endpoint = value,
            TextField::AiGroupOpencodeMembers => self.settings.ai_group_opencode_members = value,
            TextField::AiGroupCodexMembers => self.settings.ai_group_codex_members = value,
            TextField::AiGroupClaudeCodeMembers => {
                self.settings.ai_group_claude_code_members = value
            }
            TextField::SkillImportPath => self.skill_import_path = value,
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
            BoolField::AiGroupsEnabled => self.settings.ai_groups_enabled = value,
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
        .width(Fill)
        .into()
}

fn ai_chat_message<'a>(
    speaker: impl Into<String>,
    role: impl Into<String>,
    body: impl Into<String>,
    theme: Theme,
) -> Element<'a, Message> {
    let speaker = speaker.into();
    let role = role.into();
    let body = body.into();
    let text_color = theme.extended_palette().background.base.text;
    let role_color = theme.extended_palette().primary.strong.color;

    container(
        row![
            container(text(speaker.chars().next().unwrap_or('A').to_string()).size(18))
                .width(42)
                .height(42)
                .center_x(Fill)
                .center_y(Fill)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    container::Style {
                        background: Some(Background::Color(palette.primary.strong.color)),
                        border: Border {
                            radius: 21.0.into(),
                            ..Border::default()
                        },
                        text_color: Some(palette.background.base.color),
                        ..container::Style::default()
                    }
                }),
            column![
                row![
                    text(speaker).size(15).color(text_color),
                    text(role).size(13).color(role_color),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text(body)
                    .size(16)
                    .line_height(iced::widget::text::LineHeight::Relative(1.45))
                    .color(text_color),
            ]
            .spacing(6)
            .width(Fill),
        ]
        .spacing(14),
    )
    .padding([10, 6])
    .width(Fill)
    .into()
}

fn action_tile<'a>(label: &'a str, mark: &'a str, theme: Theme) -> Element<'a, Message> {
    container(
        column![
            text(mark)
                .size(22)
                .align_x(Horizontal::Center)
                .color(theme.extended_palette().primary.strong.color),
            text(label)
                .size(13)
                .align_x(Horizontal::Center)
                .color(theme.extended_palette().background.base.text),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .padding([14, 10])
    .width(Length::FillPortion(1))
    .style(feature_surface_style)
    .into()
}

fn member_card<'a>(
    name: impl Into<String>,
    role: impl Into<String>,
    status: impl Into<String>,
    theme: Theme,
) -> Element<'a, Message> {
    let name = name.into();
    let role = role.into();
    let status = status.into();

    container(
        row![
            container(text(name.chars().next().unwrap_or('A').to_string()).size(15))
                .width(36)
                .height(36)
                .center_x(Fill)
                .center_y(Fill)
                .style(move |theme: &Theme| {
                    let palette = theme.extended_palette();
                    container::Style {
                        background: Some(Background::Color(palette.primary.base.color)),
                        border: Border {
                            radius: 18.0.into(),
                            ..Border::default()
                        },
                        text_color: Some(palette.background.base.color),
                        ..container::Style::default()
                    }
                }),
            column![
                text(name)
                    .size(14)
                    .color(theme.extended_palette().background.base.text),
                text(role)
                    .size(12)
                    .color(theme.extended_palette().background.base.text),
            ]
            .spacing(4)
            .width(Fill),
            text(status)
                .size(12)
                .color(theme.extended_palette().primary.strong.color),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding(12)
    .style(feature_surface_style)
    .into()
}

fn empty_member_card<'a>(title: &'a str, detail: &'a str, theme: Theme) -> Element<'a, Message> {
    container(
        column![
            text(title)
                .size(14)
                .color(theme.extended_palette().background.base.text),
            text(detail)
                .size(12)
                .color(theme.extended_palette().background.base.text),
        ]
        .spacing(6),
    )
    .padding(12)
    .style(feature_surface_style)
    .into()
}

fn parse_member_lines(source: &str, fallback_role: &str) -> Vec<(String, String, String)> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut parts = line.split('|').map(str::trim);
            let name = parts.next().unwrap_or("Member").to_string();
            let role = parts
                .next()
                .filter(|value| !value.is_empty())
                .unwrap_or(fallback_role)
                .to_string();
            let status = parts
                .next()
                .filter(|value| !value.is_empty())
                .unwrap_or("configured")
                .to_string();
            (name, role, status)
        })
        .collect()
}

fn append_unique_line(target: &mut String, line: &str) {
    if target.lines().any(|current| current.trim() == line.trim()) {
        return;
    }

    if !target.trim().is_empty() {
        target.push('\n');
    }
    target.push_str(line);
}

fn helper_text<'a>(value: &'a str, theme: Theme) -> Element<'a, Message> {
    let base = theme.extended_palette().background.base.text;
    text(value).size(13).color(Color { a: 0.94, ..base }).into()
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

fn info_pill<'a>(
    label: impl Into<String>,
    value: impl Into<String>,
    theme: Theme,
) -> Element<'a, Message> {
    let label = label.into();
    let value = value.into();
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

fn multiline_input<'a>(
    placeholder: &'a str,
    value: &'a str,
    field: TextField,
    _height: u16,
) -> Element<'a, Message> {
    text_input(placeholder, value)
        .on_input(move |next| Message::TextChanged(field, next))
        .padding(10)
        .width(Fill)
        .size(14)
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

fn top_nav_button<'a>(label: &'a str, selected: bool, page: Page) -> Element<'a, Message> {
    let btn = text(label)
        .size(13)
        .color(if selected { Color::WHITE } else { Color { a: 0.75, ..Color::WHITE } });
    let content = if selected {
        column![btn, horizontal_rule(2).style(|_: &Theme| iced::widget::rule::Style { color: Color::WHITE, width: 2, radius: 0.0.into(), fill_mode: iced::widget::rule::FillMode::Full })].spacing(2).align_x(Alignment::Center)
    } else {
        column![btn, horizontal_rule(2).style(|_: &Theme| iced::widget::rule::Style { color: Color::TRANSPARENT, width: 2, radius: 0.0.into(), fill_mode: iced::widget::rule::FillMode::Full })].spacing(2).align_x(Alignment::Center)
    };
    button(content)
        .padding([8, 14])
        .style(|_: &Theme, _: button::Status| button::Style {
            background: None,
            text_color: Color::WHITE,
            ..button::Style::default()
        })
        .on_press(Message::SidebarClick(page))
        .into()
}

fn window_ctrl_button<'a>(
    label: &'a str,
    msg: Message,
) -> iced::widget::Button<'a, Message> {
    button(text(label).size(14).color(Color::WHITE))
        .padding([0, 0])
        .style(window_ctrl_button_style)
        .on_press(msg)
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
    let dark = palette.background.base.color.r
        + palette.background.base.color.g
        + palette.background.base.color.b
        < 1.35;
    let bg = if dark {
        Color::from_rgb8(20, 28, 48)
    } else {
        Color::from_rgb8(255, 255, 255)
    };
    let border = if dark {
        Color::from_rgb8(72, 92, 142)
    } else {
        Color::from_rgb8(199, 211, 234)
    };
    container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            width: 1.0,
            radius: 16.0.into(),
            color: border,
        },
        text_color: Some(palette.background.base.text),
        ..container::Style::default()
    }
}

fn feature_surface_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let dark = palette.background.base.color.r
        + palette.background.base.color.g
        + palette.background.base.color.b
        < 1.35;
    let bg = if dark {
        Color::from_rgb8(28, 38, 62)
    } else {
        Color::from_rgb8(246, 249, 255)
    };
    let border = if dark {
        Color::from_rgb8(83, 104, 156)
    } else {
        Color::from_rgb8(209, 219, 238)
    };
    container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            width: 1.0,
            radius: 14.0.into(),
            color: border,
        },
        text_color: Some(palette.background.base.text),
        ..container::Style::default()
    }
}

fn hero_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let dark = palette.background.base.color.r
        + palette.background.base.color.g
        + palette.background.base.color.b
        < 1.35;
    let bg = if dark {
        Color::from_rgb8(18, 36, 66)
    } else {
        Color::from_rgb8(238, 246, 255)
    };
    let border = if dark {
        Color::from_rgb8(72, 133, 202)
    } else {
        Color::from_rgb8(160, 190, 230)
    };

    container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            width: 1.0,
            radius: 22.0.into(),
            color: border,
        },
        text_color: Some(palette.background.base.text),
        ..container::Style::default()
    }
}

fn aside_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let dark = palette.background.base.color.r
        + palette.background.base.color.g
        + palette.background.base.color.b
        < 1.35;
    let bg = if dark {
        Color::from_rgb8(10, 15, 29)
    } else {
        Color::from_rgb8(242, 247, 255)
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

fn top_bar_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let dark = palette.background.base.color.r
        + palette.background.base.color.g
        + palette.background.base.color.b
        < 1.35;
    let bg = if dark {
        Color::from_rgb8(20, 50, 110)
    } else {
        Color::from_rgb8(40, 95, 185)
    };
    container::Style {
        background: Some(Background::Color(bg)),
        border: Border { width: 0.0, radius: 0.0.into(), color: Color::TRANSPARENT },
        text_color: Some(Color::WHITE),
        ..container::Style::default()
    }
}

fn window_ctrl_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let dark = palette.background.base.color.r
        + palette.background.base.color.g
        + palette.background.base.color.b
        < 1.35;
    let bg = match status {
        button::Status::Hovered => {
            if dark { Color::from_rgb8(50, 90, 150) } else { Color::from_rgb8(60, 120, 210) }
        }
        button::Status::Pressed => {
            if dark { Color::from_rgb8(30, 60, 120) } else { Color::from_rgb8(30, 80, 170) }
        }
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border { width: 0.0, radius: 0.0.into(), color: Color::TRANSPARENT },
        ..button::Style::default()
    }
}

fn close_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb8(196, 43, 28),
        button::Status::Pressed => Color::from_rgb8(160, 30, 20),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border { width: 0.0, radius: 0.0.into(), color: Color::TRANSPARENT },
        ..button::Style::default()
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

fn looks_mojibake(value: &str) -> bool {
    value.chars().any(|ch| {
        matches!(
            ch,
            '鍚' | '璇'
                | '缇'
                | '灞'
                | '鎵'
                | '閫'
                | '妗'
                | '绛'
                | '鐩'
                | '灏'
                | '寰'
                | ''
                | '€'
                | '�'
        )
    })
}

fn zh_text(english: &str) -> &str {
    match english {
        "(empty)" => "（空）",
        "AES Key" => "AES Key",
        "AI Model" => "AI 模型",
        "API Key" => "API Key",
        "Action Delay (sec)" => "动作延迟（秒）",
        "Agent ID" => "Agent ID",
        "App Key" => "App Key",
        "App Secret" => "App Secret",
        "Ask AI to complete a desktop task..." => "让 AI 完成一个桌面任务...",
        "AstrBot" => "AstrBot",
        "Base URL" => "Base URL",
        "Bot AppID" => "Bot AppID",
        "Bot Token" => "Bot Token",
        "Capture" => "截图",
        "Channel Bridge Settings" => "通道桥接设置",
        "Channels" => "通道",
        "Claude Code" => "Claude Code",
        "Codex" => "Codex",
        "Coordinate daily work across specialized agents" => "协助多个专长代理完成日常任务",
        "Corp ID" => "Corp ID",
        "Current Form" => "当前表单",
        "DingTalk needs at least the App Key, App Secret, and Webhook." => "钉钉至少需要 App Key、App Secret 和 Webhook。",
        "Disabled" => "已禁用",
        "Dragon Group" => "龙虾群",
        "Enable Wake Word" => "启用唤醒词",
        "Enable vision mode" => "启用视觉模式",
        "Enabled" => "已启用",
        "Enabled Skills" => "启用技能",
        "Enter enabled skill names here, separated by commas. Example: browser, shell, planner" => {
            "在这里输入启用的技能名，用英文逗号分隔。例如：browser, shell, planner"
        }
        "Execution Log" => "执行日志",
        "Font Size" => "字体大小",
        "Gateway disabled" => "网关已禁用",
        "General" => "常规",
        "Host" => "主机",
        "I handle implementation, debugging, and verification when the project needs changes." => {
            "我负责实现、调试和验证。需要改项目时可以直接分配给我。"
        }
        "I have the archive, collection, and research-report duties noted. Role memory is ready." => {
            "收到，资料归档、信息搜集、研究报告这几块我记下了。角色定位存好，随时待命。"
        }
        "If you previously relied on QR login, the entry point stays here." => "如果之前依赖扫码登录，入口仍然保留在这里。",
        "Language" => "语言",
        "Launch" => "启动",
        "Live preview, save to persist" => "实时预览，保存后持久化",
        "Login succeeded and the token was filled in automatically." => "登录成功，Token 已自动填入。",
        "Login succeeded." => "登录成功。",
        "Mention multiple Claws to start collaboration" => "@多个 Claw，马上开始协作",
        "Model" => "模型",
        "No result yet" => "还没有结果",
        "OpenClaw" => "OpenClaw",
        "OpenCode" => "OpenCode",
        "Please scan the QR code" => "请扫描二维码",
        "Port" => "端口",
        "Provider" => "提供商",
        "QQ supports both go-cqhttp reverse WebSocket and QQ Official Bot mode." => {
            "QQ 同时支持 go-cqhttp 反向 WebSocket 和 QQ 官方机器人模式。"
        }
        "QR Login" => "扫码登录",
        "Quick Input Hotkey" => "快捷输入热键",
        "Requesting the QR code..." => "正在获取二维码...",
        "Runtime" => "运行时",
        "Rustpotter Model Files" => "Rustpotter 模型文件",
        "Saving writes back into the shared Rust config shape and preserves unknown fields." => {
            "保存会写回共享 Rust 配置结构，并保留未知字段。"
        }
        "Scan the QR code with WeChat and confirm the login on your phone." => "请用微信扫码，并在手机上确认登录。",
        "Screenshot Quality" => "截图质量",
        "Secret" => "Secret",
        "Settings Surface" => "设置面板",
        "Skill Settings" => "技能设置",
        "Status" => "状态",
        "The QR code will appear here after you click QR Login." => "点击扫码登录后，二维码会显示在这里。",
        "The QR code will be shown in the settings page. Scan it with WeChat and confirm on your phone." => {
            "二维码会显示在设置页中，请用微信扫码并在手机上确认。"
        }
        "The desktop task button now calls the Rust-native backend directly. The Web UI can also talk to Rust's own WebSocket gateway, with no Python dependency." => {
            "桌面任务按钮现在直接调用 Rust 原生后端；Web UI 也会连接 Rust 自己的 WebSocket 网关，不再依赖 Python。"
        }
        "The original QR login flow is not fully migrated yet. For now, the UI keeps both the button and the token field." => {
            "原扫码登录流程还没有完全迁移完成，目前界面同时保留按钮和 Token 输入框。"
        }
        "Theme" => "主题",
        "This keeps the original Skill feature entry point. Directory scanning, import, and per-skill toggles can be connected next." => {
            "这里保留原有 Skill 功能入口，后续可以继续接入目录扫描、导入和单技能开关。"
        }
        "This now runs through the Rust-native execution chain. `shell:` executes a local command, `wait:` delays, and other tasks enter the native backend placeholder flow." => {
            "现在会走 Rust 原生执行链。`shell:` 执行本地命令，`wait:` 延迟，其他任务进入原生后端流程。"
        }
        "This room coordinates daily work. Drop a file, describe a goal, or mention a member to collaborate." => {
            "这里专门协助日常任务。可以上传文件、丢一个目标，或者直接 @ 某个成员开始协作。"
        }
        "Token" => "Token",
        "Use comma-separated Rustpotter .rpw model/reference file paths. Picovoice AccessKey and Porcupine libraries are no longer required." => {
            "用英文逗号分隔 Rustpotter .rpw 模型/参考文件路径。现在不再需要 Picovoice AccessKey 和 Porcupine 库。"
        }
        "Voice Input Hotkey" => "语音输入热键",
        "Wake Word" => "唤醒词",
        "WeChat Login QR Code" => "微信登录二维码",
        "WeCom needs a full callback configuration, not just the toggle. All fields below are saved into the shared config." => {
            "企业微信需要完整回调配置，不只是开关；下面字段都会保存到共享配置。"
        }
        "WebSocket Host" => "WebSocket 主机",
        "WebSocket Port" => "WebSocket 端口",
        "Webhook" => "Webhook",
        "Welcome to the AI group. Tasks can be routed into the right member by role." => {
            "欢迎来到 AI 群组。后续任务可以按角色路由给对应成员。"
        }
        "disabled" => "未启用",
        "hapi dispatch" => "hapi 调度",
        "Home" => "首页",
        "Settings" => "设置",
        "Skills" => "技能",
        "AI Groups" => "AI 群组",
        "Desktop Console" => "桌面控制台",
        "Rust Native" => "Rust 原生",
        "AI Screen Control Assistant" => "AI 屏幕操控助手",
        "Desktop Command Center" => "桌面指挥中心",
        "Run native actions, inspect live feedback, and keep the browser gateway ready on port 9178." => {
            "执行原生动作、查看实时反馈，并保持 9178 浏览器网关在线。"
        }
        "Execution State" => "执行状态",
        "Running" => "运行中",
        "Idle" => "空闲",
        "Gateway" => "网关",
        "Always on" => "常驻",
        "Current Model" => "当前模型",
        "Vision Preview" => "视觉预览",
        "Mode" => "模式",
        "Vision enabled" => "视觉已开启",
        "Command only" => "仅命令",
        "Entry" => "入口",
        "Desktop + Web" => "桌面 + 网页",
        "Task Input" => "任务输入",
        "Start" => "开始",
        "Save Settings" => "保存设置",
        "Activity Log" => "活动日志",
        "No logs yet" => "暂无日志",
        "Waiting for screenshot..." => "等待截图...",
        "Screen recognition disabled" => "屏幕识别已关闭",
        "AI Model Settings" => "AI 模型设置",
        "Capture and Execution" => "截图与执行",
        "General Settings" => "常规设置",
        "Channel Bridge" => "通道桥接",
        "WebSocket Gateway" => "WebSocket 网关",
        "WeChat iLink" => "微信 iLink",
        "WeCom" => "企业微信",
        "DingTalk" => "钉钉",
        "QQ" => "QQ",
        "Enable AI Groups" => "启用 AI 群组",
        "HAPI Endpoint" => "HAPI 入口",
        "Dispatch Strategy" => "调度策略",
        "Members" => "成员列表",
        "Group Settings" => "群聊设置",
        "Invite Member" => "邀请成员",
        "Add Claw" => "添加 Claw",
        "Edit Group" => "编辑群信息",
        "People" => "成员",
        "Claw" => "Claw",
        "Owner" => "群主",
        "Member" => "真人",
        "Available" => "可聊天",
        "online" => "在线",
        "messages" => "条消息",
        "Coordinator" => "协调者",
        "Script Specialist" => "脚本专家",
        "Research Archivist" => "资料归档",
        "Code Implementer" => "代码执行",
        "Review Specialist" => "审查专家",
        "The AI group coordinates specialized agents like a work chat. Route tasks to planner, coder, reviewer, and archive roles without leaving EyeForge." => {
            "AI 群组像工作群一样协调多个专长代理，把任务分发给规划、编码、审查和归档角色。"
        }
        "Mention a member or broadcast to the group. The runtime will dispatch through the configured hapi endpoint." => {
            "可以 @ 某个成员，也可以广播给全群；运行时会通过配置的 hapi 入口调度。"
        }
        "One member per line, format: name | role | hapi-endpoint\nExample: opencode-1 | coder | http://127.0.0.1:9101" => {
            "每行一个成员，格式：名称 | 角色 | hapi-endpoint\n示例：opencode-1 | coder | http://127.0.0.1:9101"
        }
        "AI Groups connect OpenClaw, AstrBot, OpenCode, Codex, and Claude Code members through hapi." => {
            "AI 群组通过 hapi 连接 OpenClaw、AstrBot、OpenCode、Codex 和 Claude Code 成员。"
        }
        _ => english,
    }
}
