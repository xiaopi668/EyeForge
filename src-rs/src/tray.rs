#[cfg(target_os = "windows")]
use std::cell::RefCell;

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

        let icon = match make_icon() {
            Some(icon) => icon,
            None => return,
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
fn make_icon() -> Option<Icon> {
    let svg_data = include_bytes!("../../logo.svg");
    let tree = usvg::Tree::from_data(svg_data, &usvg::Options::default()).ok()?;
    let svg_size = tree.size();
    let sx = 32.0 / svg_size.width();
    let sy = 32.0 / svg_size.height();
    let s = sx.min(sy);
    let ts = tiny_skia::Transform::from_scale(s, s);
    let mut pixmap = tiny_skia::Pixmap::new(32, 32)?;
    let mut pm = pixmap.as_mut();
    resvg::render(&tree, ts, &mut pm);
    Icon::from_rgba(pixmap.data().to_vec(), 32, 32).ok()
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_tray() {}

#[cfg(not(target_os = "windows"))]
pub fn next_command() -> Option<TrayCommand> {
    None
}
