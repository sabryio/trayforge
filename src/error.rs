//! The application-wide error model.
//!
//! One error enum is shared by every adapter so that the orchestrator
//! ([`crate::app::Application`]) can stay generic. Adapters convert their
//! library-specific failures into the relevant variant at the boundary, keeping
//! third-party error types out of the core.

use thiserror::Error;

/// Everything that can go wrong while running the application.
///
/// Variants are grouped by the *port* that produced them rather than by the
/// underlying library, so the core never has to know which crate failed.
#[derive(Debug, Error)]
pub enum AppError {
    /// Invalid or unusable configuration (bad paths, missing environment, ...).
    #[error("configuration error: {0}")]
    Config(String),

    /// The auto-start adapter could not register/unregister or query login items.
    #[error("auto-start error: {0}")]
    AutoStart(String),

    /// The tray adapter could not build the icon, menu or event loop.
    #[error("tray error: {0}")]
    Tray(String),

    /// Another instance of the application already holds the instance lock.
    #[error("another instance is already running")]
    AlreadyRunning,

    /// A plain I/O failure (file system, process introspection, ...).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience alias used throughout the crate.
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_errors_convert_automatically() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: AppError = io.into();

        assert!(matches!(err, AppError::Io(_)));
    }

    #[test]
    fn already_running_has_a_clear_message() {
        assert_eq!(
            AppError::AlreadyRunning.to_string(),
            "another instance is already running"
        );
    }
}
