//! The application layer: use-case orchestration.
//!
//! [`Application`] is the only place that knows *what* the program does. It
//! knows nothing about the registry, tray icons, mutexes or event loops — only
//! about the narrow ports it was handed. This is the dependency-inversion
//! boundary: high-level policy depends on abstractions, and `main.rs` decides
//! which concrete adapters satisfy them.
//!
//! Crucially the core never prints. It returns a presentation-agnostic
//! [`Outcome`], which lets the very same use cases be driven by the one-shot CLI
//! and by the interactive dashboard without one duplicating the other.

use crate::{
    autostart::AutoStartService,
    error::{AppError, AppResult},
    instance::InstanceLock,
    tray::TrayRunner,
};

/// A single headless use case the core can perform.
///
/// This is deliberately distinct from [`crate::cli::Command`]: the interactive
/// dashboard is a presentation concern and has no use-case equivalent, so it can
/// never reach the core by accident.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseCase {
    /// Start the long-lived tray process.
    Run,
    /// Register the app to start at login.
    EnableAutoStart,
    /// Remove the app from the login startup items.
    DisableAutoStart,
    /// Report whether auto-start is currently enabled.
    ReportStatus,
}

/// What a use case produced, described without any presentation formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The tray event loop has finished (the process is about to exit).
    TrayStarted,
    /// Auto-start is now enabled.
    AutoStartEnabled,
    /// Auto-start is now disabled.
    AutoStartDisabled,
    /// The current auto-start state.
    AutoStartStatus {
        /// Whether auto-start is enabled.
        enabled: bool,
    },
}

/// Coordinates the application's use cases over injected ports.
pub struct Application {
    autostart: Box<dyn AutoStartService>,
    tray: Box<dyn TrayRunner>,
    instance: Box<dyn InstanceLock>,
}

impl Application {
    /// Compose the application from its collaborators.
    ///
    /// The composition root (`main.rs`) is the only caller that passes real
    /// adapters; tests pass mock adapters.
    #[must_use]
    pub fn new(
        autostart: Box<dyn AutoStartService>,
        tray: Box<dyn TrayRunner>,
        instance: Box<dyn InstanceLock>,
    ) -> Self {
        Self {
            autostart,
            tray,
            instance,
        }
    }

    /// Execute a single use case.
    ///
    /// # Errors
    /// Propagates any adapter failure, and returns [`AppError::AlreadyRunning`]
    /// when `run` is requested while another instance holds the lock.
    pub fn execute(&self, use_case: UseCase) -> AppResult<Outcome> {
        match use_case {
            UseCase::Run => self.run_tray(),
            UseCase::EnableAutoStart => self.enable_autostart(),
            UseCase::DisableAutoStart => self.disable_autostart(),
            UseCase::ReportStatus => self.report_status(),
        }
    }

    /// Whether auto-start is currently enabled.
    ///
    /// A convenience query so interactive front-ends can poll state without
    /// fabricating a use-case call.
    ///
    /// # Errors
    /// Propagates the auto-start adapter failure.
    pub fn auto_start_enabled(&self) -> AppResult<bool> {
        self.autostart.is_enabled()
    }

    /// Whether the instance lock is free, i.e. no other copy is running.
    ///
    /// # Errors
    /// Propagates the instance-lock adapter failure.
    pub fn instance_available(&self) -> AppResult<bool> {
        self.instance.is_available()
    }

    /// Start the long-lived tray process, guarding against a second instance.
    fn run_tray(&self) -> AppResult<Outcome> {
        if !self.instance.try_acquire()? {
            return Err(AppError::AlreadyRunning);
        }

        tracing::info!("starting tray process");
        self.tray.run()?;
        Ok(Outcome::TrayStarted)
    }

    fn enable_autostart(&self) -> AppResult<Outcome> {
        self.autostart.enable()?;
        Ok(Outcome::AutoStartEnabled)
    }

    fn disable_autostart(&self) -> AppResult<Outcome> {
        self.autostart.disable()?;
        Ok(Outcome::AutoStartDisabled)
    }

    fn report_status(&self) -> AppResult<Outcome> {
        Ok(Outcome::AutoStartStatus {
            enabled: self.autostart.is_enabled()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        autostart::MockAutoStartService, instance::MockInstanceLock, tray::MockTrayRunner,
    };

    /// The three mocks the application talks to, kept together for assertions.
    struct Harness {
        app: Application,
        autostart: MockAutoStartService,
        tray: MockTrayRunner,
    }

    fn harness(instance: MockInstanceLock, autostart: MockAutoStartService) -> Harness {
        let tray = MockTrayRunner::new();
        let app = Application::new(
            Box::new(autostart.clone()),
            Box::new(tray.clone()),
            Box::new(instance),
        );
        Harness {
            app,
            autostart,
            tray,
        }
    }

    #[test]
    fn run_delegates_to_the_tray_runner() {
        let harness = harness(MockInstanceLock::available(), MockAutoStartService::new());

        let outcome = harness.app.execute(UseCase::Run).expect("run succeeds");

        assert_eq!(outcome, Outcome::TrayStarted);
        assert_eq!(harness.tray.run_count(), 1);
    }

    #[test]
    fn run_is_refused_when_another_instance_holds_the_lock() {
        let harness = harness(MockInstanceLock::taken(), MockAutoStartService::new());

        let err = harness
            .app
            .execute(UseCase::Run)
            .expect_err("must be refused");

        assert!(matches!(err, AppError::AlreadyRunning));
        assert_eq!(harness.tray.run_count(), 0);
    }

    #[test]
    fn enable_autostart_flips_the_service() {
        let harness = harness(MockInstanceLock::available(), MockAutoStartService::new());

        let outcome = harness
            .app
            .execute(UseCase::EnableAutoStart)
            .expect("enable succeeds");

        assert_eq!(outcome, Outcome::AutoStartEnabled);
        assert!(harness.autostart.state());
    }

    #[test]
    fn disable_autostart_flips_the_service_back() {
        let harness = harness(
            MockInstanceLock::available(),
            MockAutoStartService::enabled(),
        );

        let outcome = harness
            .app
            .execute(UseCase::DisableAutoStart)
            .expect("disable succeeds");

        assert_eq!(outcome, Outcome::AutoStartDisabled);
        assert!(!harness.autostart.state());
    }

    #[test]
    fn status_queries_without_mutating() {
        let harness = harness(
            MockInstanceLock::available(),
            MockAutoStartService::enabled(),
        );

        let outcome = harness.app.execute(UseCase::ReportStatus).expect("status");

        assert_eq!(outcome, Outcome::AutoStartStatus { enabled: true });
        assert!(harness.autostart.state());
        assert_eq!(harness.tray.run_count(), 0);
    }

    #[test]
    fn convenience_queries_reflect_the_ports() {
        let harness = harness(
            MockInstanceLock::available(),
            MockAutoStartService::enabled(),
        );

        assert!(harness.app.auto_start_enabled().expect("autostart"));
        assert!(harness.app.instance_available().expect("instance"));
    }
}
