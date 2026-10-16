//! Tray context-menu construction.
//!
//! Isolated from the event loop so the menu can grow (settings, about, logs, ...)
//! without touching the runtime. The returned [`TrayMenu`] owns every native
//! handle, which keeps the items alive for as long as the menu is shown.

use muda::{Menu, MenuId, MenuItem, PredefinedMenuItem};

use crate::error::{AppError, AppResult};

/// A built tray menu together with its identifying handles.
pub struct TrayMenu {
    menu: Menu,
    quit_id: MenuId,
    // Native menu items must outlive the menu itself; keep them owned here.
    _status_item: MenuItem,
    _separator: PredefinedMenuItem,
    _quit_item: MenuItem,
}

impl TrayMenu {
    /// Build the default menu: a disabled status line, a separator and *Quit*.
    ///
    /// # Errors
    /// Returns [`AppError::Tray`] if the native menu cannot be assembled.
    pub fn build(app_name: &str) -> AppResult<Self> {
        let menu = Menu::new();
        let status_item = MenuItem::new(format!("{app_name} is running"), false, None);
        let separator = PredefinedMenuItem::separator();
        let quit_item = MenuItem::new("Quit", true, None);
        let quit_id = quit_item.id().clone();

        menu.append_items(&[&status_item, &separator, &quit_item])
            .map_err(|err| AppError::Tray(format!("failed to assemble tray menu: {err}")))?;

        Ok(Self {
            menu,
            quit_id,
            _status_item: status_item,
            _separator: separator,
            _quit_item: quit_item,
        })
    }

    /// The underlying menu, ready to hand to the tray icon.
    #[must_use]
    pub fn menu(&self) -> &Menu {
        &self.menu
    }

    /// The id that should be interpreted as "quit".
    #[must_use]
    pub fn quit_id(&self) -> &MenuId {
        &self.quit_id
    }

    /// Whether the given event id is the *Quit* item.
    #[must_use]
    pub fn is_quit(&self, id: &MenuId) -> bool {
        id == &self.quit_id
    }
}
