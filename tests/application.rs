//! Integration tests for the public orchestration API.
//!
//! These live outside `src/` on purpose: they prove that the crate is usable
//! through its public surface and that the ports can be satisfied by stand-ins
//! that the core has never heard of.

use tray_cli::{
    app::Application, autostart::MockAutoStartService, cli::Command, error::AppError,
    instance::MockInstanceLock, tray::MockTrayRunner,
};

/// Wire an [`Application`] from test doubles and hand back the spies.
fn harness(
    instance: MockInstanceLock,
    autostart: MockAutoStartService,
) -> (Application, MockTrayRunner, MockAutoStartService) {
    let tray = MockTrayRunner::new();
    let application = Application::new(
        Box::new(autostart.clone()),
        Box::new(tray.clone()),
        Box::new(instance),
    );
    (application, tray, autostart)
}

#[test]
fn run_starts_the_tray_process_once() {
    let (application, tray, _) =
        harness(MockInstanceLock::available(), MockAutoStartService::new());

    application.execute(Command::Run).expect("run");

    assert_eq!(tray.run_count(), 1);
}

#[test]
fn a_second_instance_is_refused_before_the_tray_starts() {
    let (application, tray, _) = harness(MockInstanceLock::taken(), MockAutoStartService::new());

    let error = application.execute(Command::Run).expect_err("refused");

    assert!(matches!(error, AppError::AlreadyRunning));
    assert_eq!(tray.run_count(), 0);
}

#[test]
fn autostart_can_be_toggled_from_the_cli() {
    let (application, _, autostart) =
        harness(MockInstanceLock::available(), MockAutoStartService::new());

    application
        .execute(Command::EnableAutostart)
        .expect("enable");
    assert!(autostart.state());

    application
        .execute(Command::DisableAutostart)
        .expect("disable");
    assert!(!autostart.state());
}
