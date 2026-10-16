//! Runtime configuration detection.
//!
//! [`AppConfig`] is a small, platform-agnostic value object describing *where*
//! the app lives and *how it should identify itself*. Adapters receive it; the
//! core creates it once from the environment.

use std::{
    env,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

use crate::error::AppResult;

/// Reverse-DNS-ish identity used for OS-level registration (instance mutex,
/// login item name, log directory).
const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "trayforge";

/// Immutable configuration shared by every adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    /// Human/app name, e.g. `trayforge`.
    pub app_name: String,
    /// Stable, filesystem-safe identifier used for locking and registration.
    pub app_id: String,
    /// Absolute path to the currently running executable.
    pub executable_path: PathBuf,
    /// Tooltip shown when hovering the tray icon.
    pub tooltip: String,
    /// Directory where structured logs are written.
    pub log_dir: PathBuf,
}

impl AppConfig {
    /// Detect the configuration for the current process.
    ///
    /// # Errors
    /// Returns [`crate::error::AppError::Io`] when the executable path cannot be resolved.
    pub fn detect() -> AppResult<Self> {
        let executable_path = env::current_exe()?;
        let app_name = env!("CARGO_PKG_NAME").to_string();

        Ok(Self::new(app_name, executable_path))
    }

    /// Build a configuration from explicit inputs (used by [`Self::detect`] and
    /// by tests that must not depend on the real process).
    #[must_use]
    pub fn new(app_name: impl Into<String>, executable_path: PathBuf) -> Self {
        let app_name = app_name.into();
        let app_id = format!("{QUALIFIER}.{ORGANIZATION}.{app_name}");
        let log_dir = default_log_dir(&app_name);
        let tooltip = format!("{app_name} — running");

        Self {
            app_id,
            tooltip,
            log_dir,
            app_name,
            executable_path,
        }
    }

    /// The executable path as a lossy [`String`], which is the shape OS
    /// registration APIs expect.
    #[must_use]
    pub fn executable_path_string(&self) -> String {
        self.executable_path.to_string_lossy().into_owned()
    }
}

/// Pick a per-user log directory, falling back to the temp dir when the OS
/// conventions cannot be resolved.
fn default_log_dir(app_name: &str) -> PathBuf {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, app_name)
        .map(|dirs| {
            dirs.state_dir()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| dirs.data_local_dir().to_path_buf())
        })
        .unwrap_or_else(|| env::temp_dir().join(app_name))
        .join("logs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_derives_stable_identity() {
        let config = AppConfig::new("trayforge", PathBuf::from("/opt/trayforge"));

        assert_eq!(config.app_id, "dev.trayforge.trayforge");
        assert_eq!(config.tooltip, "trayforge — running");
        assert!(config.log_dir.ends_with("logs"));
    }

    #[test]
    fn executable_path_is_reported_lossily() {
        let config = AppConfig::new("trayforge", PathBuf::from("/opt/trayforge"));
        assert_eq!(config.executable_path_string(), "/opt/trayforge");
    }

    #[test]
    fn detect_resolves_the_running_binary() {
        let config = AppConfig::detect().expect("detect should succeed in tests");
        assert!(config.executable_path.is_absolute());
    }
}
