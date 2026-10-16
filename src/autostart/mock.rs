//! In-memory test double for [`AutoStartService`].

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use super::AutoStartService;
use crate::error::AppResult;

/// Records the requested auto-start state without touching the OS.
#[derive(Debug, Clone, Default)]
pub struct MockAutoStartService {
    enabled: Arc<AtomicBool>,
}

impl MockAutoStartService {
    /// Create a service starting in the *disabled* state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a service starting in the *enabled* state.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Inspect the current recorded state.
    #[must_use]
    pub fn state(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }
}

impl AutoStartService for MockAutoStartService {
    fn enable(&self) -> AppResult<()> {
        self.enabled.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn disable(&self) -> AppResult<()> {
        self.enabled.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_enabled(&self) -> AppResult<bool> {
        Ok(self.state())
    }
}
