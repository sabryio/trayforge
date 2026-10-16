//! In-memory test double for [`TraySupervisor`].

use std::sync::{Arc, Mutex, MutexGuard};

use super::{TrayState, TraySupervisor};
use crate::error::AppResult;

/// Tracks a synthetic tray state without spawning anything.
#[derive(Debug, Clone, Default)]
pub struct MockTraySupervisor {
    state: Arc<Mutex<TrayState>>,
}

impl MockTraySupervisor {
    /// A supervisor whose child is stopped.
    #[must_use]
    pub fn stopped() -> Self {
        Self::default()
    }

    /// A supervisor whose child is already running.
    #[must_use]
    pub fn running() -> Self {
        Self {
            state: Arc::new(Mutex::new(TrayState::Running { pid: 4242 })),
        }
    }

    fn lock(&self) -> MutexGuard<'_, TrayState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl TraySupervisor for MockTraySupervisor {
    fn start(&self) -> AppResult<TrayState> {
        let mut state = self.lock();
        *state = TrayState::Running { pid: 4242 };
        Ok(*state)
    }

    fn stop(&self) -> AppResult<()> {
        *self.lock() = TrayState::Stopped;
        Ok(())
    }

    fn state(&self) -> TrayState {
        *self.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_stop_transition_state() {
        let supervisor = MockTraySupervisor::stopped();
        assert_eq!(supervisor.state(), TrayState::Stopped);

        let running = supervisor.start().expect("start");
        assert!(running.is_running());
        assert!(supervisor.state().is_running());

        supervisor.stop().expect("stop");
        assert_eq!(supervisor.state(), TrayState::Stopped);
    }
}
