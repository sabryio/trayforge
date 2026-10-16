//! Single-instance guard port.
//!
//! A tray utility is easy to launch twice — once manually and once from the
//! login item — which would produce duplicate icons. The core depends only on
//! this narrow trait; [`desktop::DesktopInstanceLock`] implements it with an OS
//! mutex/socket/flock, and [`mock::MockInstanceLock`] makes it testable.

pub mod desktop;
pub mod mock;

pub use desktop::DesktopInstanceLock;
pub use mock::MockInstanceLock;

use crate::error::AppResult;

/// Guards against running more than one copy of the application.
///
/// Implementations must hold whatever OS resource backs the guard for as long as
/// the value is alive, so that the lock is released on drop/exit.
pub trait InstanceLock: Send + Sync {
    /// Attempt to become the primary instance.
    ///
    /// Returns `Ok(true)` when this process now owns the lock, and `Ok(false)`
    /// when another live instance already holds it.
    ///
    /// # Errors
    /// Returns an error when the underlying OS mechanism fails outright (as
    /// opposed to simply being held by someone else).
    fn try_acquire(&self) -> AppResult<bool>;
}
