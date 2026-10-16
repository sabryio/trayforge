//! Tray runtime port.
//!
//! The core does not know that a tray icon is a desktop GUI concept. It only
//! knows that "running the tray process" is something a [`TrayRunner`] can do.
//! Swapping winit for another event loop, or running headless in a test, is a
//! matter of providing a different adapter.

pub mod desktop;
pub mod icon;
pub mod menu;
pub mod mock;

pub use desktop::DesktopTrayRunner;
pub use mock::MockTrayRunner;

use crate::error::AppResult;

/// Runs the long-lived tray process until the user quits.
pub trait TrayRunner: Send {
    /// Block, showing the tray icon and pumping the desktop event loop, until
    /// the user requests to quit.
    ///
    /// # Errors
    /// Returns [`crate::error::AppError::Tray`] if the icon, menu or event loop
    /// cannot be created or the loop fails.
    fn run(&self) -> AppResult<()>;
}
