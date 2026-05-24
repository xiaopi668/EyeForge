#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod ai_groups;
mod app;
mod channels;
mod config;
mod crypto;
mod llm;
mod runtime;
mod server;
mod tray;
mod voice;
mod wakeword;
mod wechat;

use iced::{Element, Font, Task, Theme};

fn main() -> iced::Result {
    iced::application(title, update, view)
        .default_font(Font::with_name("Microsoft YaHei UI"))
        .theme(theme)
        .run_with(app::EyeForge::new)
}

fn title(state: &app::EyeForge) -> String {
    state.title()
}

fn update(state: &mut app::EyeForge, message: app::Message) -> Task<app::Message> {
    state.update(message)
}

fn view(state: &app::EyeForge) -> Element<'_, app::Message> {
    state.view()
}

fn theme(state: &app::EyeForge) -> Theme {
    state.theme()
}
