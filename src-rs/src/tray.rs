#[cfg(target_os = "windows")]
use std::{cell::RefCell, path::PathBuf};

#[cfg(target_os = "windows")]
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

#[cfg(target_os = "windows")]
thread_local! {
    static TRAY_ICON: RefCell<Option<TrayIcon>> = const { RefCell::new(None) };
}

#[cfg(target_os = "windows")]
pub fn ensure_tray() {
    TRAY_ICON.with(|slot| {
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

        let tray = TrayIconBuilder::new()
            .with_tooltip("EyeForge")
            .with_icon(icon)
            .with_menu_on_left_click(false)
            .with_menu_on_right_click(false)
            .build()
            .ok();

        *slot.borrow_mut() = tray;
    });
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
