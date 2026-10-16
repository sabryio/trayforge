//! Tray-process supervision port.
//!
//! The interactive dashboard (in the `trayforge-cli` crate) must stay responsive
//! while the tray agent runs, and the tray agent needs the desktop event loop on
//! the *main* thread. So instead of running the tray in-process, the dashboard
//! launches it as a child process and supervises it through this port.
//!
//! `run` (the auto-start entrypoint) still executes the tray in-process; only the
//! dashboard uses supervision.

pub mod desktop;
pub mod mock;

pub use desktop::DesktopTraySupervisor;
pub use mock::MockTraySupervisor;

use crate::error::AppResult;

/// Lifecycle snapshot of the supervised tray process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrayState {
    /// No child process is running.
    #[default]
    Stopped,
    /// A child process is alive with the given OS process id.
    Running {
        /// Operating-system process id.
        pid: u32,
    },
}

impl TrayState {
    /// Whether a child process is currently alive.
    #[must_use]
    pub fn is_running(self) -> bool {
        matches!(self, Self::Running { .. })
    }
}

/// Starts, stops and reports on the tray child process.
pub trait TraySupervisor: Send + Sync {
    /// Ensure a tray child process is running, returning its new state.
    ///
    /// # Errors
    /// Returns [`crate::error::AppError::Tray`] if the process cannot be spawned.
    fn start(&self) -> AppResult<TrayState>;

    /// Terminate the child process started by this supervisor, if any.
    ///
    /// # Errors
    /// Returns [`crate::error::AppError::Tray`] if the process cannot be killed.
    fn stop(&self) -> AppResult<()>;

    /// Current state, reaping the child if it has already exited.
    fn state(&self) -> TrayState;
}
