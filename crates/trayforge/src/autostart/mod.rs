//! Login / auto-start port.
//!
//! The application never talks to the Windows registry, a LaunchAgent or an XDG
//! desktop entry directly. It depends on this abstraction, which keeps the
//! mechanism replaceable and, more importantly, testable.

pub mod desktop;
pub mod mock;

pub use desktop::DesktopAutoStartService;
pub use mock::MockAutoStartService;

use crate::error::AppResult;

/// Registers and queries whether the application starts at login.
pub trait AutoStartService: Send + Sync {
    /// Register the application to start at login.
    ///
    /// # Errors
    /// Returns [`crate::error::AppError::AutoStart`] when registration fails.
    fn enable(&self) -> AppResult<()>;

    /// Remove the application from the login startup items.
    ///
    /// # Errors
    /// Returns [`crate::error::AppError::AutoStart`] when unregistration fails.
    fn disable(&self) -> AppResult<()>;

    /// Report whether the application is currently registered.
    ///
    /// # Errors
    /// Returns [`crate::error::AppError::AutoStart`] when the query fails.
    fn is_enabled(&self) -> AppResult<bool>;
}
