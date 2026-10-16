//! The application layer: use-case orchestration.
//!
//! [`Application`] is the only place that knows *what* the program does. It
//! knows nothing about the registry, tray icons, mutexes or event loops — only
//! about the narrow ports it was handed. This is the dependency-inversion
//! boundary: high-level policy depends on abstractions, and `main.rs` decides
//! which concrete adapters satisfy them.

use crate::{
    autostart::AutoStartService,
    cli::Command,
    error::{AppError, AppResult},
    instance::InstanceLock,
    tray::TrayRunner,
};

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
    pub fn execute(&self, command: Command) -> AppResult<()> {
        match command {
            Command::Run => self.run_tray(),
            Command::EnableAutostart => self.enable_autostart(),
            Command::DisableAutostart => self.disable_autostart(),
            Command::Status => self.report_status(),
        }
    }

    /// Start the long-lived tray process, guarding against a second instance.
    fn run_tray(&self) -> AppResult<()> {
        if !self.instance.try_acquire()? {
            return Err(AppError::AlreadyRunning);
        }

        println!("Starting tray process. Use the tray menu to quit.");
        tracing::info!("starting tray process");
        self.tray.run()
    }

    fn enable_autostart(&self) -> AppResult<()> {
        self.autostart.enable()?;
        println!("Auto-start enabled.");
        Ok(())
    }

    fn disable_autostart(&self) -> AppResult<()> {
        self.autostart.disable()?;
        println!("Auto-start disabled.");
        Ok(())
    }

    fn report_status(&self) -> AppResult<()> {
        let enabled = self.autostart.is_enabled()?;
        println!("Auto-start enabled: {enabled}");
        Ok(())
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

        harness.app.execute(Command::Run).expect("run succeeds");

        assert_eq!(harness.tray.run_count(), 1);
    }

    #[test]
    fn run_is_refused_when_another_instance_holds_the_lock() {
        let harness = harness(MockInstanceLock::taken(), MockAutoStartService::new());

        let err = harness
            .app
            .execute(Command::Run)
            .expect_err("must be refused");

        assert!(matches!(err, AppError::AlreadyRunning));
        assert_eq!(harness.tray.run_count(), 0);
    }

    #[test]
    fn enable_autostart_flips_the_service() {
        let harness = harness(MockInstanceLock::available(), MockAutoStartService::new());

        harness
            .app
            .execute(Command::EnableAutostart)
            .expect("enable succeeds");

        assert!(harness.autostart.state());
    }

    #[test]
    fn disable_autostart_flips_the_service_back() {
        let harness = harness(
            MockInstanceLock::available(),
            MockAutoStartService::enabled(),
        );

        harness
            .app
            .execute(Command::DisableAutostart)
            .expect("disable succeeds");

        assert!(!harness.autostart.state());
    }

    #[test]
    fn status_queries_without_mutating() {
        let harness = harness(
            MockInstanceLock::available(),
            MockAutoStartService::enabled(),
        );

        harness
            .app
            .execute(Command::Status)
            .expect("status succeeds");

        assert!(harness.autostart.state());
        assert_eq!(harness.tray.run_count(), 0);
    }
}
