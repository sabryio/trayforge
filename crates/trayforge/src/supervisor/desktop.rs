//! Real, OS process-backed tray supervisor.

use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Mutex, MutexGuard},
};

use super::{TrayState, TraySupervisor};
use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
};

/// Windows flag that prevents a console window from flashing up for the child.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Spawns `trayforge run` as a detached child and tracks its handle.
///
/// The child's stdio is redirected to the null device so it cannot scribble over
/// the dashboard's alternate screen. The child is killed when the supervisor is
/// dropped, so closing the dashboard never strands an agent process.
pub struct DesktopTraySupervisor {
    executable: PathBuf,
    child: Mutex<Option<Child>>,
}

impl DesktopTraySupervisor {
    /// Create a supervisor that will launch the binary in `config`.
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            executable: config.executable_path,
            child: Mutex::new(None),
        }
    }

    fn slot(&self) -> AppResult<MutexGuard<'_, Option<Child>>> {
        self.child
            .lock()
            .map_err(|_| AppError::Tray("tray supervisor mutex poisoned".to_string()))
    }

    /// Build the command used to launch the tray child.
    fn command(&self, extra_args: &[String]) -> Command {
        let mut command = Command::new(&self.executable);
        command
            .arg("run")
            .args(extra_args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        command
    }
}

impl TraySupervisor for DesktopTraySupervisor {
    fn start(&self, extra_args: &[String]) -> AppResult<TrayState> {
        let mut slot = self.slot()?;

        // Reuse an already-running child instead of spawning a second one.
        if let Some(child) = slot.as_mut()
            && child.try_wait().ok().flatten().is_none()
        {
            return Ok(TrayState::Running { pid: child.id() });
        }

        let child = self
            .command(extra_args)
            .spawn()
            .map_err(|err| AppError::Tray(format!("could not start tray process: {err}")))?;
        let pid = child.id();
        tracing::info!(pid, "tray process spawned");

        *slot = Some(child);
        Ok(TrayState::Running { pid })
    }

    fn stop(&self) -> AppResult<()> {
        let mut slot = self.slot()?;
        if let Some(mut child) = slot.take() {
            child
                .kill()
                .map_err(|err| AppError::Tray(format!("could not stop tray process: {err}")))?;
            // Reap the process so it does not linger as a zombie.
            let _ = child.wait();
        }
        Ok(())
    }

    fn state(&self) -> TrayState {
        let Ok(mut slot) = self.child.lock() else {
            return TrayState::Stopped;
        };

        match slot.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(None) => TrayState::Running { pid: child.id() },
                _ => {
                    *slot = None;
                    TrayState::Stopped
                }
            },
            None => TrayState::Stopped,
        }
    }
}

impl Drop for DesktopTraySupervisor {
    fn drop(&mut self) {
        if let Ok(slot) = self.child.get_mut()
            && let Some(mut child) = slot.take()
        {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
