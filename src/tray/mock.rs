//! In-memory test double for [`TrayRunner`].

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::TrayRunner;
use crate::error::AppResult;

/// Counts how many times [`TrayRunner::run`] was invoked, without touching any
/// desktop APIs.
#[derive(Debug, Clone, Default)]
pub struct MockTrayRunner {
    runs: Arc<AtomicUsize>,
}

impl MockTrayRunner {
    /// Create a fresh, never-run mock.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many times the runner has been asked to run.
    #[must_use]
    pub fn run_count(&self) -> usize {
        self.runs.load(Ordering::SeqCst)
    }
}

impl TrayRunner for MockTrayRunner {
    fn run(&self) -> AppResult<()> {
        self.runs.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
