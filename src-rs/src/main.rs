mod ai_groups;
mod app;
mod channels;
mod commands;
mod config;
mod crypto;
mod hapi;
mod llm;
mod model_manager;
mod runtime;
mod server;
mod skills;
mod theme;
mod tray;
mod voice;
mod wakeword;
mod wechat;

use std::env;

use iced::{
    font::Font,
    window,
    Theme,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn title(_app: &app::EyeForge) -> String {
    format!("EyeForge v{}", VERSION)
}

fn update(app: &mut app::EyeForge, message: app::Message) -> iced::Task<app::Message> {
    app.update(message)
}

fn view(app: &app::EyeForge) -> iced::Element<'_, app::Message> {
    app.view()
}

fn theme(_app: &app::EyeForge) -> Theme {
    Theme::Dark
}

fn subscription(app: &app::EyeForge) -> iced::Subscription<app::Message> {
    app.subscription()
}

fn main() -> iced::Result {
    iced::application(title, update, view)
        .window(window::Settings {
            decorations: false,
            ..Default::default()
        })
        .default_font(Font::with_name("Microsoft YaHei UI"))
        .theme(theme)
        .subscription(subscription)
        .exit_on_close_request(false)
        .run_with(app::EyeForge::new)
}
