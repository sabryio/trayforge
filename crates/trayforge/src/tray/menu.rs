//! Tray context-menu construction.
//!
//! Isolated from the event loop, and driven entirely by a [`TraySpec`]: the menu
//! is *data*, so it can grow (settings, about, logs, custom actions, ...) without
//! this module knowing anything new. The returned [`TrayMenu`] owns every native
//! handle, which keeps the items alive for as long as the menu is shown.

use muda::{IsMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem};

use super::{
    TrayItem,
    spec::{TrayAction, TraySpec},
};
use crate::error::{AppError, AppResult};

/// A built tray menu together with the *id → action* mapping.
pub struct TrayMenu {
    menu: Menu,
    /// Which native entry triggers which action. Only interactive entries appear.
    actions: Vec<(MenuId, TrayAction)>,
    // Native menu items must outlive the menu itself; keep them owned here.
    _items: Vec<OwnedItem>,
}

/// A native menu item, so entries and dividers can be owned together.
enum OwnedItem {
    /// An enabled or disabled entry.
    Entry(MenuItem),
    /// A horizontal divider.
    Divider(PredefinedMenuItem),
}

impl OwnedItem {
    fn as_item(&self) -> &dyn IsMenuItem {
        match self {
            Self::Entry(entry) => entry,
            Self::Divider(divider) => divider,
        }
    }
}

impl TrayMenu {
    /// Build the menu described by `spec`.
    ///
    /// # Errors
    /// Returns [`AppError::Tray`] if the native menu cannot be assembled.
    pub fn build(spec: &TraySpec) -> AppResult<Self> {
        let menu = Menu::new();
        let mut items = Vec::with_capacity(spec.items().len());
        let mut actions = Vec::new();

        for item in spec.items() {
            let owned = native_item(item);

            // Labels and dividers are inert; everything else does something.
            if let Some(action) = item.action() {
                actions.push((owned.as_item().id().clone(), action));
            }

            menu.append(owned.as_item())
                .map_err(|err| AppError::Tray(format!("failed to assemble tray menu: {err}")))?;
            items.push(owned);
        }

        Ok(Self {
            menu,
            actions,
            _items: items,
        })
    }

    /// The underlying menu, ready to hand to the tray icon.
    #[must_use]
    pub fn menu(&self) -> &Menu {
        &self.menu
    }

    /// What choosing `id` should do, or [`None`] for an unknown entry.
    #[must_use]
    pub fn resolve(&self, id: &MenuId) -> Option<&TrayAction> {
        self.actions
            .iter()
            .find(|(key, _)| key == id)
            .map(|(_, action)| action)
    }
}

/// Turn one declarative entry into its native counterpart.
fn native_item(item: &TrayItem) -> OwnedItem {
    match item {
        TrayItem::Separator => OwnedItem::Divider(PredefinedMenuItem::separator()),
        TrayItem::Label { text } => OwnedItem::Entry(MenuItem::new(text, false, None)),
        TrayItem::Quit => OwnedItem::Entry(MenuItem::new("Quit", true, None)),
        TrayItem::Action { label, .. } => OwnedItem::Entry(MenuItem::new(label, true, None)),
        TrayItem::Open { label, .. } => OwnedItem::Entry(MenuItem::new(label, true, None)),
    }
}
