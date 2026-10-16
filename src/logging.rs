//! Structured, file-backed logging.
//!
//! A tray process often runs without an attached terminal (for example when
//! started by the OS at login), so logs go to a per-user file *and* to stderr.
//! The returned [`LoggingGuard`] owns the background writer thread and must be
//! kept alive for the duration of the process.

use std::fs;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{config::AppConfig, error::AppResult};

/// Keeps the asynchronous log writer alive.
///
/// Dropping it flushes pending records and stops the writer thread; bind it to a
/// variable in `main` and let it drop at process exit.
#[derive(Debug)]
#[must_use = "dropping the guard flushes and stops the log writer thread"]
pub struct LoggingGuard {
    _file: Option<WorkerGuard>,
}

/// Install the global tracing subscriber, writing to `<log_dir>/tray-cli.log`
/// and to stderr.
///
/// `verbose` counts `-v` flags: `0` = `info`, `1` = `debug`, `2+` = `trace`. A
/// `RUST_LOG` environment variable, when set, takes precedence.
///
/// # Errors
/// Returns [`crate::error::AppError::Io`] when the log directory cannot be created.
pub fn init(config: &AppConfig, verbose: u8) -> AppResult<LoggingGuard> {
    fs::create_dir_all(&config.log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&config.log_dir, "tray-cli.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level_for(verbose)));

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(non_blocking);
    let stderr_layer = fmt::layer().with_target(false).with_writer(std::io::stderr);

    let installed = tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stderr_layer)
        .try_init();

    // A second `init` (as happens in a test binary) must not abort the app; we
    // simply keep logging through the already-installed subscriber.
    if installed.is_err() {
        tracing::debug!("global tracing subscriber already installed");
        return Ok(LoggingGuard { _file: None });
    }

    Ok(LoggingGuard { _file: Some(guard) })
}

/// Map a verbosity count to a tracing level directive.
#[must_use]
pub fn level_for(verbose: u8) -> &'static str {
    match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbosity_maps_to_levels() {
        assert_eq!(level_for(0), "info");
        assert_eq!(level_for(1), "debug");
        assert_eq!(level_for(9), "trace");
    }

    #[test]
    fn init_creates_the_log_directory() {
        let dir = std::env::temp_dir().join(format!("tray-cli-log-test-{}", std::process::id()));
        let config = AppConfig {
            app_name: "tray-cli".to_string(),
            app_id: "dev.traycli.tray-cli".to_string(),
            executable_path: std::env::current_exe().expect("exe"),
            tooltip: "tray-cli".to_string(),
            log_dir: dir.clone(),
        };

        let guard = init(&config, 0).expect("logging init");
        assert!(dir.is_dir());

        drop(guard);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
