#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod ai_groups;
mod app;
mod channels;
mod config;
mod crypto;
mod llm;
mod runtime;
mod server;
mod skills;
mod tray;
mod voice;
mod wakeword;
mod wechat;

use iced::{Element, Font, Subscription, Task, Theme};

fn main() -> iced::Result {
    iced::application(title, update, view)
        .default_font(Font::with_name("Microsoft YaHei UI"))
        .theme(theme)
        .subscription(subscription)
        .exit_on_close_request(false)
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

fn subscription(state: &app::EyeForge) -> Subscription<app::Message> {
    state.subscription()
}
