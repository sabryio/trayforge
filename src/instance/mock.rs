//! In-memory test double for [`InstanceLock`].

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use super::InstanceLock;
use crate::error::AppResult;

/// A deterministic, side-effect-free lock.
///
/// The shared [`AtomicBool`] mirrors the lock state so tests can inspect what
/// the orchestrator observed.
#[derive(Debug, Clone, Default)]
pub struct MockInstanceLock {
    available: Arc<AtomicBool>,
}

impl MockInstanceLock {
    /// A lock that reports itself as free (the usual "primary instance" case).
    #[must_use]
    pub fn available() -> Self {
        Self {
            available: Arc::new(AtomicBool::new(true)),
        }
    }

    /// A lock that reports itself as taken (the "already running" case).
    #[must_use]
    pub fn taken() -> Self {
        Self {
            available: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl InstanceLock for MockInstanceLock {
    fn try_acquire(&self) -> AppResult<bool> {
        Ok(self.available.swap(false, Ordering::SeqCst))
    }
}
