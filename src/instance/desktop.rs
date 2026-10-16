//! Real, OS-backed single-instance lock.

use std::sync::Mutex;

use single_instance::SingleInstance;

use super::InstanceLock;
use crate::error::{AppError, AppResult};

/// Uses the [`single_instance`] crate, which is backed by a named mutex on
/// Windows, an abstract Unix socket on Linux and an advisory `flock` on macOS.
///
/// The acquired [`SingleInstance`] handle is retained inside the lock so the OS
/// resource is not released until the lock itself is dropped.
pub struct DesktopInstanceLock {
    app_id: String,
    held: Mutex<Option<SingleInstance>>,
}

impl DesktopInstanceLock {
    /// Create a lock for the given stable application id.
    #[must_use]
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            held: Mutex::new(None),
        }
    }
}

impl InstanceLock for DesktopInstanceLock {
    fn try_acquire(&self) -> AppResult<bool> {
        let mut held = self
            .held
            .lock()
            .map_err(|_| AppError::Config("instance lock mutex poisoned".to_string()))?;

        if held.is_some() {
            // Already acquired by this process.
            return Ok(true);
        }

        let instance = SingleInstance::new(&self.app_id).map_err(|err| {
            AppError::Config(format!(
                "could not create instance lock '{}': {err}",
                self.app_id
            ))
        })?;

        if instance.is_single() {
            *held = Some(instance);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_acquire_succeeds_and_is_idempotent() {
        let id = format!("tray-cli-test-{}", std::process::id());
        let lock = DesktopInstanceLock::new(id);

        assert!(lock.try_acquire().expect("first acquire"));
        // A second call from the same lock object is a no-op success.
        assert!(lock.try_acquire().expect("second acquire"));
    }

    #[test]
    fn a_second_lock_object_cannot_acquire_the_same_id() {
        let id = format!("tray-cli-test-dup-{}", std::process::id());
        let primary = DesktopInstanceLock::new(id.clone());
        let secondary = DesktopInstanceLock::new(id);

        assert!(primary.try_acquire().expect("primary acquires"));
        assert!(!secondary.try_acquire().expect("secondary blocked"));
    }
}
