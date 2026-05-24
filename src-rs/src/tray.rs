#[cfg(target_os = "windows")]
use std::{cell::RefCell, path::PathBuf};

#[cfg(target_os = "windows")]
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    Show,
    Exit,
}

#[cfg(target_os = "windows")]
struct TrayState {
    _icon: TrayIcon,
    _menu: Menu,
    _show_item: MenuItem,
    _exit_item: MenuItem,
    show_id: MenuId,
    exit_id: MenuId,
}

#[cfg(target_os = "windows")]
thread_local! {
    static TRAY_STATE: RefCell<Option<TrayState>> = const { RefCell::new(None) };
}

#[cfg(target_os = "windows")]
pub fn ensure_tray() {
    TRAY_STATE.with(|slot| {
        if slot.borrow().is_some() {
            return;
        }

        let icon_path = match locate_icon() {
            Some(path) => path,
            None => return,
        };

        let icon = match Icon::from_path(icon_path, None) {
            Ok(icon) => icon,
            Err(_) => return,
        };

        let menu = Menu::new();
        let show_item = MenuItem::new("显示主界面", true, None);
        let exit_item = MenuItem::new("退出", true, None);

        if menu.append(&show_item).is_err() || menu.append(&exit_item).is_err() {
            return;
        }

        let show_id = show_item.id().clone();
        let exit_id = exit_item.id().clone();

        let tray = TrayIconBuilder::new()
            .with_tooltip("EyeForge")
            .with_icon(icon)
            .with_menu_on_left_click(false)
            .with_menu_on_right_click(true)
            .with_menu(Box::new(menu.clone()))
            .build()
            .ok();

        if let Some(icon) = tray {
            *slot.borrow_mut() = Some(TrayState {
                _icon: icon,
                _menu: menu,
                _show_item: show_item,
                _exit_item: exit_item,
                show_id,
                exit_id,
            });
        }
    });
}

#[cfg(target_os = "windows")]
pub fn next_command() -> Option<TrayCommand> {
    for event in MenuEvent::receiver().try_iter() {
        let command = TRAY_STATE.with(|slot| {
            let state = slot.borrow();
            let state = state.as_ref()?;

            if event.id == state.show_id {
                Some(TrayCommand::Show)
            } else if event.id == state.exit_id {
                Some(TrayCommand::Exit)
            } else {
                None
            }
        });

        if command.is_some() {
            return command;
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn locate_icon() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;

    for dir in exe.ancestors() {
        let candidate = dir.join("logo.ico");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    let cwd_candidate = std::env::current_dir().ok()?.join("logo.ico");
    if cwd_candidate.exists() {
        return Some(cwd_candidate);
    }

    None
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_tray() {}

#[cfg(not(target_os = "windows"))]
pub fn next_command() -> Option<TrayCommand> {
    None
}
