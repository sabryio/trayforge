//! Real, OS-backed auto-start adapter.

use auto_launch::{AutoLaunchBuilder, MacOSLaunchMode};

use super::AutoStartService;
use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
};

/// Implements [`AutoStartService`] on top of the [`auto_launch`] crate:
///
/// * **Windows** — `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
/// * **macOS** — a per-user LaunchAgent
/// * **Linux/BSD** — an XDG autostart `.desktop` entry
///
/// The executable path is taken from [`AppConfig`]; in production this should be
/// the path of an *installed* binary, not a `target/debug` artifact.
#[derive(Debug, Clone)]
pub struct DesktopAutoStartService {
    config: AppConfig,
}

impl DesktopAutoStartService {
    /// Create the adapter for the given configuration.
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Build a fresh `auto_launch` handle for the current configuration.
    fn build(&self) -> AppResult<auto_launch::AutoLaunch> {
        AutoLaunchBuilder::new()
            .set_app_name(&self.config.app_name)
            .set_app_path(&self.config.executable_path_string())
            .set_macos_launch_mode(MacOSLaunchMode::LaunchAgent)
            .build()
            .map_err(|err| AppError::AutoStart(err.to_string()))
    }
}

impl AutoStartService for DesktopAutoStartService {
    fn enable(&self) -> AppResult<()> {
        self.build()?
            .enable()
            .map_err(|err| AppError::AutoStart(err.to_string()))
    }

    fn disable(&self) -> AppResult<()> {
        self.build()?
            .disable()
            .map_err(|err| AppError::AutoStart(err.to_string()))
    }

    fn is_enabled(&self) -> AppResult<bool> {
        self.build()?
            .is_enabled()
            .map_err(|err| AppError::AutoStart(err.to_string()))
    }
}
